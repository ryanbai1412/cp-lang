//! A view walked straight from the call that makes it, `for w in
//! xs.windows(3)`: when the call only stores its params in a struct and the
//! struct's walk reads only those fields, the walk takes the params instead,
//! so a view that is only walked leaves no struct and no constructor.

use super::{all_returns, Cx, Remap, Sub};
use crate::walk::deep_exprs;
use crate::{Arg, Block, ClosureDef, Expr, ExprKind, Function, LoopId, Stmt};
use std::collections::HashMap;

/// Rewrites each library walk every call of which is `walk(make(args))`,
/// outside a closure, to take `args`; gives each such walk's `make`, or
/// `usize::MAX`.
pub(super) fn fold(
    fns: &mut [Option<Function>],
    hosts: [&mut Block; 2],
    closures: &mut Vec<ClosureDef>,
    library: &[bool],
    next_loop: &mut LoopId,
) -> Vec<usize> {
    let n = fns.len();
    // Per fn: its calls, and the fn every call of it walks the result of.
    let (mut calls, mut made) = (vec![0usize; n], vec![None; n]);
    let mut note = |b: &Block, closure: bool| {
        deep_exprs(b, &mut |e| {
            if let ExprKind::Call(f, args) = &e.kind {
                let c = match args.as_slice() {
                    [Arg::Value(Expr {
                        kind: ExprKind::Call(c, _),
                        ..
                    })] => *c,
                    _ => usize::MAX,
                };
                if let (Some(k), Some(m)) = (calls.get_mut(*f), made.get_mut(*f)) {
                    *m = if !closure && (*k == 0 || *m == Some(c)) {
                        Some(c)
                    } else {
                        None
                    };
                    *k += 1;
                }
            }
        })
    };
    fns.iter().flatten().for_each(|f| note(&f.body, false));
    closures.iter().for_each(|c| note(&c.body, true));
    hosts.iter().for_each(|b| note(b, false));
    let lib = |f: usize| library.get(f).copied().unwrap_or(false);
    let mut walks = vec![usize::MAX; n];
    for f in 0..n {
        let Some(c) = made[f].filter(|&c| c < n && c != f && lib(f) && lib(c)) else {
            continue;
        };
        let (Some(walk), Some(make)) = (&fns[f], &fns[c]) else {
            continue;
        };
        if let Some(mut new) = fold_fn(walk, make, closures, next_loop) {
            // With no other call left, `make` is gone and the walk is what it was.
            let lone = calls[c] == calls[f];
            if lone {
                std::mem::swap(
                    &mut new.name,
                    fns[c].as_mut().map_or(&mut String::new(), |m| &mut m.name),
                );
            }
            fns[f] = Some(new);
            walks[f] = c;
        }
    }
    walks
}

/// `walk` taking `make`'s params in place of the struct `make` builds of them.
fn fold_fn(
    walk: &Function,
    make: &Function,
    closures: &mut Vec<ClosureDef>,
    next_loop: &mut LoopId,
) -> Option<Function> {
    let m = make.params.len();
    let (Stmt::Return(Some(made)), pre) = make.body.stmts.split_last()? else {
        return None;
    };
    // Each `self.f` is then a field of `made`, which the inliner folds.
    let simple =
        matches!(&made.kind, ExprKind::Tuple(items) if items.iter().all(crate::walk::is_simple));
    let one_param = matches!(walk.params.as_slice(), [p] if !p.mutable && !p.fills_cache);
    if !simple
        || !one_param
        || make.locals.len() != m
        || !pre.iter().all(|s| matches!(s, Stmt::Assert { .. }))
        || all_returns(&walk.body) != 1
        || !matches!(walk.body.stmts.last(), Some(Stmt::Return(Some(_))))
    {
        return None;
    }
    let mut body = walk.body.clone();
    let mut map = vec![Sub::Expr(made.clone())];
    for s in m..walk.locals.len() + m - 1 {
        map.push(Sub::Slot(s));
    }
    let mut remap = Remap {
        map: &map,
        loops: HashMap::new(),
        next_loop,
        closures,
        span: None,
        ok: true,
    };
    remap.block(&mut body);
    let Some(Stmt::Give(v)) = body.stmts.pop().filter(|_| remap.ok) else {
        return None;
    };
    let mut stmts = pre.to_vec();
    stmts.append(&mut body.stmts);
    stmts.push(Stmt::Return(Some(v)));
    body.stmts = stmts;
    let mut locals = make.locals.clone();
    locals.extend_from_slice(&walk.locals[1..]);
    Some(Function {
        params: make.params.clone(),
        locals,
        body,
        ..walk.clone()
    })
}

impl Cx<'_, '_> {
    /// `walk(make(args))` as `walk(args)`, at `make`'s call.
    pub(super) fn retarget(&mut self, e: &mut Expr) {
        let ExprKind::Call(f, args) = &mut e.kind else {
            return;
        };
        let [Arg::Value(Expr {
            kind: ExprKind::Call(c, inner),
            span,
            ..
        })] = args.as_mut_slice()
        else {
            return;
        };
        if self.inl.walks.get(*f) != Some(c) {
            return;
        }
        let (inner, span) = (std::mem::take(inner), *span);
        if self.user {
            let name = self.inl.shown.get(*f).map_or("", String::as_str);
            let why = format!("{name}: walks the view's fields, with no struct");
            self.inl.inlined.push((span, why));
        }
        *args = inner;
        e.span = span;
    }
}
