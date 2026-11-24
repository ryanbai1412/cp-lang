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
