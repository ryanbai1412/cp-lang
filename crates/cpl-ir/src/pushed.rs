//! Which `for x in f(..)` loops over a `yields` fn run their body as `f`
//! makes each value, with no list in between. Both back ends ask here, so
//! a loop that stops early, or a fn that never stops yielding, runs the
//! same way in each.

use crate::walk::{self, Effects};
use crate::{Arg, Block, ClosureDef, Expr, ExprKind, Function, Iter, LoopId, Program, Stmt, Ty};
use cpl_diag::Diagnostic;
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
    /// The fns whose frames a closure it may call writes by reference
    /// ([`TOP`]: the program's top level). Only another fn's frame makes it
    /// `opaque` to a loop it feeds. A set of `usize`, as `reads` is, so the
    /// browser build carries one set's code.
    pub lent_from: BTreeSet<usize>,
    /// The fns whose locals a closure it may call writes by reference
    /// ([`LENT_ELSEWHERE`]: a closure's own frame, or a global), and those
    /// locals' slots in their owner's frame.
    pub lent_written_from: BTreeSet<usize>,
    pub lent_written: BTreeSet<usize>,
    /// The same for every local such a closure reads or writes by reference.
    pub lent_seen_from: BTreeSet<usize>,
    pub lent_seen: BTreeSet<usize>,
    pub reads: BTreeSet<usize>,
    pub writes: BTreeSet<usize>,
}

/// What each fn reads, writes and prints, through the fns it calls.
/// A call through a fn value may run any closure of the program, so it
/// does what all of them do together: the summary at index
/// `program.fns.len()` is that union. A call through a value of a known
/// fn type runs only a closure of that type (the program is
/// monomorphized), so each type called through gets the union of those,
/// after it.
pub fn summarize(program: &Program) -> Vec<FnSummary> {
    summarize_typed(program).0
}

/// [`summarize`], with the fn types whose values are called, in the order
/// of their summaries after the all-closures one.
pub fn summarize_typed(program: &Program) -> (Vec<FnSummary>, Vec<Ty>) {
    let values = program.fns.len();
    let mut called: Vec<Ty> = Vec::new();
    let mut direct_of = |body: &Block, by_ref: bool| {
        let effects = Effects::of_block(body);
        let mut summary = FnSummary {
            writes: effects.globals.clone(),
            opaque: by_ref,
            ..FnSummary::default()
        };
        let mut through = BTreeSet::new();
        walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Read(_) | ExprKind::Print(..) => summary.io = true,
            ExprKind::CallExpr(callee, _) => {
                let k = match called.iter().position(|t| *t == callee.ty) {
                    Some(k) => k,
                    None => {
                        called.push(callee.ty.clone());
                        called.len() - 1
                    }
                };
                through.insert(values + 1 + k);
            }
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
        summary.calls_values |= !through.is_empty();
        (summary, effects.calls, through)
    };
    let mut direct: Vec<Direct> = program
        .fns
        .iter()
        .map(|f| direct_of(&f.body, false))
        .collect();
    let owners = closure_owners(program);
    let closures: Vec<Direct> = program
        .closures
        .iter()
        .zip(owners)
        .map(|(c, (owner, nested))| {
            let by_ref = c.captures.iter().any(|cap| cap.by_ref);
            let (mut summary, calls, through) = direct_of(&c.body, by_ref);
            if by_ref {
                summary.lent_from.insert(owner);
            }
            let written = Effects::of_block(&c.body).locals;
            for cap in c.captures.iter().filter(|cap| cap.by_ref) {
                let from = if nested || cap.is_global {
                    LENT_ELSEWHERE
                } else {
                    owner
                };
                summary.lent_seen_from.insert(from);
                summary.lent_seen.insert(cap.outer);
                if written.contains(&cap.inner) {
                    summary.lent_written_from.insert(from);
                    summary.lent_written.insert(cap.outer);
                }
            }
            (summary, calls, through)
        })
        .collect();
    let union = |of: &dyn Fn(&ClosureDef) -> bool| {
        let mut all: Direct = Default::default();
        for (c, (summary, calls, through)) in program.closures.iter().zip(&closures) {
            if of(c) {
                all.0.merge(summary);
                all.1.extend(calls);
                all.2.extend(through);
            }
        }
        all
    };
    direct.push(union(&|_| true));
    for ty in &called {
        direct.push(match ty {
            Ty::Fn(args, ret) => union(&|c| closure_has_type(c, args, ret)),
            _ => union(&|_| true),
        });
    }
    let mut summaries: Vec<FnSummary> = direct.iter().map(|(s, ..)| s.clone()).collect();
    for (summary, f) in summaries.iter_mut().zip(&program.fns) {
        summary.yields_fns = f
            .locals
            .iter()
            .any(|local| local.name == "__yields" && local.ty.has_fn(program));
    }
    loop {
        let before = summaries.clone();
        for (id, (_, calls, through)) in direct.iter().enumerate() {
            for &callee in calls.iter().chain(through) {
                let other = before[callee].clone();
                summaries[id].merge(&other);
            }
        }
        if summaries == before {
            return (summaries, called);
        }
    }
}

