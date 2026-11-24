//! Fn views: a `fn` or `?fn` parameter its fn only tests and calls
//! ([`crate::lend::fn_param_only_called`],
//! [`crate::lend::opt_fn_param_only_called`]) is a `cpl::FnRef`, a pointer
//! to the caller's callable and a trampoline (null is `None`), not a
//! `std::function` that owns a copy of it. So is each name the body binds
//! from one: the `if let` test, what it unwraps, `?? default`. A name
//! written while it lives is not one: it may be set to a closure that dies
//! before it.

use super::{Home, Uses};
use crate::devirt::Devirt;
use crate::walk;
use cpl_ir::{BinOp, Expr, ExprKind, Program, Stmt, Ty};
use std::collections::BTreeSet;

/// Per fn, the slots held as a `cpl::FnRef`.
pub(crate) fn fn_refs(program: &Program, devirt: &Devirt, uses: &Uses) -> Vec<BTreeSet<usize>> {
    let fixed: BTreeSet<usize> = program
        .judge_fns
        .iter()
        .chain(&program.entry_fns)
        .copied()
        .collect();
    let mut out = vec![BTreeSet::new(); program.fns.len()];
    for (id, f) in program.fns.iter().enumerate() {
        if fixed.contains(&id) || program.is_hook(id) {
            continue;
        }
        let held = &mut out[id];
        for slot in 0..f.params.len() {
            let free = devirt.template_param_index(id, slot).is_none()
                && !devirt.param_elidable(id, slot)
                && !devirt.param_known(id, slot);
            let viewed = match &f.locals[slot].ty {
                Ty::Fn(..) => crate::lend::fn_param_only_called(program, id, slot),
                Ty::Opt(inner) if matches!(**inner, Ty::Fn(..)) => {
                    crate::lend::opt_fn_param_only_called(program, id, slot)
                }
                _ => false,
            };
            if free && viewed && !uses.of_binding(Home::Fn(id), slot).mutated {
                held.insert(slot);
            }
        }
        loop {
            let before = held.len();
            walk::deep_stmts(&f.body, &mut |stmt| {
                if let Stmt::Let(s, Some(init)) = stmt {
                    let written = uses.of_binding(Home::Fn(id), *s).mutated;
                    if !written && bound_from(program, init, held) {
                        held.insert(*s);
                    }
                }
            });
            if held.len() == before {
                break;
            }
        }
    }
    out
}

/// Whether `init` names a view in `held`, unwraps one, or defaults one to a
/// closure that captures nothing (held as its fn pointer).
fn bound_from(program: &Program, init: &Expr, held: &BTreeSet<usize>) -> bool {
    let view = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if held.contains(&s));
    match &init.kind {
        ExprKind::Local(_) => view(init),
        ExprKind::Unwrap(inner, _) => view(inner),
        ExprKind::Binary(BinOp::Coalesce, opt, default) => {
            view(opt) && crate::lend::plain_closure(program, default)
        }
        _ => false,
    }
}
