//! Sunk params no caller can move into: each call hands a variable read
//! after it, so the fn lends the param and copies where its body writes.

use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, Program, Stmt};
use std::collections::BTreeSet;

/// The parameters of `sunk` a fn moves into the struct it returns that no
/// call can move into: every call hands a variable a later statement of
/// its block still reads. Taken by value, such a parameter only moves the
/// copy from where the body writes it to the call, so it is taken by
/// `const&` instead. A parameter the body copies into a local or a store
/// stays sunk, since the copy is where the source names it whole.
pub(crate) fn unmovable_sinks(program: &Program, sunk: &[BTreeSet<usize>]) -> Vec<(usize, usize)> {
    let (mut called, mut movable) = (BTreeSet::new(), BTreeSet::new());
    let reads = |stmts: &[Stmt], var: (bool, usize)| {
        let mut found = false;
        for stmt in stmts {
            walk::stmt_exprs(stmt, &mut |e| {
                walk::deep_expr(e, &mut |inner| match inner.kind {
                    ExprKind::Local(s) => found |= var == (false, s),
                    ExprKind::Global(g) => found |= var == (true, g),
                    ExprKind::Closure(cid) => {
                        found |= (program.closures[cid].captures.iter())
                            .any(|c| (c.is_global, c.outer) == var)
                    }
                    _ => {}
                })
            });
            walk::stmt_blocks(stmt, &mut |inner| {
                walk::deep_exprs(inner, &mut |e| match e.kind {
                    ExprKind::Local(s) => found |= var == (false, s),
                    ExprKind::Global(g) => found |= var == (true, g),
                    _ => {}
                })
            });
        }
        found
    };
    // The calls of a statement outside its nested blocks, which `blocks`
    // visits as blocks of their own.
    fn calls<'p>(e: &'p Expr, out: &mut Vec<&'p Expr>) {
        if matches!(e.kind, ExprKind::Do(_)) {
            return;
        }
        if matches!(e.kind, ExprKind::Call(..)) {
            out.push(e);
        }
        walk::children(e, &mut |child| calls(child, out));
    }
    let mut visit = |block: &Block| {
        for (k, stmt) in block.stmts.iter().enumerate() {
            let mut found = Vec::new();
            walk::stmt_exprs(stmt, &mut |e| calls(e, &mut found));
            for call in found {
                let ExprKind::Call(fid, args) = &call.kind else {
                    continue;
                };
                for &slot in &sunk[*fid] {
                    let Some(Arg::Value(value)) = args.get(slot) else {
                        continue;
                    };
                    called.insert((*fid, slot));
                    let var = match value.kind {
                        ExprKind::Local(s) => Some((false, s)),
                        ExprKind::Global(g) => Some((true, g)),
                        _ => None,
                    };
                    if !var.is_some_and(|var| reads(&block.stmts[k + 1..], var)) {
                        movable.insert((*fid, slot));
                    }
                }
            }
        }
    };
    let bodies = (program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body))
        .chain([&program.setup, &program.main]);
    for body in bodies {
        walk::blocks(body, &mut visit);
    }
    let named_whole = |&(fid, slot): &(usize, usize)| {
        let f = &program.fns[fid];
        super::copied_once(f).contains(&slot) || super::stored_last(program, f).contains(&slot)
    };
    (called.difference(&movable).copied())
        .filter(|sink| !named_whole(sink))
        .collect()
}

/// The parameters of `params` some call hands a value a variable holds,
/// not one made for the call: a `Value` argument rooted at a local or a
/// global, or any other kind of argument. Taken by value, such a call
/// would copy where the C++ shows no copy.
pub(crate) fn held_args(program: &Program, params: &[BTreeSet<usize>]) -> BTreeSet<(usize, usize)> {
    fn rooted(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => true,
            ExprKind::Field(base, _) | ExprKind::Index(base, _) | ExprKind::Unwrap(base, _) => {
                rooted(base)
            }
            _ => false,
        }
    }
    let mut held = BTreeSet::new();
    let bodies = (program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body))
        .chain([&program.setup, &program.main]);
    for body in bodies {
        walk::deep_exprs(body, &mut |e| {
            let ExprKind::Call(fid, args) = &e.kind else {
                return;
            };
            for &slot in &params[*fid] {
                match args.get(slot) {
                    Some(Arg::Value(value)) if !rooted(value) => {}
                    _ => {
                        held.insert((*fid, slot));
                    }
                }
            }
        });
    }
    held
}

/// Value parameters a fn hands, whole and at their last read, to a sunk
/// parameter of the fn it calls: `fn new(id: T) = Set(0, Bag.new(id))`.
/// Taken by value, the call moves the parameter on instead of copying it.
pub(crate) fn forwarded(program: &Program, sunk: &[BTreeSet<usize>]) -> Vec<BTreeSet<usize>> {
    let mut out = vec![BTreeSet::new(); program.fns.len()];
    for (fid, f) in program.fns.iter().enumerate() {
        for (k, stmt) in f.body.stmts.iter().enumerate() {
            let (mut handed, mut named) = (Vec::new(), Vec::new());
            walk::stmt_exprs(stmt, &mut |e| {
                walk::deep_expr(e, &mut |inner| match &inner.kind {
                    ExprKind::Call(callee, args) => {
                        for &slot in &sunk[*callee] {
                            if let Some(Arg::Value(Expr {
                                kind: ExprKind::Local(p),
                                ..
                            })) = args.get(slot)
                            {
                                handed.push(*p);
                            }
                        }
                    }
                    ExprKind::Local(p) => named.push(*p),
                    _ => {}
                })
            });
            walk::stmt_blocks(stmt, &mut |inner| {
                walk::deep_exprs(inner, &mut |e| {
                    if let ExprKind::Local(p) = e.kind {
                        named.push(p);
                    }
                })
            });
            for p in handed {
                let ty = &f.locals[p].ty;
                let heap = f.params.get(p).is_some_and(|param| !param.mutable)
                    && !ty.is_scalar()
                    && !matches!(ty, cpl_ir::Ty::Fn(..));
                let once = named.iter().filter(|&&n| n == p).count() == 1;
                if heap && once && super::unread_after(program, &f.body, k, p) {
                    out[fid].insert(p);
                }
            }
        }
    }
    out
}