/// A fn's or closure's own effects, the fns it calls and the
/// [`summarize`] entries of the fn types it calls values of.
type Direct = (FnSummary, BTreeSet<usize>, BTreeSet<usize>);

/// Whether closure `c` can be a value of fn type `args -> ret`.
pub fn closure_has_type(c: &ClosureDef, args: &[Ty], ret: &Ty) -> bool {
    let same = |param: &Ty, arg: &Ty| param == arg || matches!(arg, Ty::Mut(of) if **of == *param);
    c.ret == *ret
        && c.params.len() == args.len()
        && c.params
            .iter()
            .zip(args)
            .all(|(&p, a)| same(&c.locals[p].ty, a))
}

/// The owner of what the program's top level makes, beside its fns.
pub const TOP: usize = usize::MAX;

/// The owner of a local no loop's fn can name: a closure's own, a global
/// captured by reference, or a loop whose fn is not known.
pub const LENT_ELSEWHERE: usize = usize::MAX - 1;

/// The fn each closure is made in, through the closures it is made in
/// ([`TOP`]: the top level), and whether it is made in another closure.
fn closure_owners(program: &Program) -> Vec<(usize, bool)> {
    #[derive(Clone, Copy)]
    enum Made {
        Top,
        Fn(usize),
        In(usize),
    }
    let mut made = vec![Made::Top; program.closures.len()];
    let mut mark = |body: &Block, by: Made| {
        walk::deep_exprs(body, &mut |e| {
            if let ExprKind::Closure(c) = e.kind {
                made[c] = by;
            }
        })
    };
    for (id, f) in program.fns.iter().enumerate() {
        mark(&f.body, Made::Fn(id));
    }
    for (id, c) in program.closures.iter().enumerate() {
        mark(&c.body, Made::In(id));
    }
    (0..made.len())
        .map(|mut c| {
            let nested = matches!(made[c], Made::In(_));
            for _ in 0..made.len() {
                match made[c] {
                    Made::Top => return (TOP, nested),
                    Made::Fn(id) => return (id, nested),
                    Made::In(outer) => c = outer,
                }
            }
            (TOP, nested)
        })
        .collect()
}

