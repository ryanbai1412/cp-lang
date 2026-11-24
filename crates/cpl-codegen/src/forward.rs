//! A call to a fn whose whole body returns one small expression of its
//! params is that expression with the arguments in place:
//! `op.lift(x)` with `fn lift(self, x: T) -> T = x` is `x`, and `o.is_some`
//! with `fn is_some(self: ?T) -> Bool = self != None` is `o != None`.
//! Taken when every argument is pure, so dropping or moving it changes
//! nothing, or when the body is the one param a lone impure argument fills,
//! which then runs once where the call did. A fn that only forwards its
//! params, in order, to one other call is that call:
//! `fn set(mut self, i: Int, x: T) = self.put(i, x, true)`. The fn goes when
//! nothing else calls it. It majorizes: the same value, minus a call.

use crate::optfn::exprs_mut;
use crate::uses::effects::trap_free_with;
use crate::uses::{forwarded_call, returned_expr, MAX_RETURNED_NODES};
use crate::walk;
use cpl_ir::{Arg, Expr, ExprKind, Place, PlaceKind, Program, Ty};

pub(crate) fn forward_calls(program: &mut Program, traps: bool) {
    let bodies: Vec<Option<Forward>> = (program.fns.iter().enumerate())
        .map(|(me, f)| {
            let params = || {
                f.locals[..f.params.len()]
                    .iter()
                    .map(|l| l.ty.clone())
                    .collect()
            };
            if let Some(e) = returned_expr(f, traps) {
                return Some(Forward::Expr(e.clone(), params()));
            }
            let (e, extra) = forwarded_call(f, me, traps)?;
            Some(Forward::Call(e.clone(), params(), extra))
        })
        .collect();
    if bodies.iter().all(Option::is_none) {
        return;
    }
    let forwarders: Vec<bool> = (0..program.closures.len())
        .map(|c| crate::devirt::forwards_to(program, c).is_some())
        .collect();
    let mut sites = vec![0usize; program.fns.len()];
    every_expr(program, &[], &mut |e| {
        if let ExprKind::Call(f, _) = e.kind {
            sites[f] += 1;
        }
    });
    // A forwarded call's other args are written at every site, so all its
    // sites together may add no more than the cap.
    let mut bodies = bodies;
    for (f, body) in bodies.iter_mut().enumerate() {
        if matches!(body, Some(Forward::Call(_, _, extra)) if sites[f] * *extra > MAX_RETURNED_NODES)
        {
            *body = None;
        }
    }
    if bodies.iter().all(Option::is_none) {
        return;
    }
    let mut rewrite = |e: &mut Expr| {
        let ExprKind::Call(f, args) = &mut e.kind else {
            return;
        };
        match &bodies[*f] {
            Some(Forward::Expr(body, params)) => {
                let forwards = |i: usize| matches!(body.kind, ExprKind::Local(k) if k == i);
                let fits = args.len() == params.len()
                    && args.iter().zip(params).enumerate().all(|(i, (a, ty))| {
                        matches!(a, Arg::Value(v) if v.ty == *ty
                            && (forwards(i) || trap_free_with(v, traps, &|_| false)))
                    });
                if !fits || body.ty != e.ty {
                    return;
                }
                let mut values: Vec<Option<Expr>> = (args.drain(..))
                    .map(|a| match a {
                        Arg::Value(v) => Some(v),
                        Arg::Mut(_) => None,
                    })
                    .collect();
                let mut out = body.clone();
                fill(&mut out, &mut values);
                *e = out;
            }
            Some(Forward::Call(call, params, _)) => {
                let ExprKind::Call(g, inner) = &call.kind else {
                    return;
                };
                let fits = args.len() == params.len()
                    && args.iter().zip(params).all(|(a, ty)| match a {
                        Arg::Value(v) => v.ty == *ty,
                        Arg::Mut(p) => p.ty == *ty,
                    })
                    && call.ty == e.ty;
                if !fits {
                    return;
                }
                let mut given = args.drain(..);
                let forwarded = (inner.iter())
                    .map(|a| match a {
                        Arg::Value(Expr {
                            kind: ExprKind::Local(_),
                            ..
                        })
                        | Arg::Mut(Place {
                            kind: PlaceKind::Local(_),
                            ..
                        }) => given.next().expect("one argument per param"),
                        other => other.clone(),
                    })
                    .collect();
                drop(given);
                e.kind = ExprKind::Call(*g, forwarded);
            }
            None => {}
        }
    };
    // A closure that only calls a fn is that fn (`devirt::same_closures`).
    every_expr(program, &forwarders, &mut rewrite);
}

/// Runs `visit` on every expression of `program`, children first, but for
/// the closures `skip` marks.
fn every_expr(program: &mut Program, skip: &[bool], visit: &mut dyn FnMut(&mut Expr)) {
    let Program {
        setup,
        main,
        fns,
        closures,
        ..
    } = program;
    for block in [setup, main] {
        exprs_mut(block, visit);
    }
    for f in fns.iter_mut() {
        exprs_mut(&mut f.body, visit);
    }
    for (_, c) in
        (closures.iter_mut().enumerate()).filter(|(i, _)| !skip.get(*i).is_some_and(|&s| s))
    {
        exprs_mut(&mut c.body, visit);
    }
}

/// `e` with each param read in it replaced by its argument.
fn fill(e: &mut Expr, values: &mut [Option<Expr>]) {
    if let ExprKind::Local(s) = e.kind {
        if let Some(v) = values.get_mut(s).and_then(Option::take) {
            *e = v;
        }
        return;
    }
    walk::children_mut(e, &mut |child| fill(child, values));
}

/// What a call to a fn that forwards is, with its arguments in place.
enum Forward {
    /// The one expression the fn returns, and its params' types.
    Expr(Expr, Vec<Ty>),
    /// The one call the fn makes, its params' types, and the node count of
    /// its other args.
    Call(Expr, Vec<Ty>, usize),
}
