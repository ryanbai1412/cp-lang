//! A call to a fn whose whole body returns one small expression of its
//! params is that expression with the arguments in place:
//! `op.lift(x)` with `fn lift(self, x: T) -> T = x` is `x`, and `o.is_some`
//! with `fn is_some(self: ?T) -> Bool = self != None` is `o != None`.
//! Taken when every argument is pure, so dropping or moving it changes
//! nothing, or when the body is the one param a lone impure argument fills,
//! which then runs once where the call did. The fn goes when nothing else
//! calls it. It majorizes: the same value, minus a call.

use crate::optfn::exprs_mut;
use crate::uses::effects::trap_free_with;
use crate::uses::returned_expr;
use crate::walk;
use cpl_ir::{Arg, Expr, ExprKind, Program, Ty};

pub(crate) fn forward_calls(program: &mut Program, traps: bool) {
    let bodies: Vec<Option<(Expr, Vec<Ty>)>> = (program.fns.iter())
        .map(|f| {
            let e = returned_expr(f, traps)?;
            Some((
                e.clone(),
                f.locals[..f.params.len()]
                    .iter()
                    .map(|l| l.ty.clone())
                    .collect(),
            ))
        })
        .collect();
    if bodies.iter().all(Option::is_none) {
        return;
    }
    let forwarders: Vec<bool> = (0..program.closures.len())
        .map(|c| crate::devirt::forwards_to(program, c).is_some())
        .collect();
    let mut rewrite = |e: &mut Expr| {
        let ExprKind::Call(f, args) = &mut e.kind else {
            return;
        };
        let Some((body, params)) = &bodies[*f] else {
            return;
        };
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
    };
    let Program {
        setup,
        main,
        fns,
        closures,
        ..
    } = program;
    for block in [setup, main] {
        exprs_mut(block, &mut rewrite);
    }
    for f in fns.iter_mut() {
        exprs_mut(&mut f.body, &mut rewrite);
    }
    // A closure that only calls a fn is that fn (`devirt::same_closures`).
    for (c, _) in closures.iter_mut().zip(&forwarders).filter(|(_, &f)| !f) {
        exprs_mut(&mut c.body, &mut rewrite);
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