impl FnSummary {
    pub(crate) fn merge(&mut self, other: &FnSummary) {
        self.io |= other.io;
        self.opaque |= other.opaque;
        self.calls_values |= other.calls_values;
        self.sizes |= other.sizes;
        self.lent_from.extend(other.lent_from.iter().copied());
        self.lent_written_from
            .extend(other.lent_written_from.iter().copied());
        self.lent_written.extend(other.lent_written.iter().copied());
        self.lent_seen_from
            .extend(other.lent_seen_from.iter().copied());
        self.lent_seen.extend(other.lent_seen.iter().copied());
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
///
/// A fn returning a list it only makes, pushes to and returns is one too,
/// but it runs to its end whatever takes its values: only a reducer that
/// takes every value is fed by it, never a loop that may stop early.
fn pusher(id: usize, f: &Function, summary: &FnSummary) -> Option<usize> {
    let yields = f.locals.iter().position(|local| local.name == "__yields");
    let acc = yields.or_else(|| returned_list(f))?;
    // A closure it makes writing its own locals is part of its own run.
    let opaque = summary.lent_from.iter().any(|&owner| owner != id);
    if acc < f.params.len() || summary.sizes || (!summary.yields_fns && opaque) {
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
    let mut empty = false;
    let (mut named, mut filled, mut stored) = (0, 0, 0);
    let at_acc = |place: &crate::Place| walk::root(place) == (false, acc);
    walk::deep_stmts(&f.body, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(init)) if *slot == acc => {
            empty = matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty())
        }
        Stmt::Assign(place, ..) => stored += usize::from(at_acc(place)),
        _ => {}
    });
    walk::deep_exprs(&f.body, &mut |e| {
        named += usize::from(is_acc(e));
        filled += usize::from(fills(e));
        match &e.kind {
            ExprKind::Mutate(_, place, _)
            | ExprKind::MutSlice(place, ..)
            | ExprKind::MutLine(place, ..)
            | ExprKind::Sort(place, ..) => stored += usize::from(at_acc(place)),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                stored += args
                    .iter()
                    .filter(|a| matches!(a, Arg::Mut(place) if at_acc(place)))
                    .count()
            }
            _ => {}
        }
    });
    // A list it starts with values in, or changes past pushing, is not what it yields.
    (!odd && lets == 1 && empty && named == returns && filled == pushes && stored == filled)
        .then_some(acc)
}

/// Whether a closure `f` makes captures its local `slot`: its body runs
/// elsewhere, where a push to the list cannot become a call of the loop body.
fn captured(program: &Program, f: &Function, slot: usize) -> bool {
    let mut seen = false;
    walk::deep_exprs(&f.body, &mut |e| {
        if let ExprKind::Closure(c) = e.kind {
            seen |= program.closures[c]
                .captures
                .iter()
                .any(|cap| !cap.is_global && cap.outer == slot);
        }
    });
    seen
}

/// The local a fn returns when it is a list it made itself.
fn returned_list(f: &Function) -> Option<usize> {
    let mut acc = None;
    walk::deep_stmts(&f.body, &mut |stmt| {
        if let Stmt::Return(Some(e)) = stmt {
            if let ExprKind::Local(slot) = e.kind {
                acc = acc.or(Some(slot));
            }
        }
    });
    acc.filter(|&slot| slot >= f.params.len() && matches!(f.locals[slot].ty, crate::Ty::List(_)))
}

/// The summaries of a program's fns and its `yields` fns that can feed a
/// loop body: each to the slot of the list it builds.
#[derive(Default, Clone, Debug)]
pub struct Feeds {
    pub summaries: Vec<FnSummary>,
    pub pushers: BTreeMap<usize, usize>,
    /// The pushers that are not `yields` fns: they run to their end.
    pub eager: BTreeSet<usize>,
    pub scope: FeedScope,
}

/// Where each loop is and which fn types are called, so a loop knows whose
/// locals a closure its body calls writes.
#[derive(Default, Clone, Debug)]
pub struct FeedScope {
    /// The fn types whose summaries follow the all-closures one.
    pub values: Vec<Ty>,
    /// The fn each loop is in ([`TOP`]: the top level; [`LENT_ELSEWHERE`]:
    /// a closure, or more than one place).
    pub homes: BTreeMap<LoopId, usize>,
}

impl FeedScope {
    fn of(program: &Program, values: Vec<Ty>) -> FeedScope {
        let mut homes = BTreeMap::new();
        let bodies = [(&program.main, TOP), (&program.setup, TOP)]
            .into_iter()
            .chain(program.fns.iter().enumerate().map(|(id, f)| (&f.body, id)))
            .chain(program.closures.iter().map(|c| (&c.body, LENT_ELSEWHERE)));
        for (body, home) in bodies {
            walk::deep_stmts(body, &mut |stmt| {
                if let Stmt::For(id, ..) = stmt {
                    let at = homes.entry(*id).or_insert(home);
                    if *at != home {
                        *at = LENT_ELSEWHERE;
                    }
                }
            });
        }
        FeedScope { values, homes }
    }
}

