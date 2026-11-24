//! What a body does, as the inline pass (`crate::splice`) asks before it runs a call in place.

use cpl_ir::{Arg, Block, ClosureDef, Expr, ExprKind, FnId, Place, Slot, Stmt};
use std::collections::BTreeSet;

/// Whether `body` sorts a place itself or calls one of `seeds`.
pub(crate) fn sorts_or_calls(body: &Block, seeds: &BTreeSet<FnId>) -> bool {
    let mut sorts = false;
    cpl_ir::walk::deep_exprs(body, &mut |e| {
        sorts |= match e.kind {
            ExprKind::Sort(..) => true,
            ExprKind::Call(g, _) => seeds.contains(&g),
            _ => false,
        };
    });
    sorts
}

/// The `return`s in `body`, at any depth.
pub(crate) fn return_count(body: &Block) -> usize {
    let mut returns = 0;
    cpl_ir::walk::deep_stmts(body, &mut |s| {
        returns += matches!(s, Stmt::Return(_)) as usize
    });
    returns
}

/// Whether a closure in `body` captures one of its locals.
pub(crate) fn captures_a_local(body: &Block, closures: &[ClosureDef]) -> bool {
    let mut captures = false;
    cpl_ir::walk::deep_exprs(body, &mut |e| {
        captures |= matches!(e.kind, ExprKind::Closure(c)
            if closures[c].captures.iter().any(|cap| !cap.is_global));
    });
    captures
}

/// How often `body` reads each of its first `params` locals.
pub(crate) fn param_reads(body: &Block, params: usize) -> Vec<usize> {
    let mut uses = vec![0usize; params];
    cpl_ir::walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            if s < params {
                uses[s] += 1;
            }
        }
    });
    uses
}

/// How often `body` reads local `slot`.
pub(crate) fn local_reads(body: &Block, slot: Slot) -> usize {
    let mut reads = 0;
    cpl_ir::walk::deep_exprs(body, &mut |e| {
        reads += matches!(e.kind, ExprKind::Local(s) if s == slot) as usize
    });
    reads
}

/// Whether running `b` may write global `g`: a call it still makes may.
pub(crate) fn may_write_global(b: &Block, g: cpl_ir::GlobalId) -> bool {
    let at = |p: &Place| cpl_ir::walk::root(p) == (true, g);
    let mut writes = false;
    cpl_ir::walk::deep_exprs(b, &mut |e| {
        writes |= match &e.kind {
            ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, _, _) => at(p),
            ExprKind::Call(..) | ExprKind::CallExpr(..) => true,
            _ => false,
        };
    });
    cpl_ir::walk::deep_stmts(b, &mut |s| {
        writes |= matches!(s, Stmt::Assign(p, _, _) if at(p))
    });
    writes
}

/// Whether working out `v` may write one of the variables `slots`.
pub(crate) fn writes_any(v: &Expr, slots: &[Slot]) -> bool {
    let at = |p: &Place| matches!(cpl_ir::walk::root(p), (false, s) if slots.contains(&s));
    let mut writes = false;
    cpl_ir::walk::deep_expr(v, &mut |e| {
        writes |= match &e.kind {
            ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, _, _) => at(p),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                args.iter().any(|a| matches!(a, Arg::Mut(p) if at(p)))
            }
            _ => false,
        };
    });
    if let ExprKind::Do(b) = &v.kind {
        cpl_ir::walk::deep_stmts(b, &mut |s| {
            writes |= matches!(s, Stmt::Assign(p, _, _) if at(p))
        });
    }
    writes
}
