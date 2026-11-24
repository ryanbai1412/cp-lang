//! What is known about the sign of integers, so that Euclidean `/` and
//! `%` cost nothing where C++'s own would give the same answer.
//!
//! C++ rounds a quotient toward zero and the language rounds so that the
//! remainder is never negative. The two agree when the dividend is not
//! negative and the divisor is positive, and that is what this proves:
//! from literals, from the declared ranges of input (`let n: 1..=2e5 =
//! read()`), from loop bounds, and from what arithmetic keeps. A variable
//! counts only if it is stored exactly once, so a later assignment can't
//! undo the fact.

use crate::uses::Home;

mod order;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, DomainId, Expr, ExprKind, Iter, Place,
    PlaceKind, Program, Spec, Stmt, Ty, UnOp,
};
use std::collections::{BTreeMap, BTreeSet};

/// The most elements a list or string holds: memory runs out first.
const LEN_MAX: i64 = 1 << 31;
/// The most returns a fn's call result is bounded through.
const RETURNS_MAX: usize = 4;

/// A variable: a global, or a local of one fn (`None` is the top level).
pub(crate) type Var = (Option<usize>, bool, usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Sign {
    Unknown,
    /// `>= 0`.
    NonNeg,
    /// `> 0`.
    Positive,
}

#[derive(Clone, Copy)]
enum StoreOp {
    /// `v = rhs` (or `let v = rhs`): NonNeg if `rhs` is NonNeg/Positive.
    Set,
    /// `+=`, `*=`, `|=`, `^=`, `/=`, `>>=`, `max=`: preserves NonNeg if `rhs` is NonNeg/Positive.
    KeepNonNeg,
    /// `v += rhs`: as `KeepNonNeg`, and bounded by the guards on `v` there.
    Add,
    /// `&=`: preserves NonNeg regardless of `rhs`.
    BitAnd,
    /// `%=`: Euclidean remainder is always NonNeg.
    Rem,
}

type Store<'p> = (Option<usize>, StoreOp, &'p Expr);

/// What a branch or loop condition in force says about a local `v`:
/// `v <= e + k` or `v >= e + k`, with `e` read where the condition is.
#[derive(Clone, Copy)]
enum Fact<'p> {
    Le(&'p Expr, i64),
    Ge(&'p Expr, i64),
    /// `v <= k`, from a shift by `v` that stays positive.
    Most(i64),
}

/// A condition's facts: a guard on a local, and when the guard was made.
type Guard<'p> = (Var, Fact<'p>, usize);

/// Where an expression is: a guard's facts hold for one read of a local.
fn at(e: &Expr) -> usize {
    std::ptr::from_ref(e) as usize
}

/// A block that never falls through to the statement after it.
fn diverges(b: &Block) -> bool {
    matches!(
        b.stmts.last(),
        Some(Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..))
    )
}

fn read_var(fn_id: Option<usize>, e: &Expr) -> Option<Var> {
    match &e.kind {
        ExprKind::Local(slot) => Some((fn_id, false, *slot)),
        ExprKind::Global(id) => Some((None, true, *id)),
        ExprKind::Field(base, idx) => match base.kind {
            ExprKind::Local(slot) if slot < 1 << 15 && *idx < 1 << 16 => {
                Some((fn_id, true, FIELD_KEY | slot << 16 | idx))
            }
            _ => None,
        },
        _ => None,
    }
}

/// A guard's key for a field of a local, `s.f`: its own number past every
/// global's, so a guard on it dies with any store to `s`. 31 bits, so it
/// packs the same on wasm32's 32-bit `usize`.
const FIELD_KEY: usize = 1 << 31;

/// The local whose field the guard key `v` is.
fn field_base(v: Var) -> Option<Var> {
    (v.1 && v.2 & FIELD_KEY != 0).then_some((v.0, false, (v.2 & !FIELD_KEY) >> 16))
}

