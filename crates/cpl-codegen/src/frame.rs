//! A recursive fn's stack frame is as large as its largest region, at every
//! level of the recursion. The statements that run after a block's last
//! call back into the recursion need their locals once, at the return:
//! emitted as a noinline lambda the block returns through, those locals
//! leave the recursive frame (R13.3.6).

use crate::uses::effects;
use crate::uses::loops::each_loop;
use crate::uses::places::{each_put, Put};
use crate::uses::split::{each_sink, each_value, Sink};
use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, FnId, Program, Stmt};
use std::collections::{BTreeMap, BTreeSet};

/// The outer locals a cold suffix reads, each passed by reference whatever
/// its type, so the instances of a generic fn still differ only in types.
pub(crate) type Inputs = Vec<usize>;

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

/// A suffix that can leave the frame: where it starts, by address and by
/// its path of statement and branch indices (alike in every instance of a
/// generic fn), and whether it holds enough to be worth it.
struct Candidate {
    path: Vec<usize>,
    at: At,
    heavy: bool,
    inputs: Inputs,
}

/// The cold suffixes of the fn's body, with their inputs. A suffix that is
/// heavy in one of `siblings`, the other instances of the same generic fn,
/// is cold in each: whether a local is a scalar differs from one instance
/// to the next, and instances whose suffixes split alike stay one template.
pub(crate) fn cold_suffixes(f: &Fn, siblings: &[Fn]) -> BTreeMap<At, Inputs> {
    let heavy_in_some: BTreeSet<Vec<usize>> = siblings
        .iter()
        .flat_map(candidates)
        .filter(|c| c.heavy)
        .map(|c| c.path)
        .collect();
    candidates(f)
        .into_iter()
        .filter(|c| c.heavy || heavy_in_some.contains(&c.path))
        .map(|c| (c.at, c.inputs))
        .collect()
}

fn candidates(f: &Fn) -> Vec<Candidate> {
    let reach = effects::reaching(f.program, f.id);
    let mut out = Vec::new();
    tail_block(
        f,
        &reach,
        &f.program.fns[f.id].body,
        &mut Vec::new(),
        &mut out,
    );
    out
}

/// `b` returns on every path, or falls off the end of the fn; `path` leads
/// to it from the body, which it is when empty.
fn tail_block(
    f: &Fn,
    reach: &BTreeSet<FnId>,
    b: &Block,
    path: &mut Vec<usize>,
    out: &mut Vec<Candidate>,
) {
    let hot: Vec<bool> = b
        .stmts
        .iter()
        .map(|s| effects::calls_back(f.program, reach, s))
        .collect();
    let Some(last) = hot.iter().rposition(|&h| h) else {
        if !path.is_empty() {
            note(f, b, 0, path, out);
        }
        return;
    };
    // A suffix that can't move out whole may still have branches that can.
    let upto = if note(f, b, last + 1, path, out) {
        last + 1
    } else {
        b.stmts.len()
    };
    for (i, stmt) in b.stmts[..upto].iter().enumerate() {
        if let Stmt::If(_, then, otherwise) = stmt {
            for (k, branch) in [then, otherwise].into_iter().enumerate() {
                if returns(&branch.stmts) || i + 1 == b.stmts.len() {
                    path.extend([i, k]);
                    tail_block(f, reach, branch, path, out);
                    path.truncate(path.len() - 2);
                }
            }
        }
    }
}

/// Records `b.stmts[start..]` as a candidate; true when it can leave the
/// frame, or when it is empty.
fn note(f: &Fn, b: &Block, start: usize, path: &[usize], out: &mut Vec<Candidate>) -> bool {
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
    out.push(Candidate {
        path: [path, &[start]].concat(),
        at: (b as *const Block, start),
        heavy: heavy(f, &suffix),
        inputs: inputs(f, &suffix),
    });
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
    each_sink(&b, &mut |sink| {
        if let Sink::Return(Expr {
            kind: ExprKind::Local(slot),
            ..
        }) = sink
        {
            found |= !inner.contains(slot);
        }
    });
    found
}

fn declared(b: &Block) -> BTreeSet<usize> {
    let mut inner = BTreeSet::new();
    walk::binders(b, &mut |slot| {
        inner.insert(slot);
    });
    inner
}

/// The suffix holds something in memory: a loop, a non-scalar local, or a
/// non-scalar value a call, builtin or block builds.
fn heavy(f: &Fn, b: &Block) -> bool {
    let mut heavy = false;
    each_loop(b, &mut |_, _| heavy = true);
    each_sink(b, &mut |sink| {
        if let Sink::Let(slot, _) = sink {
            heavy |= !(f.scalar)(slot);
        }
    });
    each_value(b, &mut |e| {
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
    let mut read = BTreeSet::new();
    let write = |p: &cpl_ir::Place, read: &mut BTreeSet<usize>| {
        if let (false, slot) = walk::root(p) {
            read.insert(slot);
        }
    };
    each_put(b, &mut |put, _| {
        if let Put::Store(p, _) = put {
            write(p, &mut read);
        }
    });
    each_value(b, &mut |e| match &e.kind {
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
        .collect()
}
