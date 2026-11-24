//! A call to a fn whose whole body hands each of its params, once and in
//! order, to one builtin (`fn clone<T>(self: T) -> T = copy(self)`) runs as
//! that builtin on the arguments, at the calling line: the arguments are
//! evaluated once and in order either way, and both engines see the builtin
//! itself, so a copy of a dead argument still moves and is listed where it's
//! written. The first case of the one IR inliner.

use crate::walk::{children_mut, stmt_blocks_mut, stmt_exprs_mut};
use crate::{Arg, Block, Builtin, ClosureDef, Expr, ExprKind, Function, Stmt, Ty};

type Forwards = [Option<(Builtin, Ty)>];

pub(super) fn inline_forwarders(
    fns: &mut [Option<Function>],
    blocks: [&mut Block; 2],
    closures: &mut [ClosureDef],
) {
    let ops: Vec<_> = fns.iter().map(|f| f.as_ref().and_then(forwards)).collect();
    if ops.iter().all(Option::is_none) {
        return;
    }
    for f in fns.iter_mut().flatten() {
        block(&mut f.body, &ops);
    }
    for b in blocks {
        block(b, &ops);
    }
    for c in closures {
        block(&mut c.body, &ops);
    }
}

/// The builtin `f`'s body is, when it hands it each param in order.
fn forwards(f: &Function) -> Option<(Builtin, Ty)> {
    let [Stmt::Return(Some(body))] = f.body.stmts.as_slice() else {
        return None;
    };
    let ExprKind::Builtin(op, args) = &body.kind else {
        return None;
    };
    if args.len() != f.params.len() || body.ty != f.ret {
        return None;
    }
    for (slot, (arg, p)) in args.iter().zip(&f.params).enumerate() {
        if p.mutable || p.range.is_some() || p.fills_cache || arg.kind != ExprKind::Local(slot) {
            return None;
        }
    }
    Some((*op, body.ty.clone()))
}

fn block(b: &mut Block, ops: &Forwards) {
    for stmt in &mut b.stmts {
        stmt_exprs_mut(stmt, &mut |e| expr(e, ops));
        stmt_blocks_mut(stmt, &mut |inner| block(inner, ops));
    }
}

fn expr(e: &mut Expr, ops: &Forwards) {
    if let ExprKind::Do(inner) = &mut e.kind {
        block(inner, ops);
    }
    children_mut(e, &mut |child| expr(child, ops));
    let ExprKind::Call(f, args) = &mut e.kind else {
        return;
    };
    let Some(Some((op, ty))) = ops.get(*f) else {
        return;
    };
    if *ty != e.ty || args.iter().any(|a| matches!(a, Arg::Mut(_))) {
        return;
    }
    let values = std::mem::take(args).into_iter().filter_map(|a| match a {
        Arg::Value(v) => Some(v),
        Arg::Mut(_) => None,
    });
    e.kind = ExprKind::Builtin(*op, values.collect());
}