/// What `cond` being `holds` says about the locals it compares.
fn cond_facts<'p>(
    fn_id: Option<usize>,
    cond: &'p Expr,
    holds: bool,
    out: &mut Vec<(Var, Fact<'p>)>,
) {
    use CmpOp::*;
    match &cond.kind {
        ExprKind::Unary(UnOp::Not, x) => cond_facts(fn_id, x, !holds, out),
        ExprKind::Binary(BinOp::And, a, b) if holds => {
            cond_facts(fn_id, a, true, out);
            cond_facts(fn_id, b, true, out);
        }
        ExprKind::Binary(BinOp::Or, a, b) if !holds => {
            cond_facts(fn_id, a, false, out);
            cond_facts(fn_id, b, false, out);
        }
        ExprKind::Cmp(op, l, r) if l.ty == Ty::Int && r.ty == Ty::Int => {
            let op = match (holds, op) {
                (true, op) => *op,
                (false, Lt) => Ge,
                (false, Le) => Gt,
                (false, Gt) => Le,
                (false, Ge) => Lt,
                (false, Eq) => Ne,
                (false, Ne) => Eq,
                _ => return,
            };
            let flip = match op {
                Lt => Gt,
                Le => Ge,
                Gt => Lt,
                Ge => Le,
                op => op,
            };
            for (x, op, y) in [(l, op, r), (r, flip, l)] {
                // `(x >> s) > k >= 0`: a shift of 63 or more leaves 0 or -1.
                if let (ExprKind::Binary(BinOp::Shr, _, s), Gt | Ge, ExprKind::Int(k)) =
                    (&x.kind, op, &y.kind)
                {
                    if let Some(s) = read_var(fn_id, s).filter(|_| *k >= i64::from(op == Ge)) {
                        out.push((s, Fact::Most(62)));
                    }
                }
                let Some(v) = read_var(fn_id, x) else {
                    continue;
                };
                match op {
                    Lt => out.push((v, Fact::Le(y, -1))),
                    Le => out.push((v, Fact::Le(y, 0))),
                    Gt => out.push((v, Fact::Ge(y, 1))),
                    Ge => out.push((v, Fact::Ge(y, 0))),
                    Eq => out.extend([(v, Fact::Le(y, 0)), (v, Fact::Ge(y, 0))]),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[derive(Default)]
pub(crate) struct Facts<'p> {
    /// How many times each variable is stored to, in any way.
    stores: BTreeMap<Var, u32>,
    /// The value a variable starts with, or the spec it is read with.
    init: BTreeMap<Var, &'p Expr>,
    /// Every direct store to a local/global variable, for multi-store NonNeg proof.
    var_stores: BTreeMap<Var, Vec<Store<'p>>>,
    /// Variables excluded from multi-store NonNeg proof.
    tainted: BTreeSet<Var>,
    /// Multi-store variables proven `>= 0` across every store.
    nonneg_vars: BTreeSet<Var>,
    /// Loop variables: the low end of the range they run over.
    loop_lo: BTreeMap<Var, &'p Expr>,
    /// Loop variables: what they walk ([`order`]).
    loop_iter: BTreeMap<Var, &'p cpl_ir::Iter>,
    /// Loop variables: the high end of the range, and whether it is reached.
    loop_hi: BTreeMap<Var, (&'p Expr, bool)>,
    /// Each domain's length expression, when defined once.
    domain_len: BTreeMap<usize, Option<(Option<usize>, &'p Expr)>>,
    /// Each domain's length bound, worked out once from its definition: a
    /// bound reached through a call or a field doesn't spend its depth on it.
    domain_hi: BTreeMap<usize, i64>,
    /// Struct fields whose declared range narrows to unsigned 32-bit (`>= 0`).
    u32_fields: BTreeSet<(usize, usize)>,
    /// Struct fields' declared ranges (`StructDef::int_ranges`): every value holds them.
    field_ranges: BTreeMap<(usize, usize), (i64, i64)>,
    /// The domains whose positions are held in 64 bits; `None` when the
    /// facts were not collected over a whole program, so no count is bounded.
    wide: Option<BTreeSet<usize>>,
    /// Each domain's lower-bound expression, when defined once at top level.
    domain_lo: BTreeMap<usize, Option<(Option<usize>, &'p Expr)>>,
    /// Ranged params: every call checks its argument into the range, and
    /// every store to the param is checked too (R04.4.24).
    param_ranges: BTreeMap<Var, &'p Spec>,
    /// The fns before closure 0 in the [`Facts::key`] numbering.
    fns: usize,
    /// Where each closure is made: a home for each `Closure` expr.
    closure_homes: BTreeMap<usize, BTreeSet<Option<usize>>>,
    /// A closure's by-value capture: the variable it holds a copy of.
    captured: BTreeMap<Var, Var>,
    /// Variables with a store other than a set or a step by one.
    odd_stores: BTreeSet<Var>,
    /// Variables only set to bounded values `>= 0` and stepped by one: see
    /// [`Facts::unit_step`].
    unit_steps: BTreeSet<Var>,
    /// Variables set more than once, each time to a value under the bound
    /// (or kept under it by `&=`/`%=` while `>= 0`): see [`Facts::stores_hi`].
    var_hi: BTreeMap<Var, i64>,
    /// While `collect` walks a body, the guards in force; a store to the
    /// guarded local kills its guard for the rest of the walk.
    active: Vec<Option<Guard<'p>>>,
    /// How many guards have been made, numbering them in walk order.
    made: usize,
    /// Every store to a local in walk order, so a loop can drop the facts
    /// its body's stores undo on the next lap.
    store_log: Vec<Var>,
    /// Each read of a guarded local: where it is, and the guard.
    leaf_log: Vec<(usize, Guard<'p>)>,
    /// What the conditions in force say about a read of a local, by where
    /// the read is (R09.3.4). One walk finds them; no loop is iterated.
    guarded: BTreeMap<usize, Vec<(Var, Fact<'p>)>>,
    /// Each fn's returned values; a call's result is one of them, read
    /// with the facts of the fn's own body.
    returns: BTreeMap<usize, Vec<&'p Expr>>,
}

fn var(fn_id: Option<usize>, kind: &PlaceKind) -> Option<Var> {
    match kind {
        PlaceKind::Local(slot) => Some((fn_id, false, *slot)),
        PlaceKind::Global(id) => Some((None, true, *id)),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            var(fn_id, &base.kind)
        }
    }
}

impl<'p> Facts<'p> {
    pub fn collect(program: &'p Program) -> Facts<'p> {
        let mut facts = Facts {
            fns: program.fns.len(),
            ..Facts::default()
        };
        facts.block(None, &program.setup);
        facts.block(None, &program.main);
        for (id, f) in program.fns.iter().enumerate() {
            // A parameter's value comes from each caller: unknown.
            for (slot, param) in f.params.iter().enumerate() {
                let v = (Some(id), false, slot);
                *facts.stores.entry(v).or_default() += 2;
                facts.taint(v);
                if let Some(range) = &param.range {
                    facts.param_ranges.insert(v, range);
                }
            }
            facts.block(Some(id), &f.body);
        }
        for (cid, c) in program.closures.iter().enumerate() {
            let home = facts.key(Home::Closure(cid));
            // A closure's parameter comes from each call: unknown.
            for &slot in &c.params {
                let v = (home, false, slot);
                *facts.stores.entry(v).or_default() += 2;
                facts.taint(v);
            }
            facts.block(home, &c.body);
        }
        facts.settle_guards(program);
        for (cid, c) in program.closures.iter().enumerate() {
            let homes = facts.closure_homes.get(&cid).filter(|h| h.len() == 1);
            let Some(&made) = homes.and_then(BTreeSet::first) else {
                continue;
            };
            for cap in c.captures.iter().filter(|cap| !cap.by_ref) {
                let outer = match cap.is_global {
                    true => (None, true, cap.outer),
                    false => (made, false, cap.outer),
                };
                let inner = (facts.key(Home::Closure(cid)), false, cap.inner);
                facts.captured.insert(inner, outer);
            }
        }
        // `width::domain_sizes` collects facts before it decides the widths.
        facts.wide = (!program.wide_domains.is_empty()).then(|| {
            (program.wide_domains.iter().enumerate())
                .filter(|(_, w)| **w)
                .map(|(d, _)| d)
                .collect()
        });
        for (sid, s) in program.structs.iter().enumerate() {
            for (&fi, &range) in &s.int_ranges {
                facts.field_ranges.insert((sid, fi), range);
            }
            for (&fi, &w) in &s.narrow_ints {
                if w.is_unsigned() {
                    facts.u32_fields.insert((sid, fi));
                }
            }
        }
        let mut nonneg: BTreeSet<Var> = facts
            .var_stores
            .keys()
            .copied()
            .filter(|v| !facts.tainted.contains(v) && !facts.stable(*v))
            .collect();
        loop {
            facts.nonneg_vars = nonneg.clone();
            let mut changed = false;
            nonneg.retain(|v| {
                let ok = facts.var_stores[v]
                    .iter()
                    .all(|&(home, sop, rhs)| match sop {
                        StoreOp::Rem | StoreOp::BitAnd => true,
                        StoreOp::Set | StoreOp::KeepNonNeg | StoreOp::Add => {
                            facts.sign(home, rhs) != Sign::Unknown
                        }
                    });
                if !ok {
                    changed = true;
                }
                ok
            });
            if !changed {
                break;
            }
        }
        facts.nonneg_vars = nonneg;
        // A closure that captures a local by reference stores to it under
        // its own slot.
        for (cid, c) in program.closures.iter().enumerate() {
            let inner_home = facts.key(Home::Closure(cid));
            let homes: Vec<_> = facts
                .closure_homes
                .get(&cid)
                .map(|h| h.iter().copied().collect())
                .unwrap_or_default();
            for cap in c.captures.iter().filter(|cap| cap.by_ref && !cap.is_global) {
                if facts.stores((inner_home, false, cap.inner)) > 0 {
                    for &home in &homes {
                        facts.odd_stores.insert((home, false, cap.outer));
                    }
                }
            }
        }
        let unit: BTreeSet<Var> = facts
            .stores
            .keys()
            .copied()
            .filter(|v| !facts.odd_stores.contains(v))
            .filter(|v| {
                facts.var_stores.get(v).is_some_and(|stores| {
                    stores.iter().all(|&(home, sop, rhs)| {
                        !matches!(sop, StoreOp::Set)
                            || (facts.sign(home, rhs) != Sign::Unknown
                                && facts
                                    .upper_bound(home, rhs, 1)
                                    .is_some_and(|hi| hi < 1 << 62))
                    })
                })
            })
            .collect();
        facts.unit_steps = unit;
        let hi: BTreeMap<Var, i64> = facts
            .var_stores
            .keys()
            .filter_map(|&v| Some((v, facts.stores_hi(v)?)))
            .collect();
        facts.var_hi = hi;
        let domains: Vec<usize> = facts.domain_len.keys().copied().collect();
        for d in domains {
            if let Some(hi) = facts.domain_max_len_at(d, 0) {
                facts.domain_hi.insert(d, hi);
            }
        }
        facts
    }

    /// The largest value variable `v` is ever set to, when every store to
    /// it is a plain set of a bounded value, or a `&=`/`%=` of a variable
    /// proven `>= 0`, which never grows it. A store that reads `v` itself
    /// is unbounded here, so one pass over the stores decides it.
    fn stores_hi(&self, v: Var) -> Option<i64> {
        let stores = self.var_stores.get(&v)?;
        if self.tainted.contains(&v)
            || self.odd_stores.contains(&v)
            || self.stores(v) as usize != stores.len()
        {
            return None;
        }
        let mut hi = i64::MIN;
        for &(home, sop, rhs) in stores {
            match sop {
                StoreOp::Set => hi = hi.max(self.upper_bound(home, rhs, 1)?),
                StoreOp::Add => {
                    let before = self.guard_hi_at(at(rhs) + 1, 1)?;
                    hi = hi.max(before.checked_add(self.upper_bound(home, rhs, 1)?)?);
                }
                StoreOp::Rem | StoreOp::BitAnd if self.nonneg_vars.contains(&v) => {}
                _ => return None,
            }
        }
        Some(hi)
    }

    /// A variable only ever set to a value in `0..2^62` and stepped by one
    /// (`+= 1`, `-= 1`) stays more than `2^62` from either end of `Int`
    /// in any run: getting there takes `2^62` steps, centuries at any
    /// speed. So its step by one needs no overflow check (R09.3.4).
    pub fn unit_step(&self, v: Var) -> bool {
        self.unit_steps.contains(&v)
    }

    fn unit_stepped(&self, fn_id: Option<usize>, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Local(slot) => self.unit_step(self.local(fn_id, *slot)),
            ExprKind::Global(id) => self.unit_step((None, true, *id)),
            _ => false,
        }
    }

    /// The stores the body of one fn makes, on its own.
    pub fn of_fn(id: usize, body: &'p Block) -> Facts<'p> {
        let mut facts = Facts::default();
        facts.block(Some(id), body);
        facts
    }

    fn taint(&mut self, v: Var) {
        self.tainted.insert(v);
        self.odd_stores.insert(v);
    }

    fn store(&mut self, fn_id: Option<usize>, place: &Place) {
        if let Some(v) = var(fn_id, &place.kind) {
            *self.stores.entry(v).or_default() += 1;
        }
    }

    fn block(&mut self, fn_id: Option<usize>, block: &'p Block) {
        let mark = self.active.len();
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let(slot, init) => {
                    let v = (fn_id, false, *slot);
                    *self.stores.entry(v).or_default() += 1;
                    if let Some(init) = init {
                        self.init.insert(v, init);
                        self.var_stores
                            .entry(v)
                            .or_default()
                            .push((fn_id, StoreOp::Set, init));
                        self.expr(fn_id, init);
                    } else {
                        self.taint(v);
                    }
                    self.kill(v);
                }
                Stmt::Assign(place, op, value) => {
                    self.store(fn_id, place);
                    if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                        (&place.kind, var(fn_id, &place.kind))
                    {
                        self.init.entry(v).or_insert(value);
                        let unit = matches!(
                            (op, &value.kind),
                            (AssignOp::Set, _)
                                | (AssignOp::Binary(BinOp::Add | BinOp::Sub), ExprKind::Int(1))
                        );
                        if !unit {
                            self.odd_stores.insert(v);
                        }
                        let sop = match op {
                            AssignOp::Set => Some(StoreOp::Set),
                            AssignOp::Max => Some(StoreOp::KeepNonNeg),
                            AssignOp::Binary(BinOp::Add) => Some(StoreOp::Add),
                            AssignOp::Binary(
                                BinOp::Mul | BinOp::BitOr | BinOp::BitXor | BinOp::Div | BinOp::Shr,
                            ) => Some(StoreOp::KeepNonNeg),
                            AssignOp::Binary(BinOp::BitAnd) => Some(StoreOp::BitAnd),
                            AssignOp::Binary(BinOp::Rem) => Some(StoreOp::Rem),
                            _ => None,
                        };
                        if let Some(sop) = sop {
                            self.var_stores
                                .entry(v)
                                .or_default()
                                .push((fn_id, sop, value));
                        } else {
                            self.tainted.insert(v);
                        }
                    }
                    self.place(fn_id, place);
                    self.expr(fn_id, value);
                    // `v += value` reads `v` here too.
                    if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                        (&place.kind, var(fn_id, &place.kind))
                    {
                        for &g in self.active.iter().flatten().filter(|g| g.0 == v) {
                            self.leaf_log.push((at(value) + 1, g));
                        }
                    }
                    self.kill_place(fn_id, place);
                }
                Stmt::Return(Some(e)) => {
                    if let Some(id) = fn_id.filter(|&id| id < self.fns) {
                        self.returns.entry(id).or_default().push(e);
                    }
                    self.expr(fn_id, e);
                }
                Stmt::Expr(e) | Stmt::Give(e) => self.expr(fn_id, e),
                Stmt::If(cond, then, otherwise) => {
                    self.expr(fn_id, cond);
                    let (mark, stored) = (self.active.len(), self.store_log.len());
                    self.assume(fn_id, cond, true, stored);
                    self.block(fn_id, then);
                    self.active.truncate(mark);
                    self.assume(fn_id, cond, false, stored);
                    self.block(fn_id, otherwise);
                    self.active.truncate(mark);
                    // Past a branch that leaves, the other one's condition holds.
                    match (diverges(then), diverges(otherwise)) {
                        (true, false) => self.assume(fn_id, cond, false, stored),
                        (false, true) => self.assume(fn_id, cond, true, stored),
                        _ => {}
                    }
                }
                Stmt::While(_, cond, body) => {
                    let lap = self.lap();
                    self.expr(fn_id, cond);
                    let mark = self.active.len();
                    self.assume(fn_id, cond, true, self.store_log.len());
                    self.block(fn_id, body);
                    self.active.truncate(mark);
                    self.lapped(lap);
                }
                Stmt::For(_, slot, iter, body) => {
                    let v = (fn_id, false, *slot);
                    *self.stores.entry(v).or_default() += 1;
                    self.loop_iter.insert(v, iter);
                    self.taint(v);
                    self.kill(v);
                    match iter {
                        Iter::Walk { .. } => {
                            crate::fuse::iter_exprs(iter, &mut |e| self.expr(fn_id, e))
                        }
                        Iter::Range {
                            lo,
                            hi,
                            step,
                            closed,
                            ..
                        } => {
                            self.loop_lo.insert(v, lo);
                            self.loop_hi.insert(v, (hi, *closed));
                            self.expr(fn_id, lo);
                            self.expr(fn_id, hi);
                            if let Some(step) = step {
                                self.expr(fn_id, step);
                            }
                        }
                        Iter::RangeVal(e, rev) => {
                            if let (false, ExprKind::Range { lo, hi }) = (rev, &e.kind) {
                                self.loop_lo.insert(v, lo);
                                self.loop_hi.insert(v, (hi, false));
                            }
                            self.expr(fn_id, e);
                        }
                        Iter::Seq(e, _) => {
                            // A walk of a literal list of constants takes values between
                            // its least and greatest.
                            if let (Ty::List(_), ExprKind::Tuple(items)) = (&e.ty, &e.kind) {
                                let int = |x: &&'p Expr| match x.kind {
                                    ExprKind::Int(n) => Some(n),
                                    _ => None,
                                };
                                if items.iter().all(|x| int(&x).is_some()) {
                                    if let (Some(lo), Some(hi)) = (
                                        items.iter().min_by_key(|x| int(x)),
                                        items.iter().max_by_key(|x| int(x)),
                                    ) {
                                        self.loop_lo.insert(v, lo);
                                        self.loop_hi.insert(v, (hi, true));
                                    }
                                }
                            }
                            self.expr(fn_id, e);
                        }
                        Iter::TreeRange {
                            container, lo, hi, ..
                        } => {
                            self.expr(fn_id, container);
                            if let Some((e, _)) = lo {
                                self.expr(fn_id, e);
                            }
                            if let Some((e, _)) = hi {
                                self.expr(fn_id, e);
                            }
                        }
                    }
                    let lap = self.lap();
                    self.block(fn_id, body);
                    self.lapped(lap);
                }
                Stmt::Assert {
                    cond,
                    message,
                    kept,
                    ..
                } => {
                    self.expr(fn_id, cond);
                    if let Some(message) = message {
                        self.expr(fn_id, message);
                    }
                    // An assert `checks: none` drops proves nothing.
                    if *kept {
                        self.assume(fn_id, cond, true, self.store_log.len());
                    }
                }
                Stmt::DefDomain { id, lo, len, .. } => {
                    self.domain_len
                        .entry(*id)
                        .and_modify(|slot| *slot = None)
                        .or_insert(Some((fn_id, len)));
                    if fn_id.is_none() {
                        self.domain_lo
                            .entry(*id)
                            .and_modify(|slot| *slot = None)
                            .or_insert(Some((fn_id, lo)));
                    } else {
                        self.domain_lo.insert(*id, None);
                    }
                    self.expr(fn_id, lo);
                    self.expr(fn_id, len);
                }
                Stmt::DefDomainFromVal { id, val } => {
                    self.domain_len.insert(*id, None);
                    self.domain_lo.insert(*id, None);
                    self.expr(fn_id, val);
                }
                Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
            }
        }
        self.active.truncate(mark);
    }

    /// `cond` holds from here to the end of the scope, for the locals it
    /// compares that no store since `since` in the store log has changed.
    fn assume(&mut self, fn_id: Option<usize>, cond: &'p Expr, holds: bool, since: usize) {
        let mut out = Vec::new();
        cond_facts(fn_id, cond, holds, &mut out);
        for (v, fact) in out {
            if !(self.store_log[since..].iter()).any(|&s| s == v || field_base(v) == Some(s)) {
                self.made += 1;
                self.active.push(Some((v, fact, self.made)));
            }
        }
    }

    /// A store to `v`: the guards on it no longer hold.
    fn kill(&mut self, v: Var) {
        self.store_log.push(v);
        for g in &mut self.active {
            if let Some((gv, ..)) = *g {
                if gv == v || field_base(gv) == Some(v) {
                    self.store_log.push(gv);
                    *g = None;
                }
            }
        }
    }

    fn kill_place(&mut self, fn_id: Option<usize>, place: &Place) {
        if let Some(v) = var(fn_id, &place.kind) {
            self.kill(v);
        }
    }

    /// Where a loop body starts: the store log, the read log and the guards made.
    fn lap(&self) -> (usize, usize, usize) {
        (self.store_log.len(), self.leaf_log.len(), self.made)
    }

    /// The loop body that started at `lap` is walked: a guard made before
    /// it does not hold at a read in it of a local the body stores to,
    /// since the store may come first on the next lap.
    fn lapped(&mut self, (stores, reads, made): (usize, usize, usize)) {
        let stored: BTreeSet<Var> = self.store_log[stores..].iter().copied().collect();
        if stored.is_empty() {
            return;
        }
        let body = self.leaf_log.split_off(reads);
        (self.leaf_log).extend(
            body.into_iter()
                .filter(|&(_, (v, _, m))| m > made || !stored.contains(&v)),
        );
    }

    /// The reads' guards, once every body is walked. A local a closure
    /// captures by reference can change under a call, so none hold for it.
    fn settle_guards(&mut self, program: &Program) {
        let mut shared = BTreeSet::new();
        for (cid, c) in program.closures.iter().enumerate() {
            let inner = self.key(Home::Closure(cid));
            for cap in c.captures.iter().filter(|cap| cap.by_ref && !cap.is_global) {
                shared.insert((inner, false, cap.inner));
                for &home in self.closure_homes.get(&cid).into_iter().flatten() {
                    shared.insert((home, false, cap.outer));
                }
            }
        }
        // A global a fn stores to can change under any call.
        for (&v, stores) in &self.var_stores {
            if v.1 && stores.iter().any(|&(home, ..)| home.is_some()) {
                shared.insert(v);
            }
        }
        shared.extend(self.tainted.iter().filter(|v| v.1));
        for (read, (v, fact, _)) in std::mem::take(&mut self.leaf_log) {
            if !shared.contains(&v) && !field_base(v).is_some_and(|b| shared.contains(&b)) {
                self.guarded.entry(read).or_default().push((v, fact));
            }
        }
        self.active = Vec::new();
        self.store_log = Vec::new();
    }

    /// The least upper bound the guards in force give the read `e`.
    fn guard_hi(&self, e: &Expr, depth: u8) -> Option<i64> {
        self.guard_hi_at(at(e), depth)
    }

    /// As `guard_hi`, for the read at `read`; `at(value) + 1` is the read
    /// of `v` that `v += value` makes.
    fn guard_hi_at(&self, read: usize, depth: u8) -> Option<i64> {
        (self.guarded.get(&read)?.iter())
            .filter_map(|&(v, fact)| match fact {
                Fact::Le(b, k) => self.upper_bound(v.0, b, depth + 1)?.checked_add(k),
                Fact::Most(k) => Some(k),
                Fact::Ge(..) => None,
            })
            .min()
    }

    /// The greatest lower bound the guards in force give the read `e`.
    pub(crate) fn guard_lo(&self, e: &Expr, depth: u8) -> Option<i64> {
        (self.guarded.get(&at(e))?.iter())
            .filter_map(|&(v, fact)| match fact {
                Fact::Ge(b, k) => self.lower_bound(v.0, b, depth + 1)?.checked_add(k),
                Fact::Le(..) | Fact::Most(_) => None,
            })
            .max()
    }

    fn place(&mut self, fn_id: Option<usize>, place: &'p Place) {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(fn_id, base),
            PlaceKind::Index(base, indices) => {
                self.place(fn_id, base);
                for index in indices {
                    self.expr(fn_id, index);
                }
            }
        }
    }

    /// Finds the stores and blocks inside an expression.
    fn expr(&mut self, fn_id: Option<usize>, e: &'p Expr) {
        if let ExprKind::Closure(cid) = e.kind {
            self.closure_homes.entry(cid).or_default().insert(fn_id);
        }
        if let Some(v) = read_var(fn_id, e) {
            for &g in self.active.iter().flatten().filter(|g| g.0 == v) {
                self.leaf_log.push((at(e), g));
            }
        }
        match &e.kind {
            ExprKind::Do(block) => self.block(fn_id, block),
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), a, b) => {
                self.expr(fn_id, a);
                let mark = self.active.len();
                self.assume(fn_id, a, *op == BinOp::And, self.store_log.len());
                self.expr(fn_id, b);
                self.active.truncate(mark);
            }
            ExprKind::If(cond, then, otherwise) => {
                self.expr(fn_id, cond);
                let (mark, stored) = (self.active.len(), self.store_log.len());
                self.assume(fn_id, cond, true, stored);
                self.expr(fn_id, then);
                self.active.truncate(mark);
                self.assume(fn_id, cond, false, stored);
                self.expr(fn_id, otherwise);
                self.active.truncate(mark);
            }
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                if let ExprKind::CallExpr(callee, _) = &e.kind {
                    self.expr(fn_id, callee);
                }
                for arg in args {
                    match arg {
                        Arg::Value(value) => self.expr(fn_id, value),
                        Arg::Mut(place) => {
                            self.store(fn_id, place);
                            if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                                (&place.kind, var(fn_id, &place.kind))
                            {
                                self.taint(v);
                            }
                            self.place(fn_id, place);
                            self.kill_place(fn_id, place);
                        }
                    }
                }
                // A callee may reach a field through an alias of its base.
                for g in &mut self.active {
                    if let Some((gv, ..)) = *g {
                        if field_base(gv).is_some() {
                            self.store_log.push(gv);
                            *g = None;
                        }
                    }
                }
            }
            ExprKind::Mutate(_, place, args) => {
                self.store(fn_id, place);
                if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                    (&place.kind, var(fn_id, &place.kind))
                {
                    self.taint(v);
                }
                self.place(fn_id, place);
                for arg in args {
                    self.expr(fn_id, arg);
                }
                self.kill_place(fn_id, place);
            }
            ExprKind::Sort(place, sort_by, desc) => {
                self.store(fn_id, place);
                if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                    (&place.kind, var(fn_id, &place.kind))
                {
                    self.taint(v);
                }
                self.place(fn_id, place);
                if let Some(desc) = desc {
                    self.expr(fn_id, desc);
                }
                match sort_by {
                    cpl_ir::SortBy::Natural => {}
                    cpl_ir::SortBy::Key(slot, key) => {
                        let v = (fn_id, false, *slot);
                        *self.stores.entry(v).or_default() += 2;
                        self.taint(v);
                        self.expr(fn_id, key);
                    }
                    cpl_ir::SortBy::Fn { func, .. } => self.expr(fn_id, func),
                }
                self.kill_place(fn_id, place);
            }
            _ => crate::walk::children(e, &mut |child| self.expr(fn_id, child)),
        }
    }

    /// How many times the variable is stored to. A parameter starts at two.
    pub fn stores(&self, v: Var) -> u32 {
        self.stores.get(&v).copied().unwrap_or(0)
    }

    /// A static upper bound on `domain`'s length when provable from literals
    /// and declared input ranges (`let n: 5..=300 = read()`).
    pub fn domain_max_len(&self, id: usize) -> Option<i64> {
        self.domain_max_len_at(id, 0)
    }

    fn domain_max_len_at(&self, id: usize, depth: u8) -> Option<i64> {
        let (fn_id, len) = self.domain_len.get(&id).copied().flatten()?;
        self.upper_bound(fn_id, len, depth)
    }

    /// The most positions `domain` has: its length's bound, else below
    /// 2^31 when its positions are held in an `int`.
    fn domain_len_hi(&self, id: usize, depth: u8) -> Option<i64> {
        let known = self.domain_hi.get(&id).copied();
        known
            .or_else(|| self.domain_max_len_at(id, depth))
            .or_else(|| {
                let narrow = self.wide.as_ref().is_some_and(|w| !w.contains(&id));
                narrow.then_some(i64::from(i32::MAX))
            })
    }

    /// `domain`'s length expression, when it is defined once.
    pub fn domain_len_expr(&self, id: usize) -> Option<&'p Expr> {
        self.domain_len
            .get(&id)
            .copied()
            .flatten()
            .map(|(_, len)| len)
    }

    /// A constant lower bound for `domain` when every definition of it in the
    /// program is a single `DefDomain` whose `lo` resolves to an integer literal.
    pub fn const_domain_lo(&self, id: usize) -> Option<i64> {
        let (fn_id, lo) = self.domain_lo.get(&id).copied().flatten()?;
        self.const_int(fn_id, lo, 0)
    }

    /// `domain`'s count when every run gives it the same one, folded from its
    /// one definition: literals, variables stored once, other such domains,
    /// `+`, `-`, `*`, `max` and `min`.
    pub fn fixed_domain_len(&self, id: usize) -> Option<i64> {
        let (fn_id, len) = self.domain_len.get(&id).copied().flatten()?;
        self.exact(fn_id, len, 0).map(|n| n.max(0))
    }

    /// [`Self::fixed_domain_len`] when it fits a position: the C++ holds it as a constant.
    pub fn constexpr_domain_len(&self, id: usize) -> Option<i64> {
        self.fixed_domain_len(id)
            .filter(|n| *n <= i64::from(i32::MAX))
    }

    fn exact(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> Option<i64> {
        if depth > 8 {
            return None;
        }
        let var = |v: Var| (self.stores(v) == 1).then(|| self.init(v)).flatten();
        match &e.kind {
            ExprKind::Int(n) => Some(*n),
            ExprKind::Cast(Cast::CharToInt, c) => match c.kind {
                ExprKind::Char(c) => Some(i64::from(c)),
                _ => None,
            },
            ExprKind::Local(slot) => {
                let v = self.local(fn_id, *slot);
                self.exact(v.0, var(v)?, depth + 1)
            }
            ExprKind::Global(id) => self.exact(None, var((None, true, *id))?, depth + 1),
            ExprKind::DomainLen(d) => {
                let (f, len) = self.domain_len.get(d).copied().flatten()?;
                self.exact(f, len, depth + 1)
            }
            ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub | BinOp::Mul), l, r) => {
                let (l, r) = (
                    self.exact(fn_id, l, depth + 1)?,
                    self.exact(fn_id, r, depth + 1)?,
                );
                match op {
                    BinOp::Add => l.checked_add(r),
                    BinOp::Sub => l.checked_sub(r),
                    _ => l.checked_mul(r),
                }
            }
            ExprKind::Builtin(op @ (Builtin::Max | Builtin::Min), args) => {
                let vs: Vec<i64> = args
                    .iter()
                    .map(|a| self.exact(fn_id, a, depth + 1))
                    .collect::<Option<_>>()?;
                if *op == Builtin::Max {
                    vs.into_iter().max()
                } else {
                    vs.into_iter().min()
                }
            }
            _ => None,
        }
    }

    fn const_int(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> Option<i64> {
        if depth > 8 {
            return None;
        }
        match &e.kind {
            ExprKind::Int(n) => Some(*n),
            ExprKind::Local(slot) => {
                let v = self.local(fn_id, *slot);
                let init = self.init(v).filter(|_| self.stable(v))?;
                self.const_int(v.0, init, depth + 1)
            }
            ExprKind::Global(id) => {
                let v = (None, true, *id);
                let init = self.init(v).filter(|_| self.stable(v))?;
                self.const_int(None, init, depth + 1)
            }
            _ => None,
        }
    }

    pub(crate) fn upper_bound(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> Option<i64> {
        if depth > 8 {
            return None;
        }
        match &e.kind {
            ExprKind::Int(n) => Some(*n),
            ExprKind::Read(spec) => match spec.as_ref() {
                Spec::Int { hi: Some(hi), .. } => self.upper_bound(fn_id, hi, depth + 1),
                _ => None,
            },
            ExprKind::Local(slot) => {
                let own = self.local_hi(self.local(fn_id, *slot), depth);
                own.into_iter().chain(self.guard_hi(e, depth)).min()
            }
            ExprKind::Global(id) => {
                let v = (None, true, *id);
                let own = if self.stores(v) == 1 {
                    self.init
                        .get(&v)
                        .and_then(|init| self.upper_bound(None, init, depth + 1))
                } else {
                    self.var_hi.get(&v).copied()
                };
                own.into_iter().chain(self.guard_hi(e, depth)).min()
            }
            // A count held in an `int` is below 2^31.
            ExprKind::DomainLen(d) => self.domain_len_hi(*d, depth + 1),
            ExprKind::Builtin(Builtin::Len | Builtin::AxisLen, _) => Some(LEN_MAX),
            ExprKind::Field(base, idx) => (self.field_range(&base.ty, *idx).map(|r| r.1))
                .into_iter()
                .chain(
                    matches!(base.ty, Ty::Struct(sid) if self.u32_fields.contains(&(sid, *idx)))
                        .then_some(i64::from(u32::MAX)),
                )
                .chain(self.guard_hi(e, depth))
                .min(),
            ExprKind::Binary(BinOp::Mul, l, r)
                if self.sign_at(fn_id, l, u32::from(depth) * 2) != Sign::Unknown
                    && self.sign_at(fn_id, r, u32::from(depth) * 2) != Sign::Unknown =>
            {
                self.upper_bound(fn_id, l, depth + 1)?
                    .checked_mul(self.upper_bound(fn_id, r, depth + 1)?)
            }
            ExprKind::Binary(BinOp::Add, l, r) => self
                .upper_bound(fn_id, l, depth + 1)?
                .checked_add(self.upper_bound(fn_id, r, depth + 1)?),
            ExprKind::Binary(BinOp::Sub, l, r) => {
                let l_hi = self.upper_bound(fn_id, l, depth + 1)?;
                if let ExprKind::Int(c) = r.kind {
                    l_hi.checked_sub(c)
                } else if self.sign_at(fn_id, r, u32::from(depth) * 2) != Sign::Unknown {
                    Some(l_hi)
                } else {
                    None
                }
            }
            ExprKind::Builtin(op @ (Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul), a)
                if self.plain_wrap(fn_id, e).is_some() =>
            {
                let (l, r) = (&a[0], &a[1]);
                let l_hi = self.upper_bound(fn_id, l, depth + 1)?;
                match op {
                    Builtin::WrapAdd => l_hi.checked_add(self.upper_bound(fn_id, r, depth + 1)?),
                    Builtin::WrapMul => l_hi.checked_mul(self.upper_bound(fn_id, r, depth + 1)?),
                    _ => Some(l_hi),
                }
            }
            // Euclidean `l / r` with `r > 0` is at most `max(l, 0)`.
            ExprKind::Binary(BinOp::Div, l, r)
                if self.sign_at(fn_id, r, u32::from(depth) * 2) == Sign::Positive =>
            {
                let hi = self.upper_bound(fn_id, l, depth + 1)?.max(0);
                Some(match r.kind {
                    ExprKind::Int(d) => hi / d,
                    _ => hi,
                })
            }
            // Euclidean: below the divisor, whatever the dividend.
            ExprKind::Binary(BinOp::Rem, _, r)
                if self.sign_at(fn_id, r, u32::from(depth) * 2) == Sign::Positive =>
            {
                Some(self.upper_bound(fn_id, r, depth + 1)? - 1)
            }
            // At most either operand that is never negative.
            ExprKind::Binary(BinOp::BitAnd, l, r) => [l, r]
                .into_iter()
                .filter(|x| self.sign_at(fn_id, x, u32::from(depth) * 2) != Sign::Unknown)
                .filter_map(|x| self.upper_bound(fn_id, x, depth + 1))
                .min(),
            ExprKind::Binary(BinOp::Shr, l, r)
                if self.sign_at(fn_id, l, u32::from(depth) * 2) != Sign::Unknown =>
            {
                let hi = self.upper_bound(fn_id, l, depth + 1)?;
                Some(match r.kind {
                    ExprKind::Int(k @ 0..=63) => hi >> k,
                    _ => hi,
                })
            }
            ExprKind::Builtin(Builtin::Max, args) if !args.is_empty() => {
                let mut hi = i64::MIN;
                for arg in args {
                    hi = hi.max(self.upper_bound(fn_id, arg, depth + 1)?);
                }
                Some(hi)
            }
            ExprKind::If(_, then, otherwise) => Some(
                self.upper_bound(fn_id, then, depth + 1)?
                    .max(self.upper_bound(fn_id, otherwise, depth + 1)?),
            ),
            // A label is its ordinal plus the domain's first label.
            ExprKind::Cast(Cast::Label(d), p) => {
                let ord = match p.kind {
                    ExprKind::Cast(Cast::Trust, ref k) if matches!(k.kind, ExprKind::Int(_)) => {
                        self.upper_bound(fn_id, k, depth + 1)?
                    }
                    _ => self.domain_len_hi(*d, depth + 1)? - 1,
                };
                self.const_domain_lo(*d)?.checked_add(ord)
            }
            ExprKind::Cast(Cast::Ord, p) if matches!(p.ty, Ty::Pos(_)) => {
                let own = self.upper_bound(fn_id, p, depth + 1);
                let len = match p.ty {
                    Ty::Pos(d) => self.domain_len_hi(d, depth + 1).map(|n| n - 1),
                    _ => None,
                };
                own.into_iter().chain(len).min()
            }
            ExprKind::Cast(_, inner) => self.upper_bound(fn_id, inner, depth + 1),
            // The power of two at or above the argument's bound.
            ExprKind::Intrinsic(cpl_ir::Prim::BitCeil, a) if a.len() == 1 => {
                let hi = u64::try_from(self.upper_bound(fn_id, &a[0], depth + 1)?.max(1)).ok()?;
                i64::try_from(hi.checked_next_power_of_two()?).ok()
            }
            ExprKind::Call(id, _) => (self.returned(*id)?.iter())
                .map(|r| self.upper_bound(Some(*id), r, depth + 2))
                .try_fold(i64::MIN, |hi, r| Some(hi.max(r?))),
            _ => None,
        }
    }

    /// The values fn `id` returns, when it has a few: a call's result is
    /// one of them. Each is read in `id`'s own body, so a cycle of calls
    /// ends at the depth limit, never in a fixpoint.
    pub(crate) fn returned(&self, id: usize) -> Option<&[&'p Expr]> {
        self.returns
            .get(&id)
            .map(Vec::as_slice)
            .filter(|r| (1..=RETURNS_MAX).contains(&r.len()))
    }

    fn local_hi(&self, v: Var, depth: u8) -> Option<i64> {
        if let Some(Spec::Int { hi: Some(hi), .. }) = self.param_range(v) {
            return self.upper_bound(v.0, hi, depth + 1);
        }
        if let (1, Some(&(hi, closed))) = (self.stores(v), self.loop_hi.get(&v)) {
            let hi = self.upper_bound(v.0, hi, depth + 1)?;
            return if closed { Some(hi) } else { hi.checked_sub(1) };
        }
        if self.stores(v) == 1 {
            self.init
                .get(&v)
                .and_then(|init| self.upper_bound(v.0, init, depth + 1))
        } else {
            self.var_hi.get(&v).copied()
        }
    }

    /// The declared range of parameter `v`, while no store has changed it.
    pub(crate) fn param_range(&self, v: Var) -> Option<&'p Spec> {
        self.param_ranges
            .get(&v)
            .copied()
            .filter(|_| self.stores(v) == 2)
    }

    /// True when `e` can't be `Int.MAX`, so `i <= e` ends a loop that
    /// steps `i` by one: a length, a domain size, a bounded value, or
    /// less than one of those.
    /// True when `e` is `>= 0` and at most `u32::MAX`.
    /// Whether `e` is proven inside `lo..=hi`.
    pub fn proves_within(&self, fn_id: Option<usize>, e: &Expr, lo: i64, hi: i64) -> bool {
        let lo_ok = match self.sign(fn_id, e) {
            Sign::Positive => lo <= 1,
            Sign::NonNeg => lo <= 0,
            Sign::Unknown => false,
        };
        lo_ok && self.upper_bound(fn_id, e, 0).is_some_and(|h| h <= hi)
    }

    pub fn fits_u32(&self, fn_id: Option<usize>, e: &Expr) -> bool {
        self.sign(fn_id, e) != Sign::Unknown
            && self
                .upper_bound(fn_id, e, 0)
                .is_some_and(|hi| hi <= i64::from(u32::MAX))
    }

    /// True when `e` is proven in `0..n`.
    pub fn within(&self, fn_id: Option<usize>, e: &Expr, n: i64) -> bool {
        self.sign(fn_id, e) != Sign::Unknown
            && self.upper_bound(fn_id, e, 0).is_some_and(|hi| hi < n)
    }

    pub fn below_max(&self, fn_id: Option<usize>, e: &Expr) -> bool {
        self.below_max_at(fn_id, e, 0)
    }

    fn below_max_at(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> bool {
        if depth > 8 || e.ty != Ty::Int {
            return false;
        }
        if self
            .upper_bound(fn_id, e, depth)
            .is_some_and(|hi| hi < i64::MAX)
        {
            return true;
        }
        let set_once = |v: Var| {
            let home = v.0;
            self.stores(v) == 1
                && self
                    .init
                    .get(&v)
                    .is_some_and(|init| self.below_max_at(home, init, depth + 1))
        };
        match &e.kind {
            ExprKind::DomainLen(_)
            | ExprKind::Builtin(Builtin::Len | Builtin::AxisLen, _)
            | ExprKind::Cast(Cast::Ord, _) => true,
            ExprKind::Local(slot) => {
                let v = self.local(fn_id, *slot);
                set_once(v)
                    || matches!((self.stores(v), self.loop_hi.get(&v)), (1, Some(&(hi, closed)))
                        if !closed || self.below_max_at(v.0, hi, depth + 1))
            }
            ExprKind::Global(id) => set_once((None, true, *id)),
            ExprKind::Field(base, index) => {
                let v = match &base.kind {
                    ExprKind::Local(slot) => self.local(fn_id, *slot),
                    ExprKind::Global(id) => (None, true, *id),
                    _ => return false,
                };
                let home = v.0;
                self.stores(v) == 1
                    && matches!(self.init.get(&v).map(|init| &init.kind),
                        Some(ExprKind::Tuple(items)) if items
                            .get(*index)
                            .is_some_and(|item| self.below_max_at(home, item, depth + 1)))
            }
            ExprKind::Binary(BinOp::Sub, l, r) => {
                self.below_max_at(fn_id, l, depth + 1) && self.sign(fn_id, r) != Sign::Unknown
            }
            ExprKind::Builtin(Builtin::Min, args) => {
                args.iter().any(|a| self.below_max_at(fn_id, a, depth + 1))
            }
            ExprKind::If(_, a, b) => {
                self.below_max_at(fn_id, a, depth + 1) && self.below_max_at(fn_id, b, depth + 1)
            }
            _ => false,
        }
    }

    /// The declared range of field `idx` of a value of type `ty`.
    pub(crate) fn field_range(&self, ty: &Ty, idx: usize) -> Option<(i64, i64)> {
        match ty {
            Ty::Struct(sid) => self.field_ranges.get(&(*sid, idx)).copied(),
            _ => None,
        }
    }

    /// True when `e` is a position of a loop `for v in lo..hi` whose end
    /// `hi` is itself a position of the same domain, so `v + 1 <= hi` is a
    /// position too and `v.next` always exists.
    pub fn next_exists(&self, fn_id: Option<usize>, e: &Expr) -> bool {
        let (ExprKind::Local(slot), Ty::Pos(d)) = (&e.kind, &e.ty) else {
            return false;
        };
        let v = self.local(fn_id, *slot);
        let mut hi = match (self.stores(v), self.loop_hi.get(&v)) {
            (1, Some(&(hi, false))) => hi,
            _ => return false,
        };
        while let ExprKind::Cast(_, inner) = &hi.kind {
            hi = inner;
        }
        hi.ty == Ty::Pos(*d)
    }

    /// True when `e` is a position of a loop whose first value is past the
    /// domain's first position, so `v.prev` always exists.
    pub fn prev_exists(&self, fn_id: Option<usize>, e: &Expr) -> bool {
        let (ExprKind::Local(slot), Ty::Pos(_)) = (&e.kind, &e.ty) else {
            return false;
        };
        let v = self.local(fn_id, *slot);
        match (self.stores(v), self.loop_lo.get(&v)) {
            (1, Some(lo)) => matches!(self.sign_at(v.0, lo, 0), Sign::Positive),
            _ => false,
        }
    }

    /// A loop variable's first value.
    pub(crate) fn loop_lo(&self, v: Var) -> Option<&'p Expr> {
        self.loop_lo.get(&v).copied()
    }

    /// Whether `e` is the counter of a loop `for v in lo..n` with `lo >= 0`,
    /// so `0 <= v < n` wherever it is read.
    pub(crate) fn counts_below(&self, fn_id: Option<usize>, e: &Expr, n: &Expr) -> bool {
        let ExprKind::Local(slot) = &e.kind else {
            return false;
        };
        let v = self.local(fn_id, *slot);
        match (self.stores(v), self.loop_hi.get(&v), self.loop_lo.get(&v)) {
            (1, Some(&(hi, false)), Some(lo)) if v.0 == fn_id => {
                crate::span::same_count(hi, n)
                    && self.lower_bound(fn_id, lo, 0).is_some_and(|lo| lo >= 0)
            }
            _ => false,
        }
    }

    /// The value of the variable's first store.
    pub fn init(&self, v: Var) -> Option<&'p Expr> {
        self.init.get(&v).copied()
    }

    /// True when the variable is stored once, so its first value is its
    /// only value.
    /// The key a body's variables have: a fn's id, or past the fns for a
    /// closure, whose slots are its own and not its maker's.
    pub fn key(&self, home: Home) -> Option<usize> {
        match home {
            Home::Top => None,
            Home::Fn(id) => Some(id),
            Home::Closure(cid) => Some(self.fns + cid),
        }
    }

    /// Local `slot` of the body keyed `fn_id`; a by-value capture is the
    /// variable it copies, whose facts hold for the copy.
    pub fn local(&self, fn_id: Option<usize>, slot: usize) -> Var {
        let mut v = (fn_id, false, slot);
        while let Some(&outer) = self.captured.get(&v) {
            v = outer;
        }
        v
    }

    pub fn stable(&self, v: Var) -> bool {
        self.stores.get(&v).copied().unwrap_or(0) <= 1
    }

    /// The declared range of the value `e` reads, when `e` is a path into
    /// a variable that was read from input and never changed, or a copy of
    /// such a path.
    fn spec(&self, fn_id: Option<usize>, e: &Expr) -> Option<&'p Spec> {
        match &e.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => {
                let v = match &e.kind {
                    ExprKind::Local(slot) => self.local(fn_id, *slot),
                    ExprKind::Global(id) => (None, true, *id),
                    _ => return None,
                };
                if let Some(range) = self.param_ranges.get(&v) {
                    return Some(range);
                }
                if !self.stable(v) {
                    return None;
                }
                let init = self.init.get(&v)?;
                match &init.kind {
                    ExprKind::Read(spec) => Some(spec),
                    // `let (c, w) = item[v]`: a copy of a path into input.
                    ExprKind::Index(..) | ExprKind::Field(..) => self.spec(v.0, init),
                    _ => None,
                }
            }
            ExprKind::Index(base, _) => match self.spec(fn_id, base)? {
                Spec::Array { elem, .. } => Some(elem),
                _ => None,
            },
            ExprKind::Field(base, index) => match self.spec(fn_id, base)? {
                Spec::Fields(_, fields) => fields.get(*index),
                _ => None,
            },
            _ => None,
        }
    }

    /// What is known about the sign of the `Int` expression `e`, which
    /// appears in fn `fn_id`.
    /// The plain operator an `unchecked:` wrapping `+`, `-` or `*` computes
    /// when its operands are `>= 0` and bounded so that it can't wrap.
    pub fn plain_wrap(&self, fn_id: Option<usize>, e: &Expr) -> Option<BinOp> {
        let ExprKind::Builtin(op, a) = &e.kind else {
            return None;
        };
        let [l, r] = a.as_slice() else {
            return None;
        };
        let op = match op {
            Builtin::WrapAdd => BinOp::Add,
            Builtin::WrapSub => BinOp::Sub,
            Builtin::WrapMul => BinOp::Mul,
            _ => return None,
        };
        self.fits(fn_id, op, l, r).then_some(op)
    }

    /// Whether the `Int`s `l op r` (`+`, `-` or `*`) is proved to stay an
    /// `Int`: both sides are `>= 0` and their bounds keep it in range, so
    /// no build needs an overflow check (R09.3.4).
    pub fn fits(&self, fn_id: Option<usize>, op: BinOp, l: &Expr, r: &Expr) -> bool {
        let (a, b) = (self.const_int(fn_id, l, 0), self.const_int(fn_id, r, 0));
        let one_more = |c: Option<i64>, x: &Expr| {
            c == Some(1) && (self.below_max(fn_id, x) || self.unit_stepped(fn_id, x))
        };
        let trivial = match op {
            BinOp::Add => a == Some(0) || b == Some(0) || one_more(a, r) || one_more(b, l),
            BinOp::Sub => b == Some(0) || (b == Some(1) && self.unit_stepped(fn_id, l)),
            BinOp::Mul => matches!(a, Some(-1..=1)) || matches!(b, Some(-1..=1)),
            _ => false,
        };
        if trivial || self.fits_bounds(fn_id, op, l, r) {
            return true;
        }
        if self.sign(fn_id, l) == Sign::Unknown || self.sign(fn_id, r) == Sign::Unknown {
            return false;
        }
        let his = || {
            Some((
                self.upper_bound(fn_id, l, 1)?,
                self.upper_bound(fn_id, r, 1)?,
            ))
        };
        match op {
            BinOp::Add => his().and_then(|(x, y)| x.checked_add(y)).is_some(),
            BinOp::Mul => his().and_then(|(x, y)| x.checked_mul(y)).is_some(),
            BinOp::Shl => his().is_some_and(|(x, y)| (0..64).contains(&y) && x <= i64::MAX >> y),
            BinOp::Sub => true,
            _ => false,
        }
    }

    pub fn sign(&self, fn_id: Option<usize>, e: &Expr) -> Sign {
        self.sign_at(fn_id, e, 0)
    }

    pub(crate) fn sign_at(&self, fn_id: Option<usize>, e: &Expr, depth: u32) -> Sign {
        if depth > 16 || e.ty != Ty::Int {
            return Sign::Unknown;
        }
        let guarded = match e.kind {
            ExprKind::Local(_) | ExprKind::Global(_) | ExprKind::Field(..) => {
                self.guard_lo(e, depth as u8 + 1)
            }
            _ => None,
        };
        match (guarded, self.sign_own(fn_id, e, depth)) {
            (Some(lo), _) if lo > 0 => Sign::Positive,
            (Some(0), Sign::Unknown) => Sign::NonNeg,
            (_, own) => own,
        }
    }

    /// Which sides of `0..len` the ordinal `e` of domain `d` is known to
    /// keep: `(>= 0, < len)`. A position stepped up by an amount never
    /// negative stays `>= 0`; stepped down, `< len`.
    pub(crate) fn kept_sides(&self, fn_id: Option<usize>, e: &Expr, d: DomainId) -> (bool, bool) {
        let low = self.sign_at(fn_id, e, 0) != Sign::Unknown;
        let first = |z: &Expr| match &z.kind {
            ExprKind::Cast(Cast::Label(l), z) if *l == d => match &z.kind {
                ExprKind::Cast(Cast::Trust, z) => matches!(z.kind, ExprKind::Int(0)),
                k => matches!(k, ExprKind::Int(0)),
            },
            _ => false,
        };
        let (base, labels) = match &e.kind {
            ExprKind::Binary(BinOp::Sub, a, z) if first(z) => (&**a, true),
            _ => (e, false),
        };
        let pos = |p: &Expr| match &p.kind {
            ExprKind::Cast(Cast::Ord, q) => q.ty == Ty::Pos(d),
            _ => p.ty == Ty::Pos(d),
        };
        let at = |p: &Expr| match &p.kind {
            ExprKind::Cast(Cast::Label(l), p) => labels && *l == d && pos(p),
            ExprKind::Cast(Cast::Ord, p) => !labels && p.ty == Ty::Pos(d),
            _ => false,
        };
        let known = |k: &Expr| self.sign_at(fn_id, k, 0) != Sign::Unknown;
        match &base.kind {
            _ if at(base) => (true, true),
            ExprKind::Binary(BinOp::Add, p, k) if at(p) && known(k) => (true, false),
            ExprKind::Binary(BinOp::Sub, p, k) if at(p) && known(k) => (low, true),
            _ => (low, false),
        }
    }

    fn sign_own(&self, fn_id: Option<usize>, e: &Expr, depth: u32) -> Sign {
        use Sign::*;
        let of = |e: &Expr| self.sign_at(fn_id, e, depth + 1);
        let both = |a: Sign, b: Sign| match (a, b) {
            (Unknown, _) | (_, Unknown) => Unknown,
            (Positive, Positive) => Positive,
            _ => NonNeg,
        };
        let literal = |v: i64| match v {
            v if v > 0 => Positive,
            0 => NonNeg,
            _ => Unknown,
        };
        match &e.kind {
            ExprKind::Int(v) => literal(*v),
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Index(..)
            | ExprKind::Field(..) => {
                if let Some(Spec::Int { lo: Some(lo), .. }) = self.spec(fn_id, e) {
                    if let ExprKind::Int(lo) = lo.kind {
                        return literal(lo);
                    }
                }
                if let ExprKind::Field(base, idx) = &e.kind {
                    if let Ty::Struct(sid) = base.ty {
                        if self.u32_fields.contains(&(sid, *idx)) {
                            return NonNeg;
                        }
                    }
                }
                let v = match &e.kind {
                    ExprKind::Local(slot) => self.local(fn_id, *slot),
                    ExprKind::Global(id) => (None, true, *id),
                    _ => return Unknown,
                };
                if !self.stable(v) {
                    return if self.nonneg_vars.contains(&v) {
                        NonNeg
                    } else {
                        Unknown
                    };
                }
                // A global is set by top-level code, a local by its own body.
                let home = v.0;
                if let Some(lo) = self.loop_lo.get(&v) {
                    return self.sign_at(home, lo, depth + 1);
                }
                match self.init.get(&v) {
                    Some(init) => self.sign_at(home, init, depth + 1),
                    None => Unknown,
                }
            }
            ExprKind::Binary(op, l, r) => match op {
                BinOp::Add => match (of(l), of(r)) {
                    (Unknown, _) | (_, Unknown) => Unknown,
                    (NonNeg, NonNeg) => NonNeg,
                    _ => Positive,
                },
                BinOp::Mul | BinOp::Pow => both(of(l), of(r)),
                BinOp::Sub => {
                    let d = depth as u8 + 1;
                    let lo = self.lower_bound(fn_id, l, d);
                    let hi = self.upper_bound(fn_id, r, d);
                    match lo.zip(hi).and_then(|(a, b)| a.checked_sub(b)) {
                        Some(x) if x > 0 => Positive,
                        Some(0) => NonNeg,
                        _ => Unknown,
                    }
                }
                // Euclidean: never negative, whatever the operands.
                BinOp::Rem => NonNeg,
                BinOp::Div | BinOp::Shr | BinOp::BitOr | BinOp::BitXor => {
                    match both(of(l), of(r)) {
                        Unknown => Unknown,
                        _ => NonNeg,
                    }
                }
                BinOp::BitAnd => match (of(l), of(r)) {
                    (Unknown, Unknown) => Unknown,
                    _ => NonNeg,
                },
                // The interpreter rejects a shift that overflows.
                BinOp::Shl => of(l),
                _ => Unknown,
            },
            ExprKind::Builtin(Builtin::Len | Builtin::AxisLen, _) => NonNeg,
            ExprKind::Intrinsic(cpl_ir::Prim::Abs, _) => NonNeg,
            ExprKind::Intrinsic(cpl_ir::Prim::BitCeil, _) => Positive,
            ExprKind::Builtin(Builtin::WrapAdd | Builtin::WrapMul, a) => {
                match self.plain_wrap(fn_id, e) {
                    Some(BinOp::Add) => match (of(&a[0]), of(&a[1])) {
                        (NonNeg, NonNeg) => NonNeg,
                        _ => Positive,
                    },
                    Some(_) => both(of(&a[0]), of(&a[1])),
                    None => Unknown,
                }
            }
            ExprKind::Builtin(Builtin::Min, args) => {
                args.iter().map(of).reduce(both).unwrap_or(Unknown)
            }
            ExprKind::Builtin(Builtin::Max, args) => {
                let signs: Vec<Sign> = args.iter().map(of).collect();
                if signs.contains(&Positive) {
                    Positive
                } else if signs.contains(&NonNeg) {
                    NonNeg
                } else {
                    Unknown
                }
            }
            // A count is positive when its one definition is.
            ExprKind::DomainLen(d) => match self.domain_len.get(d).copied().flatten() {
                Some((home, len)) if self.sign_at(home, len, depth + 1) == Positive => Positive,
                _ => NonNeg,
            },
            ExprKind::Cast(Cast::CharToInt | Cast::Ord, _) => NonNeg,
            ExprKind::Cast(Cast::Label(d), _) => self.const_domain_lo(*d).map_or(Unknown, literal),
            ExprKind::If(_, a, b) => match both(of(a), of(b)) {
                Unknown => Unknown,
                sign => sign,
            },
            ExprKind::Call(id, _) => match self.returned(*id) {
                Some(rets) => (rets.iter())
                    .map(|r| self.sign_at(Some(*id), r, depth + 4))
                    .reduce(both)
                    .unwrap_or(Unknown),
                None => Unknown,
            },
            _ => Unknown,
        }
    }
}
