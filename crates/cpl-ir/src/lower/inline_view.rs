//! A view walked straight from the call that makes it, `for w in
//! xs.windows(3)`: when the call only stores its params in a struct and the
//! struct's walk reads only those fields, the walk takes the params instead,
//! so a view that is only walked leaves no struct and no constructor.

use super::{all_returns, Remap, Sub};
use crate::walk::{children_mut, deep_exprs, stmt_blocks_mut, stmt_exprs_mut};
use crate::{Arg, Block, ClosureDef, Expr, ExprKind, Function, LoopId, Span, Stmt, Ty};
use std::collections::HashMap;

/// Rewrites each library walk every call of which is `walk(make(args))` to
/// take `args`; gives each such call in the user's code.
pub(super) fn fold(
    fns: &mut [Option<Function>],
    mut hosts: [&mut Block; 2],
    closures: &mut Vec<ClosureDef>,
    library: &[bool],
    shown: &[String],
    next_loop: &mut LoopId,
) -> Vec<(Span, String)> {
    let n = fns.len();
    // Per fn: its calls, and the fn every call of it walks the result of.
    let (mut calls, mut made) = (vec![0usize; n], vec![None; n]);
    let mut note = |b: &Block| {
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
                    *m = if *k == 0 || *m == Some(c) {
                        Some(c)
                    } else {
                        None
                    };
                    *k += 1;
                }
            }
        })
    };
    fns.iter().flatten().for_each(|f| note(&f.body));
    closures.iter().for_each(|c| note(&c.body));
    hosts.iter().for_each(|b| note(b));
    let lib = |f: usize| library.get(f).copied().unwrap_or(false);
    let mut walks = vec![false; n];
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
            walks[f] = true;
        }
    }
    let mut out = Vec::new();
    for (i, f) in fns.iter_mut().enumerate() {
        if let Some(f) = f {
            retarget(&mut f.body, &walks, !lib(i), shown, &mut out);
        }
    }
    for c in closures.iter_mut() {
        retarget(&mut c.body, &walks, false, shown, &mut out);
    }
    for b in hosts.iter_mut() {
        retarget(b, &walks, true, shown, &mut out);
    }
    out
}

/// `walk` taking `make`'s params in place of the struct `make` builds of them.
fn fold_fn(
    walk: &Function,
    make: &Function,
    closures: &mut Vec<ClosureDef>,
    next_loop: &mut LoopId,
) -> Option<Function> {
    let m = make.params.len();
    let (last, pre) = make.body.stmts.split_last()?;
    let Stmt::Return(Some(Expr {
        kind: ExprKind::Tuple(items),
        ..
    })) = last
    else {
        return None;
    };
    let mut to = Vec::new();
    for e in items {
        match e.kind {
            ExprKind::Local(j) if j < m && make.locals[j].ty == e.ty => to.push(j),
            _ => return None,
        }
    }
    let one_param = matches!(walk.params.as_slice(), [p] if !p.mutable && !p.fills_cache);
    if !one_param
        || make.locals.len() != m
        || !pre.iter().all(|s| matches!(s, Stmt::Assert { .. }))
        || all_returns(&walk.body) != 1
        || !matches!(walk.body.stmts.last(), Some(Stmt::Return(Some(_))))
    {
        return None;
    }
    let fresh = walk.locals.len();
    let mut body = walk.body.clone();
    exprs_mut(&mut body, &mut |e| {
        if let ExprKind::Field(base, i) = &e.kind {
            if let (ExprKind::Local(0), Some(&j)) = (&base.kind, to.get(*i)) {
                if make.locals[j].ty == e.ty {
                    e.kind = ExprKind::Local(fresh + j);
                }
            }
        }
    });
    let mut whole = false;
    deep_exprs(&body, &mut |e| {
        whole |= matches!(e.kind, ExprKind::Local(0) | ExprKind::Closure(_))
    });
    if whole {
        return None;
    }
    let unit = Expr {
        kind: ExprKind::Unit,
        ty: Ty::Unit,
        span: Span::default(),
    };
    let mut map = vec![Sub::Expr(unit)];
    for s in (m..fresh + m - 1).chain(0..m) {
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

/// `walk(make(args))` as `walk(args)`, at `make`'s call.
fn retarget(
    b: &mut Block,
    walks: &[bool],
    user: bool,
    shown: &[String],
    out: &mut Vec<(Span, String)>,
) {
    exprs_mut(b, &mut |e| {
        let ExprKind::Call(f, args) = &mut e.kind else {
            return;
        };
        let [Arg::Value(Expr {
            kind: ExprKind::Call(_, inner),
            span,
            ..
        })] = args.as_mut_slice()
        else {
            return;
        };
        if !walks.get(*f).copied().unwrap_or(false) {
            return;
        }
        let (inner, span) = (std::mem::take(inner), *span);
        if user {
            let name = shown.get(*f).map_or("", String::as_str);
            out.push((
                span,
                format!("{name}: walks the view's fields, with no struct"),
            ));
        }
        *args = inner;
        e.span = span;
    });
}

fn exprs_mut(b: &mut Block, f: &mut dyn FnMut(&mut Expr)) {
    for s in &mut b.stmts {
        stmt_exprs_mut(s, &mut |e| expr_mut(e, f));
        stmt_blocks_mut(s, &mut |inner| exprs_mut(inner, f));
    }
}

fn expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    if let ExprKind::Do(b) = &mut e.kind {
        exprs_mut(b, f);
    }
    children_mut(e, &mut |c| expr_mut(c, f));
    f(e);
}
