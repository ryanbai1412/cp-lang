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
    /// It calls a fn value or makes a closure, so it cannot be turned
    /// inside out to feed a loop.
    pub calls_values: bool,
    /// What it does is not known: a fn value it may call writes a variable
    /// it captured by reference.
    pub opaque: bool,
    /// It sizes a domain, which changes what later code sees.
    pub sizes: bool,
    pub reads: BTreeSet<usize>,
    pub writes: BTreeSet<usize>,
}

/// What each fn reads, writes and prints, through the fns it calls.
/// A call through a fn value may run any closure of the program, so it
/// does what all of them do together: the last summary, at index
/// `program.fns.len()`, is that union.
pub fn summarize(program: &Program) -> Vec<FnSummary> {
    let direct_of = |body: &Block, by_ref: bool| {
        let effects = Effects::of_block(body);
        let mut summary = FnSummary {
            writes: effects.globals.clone(),
            opaque: by_ref,
            ..FnSummary::default()
        };
        let mut through = false;
        walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Read(_) | ExprKind::Print(..) => summary.io = true,
            ExprKind::CallExpr(..) => through = true,
            ExprKind::Closure(_) => summary.calls_values = true,
            ExprKind::Global(id) => {
                summary.reads.insert(*id);
            }
            _ => {}
        });
        walk::deep_stmts(body, &mut |stmt| {
            // Sizing a domain changes what later code sees.
            if matches!(stmt, Stmt::DefDomain { .. }) {
                summary.io = true;
                summary.sizes = true;
            }
        });
        summary.calls_values |= through;
        (summary, effects.calls, through)
    };
    let values = program.fns.len();
    let mut direct: Vec<(FnSummary, BTreeSet<usize>, bool)> = program
        .fns
        .iter()
        .map(|f| direct_of(&f.body, false))
        .collect();
    let mut all = (FnSummary::default(), BTreeSet::new(), false);
    for c in &program.closures {
        let (summary, calls, through) = direct_of(&c.body, c.captures.iter().any(|cap| cap.by_ref));
        all.0.merge(&summary);
        all.1.extend(calls);
        all.2 |= through;
    }
    direct.push(all);
    let mut summaries: Vec<FnSummary> = direct.iter().map(|(s, ..)| s.clone()).collect();
    loop {
        let before = summaries.clone();
        for (id, (_, calls, through)) in direct.iter().enumerate() {
            let through_values = through.then_some(values);
            for callee in calls.iter().copied().chain(through_values) {
                let other = before[callee].clone();
                summaries[id].merge(&other);
            }
        }
        if summaries == before {
            return summaries;
        }
    }
}

impl FnSummary {
    fn merge(&mut self, other: &FnSummary) {
        self.io |= other.io;
        self.opaque |= other.opaque;
        self.calls_values |= other.calls_values;
        self.sizes |= other.sizes;
        self.reads.extend(other.reads.iter().copied());
        self.writes.extend(other.writes.iter().copied());
    }
}

/// The slot of the list a `yields` fn builds, when the fn can also hand
/// each value to a callback as it is made: `for x in f(..)` then runs
/// its body between the yields with no list. What the fn prints, reads
/// from the input and stores to a global then happens between the turns
/// of the body, in both back ends alike. It must not change an argument
/// (the caller's variable is lent out for the whole call), size a domain
/// or call a fn value.
fn pusher(f: &Function, summary: &FnSummary) -> Option<usize> {
    let acc = f.locals.iter().position(|local| local.name == "__yields")?;
    if acc < f.params.len()
        || f.params.iter().any(|p| p.mutable)
        || summary.sizes
        || summary.opaque
        || summary.calls_values
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
        // The body must not change what the fn is still reading or
        // changing: a global it touches, or a list handed to it.
        let summary = &self.summaries[*callee];
        let effects = Effects::of_block(body);
        let mut calls = effects.calls.clone();
        walk::deep_exprs(body, &mut |e| {
            if matches!(e.kind, ExprKind::CallExpr(..)) {
                calls.insert(self.summaries.len() - 1);
            }
        });
        let mut opaque = false;
        let mut writes = effects.globals.clone();
        for f in &calls {
            opaque |= self.summaries[*f].opaque;
            writes.extend(self.summaries[*f].writes.iter().copied());
        }
        let mut shared = false;
        for arg in args {
            let Arg::Value(value) = arg else {
                return None;
            };
            walk::deep_expr(value, &mut |e| match e.kind {
                ExprKind::Local(slot) => {
                    shared |= !e.ty.is_scalar() && effects.locals.contains(&slot)
                }
                ExprKind::Global(g) => shared |= writes.contains(&g),
                ExprKind::Call(f, _) => shared |= !self.summaries[f].reads.is_disjoint(&writes),
                _ => {}
            });
        }
        if opaque
            || shared
            || !summary.reads.is_disjoint(&writes)
            || !summary.writes.is_disjoint(&writes)
        {
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
        jumps
            .iter()
            .all(|to| inner.contains(to))
            .then_some((*callee, acc))
    }
}

/// True when `body` has a `return` in it.
pub fn returns(body: &Block) -> bool {
    let mut returns = false;
    walk::deep_stmts(body, &mut |stmt| returns |= matches!(stmt, Stmt::Return(_)));
    returns
}
