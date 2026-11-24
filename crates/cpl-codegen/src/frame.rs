//! A recursive fn's stack frame is as large as its largest region, at every
//! level of the recursion. The statements that run after a block's last
//! call back into the recursion need their locals once, at the return:
//! emitted as a noinline lambda the block returns through, those locals
//! leave the recursive frame (R13.3.6).

use crate::uses::effects;
use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, FnId, Program, Stmt};
use std::collections::{BTreeMap, BTreeSet};

/// The outer locals a cold suffix reads, each with whether it is passed by
/// value (a scalar it never writes) or by reference.
pub(crate) type Inputs = Vec<(usize, bool)>;

/// Where a cold suffix starts: its block, by address, and statement index.
pub(crate) type At = (*const Block, usize);

pub(crate) struct Fn<'a> {
    pub program: &'a Program,
    pub id: FnId,
    /// Whether a local is a scalar, kept in a register.
    pub scalar: &'a dyn std::ops::Fn(usize) -> bool,
    /// Whether a local exists in the C++ (not dropped or `void`).
    pub present: &'a dyn std::ops::Fn(usize) -> bool,
}

/// The cold suffixes of the fn's body, with their inputs.
pub(crate) fn cold_suffixes(f: &Fn) -> BTreeMap<At, Inputs> {
    let reach = effects::reaching(f.program, f.id);
    let mut cold = BTreeMap::new();
    tail_block(f, &reach, &f.program.fns[f.id].body, true, &mut cold);
    cold
}

/// `b` returns on every path, or falls off the end of the fn.
fn tail_block(
    f: &Fn,
    reach: &BTreeSet<FnId>,
    b: &Block,
    root: bool,
    cold: &mut BTreeMap<At, Inputs>,
) {
    let hot: Vec<bool> = b
        .stmts
        .iter()
        .map(|s| effects::calls_back(f.program, reach, s))
        .collect();
    let Some(last) = hot.iter().rposition(|&h| h) else {
        if !root {
            note(f, b, 0, cold);
        }
        return;
    };
    // A suffix that can't move out whole may still have branches that can.
    let upto = if note(f, b, last + 1, cold) {
        last + 1
    } else {
        b.stmts.len()
    };
    for (i, stmt) in b.stmts[..upto].iter().enumerate() {
        if let Stmt::If(_, then, otherwise) = stmt {
            for branch in [then, otherwise] {
                if returns(&branch.stmts) || i + 1 == b.stmts.len() {
                    tail_block(f, reach, branch, false, cold);
                }
            }
        }
    }
}

/// Records `b.stmts[start..]` when it is worth its own frame; true when
/// it is, or when it is empty.
fn note(f: &Fn, b: &Block, start: usize, cold: &mut BTreeMap<At, Inputs>) -> bool {
    let suffix = &b.stmts[start..];
    if suffix.is_empty() {
        return true;
    }
    if returns_outer(suffix) {
        return false;
    }
    let suffix = Block {
        stmts: suffix.to_vec(),
    };
    if heavy(f, &suffix) {
        cold.insert((b as *const Block, start), inputs(f, &suffix));
    }
    true
}

/// True when every path through `stmts` returns.
fn returns(stmts: &[Stmt]) -> bool {
    match stmts.last() {
        Some(Stmt::Return(_)) => true,
        Some(Stmt::If(_, t, o)) => returns(&t.stmts) && returns(&o.stmts),
        _ => false,
    }
}

/// The suffix returns a local declared before it: from the lambda that
/// would be a copy of the fn's local, where the fn moves it.
fn returns_outer(stmts: &[Stmt]) -> bool {
    let b = Block {
        stmts: stmts.to_vec(),
    };
    let inner = declared(&b);
    let mut found = false;
    walk::deep_stmts(&b, &mut |s| {
        if let Stmt::Return(Some(Expr {
            kind: ExprKind::Local(slot),
            ..
        })) = s
        {
            found |= !inner.contains(slot);
        }
    });
    found
}

fn declared(b: &Block) -> BTreeSet<usize> {
    let mut inner = BTreeSet::new();
    walk::deep_stmts(b, &mut |s| match s {
        Stmt::Let(slot, _) | Stmt::For(_, slot, ..) => {
            inner.insert(*slot);
        }
        _ => {}
    });
    inner
}

/// The suffix holds something in memory: a loop, a non-scalar local, or a
/// non-scalar value a call, builtin or block builds.
fn heavy(f: &Fn, b: &Block) -> bool {
    let mut heavy = false;
    walk::deep_stmts(b, &mut |s| match s {
        Stmt::For(..) | Stmt::While(..) => heavy = true,
        Stmt::Let(slot, _) => heavy |= !(f.scalar)(*slot),
        _ => {}
    });
    walk::deep_exprs(b, &mut |e| {
        heavy |= matches!(
            e.kind,
            ExprKind::Call(..) | ExprKind::Builtin(..) | ExprKind::Do(_) | ExprKind::Closure(_)
        ) && !e.ty.is_scalar();
    });
    heavy
}

/// What `b` reads from before it.
fn inputs(f: &Fn, b: &Block) -> Inputs {
    let inner = declared(b);
    let (mut read, mut written) = (BTreeSet::new(), BTreeSet::new());
    let mut write = |p: &cpl_ir::Place, read: &mut BTreeSet<usize>| {
        if let (false, slot) = walk::root(p) {
            written.insert(slot);
            read.insert(slot);
        }
    };
    walk::deep_stmts(b, &mut |s| {
        if let Stmt::Assign(p, ..) = s {
            write(p, &mut read);
        }
    });
    walk::deep_exprs(b, &mut |e| match &e.kind {
        ExprKind::Local(s) => {
            read.insert(*s);
        }
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(p) = arg {
                    write(p, &mut read);
                }
            }
        }
        ExprKind::MutSlice(p, ..) | ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, ..) => {
            write(p, &mut read)
        }
        _ => {}
    });
    read.into_iter()
        .filter(|s| !inner.contains(s) && (f.present)(*s))
        .map(|s| (s, (f.scalar)(s) && !written.contains(&s)))
        .collect()
}
