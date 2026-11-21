//! Which `for x in f(..)` loops over a `yields` fn run their body as `f`
//! makes each value, with no list in between. Both back ends ask here, so
//! a loop that stops early, or a fn that never stops yielding, runs the
//! same way in each.

use crate::walk::{self, Effects};
use crate::{Arg, Block, Expr, ExprKind, Function, LoopId, Program, Stmt};
use std::collections::{BTreeMap, BTreeSet};

/// What a fn does beyond computing its result, counting the fns it calls.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct FnSummary {
    pub io: bool,
    /// It calls a fn value or makes a closure, so what it does is not known.
    pub opaque: bool,
    pub reads: BTreeSet<usize>,
    pub writes: BTreeSet<usize>,
}

/// What each fn reads, writes and prints, through the fns it calls.
pub fn summarize(program: &Program) -> Vec<FnSummary> {
    let direct: Vec<(FnSummary, BTreeSet<usize>)> = program
        .fns
        .iter()
        .map(|f| {
            let effects = Effects::of_block(&f.body);
            let mut summary = FnSummary {
                writes: effects.globals.clone(),
                ..FnSummary::default()
            };
            walk::deep_exprs(&f.body, &mut |e| match &e.kind {
                ExprKind::Read(_) | ExprKind::Print(..) => summary.io = true,
                ExprKind::CallExpr(..) | ExprKind::Closure(_) => summary.opaque = true,
                ExprKind::Global(id) => {
                    summary.reads.insert(*id);
                }
                _ => {}
            });
            walk::deep_stmts(&f.body, &mut |stmt| {
                // Sizing a domain changes what later code sees.
                if matches!(stmt, Stmt::DefDomain { .. }) {
                    summary.io = true;
                }
            });
            (summary, effects.calls)
        })
        .collect();
    let mut summaries: Vec<FnSummary> = direct.iter().map(|(s, _)| s.clone()).collect();
    loop {
        let before = summaries.clone();
        for (id, (_, calls)) in direct.iter().enumerate() {
            for callee in calls {
                let other = before[*callee].clone();
                summaries[id].io |= other.io;
                summaries[id].opaque |= other.opaque;
                summaries[id].reads.extend(other.reads);
                summaries[id].writes.extend(other.writes);
            }
        }
        if summaries == before {
            return summaries;
        }
    }
}

/// The slot of the list a `yields` fn builds, when the fn can also hand
/// each value to a callback as it is made: `for x in f(..)` then runs
/// its body between the yields with no list. That shows only if the fn
/// does something a loop body could see, so it must not read input,
/// print, store to a global, change an argument or call a fn value.
fn pusher(f: &Function, summary: &FnSummary) -> Option<usize> {
    let acc = f.locals.iter().position(|local| local.name == "__yields")?;
    if acc < f.params.len()
        || f.params.iter().any(|p| p.mutable)
        || summary.io
        || summary.opaque
        || !summary.writes.is_empty()
    {
        return None;
    }
    let is_acc = |e: &Expr| matches!(e.kind, ExprKind::Local(slot) if slot == acc);
    let fills = |e: &Expr| {
        matches!(&e.kind, ExprKind::Mutate(crate::Mutation::Push | crate::Mutation::Extend, place, _)
            if matches!(place.kind, crate::PlaceKind::Local(slot) if slot == acc))
    };
    // The list is only made, pushed to and returned.
    let (mut lets, mut pushes, mut returns, mut odd) = (0, 0, 0, false);
    walk::deep_stmts(&f.body, &mut |stmt| match stmt {
        Stmt::Let(slot, _) if *slot == acc => lets += 1,
        Stmt::Expr(e) if fills(e) => pushes += 1,
        Stmt::Return(Some(e)) if is_acc(e) => returns += 1,
        Stmt::Return(_) => odd = true,
        _ => {}
    });
    let (mut named, mut filled) = (0, 0);
    walk::deep_exprs(&f.body, &mut |e| {
        named += usize::from(is_acc(e));
        filled += usize::from(fills(e));
    });
    (!odd && lets == 1 && named == returns && filled == pushes).then_some(acc)
}


/// The summaries of a program's fns and its `yields` fns that can feed a
/// loop body: each to the slot of the list it builds.
#[derive(Default, Clone, Debug)]
pub struct Feeds {
    pub summaries: Vec<FnSummary>,
    pub pushers: BTreeMap<usize, usize>,
}

impl Feeds {
    pub fn of(program: &Program) -> Feeds {
        let summaries = summarize(program);
        let pushers = (0..program.fns.len())
            .filter_map(|id| Some((id, pusher(&program.fns[id], &summaries[id])?)))
            .collect();
        Feeds { summaries, pushers }
    }

    /// The fn and the slot of its list when the loop `id` over `seq`
    /// with this body runs the body as the fn yields. The loop must not
    /// be in a closure or in a fn that is itself feeding a loop.
    pub fn feeds(&self, id: LoopId, seq: &Expr, body: &Block) -> Option<(usize, usize)> {
        let ExprKind::Call(callee, args) = &seq.kind else {
            return None;
        };
        let acc = *self.pushers.get(callee)?;
        // The body must not change what the fn is still reading: a
        // global it reads, or a list handed to it.
        let summary = &self.summaries[*callee];
        let effects = Effects::of_block(body);
        let mut opaque = false;
        walk::deep_exprs(body, &mut |e| {
            opaque |= matches!(e.kind, ExprKind::CallExpr(..) | ExprKind::Closure(_));
        });
        let mut writes = effects.globals.clone();
        for f in &effects.calls {
            opaque |= self.summaries[*f].opaque;
            writes.extend(self.summaries[*f].writes.iter().copied());
        }
        let mut shared = false;
        for arg in args {
            let Arg::Value(value) = arg else {
                return None;
            };
            walk::deep_expr(value, &mut |e| match e.kind {
                ExprKind::Local(slot) => shared |= !e.ty.is_scalar() && effects.locals.contains(&slot),
                ExprKind::Global(g) => shared |= writes.contains(&g),
                ExprKind::Call(f, _) => shared |= !self.summaries[f].reads.is_disjoint(&writes),
                _ => {}
            });
        }
        if opaque || shared || !summary.reads.is_disjoint(&writes) {
            return None;
        }
        // A jump out of the body can only be to this loop.
        let mut inner = vec![id];
        let mut jumps = Vec::new();
        walk::deep_stmts(body, &mut |stmt| match stmt {
            Stmt::For(id, ..) | Stmt::While(id, ..) => inner.push(*id),
            Stmt::Break(Some(to)) | Stmt::Continue(Some(to)) => jumps.push(*to),
            _ => {}
        });
        jumps.iter().all(|to| inner.contains(to)).then_some((*callee, acc))
    }
}

/// True when `body` has a `return` in it.
pub fn returns(body: &Block) -> bool {
    let mut returns = false;
    walk::deep_stmts(body, &mut |stmt| returns |= matches!(stmt, Stmt::Return(_)));
    returns
}
