//! Which `for x in f(..)` loops over a `yields` fn run their body as `f`
//! makes each value, with no list in between. Both back ends ask here, so
//! a loop that stops early, or a fn that never stops yielding, runs the
//! same way in each.

use crate::walk::{self, Effects};
use crate::{Arg, Block, Expr, ExprKind, Function, Iter, LoopId, Program, Stmt};
use cpl_diag::{codes, Diagnostic};
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
    /// It yields fn values: they may write its locals, so a loop over it
    /// must run as it yields (R06.3.4), whatever else the body does.
    pub yields_fns: bool,
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
    for (summary, f) in summaries.iter_mut().zip(&program.fns) {
        summary.yields_fns = f
            .locals
            .iter()
            .any(|local| local.name == "__yields" && local.ty.has_fn());
    }
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
    pub(crate) fn merge(&mut self, other: &FnSummary) {
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
/// other than through a `mut` parameter, size a domain or call a fn value
/// that writes a variable it captured by reference.
fn pusher(f: &Function, summary: &FnSummary) -> Option<usize> {
    let acc = f.locals.iter().position(|local| local.name == "__yields")?;
    if acc < f.params.len() || summary.sizes || (!summary.yields_fns && summary.opaque) {
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
        let mut touched = false;
        for arg in args {
            let value = match arg {
                Arg::Value(value) => value,
                // The fn works on the caller's variable between the turns
                // of the body, so the body must not see it at all.
                Arg::Mut(place) => {
                    touched |= self.sees(walk::root(place), body, &effects, &calls);
                    continue;
                }
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
        if touched
            || !summary.yields_fns
                && (opaque
                    || shared
                    || !summary.reads.is_disjoint(&writes)
                    || !summary.writes.is_disjoint(&writes))
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

impl Feeds {
    /// Whether `body`, or a fn it calls, reads or writes the variable `root`.
    fn sees(
        &self,
        root: (bool, usize),
        body: &Block,
        effects: &Effects,
        calls: &BTreeSet<usize>,
    ) -> bool {
        let mut seen = match root {
            (false, slot) => effects.locals.contains(&slot),
            (true, g) => {
                effects.globals.contains(&g)
                    || calls.iter().any(|f| {
                        let summary = &self.summaries[*f];
                        summary.reads.contains(&g) || summary.writes.contains(&g)
                    })
            }
        };
        walk::deep_exprs(body, &mut |e| match e.kind {
            ExprKind::Local(slot) => seen |= root == (false, slot),
            ExprKind::Global(g) => seen |= root == (true, g),
            _ => {}
        });
        seen
    }
}

/// True when `body` has a `return` in it.
pub fn returns(body: &Block) -> bool {
    let mut returns = false;
    walk::deep_stmts(body, &mut |stmt| returns |= matches!(stmt, Stmt::Return(_)));
    returns
}

/// `yields` fns that yield closures writing one of their own locals, each
/// with the name of such a local: those closures live only while it runs.
fn yields_lent_closures(program: &Program) -> BTreeMap<usize, String> {
    let mut lent = BTreeMap::new();
    for (id, f) in program.fns.iter().enumerate() {
        if !f
            .locals
            .iter()
            .any(|l| l.name == "__yields" && l.ty.has_fn())
        {
            continue;
        }
        walk::deep_exprs(&f.body, &mut |e| {
            if let ExprKind::Closure(cid) = e.kind {
                let caps = &program.closures[cid].captures;
                if let Some(cap) = caps.iter().find(|c| c.by_ref && !c.is_global) {
                    lent.entry(id)
                        .or_insert_with(|| f.locals[cap.outer].name.clone());
                }
            }
        });
    }
    lent
}

/// A call of such a fn anywhere but as a `for` that runs its body as the
/// fn yields would keep its closures after its frame is gone, as a
/// `return`ed closure would (R05.10.2): each is E0603.
pub fn escaping_yields(program: &Program) -> Vec<Diagnostic> {
    let lent = yields_lent_closures(program);
    if lent.is_empty() {
        return Vec::new();
    }
    let feeds = Feeds::of(program);
    // Loops in a closure or in a fn that may itself be feeding are never fed.
    let mut fed: BTreeSet<*const Expr> = BTreeSet::new();
    let fed_bodies = [&program.main, &program.setup].into_iter().chain(
        (program.fns.iter().enumerate())
            .filter(|(id, _)| !feeds.pushers.contains_key(id))
            .map(|(_, f)| &f.body),
    );
    for body in fed_bodies {
        walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::For(id, _, Iter::Seq(seq, false), inner) = stmt {
                if feeds.feeds(*id, seq, inner).is_some() {
                    fed.insert(seq as *const Expr);
                }
            }
        });
    }
    let mut diags = Vec::new();
    let bodies = [&program.main, &program.setup]
        .into_iter()
        .chain(program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body));
    for body in bodies {
        walk::deep_exprs(body, &mut |e| {
            let ExprKind::Call(callee, _) = e.kind else {
                return;
            };
            let Some(local) = lent.get(&callee) else {
                return;
            };
            if fed.contains(&(e as *const Expr)) {
                return;
            }
            let name = &program.fns[callee].name;
            diags.push(Diagnostic::error(
                codes::LENT_ESCAPES,
                e.span,
                format!(
                    "`{name}` yields closures that write its local `{local}`, and this use keeps them after `{name}` returns; loop over it with a plain `for` instead"
                ),
            ));
        });
    }
    diags
}
