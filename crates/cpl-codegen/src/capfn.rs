//! performance/R13. A closure that captures a fn parameter known to hold
//! one closure calls that closure directly: in
//! `fn opt(self: F) = Monoid(None, (a, b) => .. self(x, y) ..)` called
//! only with `lighter`, the inner closure calls `lighter` and captures
//! nothing, so the `Monoid.op` it fills needs no `std::function`. It
//! majorizes: the same closure runs on the same arguments, and a capture
//! of a name no one changes is the value that name holds.

use crate::devirt::Devirt;
use crate::walk;
use cpl_ir::{Arg, Block, ClosureId, Expr, ExprKind, FnId, PlaceKind, Program, Slot, Stmt};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn known_captures(program: &mut Program) {
    let mut plan: BTreeMap<(FnId, ClosureId), Vec<(Slot, ClosureId)>> = BTreeMap::new();
    {
        let devirt = Devirt::collect(program);
        for (id, f) in program.fns.iter().enumerate() {
            let changed = changed(program, id);
            walk::deep_exprs(&f.body, &mut |e| {
                let ExprKind::Closure(cid) = e.kind else {
                    return;
                };
                if plan.contains_key(&(id, cid)) {
                    return;
                }
                let def = &program.closures[cid];
                let known: Vec<(Slot, ClosureId)> = def
                    .captures
                    .iter()
                    .filter(|cap| {
                        !cap.is_global
                            && !cap.by_ref
                            && cap.outer < f.params.len()
                            && !f.params[cap.outer].mutable
                            && !changed.contains(&cap.outer)
                            && only_called(&def.body, cap.inner)
                            && !nested_capture(program, &def.body, cap.inner)
                    })
                    .filter_map(|cap| Some((cap.inner, devirt.param_target(id, cap.outer)?)))
                    .collect();
                if !known.is_empty() {
                    plan.insert((id, cid), known);
                }
            });
        }
    }
    let made = made_once(program);
    // A closure made at more than one place gets a copy per fn, since each
    // fn may know a different value for what it captures.
    let mut moved: BTreeMap<(FnId, ClosureId), ClosureId> = BTreeMap::new();
    for &(id, cid) in plan.keys() {
        if !made.contains(&cid) {
            moved.insert((id, cid), program.closures.len());
            program.closures.push(program.closures[cid].clone());
        }
    }
    for (&(id, cid), &copy) in &moved {
        crate::optfn::exprs_mut(&mut program.fns[id].body, &mut |e| {
            if e.kind == ExprKind::Closure(cid) {
                e.kind = ExprKind::Closure(copy);
            }
        });
    }
    for ((id, cid), known) in plan {
        let cid = moved.get(&(id, cid)).copied().unwrap_or(cid);
        let def = &mut program.closures[cid];
        for (slot, target) in known {
            crate::optfn::exprs_mut(&mut def.body, &mut |e| {
                if let ExprKind::CallExpr(callee, _) = &mut e.kind {
                    if callee.kind == ExprKind::Local(slot) {
                        callee.kind = ExprKind::Closure(target);
                    }
                }
            });
            def.captures.retain(|c| c.is_global || c.inner != slot);
        }
    }
}

/// The closures made at one place only: a fn cloned per call keeps its
/// closures, which then capture a different value in each clone.
fn made_once(program: &Program) -> BTreeSet<ClosureId> {
    let mut seen = vec![0usize; program.closures.len()];
    let mut blocks = vec![&program.setup, &program.main];
    blocks.extend(program.fns.iter().map(|f| &f.body));
    blocks.extend(program.closures.iter().map(|c| &c.body));
    for block in blocks {
        walk::deep_exprs(block, &mut |e| {
            if let ExprKind::Closure(cid) = e.kind {
                seen[cid] += 1;
            }
        });
    }
    (0..seen.len()).filter(|&c| seen[c] == 1).collect()
}

/// The params of fn `id` it stores to, passes `mut`, or lets a closure
/// capture by reference.
fn changed(program: &Program, id: FnId) -> BTreeSet<Slot> {
    let body = &program.fns[id].body;
    let mut out = BTreeSet::new();
    walk::deep_stmts(body, &mut |s| {
        if let Stmt::Assign(place, ..) = s {
            if let (false, slot) = walk::root(place) {
                out.insert(slot);
            }
        }
    });
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(place) = arg {
                    if let (false, slot) = walk::root(place) {
                        out.insert(slot);
                    }
                }
            }
        }
        ExprKind::Closure(cid) => {
            let caps = program.closures[*cid].captures.iter();
            out.extend(caps.filter(|c| !c.is_global && c.by_ref).map(|c| c.outer));
        }
        _ => {}
    });
    out
}

/// Every read of `slot` in `body` is the callee of a call.
fn only_called(body: &Block, slot: Slot) -> bool {
    let (mut reads, mut calls) = (0, 0);
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => reads += 1,
        ExprKind::CallExpr(callee, _) if callee.kind == ExprKind::Local(slot) => calls += 1,
        _ => {}
    });
    let mut written = false;
    walk::deep_stmts(body, &mut |s| {
        if let Stmt::Assign(place, ..) = s {
            written |= matches!(place.kind, PlaceKind::Local(s) if s == slot)
                || walk::root(place) == (false, slot);
        }
    });
    reads == calls && !written
}

/// A closure made in `body` captures `slot`.
fn nested_capture(program: &Program, body: &Block, slot: Slot) -> bool {
    let mut found = false;
    walk::deep_exprs(body, &mut |e: &Expr| {
        if let ExprKind::Closure(cid) = e.kind {
            found |= program.closures[cid]
                .captures
                .iter()
                .any(|c| !c.is_global && c.outer == slot);
        }
    });
    found
}
