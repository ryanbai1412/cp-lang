//! `Str` parameters and locals that only ever hold a constant message for
//! an `assert`: `fn flip_at(.., wrong_way: Str)`, passed a literal. Each is
//! a `const char*`, so a call builds no `std::string`; only a failing
//! assert makes one, to report it.

use super::{Home, Uses};
use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

/// A constant message: a literal (with no NUL, which a `const char*` would
/// end at), a message slot of `mine`, or a choice of two on a plain value.
pub(crate) fn constant(e: &Expr, mine: &BTreeSet<usize>) -> bool {
    match &e.kind {
        ExprKind::Str(bytes) => !bytes.contains(&0),
        ExprKind::Local(s) => mine.contains(s),
        ExprKind::If(c, a, b) => walk::is_simple(c) && constant(a, mine) && constant(b, mine),
        _ => false,
    }
}

/// The message slots of `e`, a [`constant`].
fn leaves(e: &Expr, out: &mut Vec<usize>) {
    match &e.kind {
        ExprKind::Local(s) => out.push(*s),
        ExprKind::If(_, a, b) => {
            leaves(a, out);
            leaves(b, out);
        }
        _ => {}
    }
}

/// Per fn, its message slots: a parameter some call passes a constant
/// message and every call does, or a local set once to one, that is read
/// only as the message of an `assert` or passed on to such a parameter.
/// One a build without checks never reads is one too.
pub(crate) fn message_slots(program: &Program, uses: &Uses) -> Vec<BTreeSet<usize>> {
    let fixed: BTreeSet<usize> = (program.judge_fns.iter())
        .chain(&program.entry_fns)
        .copied()
        .chain(program.cpp_bodies.iter().map(|(f, _)| *f))
        .collect();
    let mut slots: Vec<BTreeSet<usize>> = (program.fns.iter().enumerate())
        .map(|(id, f)| {
            if fixed.contains(&id) || program.is_hook(id) {
                return BTreeSet::new();
            }
            let hoisted = crate::scope::hoisted(&f.body, f.params.len());
            (0..f.locals.len())
                .filter(|&s| {
                    f.locals[s].ty == Ty::Str
                        && !uses.of_binding(Home::Fn(id), s).mutated
                        && (s < f.params.len() || !hoisted.contains(&s))
                })
                .collect()
        })
        .collect();
    let empty = BTreeSet::new();
    loop {
        let n = program.fns.len();
        let (mut passed, mut refused) = (vec![BTreeSet::new(); n], vec![BTreeSet::new(); n]);
        let mut given = |body: &Block, mine: &BTreeSet<usize>| {
            args_given(body, mine, &slots, &mut passed, &mut refused)
        };
        for (id, f) in program.fns.iter().enumerate() {
            given(&f.body, &slots[id]);
        }
        for c in &program.closures {
            given(&c.body, &empty);
        }
        given(&program.main, &empty);
        given(&program.setup, &empty);
        let mut changed = false;
        for (id, f) in program.fns.iter().enumerate() {
            let mut bad = misused(program, &f.body, f.params.len(), &slots[id], &slots);
            bad.extend(&refused[id]);
            bad.extend((0..f.params.len()).filter(|j| !passed[id].contains(j)));
            let before = slots[id].len();
            slots[id].retain(|s| !bad.contains(s));
            changed |= slots[id].len() != before;
        }
        if !changed {
            return slots;
        }
    }
}

/// Notes, per fn, the message parameters a call in `body` passes a
/// constant message (`passed`) or something else (`refused`).
fn args_given(
    body: &Block,
    mine: &BTreeSet<usize>,
    slots: &[BTreeSet<usize>],
    passed: &mut [BTreeSet<usize>],
    refused: &mut [BTreeSet<usize>],
) {
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Call(g, args) = &e.kind {
            for (j, arg) in args.iter().enumerate() {
                if !slots[*g].contains(&j) {
                    continue;
                }
                if matches!(arg, Arg::Value(v) if constant(v, mine)) {
                    passed[*g].insert(j);
                } else {
                    refused[*g].insert(j);
                }
            }
        }
    });
}

/// The slots of `mine` that some use in `body` keeps from being messages.
fn misused(
    program: &Program,
    body: &Block,
    params: usize,
    mine: &BTreeSet<usize>,
    slots: &[BTreeSet<usize>],
) -> BTreeSet<usize> {
    let (mut uses, mut ok) = (
        BTreeMap::<usize, usize>::new(),
        BTreeMap::<usize, usize>::new(),
    );
    let mut bad = BTreeSet::new();
    let fine = |e: &Expr, ok: &mut BTreeMap<usize, usize>| {
        let mut found = Vec::new();
        leaves(e, &mut found);
        for s in found {
            *ok.entry(s).or_default() += 1;
        }
    };
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Local(s) => *uses.entry(*s).or_default() += 1,
        ExprKind::Call(g, args) => {
            for (j, arg) in args.iter().enumerate() {
                match arg {
                    Arg::Value(v) if slots[*g].contains(&j) && constant(v, mine) => {
                        fine(v, &mut ok)
                    }
                    _ => {}
                }
            }
        }
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
    let mut asserted = BTreeSet::new();
    walk::deep_stmts(body, &mut |stmt| match stmt {
        Stmt::Let(t, init) => {
            *lets.entry(*t).or_default() += 1;
            match init {
                Some(init) if mine.contains(t) && constant(init, mine) => fine(init, &mut ok),
                _ => _ = bad.insert(*t),
            }
        }
        Stmt::Assert {
            message: Some(message),
            ..
        } => {
            if let ExprKind::Local(s) = message.kind {
                *ok.entry(s).or_default() += 1;
                asserted.insert(s);
            }
        }
        _ => {}
    });
    for &s in mine {
        let set_once = s < params || lets.get(&s) == Some(&1);
        let read = uses.get(&s).copied().unwrap_or(0);
        let as_message = asserted.contains(&s) || forwarded(body, s, slots);
        if !set_once || read != ok.get(&s).copied().unwrap_or(0) || (read > 0 && !as_message) {
            bad.insert(s);
        }
    }
    bad
}

/// True when a call in `body` passes slot `s` on to a message parameter.
fn forwarded(body: &Block, s: usize, slots: &[BTreeSet<usize>]) -> bool {
    let mut found = false;
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Call(g, args) = &e.kind {
            found |= args.iter().enumerate().any(|(j, arg)| {
                slots[*g].contains(&j)
                    && matches!(arg, Arg::Value(v) if {
                        let mut l = Vec::new();
                        leaves(v, &mut l);
                        l.contains(&s)
                    })
            });
        }
    });
    found
}