impl Feeds {
    pub fn of(program: &Program) -> Feeds {
        let mut feeds = Feeds::candidates(program);
        feeds.keep_fed(program, &BTreeSet::new());
        feeds
    }

    /// Every `yields` fn that could feed a loop, fed or not yet.
    pub fn candidates(program: &Program) -> Feeds {
        let (summaries, values) = summarize_typed(program);
        let scope = FeedScope::of(program, values);
        // Inserted in key order: collecting a BTreeMap sorts, a copy of std's stable sort.
        let mut pushers = BTreeMap::new();
        let mut eager = BTreeSet::new();
        for (id, f) in program.fns.iter().enumerate() {
            if let Some(p) = pusher(id, f, &summaries[id]).filter(|&p| !captured(program, f, p)) {
                pushers.insert(id, p);
                if f.locals[p].name != "__yields" {
                    eager.insert(id);
                }
            }
        }
        Feeds {
            summaries,
            pushers,
            eager,
            scope,
        }
    }

    /// Keeps the fns some loop is fed by, and `also`: only they get the form that feeds.
    pub fn keep_fed(&mut self, program: &Program, also: &BTreeSet<usize>) {
        let feeds = &*self;
        let mut fed = also.clone();
        let bodies = [&program.main, &program.setup]
            .into_iter()
            .chain(program.fns.iter().map(|f| &f.body))
            .chain(program.closures.iter().map(|c| &c.body));
        for body in bodies {
            walk::deep_stmts(body, &mut |stmt| {
                if let Stmt::For(id, _, Iter::Seq(seq, false), inner) = stmt {
                    if let Some((callee, _)) = feeds.feeds(*id, seq, inner) {
                        fed.insert(callee);
                    }
                }
            });
        }
        // Rebuilt by insertion: `retain` brings in a BTree's own removal code.
        for (id, n) in std::mem::take(&mut self.pushers) {
            if fed.contains(&id) {
                self.pushers.insert(id, n);
            }
        }
        for id in std::mem::take(&mut self.eager) {
            if fed.contains(&id) {
                self.eager.insert(id);
            }
        }
    }

    /// The pushers whose values a `.len` only counts: the count is taken
    /// as they are made, with no list.
    pub fn counted(&self, program: &Program) -> BTreeSet<usize> {
        let mut counted = BTreeSet::new();
        let bodies = [&program.main, &program.setup]
            .into_iter()
            .chain(program.fns.iter().map(|f| &f.body))
            .chain(program.closures.iter().map(|c| &c.body));
        for body in bodies {
            walk::deep_exprs(body, &mut |e| {
                if let Some(callee) = self.counts(e) {
                    counted.insert(callee);
                }
            });
        }
        counted
    }

    /// The pusher `e` takes the length of, `f(..).len`.
    pub fn counts(&self, e: &Expr) -> Option<usize> {
        match &e.kind {
            ExprKind::Builtin(crate::Builtin::Len, of) => match of.as_slice() {
                [Expr {
                    kind: ExprKind::Call(callee, _),
                    ..
                }] if self.pushers.contains_key(callee) => Some(*callee),
                _ => None,
            },
            _ => None,
        }
    }

