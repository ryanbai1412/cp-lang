//! `?T` parameters whose `T` holds a heap buffer (a container, or a
//! struct, tuple or enum holding one) that the fn only reads: each is a
//! `cpl::Lent<T>`, a pointer to the caller's value, so `f(xs)` for
//! `fn f(a: ?List<Int> = None)` passes `xs` without copying it into a
//! `std::optional`. A local the fn binds to one, as `if let` does, is one
//! too.

use crate::uses::places::{each_call, each_closure, each_opened, each_put, each_write, Put, Write};
use crate::uses::read_counts;
use crate::walk;
use cpl_ir::{Arg, ExprKind, Program, Ty};
use std::collections::{BTreeMap, BTreeSet};

fn opt_heap(program: &Program, ty: &Ty) -> bool {
    matches!(ty, Ty::Opt(inner) if !matches!(**inner, Ty::Opt(_))
        && crate::lend::holds_heap(program, inner))
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
                    opt_heap(program, &f.locals[s].ty)
                        && (s >= f.params.len() || !f.params[s].mutable)
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
    for (s, &n) in read_counts(&f.body, f.locals.len()).iter().enumerate() {
        if n > 0 {
            uses.insert(s, n);
        }
    }
    each_opened(&f.body, &mut |x| {
        if let Some(s) = local(x) {
            *ok.entry(s).or_default() += 1;
        }
    });
    each_call(&f.body, &mut |g, args| {
        for (j, arg) in args.iter().enumerate() {
            if let Arg::Value(v) = arg {
                if let Some(s) = local(v) {
                    if j < program.fns[g].params.len() && lent[g].contains(&j) {
                        *ok.entry(s).or_default() += 1;
                    }
                }
            }
        }
    });
    each_write(&f.body, &mut |how, place| {
        if let Write::Store | Write::Mut = how {
            bad.insert(walk::root(place).1);
        }
    });
    each_closure(&f.body, &mut |cid| {
        let caps = program.closures[cid].captures.iter();
        bad.extend(caps.filter(|c| !c.is_global).map(|c| c.outer));
    });
    let mut lets = BTreeMap::<usize, usize>::new();
    let mut aliases = Vec::new();
    each_put(&f.body, &mut |put, init| {
        if let Put::Let(t) = put {
            *lets.entry(t).or_default() += 1;
            match local(init) {
                Some(s) if mine.contains(&t) => {
                    *ok.entry(s).or_default() += 1;
                    aliases.push((t, s));
                }
                Some(s) => {
                    bad.insert(s);
                }
                None => {
                    bad.insert(t);
                }
            }
        }
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
