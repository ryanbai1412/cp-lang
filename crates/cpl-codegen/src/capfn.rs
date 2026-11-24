//! performance/R13. A closure that captures a fn parameter known to hold
//! one closure calls that closure directly: in
//! `fn opt(self: F) = Monoid(None, (a, b) => .. self(x, y) ..)` called
//! only with `lighter`, the inner closure calls `lighter` and captures
//! nothing, so the `Monoid.op` it fills needs no `std::function`. It
//! majorizes: the same closure runs on the same arguments, and a capture
//! of a name no one changes is the value that name holds.

use crate::devirt::Devirt;
use crate::walk;
use cpl_ir::{Arg, Block, ClosureId, Expr, ExprKind, FnId, PlaceKind, Program, Slot, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

/// A capture of a value with no representation (an op value like `+`,
/// a struct of them, `()`) holds nothing: the closure makes its own
/// (`Let` without an init is the type's zero) and captures nothing, so it
/// lowers like a captureless closure (R06.2.5).
pub(crate) fn zero_size_captures(program: &mut Program) {
    let structs = &program.structs;
    for def in &mut program.closures {
        let (empty, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut def.captures)
            .into_iter()
            .partition(|cap| {
                !cap.is_global && !cap.by_ref && zero_size(structs, &def.locals[cap.inner].ty)
            });
        def.captures = kept;
        let lets = empty.iter().map(|cap| Stmt::Let(cap.inner, None));
        def.body.stmts.splice(0..0, lets);
    }
}

fn zero_size(structs: &[cpl_ir::StructDef], ty: &Ty) -> bool {
    match ty {
        Ty::Unit => true,
        Ty::Tuple(items) => items.iter().all(|t| zero_size(structs, t)),
        Ty::Record(fields) => fields.iter().all(|(_, t)| zero_size(structs, t)),
        Ty::Struct(s) => structs[*s]
            .fields
            .iter()
            .all(|(_, t)| zero_size(structs, t)),
        _ => false,
    }
}

pub(crate) fn known_captures(program: &mut Program) {
    let mut plan: BTreeMap<(FnId, ClosureId), Vec<(Slot, ClosureId)>> = BTreeMap::new();
    {
        let devirt = Devirt::collect(program);
        for (id, f) in program.fns.iter().enumerate() {
            let changed = changed(program, id);
            let aliases = param_aliases(f);
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
                    .filter_map(|cap| {
                        let param = match aliases.get(&cap.outer) {
                            Some(&p) => p,
                            None => cap.outer,
                        };
                        let ok = !cap.is_global
                            && !cap.by_ref
                            && param < f.params.len()
                            && !f.params[param].mutable
                            && !changed.contains(&param)
                            && !changed.contains(&cap.outer)
                            && only_called(&def.body, cap.inner)
                            && !nested_capture(program, &def.body, cap.inner);
                        ok.then(|| Some((cap.inner, devirt.param_target(id, param)?)))
                            .flatten()
                    })
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

/// Each local of `f` bound once to a parameter or to the value in an
/// optional one (`if let g = cmp`), with the parameter it names.
fn param_aliases(f: &cpl_ir::Function) -> BTreeMap<Slot, Slot> {
    let mut lets: BTreeMap<Slot, Option<Slot>> = BTreeMap::new();
    walk::deep_stmts(&f.body, &mut |s| {
        if let Stmt::Let(slot, init) = s {
            let param = match init.as_ref().map(|e| &e.kind) {
                Some(ExprKind::Local(p)) => Some(*p),
                Some(ExprKind::Unwrap(inner, _)) => match inner.kind {
                    ExprKind::Local(p) => Some(p),
                    _ => None,
                },
                _ => None,
            };
            let once = !lets.contains_key(slot);
            lets.insert(*slot, param.filter(|&p| once && p < f.params.len()));
        }
    });
    lets.into_iter()
        .filter_map(|(slot, p)| Some((slot, p?)))
        .collect()
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
