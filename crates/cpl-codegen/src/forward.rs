//! A call to a fn that only returns one of its params, `op.lift(x)` with
//! `fn lift(self, x: T) -> T = x`, is that argument. Taken when every other
//! argument is pure, so dropping it changes nothing; the fn then goes when
//! nothing else calls it. It majorizes: the same value, minus a call.

use crate::optfn::exprs_mut;
use crate::uses::effects::trap_free_with;
use crate::uses::returned_param;
use cpl_ir::{Arg, Expr, ExprKind, Program};

pub(crate) fn forward_calls(program: &mut Program, traps: bool) {
    let returns: Vec<Option<usize>> = program.fns.iter().map(returned_param).collect();
    if returns.iter().all(Option::is_none) {
        return;
    }
    let mut rewrite = |e: &mut Expr| {
        let ExprKind::Call(f, args) = &mut e.kind else {
            return;
        };
        let Some(k) = returns[*f] else {
            return;
        };
        let pure = |a: &Arg| matches!(a, Arg::Value(v) if trap_free_with(v, traps, &|_| false));
        let others_pure = args.iter().enumerate().all(|(i, a)| i == k || pure(a));
        if let (true, Some(Arg::Value(v))) = (others_pure, args.get(k)) {
            if v.ty == e.ty {
                if let Arg::Value(v) = args.swap_remove(k) {
                    *e = v;
                }
            }
        }
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
    for c in closures.iter_mut() {
        exprs_mut(&mut c.body, &mut rewrite);
    }
}
