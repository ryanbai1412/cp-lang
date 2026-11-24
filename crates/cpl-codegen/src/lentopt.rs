//! `?T` parameters holding a container that the fn only reads: each is a
//! `cpl::Lent<T>`, a pointer to the caller's value, so `f(xs)` for
//! `fn f(a: ?List<Int> = None)` passes `xs` without copying it into a
//! `std::optional`. A local the fn binds to one, as `if let` does, is one
//! too.

use crate::walk;
use cpl_ir::{Arg, ExprKind, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

fn opt_container(ty: &Ty) -> bool {
    matches!(ty, Ty::Opt(inner) if matches!(
        **inner,
        Ty::List(_) | Ty::Array { .. } | Ty::Str | Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_) | Ty::HashSet(_)
    ))
}

/// Per fn, the slots taken or held as a `cpl::Lent`: a parameter or a
/// local only tested (`IsSome`), read through (`Unwrap`), bound whole to
/// another such local or passed whole to such a parameter of a fn.
pub(crate) fn lent_opts(program: &Program) -> Vec<BTreeSet<usize>> {
    let mut lent: Vec<BTreeSet<usize>> = program
        .fns
        .iter()
        .map(|f| {
            (0..f.locals.len())
                .filter(|&s| {
                    opt_container(&f.locals[s].ty) && (s >= f.params.len() || !f.params[s].mutable)
                })
                .collect()
        })
        .collect();
    loop {
        let mut changed = false;
        for id in 0..program.fns.len() {
            let bad = misused(program, id, &lent);
            if !bad.is_empty() {
                changed = true;
                lent[id].retain(|s| !bad.contains(s));
            }
        }
        if !changed {
            return lent;
        }
    }
}

/// The slots of fn `id` in `lent` that some use of keeps from being one.
fn misused(program: &Program, id: usize, lent: &[BTreeSet<usize>]) -> BTreeSet<usize> {
    let f = &program.fns[id];
    let mine = &lent[id];
    let (mut uses, mut ok) = (
        BTreeMap::<usize, usize>::new(),
        BTreeMap::<usize, usize>::new(),
    );
    let mut bad = BTreeSet::new();
    let local = |e: &cpl_ir::Expr| match e.kind {
        ExprKind::Local(s) if mine.contains(&s) => Some(s),
        _ => None,
    };
    let mut_root = |arg: &Arg, bad: &mut BTreeSet<usize>| {
        if let Arg::Mut(place) = arg {
            bad.insert(walk::root(place).1);
        }
    };
    walk::deep_exprs(&f.body, &mut |e| match &e.kind {
        ExprKind::Local(s) => *uses.entry(*s).or_default() += 1,
        ExprKind::IsSome(x) | ExprKind::Unwrap(x, false) => {
            if let Some(s) = local(x) {
                *ok.entry(s).or_default() += 1;
            }
        }
        ExprKind::Call(g, args) => {
            for (j, arg) in args.iter().enumerate() {
                mut_root(arg, &mut bad);
                if let Arg::Value(v) = arg {
                    if let Some(s) = local(v) {
                        if j < program.fns[*g].params.len() && lent[*g].contains(&j) {
                            *ok.entry(s).or_default() += 1;
                        }
                    }
                }
            }
        }
        ExprKind::CallExpr(_, args) => args.iter().for_each(|arg| mut_root(arg, &mut bad)),
        ExprKind::Closure(cid) => bad.extend(
            program.closures[*cid]
                .captures
                .iter()
                .filter(|c| !c.is_global)
                .map(|c| c.outer),
        ),
        _ => {}
    });
    let mut lets = BTreeMap::<usize, usize>::new();
    let mut aliases = Vec::new();
    walk::deep_stmts(&f.body, &mut |stmt| match stmt {
        Stmt::Let(t, init) => {
            *lets.entry(*t).or_default() += 1;
            match init.as_ref().and_then(local) {
                Some(s) if mine.contains(t) => {
                    *ok.entry(s).or_default() += 1;
                    aliases.push((*t, s));
                }
                Some(s) => {
                    bad.insert(s);
                }
                None => {
                    bad.insert(*t);
                }
            }
        }
        Stmt::Assign(place, ..) => {
            bad.insert(walk::root(place).1);
        }
        _ => {}
    });
    for &s in mine {
        let let_once = s < f.params.len() || lets.get(&s) == Some(&1);
        if !let_once || uses.get(&s).copied().unwrap_or(0) != ok.get(&s).copied().unwrap_or(0) {
            bad.insert(s);
        }
    }
    for (t, s) in aliases {
        if bad.contains(&t) || bad.contains(&s) {
            bad.insert(t);
            bad.insert(s);
        }
    }
    bad.retain(|s| mine.contains(s));
    bad
}