    /// The fn and the slot of its list when the loop `id` over `seq`
    /// with this body runs the body as the fn yields. The loop must not
    /// be in a closure or in a fn that is itself feeding a loop.
    /// Whether `body` has a loop some fn feeds, which never leaves early:
    /// from inside a fed fn that loop's yields feed on out.
    pub fn walks_a_fed_fn(&self, body: &Block) -> bool {
        let mut walks = false;
        walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::For(id, _, Iter::Seq(seq, false), b) = stmt {
                walks |= !escapes(b) && self.feeds(*id, seq, b).is_some();
            }
        });
        walks
    }

    pub fn feeds(&self, id: LoopId, seq: &Expr, body: &Block) -> Option<(usize, usize)> {
        let ExprKind::Call(callee, args) = &seq.kind else {
            return None;
        };
        let acc = *self.pushers.get(callee)?;
        if self.eager.contains(callee) {
            return None;
        }
        // The body must not change what the fn is still reading or
        // changing: a global it touches, or a list handed to it.
        let summary = &self.summaries[*callee];
        let effects = Effects::of_block(body);
        let mut calls = effects.calls.clone();
        let fns = self.summaries.len() - 1 - self.scope.values.len();
        walk::deep_exprs(body, &mut |e| {
            if let ExprKind::CallExpr(callee, _) = &e.kind {
                let typed = self.scope.values.iter().position(|t| *t == callee.ty);
                calls.insert(typed.map_or(fns, |k| fns + 1 + k));
            }
        });
        // A closure the body calls may write a local by reference only in
        // the loop's own fn, and never one the fn is handed.
        let home = self.scope.homes.get(&id).copied().unwrap_or(LENT_ELSEWHERE);
        let foreign = |from: &BTreeSet<usize>| {
            from.iter()
                .any(|&owner| owner != home || owner == LENT_ELSEWHERE)
        };
        let mut opaque = false;
        let mut seen_elsewhere = false;
        let mut writes = effects.globals.clone();
        let mut lent_written = BTreeSet::new();
        let mut lent_seen = BTreeSet::new();
        for f in &calls {
            let s = &self.summaries[*f];
            opaque |= foreign(&s.lent_written_from);
            seen_elsewhere |= foreign(&s.lent_seen_from);
            writes.extend(s.writes.iter().copied());
            lent_written.extend(s.lent_written.iter().copied());
            lent_seen.extend(s.lent_seen.iter().copied());
        }
        let mut shared = false;
        let mut touched = false;
        for arg in args {
            let value = match arg {
                Arg::Value(value) => value,
                // The fn works on the caller's variable between the turns
                // of the body, so the body must not see it at all.
                Arg::Mut(place) => {
                    let root = walk::root(place);
                    touched |= self.sees(root, body, &effects, &calls)
                        || !root.0 && (seen_elsewhere || lent_seen.contains(&root.1));
                    continue;
                }
            };
            walk::deep_expr(value, &mut |e| match e.kind {
                ExprKind::Local(slot) => {
                    shared |= !e.ty.is_scalar()
                        && (effects.locals.contains(&slot) || lent_written.contains(&slot))
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

/// Whether `body` may leave its loop early: a `break`, `continue` or `return`.
pub fn escapes(body: &Block) -> bool {
    let mut escapes = false;
    walk::deep_stmts(body, &mut |stmt| {
        escapes |= matches!(stmt, Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(_));
    });
    escapes
}

/// Whether `body` may stop its loop: a `break`, a `return` or a labeled `continue`.
pub fn stops(body: &Block) -> bool {
    let mut stops = false;
    walk::deep_stmts(body, &mut |stmt| {
        stops |= matches!(
            stmt,
            Stmt::Break(_) | Stmt::Return(_) | Stmt::Continue(Some(_))
        )
    });
    stops
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
            .any(|l| l.name == "__yields" && l.ty.has_fn(program))
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
    // Addresses as `usize`, so this is the one set of `usize` the crate already carries.
    let mut fed: BTreeSet<usize> = BTreeSet::new();
    let fed_bodies = [&program.main, &program.setup].into_iter().chain(
        (program.fns.iter().enumerate())
            .filter(|(id, _)| !feeds.pushers.contains_key(id))
            .map(|(_, f)| &f.body),
    );
    for body in fed_bodies {
        walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::For(id, _, Iter::Seq(seq, false), inner) = stmt {
                if feeds.feeds(*id, seq, inner).is_some() {
                    fed.insert(seq as *const Expr as usize);
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
            if fed.contains(&(e as *const Expr as usize)) {
                return;
            }
            let name = &program.fns[callee].name;
            diags.push(
                cpl_diag::msgs::check::LENT_CLOSURES_KEPT.error(e.span, &[name, local, name]),
            );
        });
    }
    diags
}
