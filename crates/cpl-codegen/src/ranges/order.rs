//! Order between two integer reads, for a check that can never fail. In the
//! body of a loop over `lo..hi` its variable is `lo <= p < hi`; a `let`
//! stored once stands for its value; an element of a list one `push` builds
//! stands for what is pushed. Each side is a sum of such reads with constant
//! counts, and a loop variable is replaced by its own loop's bound: linear,
//! with no solver.
//!
//! `r.drop(k)` lowers to `lo + min(room, k)..hi` with `room >= hi - lo`;
//! inside its loop `p >= lo + k`, since were the clamp taken the range would
//! be empty and the body would not run. `drop_last` is the same from `hi`.
//!
//! The reads inside one pushed element share that push's values, so they
//! are told apart from every other element's by the loop over the list.

use super::{Facts, Var};
use cpl_ir::{BinOp, Block, Builtin, CmpOp, Expr, ExprKind, Iter, Program, Stmt};
use std::collections::BTreeMap;

const DEPTH: u8 = 32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Part {
    Value,
    Lo,
    Hi,
}

/// The element a read is in (the loop over the list), when it is in one.
type Tag = Option<Var>;

/// A sum of reads, each counted, plus a constant.
#[derive(Clone, PartialEq, Default, Debug)]
struct Lin {
    terms: BTreeMap<(Tag, Var, Part), i64>,
    k: i64,
}

impl Lin {
    fn k(k: i64) -> Lin {
        Lin {
            terms: BTreeMap::new(),
            k,
        }
    }

    fn atom(tag: Tag, v: Var, part: Part) -> Lin {
        Lin {
            terms: BTreeMap::from([((tag, v, part), 1)]),
            k: 0,
        }
    }

    /// `self + sign * other`.
    fn plus(mut self, other: &Lin, sign: i64) -> Option<Lin> {
        self.k = self.k.checked_add(other.k.checked_mul(sign)?)?;
        for (atom, c) in &other.terms {
            let t = self.terms.entry(*atom).or_insert(0);
            *t = t.checked_add(c.checked_mul(sign)?)?;
        }
        self.terms.retain(|_, c| *c != 0);
        Some(self)
    }

    fn constant(&self) -> Option<i64> {
        self.terms.is_empty().then_some(self.k)
    }
}

/// The order facts of one body's reads ([`Facts::order`]).
pub(crate) struct Order<'a, 'p> {
    facts: &'a Facts<'p>,
    program: &'p Program,
    home: Option<usize>,
}

impl<'p> Facts<'p> {
    /// The order facts for reads in the body keyed `home` ([`Facts::key`]).
    pub(crate) fn order<'a>(&'a self, program: &'p Program, home: Option<usize>) -> Order<'a, 'p> {
        Order {
            facts: self,
            program,
            home,
        }
    }
}

/// The value a `do` block gives.
fn given(block: &Block) -> Option<&Expr> {
    match block.stmts.last() {
        Some(Stmt::Give(e)) => Some(e),
        _ => None,
    }
}

impl<'p> Order<'_, 'p> {
    /// Whether the comparison `cond` is false wherever it is read.
    pub(crate) fn never(&self, cond: &Expr) -> bool {
        let ExprKind::Cmp(op, a, b) = &cond.kind else {
            return false;
        };
        match op {
            CmpOp::Le => self.apart(b, a, 1),
            CmpOp::Lt => self.apart(b, a, 0),
            CmpOp::Ge => self.apart(a, b, 1),
            CmpOp::Gt => self.apart(a, b, 0),
            _ => false,
        }
    }

    /// Whether `hi - lo >= gap` wherever the two are read.
    fn apart(&self, lo: &Expr, hi: &Expr, gap: i64) -> bool {
        let d = self
            .value(hi, None, 0)
            .zip(self.value(lo, None, 0))
            .and_then(|(h, l)| h.plus(&l, -1));
        let Some(mut d) = d else {
            return false;
        };
        for _ in 0..16 {
            if let Some(k) = d.constant() {
                return k >= gap;
            }
            // Bound the innermost loop variable first: one no other bound
            // mentions, so the walk ends instead of trading one for another.
            let steps: Vec<_> = d
                .terms
                .iter()
                .filter(|(&(_, _, part), _)| part == Part::Value)
                .filter_map(|(&(tag, v, _), &c)| {
                    let bound = if c > 0 {
                        self.lower(v, tag)
                    } else {
                        self.upper(v, tag)
                    }?;
                    Some((tag, v, c, bound))
                })
                .collect();
            let step = steps.iter().find(|&&(tag, v, _, _)| {
                steps
                    .iter()
                    .all(|(_, _, _, b)| !b.terms.keys().any(|&(t, u, _)| t == tag && u == v))
            });
            let step = step.cloned();
            let Some((tag, v, c, bound)) = step else {
                return false;
            };
            let next = d
                .plus(&Lin::atom(tag, v, Part::Value), -c)
                .and_then(|d| d.plus(&bound, c));
            let Some(next) = next else {
                return false;
            };
            d = next;
        }
        false
    }

    fn var(&self, slot: usize) -> Var {
        self.facts.local(self.home, slot)
    }

    /// Whether `v` holds one value wherever it is read: stored once, or a
    /// parameter never stored to.
    fn fixed(&self, v: Var) -> bool {
        let param = matches!(v, (Some(id), false, slot)
            if id < self.facts.fns && slot < self.program.fns[id].params.len());
        let stores = self.facts.stores(v);
        stores <= 1 || (param && stores == 2)
    }

    fn init(&self, v: Var) -> Option<&'p Expr> {
        self.facts.init(v).filter(|_| self.facts.stable(v))
    }

    /// `e` with each `let` stored once read through to its value.
    fn resolved<'e>(&self, mut e: &'e Expr) -> &'e Expr
    where
        'p: 'e,
    {
        for _ in 0..DEPTH {
            e = match &e.kind {
                ExprKind::Local(s) => match self.init(self.var(*s)) {
                    Some(init) => init,
                    None => return e,
                },
                ExprKind::Do(block) => match given(block) {
                    Some(g) => g,
                    None => return e,
                },
                _ => return e,
            };
        }
        e
    }

    fn value(&self, e: &Expr, tag: Tag, d: u8) -> Option<Lin> {
        let d = d.checked_add(1).filter(|d| *d < DEPTH)?;
        match &e.kind {
            ExprKind::Int(n) => Some(Lin::k(*n)),
            ExprKind::Local(s) => {
                let v = self.var(*s);
                if let Some(l) = self.init(v).and_then(|i| self.value(i, tag, d)) {
                    return Some(l);
                }
                self.fixed(v).then(|| Lin::atom(tag, v, Part::Value))
            }
            ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub), a, b) => {
                let sign = if *op == BinOp::Add { 1 } else { -1 };
                self.value(a, tag, d)?.plus(&self.value(b, tag, d)?, sign)
            }
            ExprKind::Builtin(Builtin::RangeLo, args) if args.len() == 1 => {
                self.end(&args[0], Part::Lo, tag, d)
            }
            ExprKind::Builtin(Builtin::RangeHi, args) if args.len() == 1 => {
                self.end(&args[0], Part::Hi, tag, d)
            }
            ExprKind::Builtin(which @ (Builtin::Min | Builtin::Max), args) if args.len() == 2 => {
                let x = self.value(&args[0], tag, d)?.constant()?;
                let y = self.value(&args[1], tag, d)?.constant()?;
                Some(Lin::k(if *which == Builtin::Min {
                    x.min(y)
                } else {
                    x.max(y)
                }))
            }
            ExprKind::Do(block) => self.value(given(block)?, tag, d),
            ExprKind::Cast(cpl_ir::Cast::Ord | cpl_ir::Cast::Trust, x) => self.value(x, tag, d),
            ExprKind::Field(base, i) => {
                let (e, tag) = self.part(base, *i, tag, d)?;
                self.value(e, tag, d)
            }
            _ => None,
        }
    }

    /// An end of the range `r`.
    fn end(&self, r: &Expr, part: Part, tag: Tag, d: u8) -> Option<Lin> {
        let d = d.checked_add(1).filter(|d| *d < DEPTH)?;
        match &r.kind {
            ExprKind::Range { lo, hi } => {
                self.value(if part == Part::Lo { lo } else { hi }, tag, d)
            }
            ExprKind::Local(s) => {
                let v = self.var(*s);
                if let Some(l) = self.init(v).and_then(|i| self.end(i, part, tag, d)) {
                    return Some(l);
                }
                if let Some((e, tag)) = self.element(v, tag) {
                    return self.end(e, part, tag, d);
                }
                self.fixed(v).then(|| Lin::atom(tag, v, part))
            }
            ExprKind::Do(block) => self.end(given(block)?, part, tag, d),
            ExprKind::Field(base, i) => {
                let (e, tag) = self.part(base, *i, tag, d)?;
                self.end(e, part, tag, d)
            }
            _ => None,
        }
    }

    /// Part `i` of the tuple `base`, and the element it is in.
    fn part<'e>(&self, base: &'e Expr, i: usize, tag: Tag, d: u8) -> Option<(&'e Expr, Tag)>
    where
        'p: 'e,
    {
        let d = d.checked_add(1).filter(|d| *d < DEPTH)?;
        match &base.kind {
            ExprKind::Tuple(items) => items.get(i).map(|e| (e, tag)),
            ExprKind::Local(s) => {
                let v = self.var(*s);
                if let Some(init) = self.init(v) {
                    return self.part(init, i, tag, d);
                }
                let (e, tag) = self.element(v, tag)?;
                self.part(e, i, tag, d)
            }
            ExprKind::Do(block) => self.part(given(block)?, i, tag, d),
            _ => None,
        }
    }

    /// What the one `push` of the list the loop variable `v` walks pushes.
    /// Only one element deep: an element of an element is not told apart.
    fn element(&self, v: Var, tag: Tag) -> Option<(&'p Expr, Tag)> {
        if tag.is_some() {
            return None;
        }
        let Iter::Seq(seq, false) = self.facts.loop_iter.get(&v)? else {
            return None;
        };
        let mut seq = seq;
        for _ in 0..DEPTH {
            match &seq.kind {
                ExprKind::Local(s) => seq = self.init(self.var(*s))?,
                _ => break,
            }
        }
        let ExprKind::Do(block) = &seq.kind else {
            return None;
        };
        // A spliced fn's body binds its parameters first.
        let at = block.stmts.len().checked_sub(3)?;
        let (params, tail) = block.stmts.split_at(at);
        if !params.iter().all(|s| matches!(s, Stmt::Let(..))) {
            return None;
        }
        let p = crate::fuse::stmts_producer(tail)?;
        Some((crate::uses::pushed_value(p.body, *p.vec)?, Some(v)))
    }

    /// The two ends of the range the loop variable `v` walks forward, by
    /// one, with the high end left out.
    fn walked(&self, v: Var) -> Option<(&'p Expr, &'p Expr, bool)> {
        match self.facts.loop_iter.get(&v)? {
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed,
            } => Some((lo, hi, *closed)),
            Iter::RangeVal(e, false) => match &self.resolved(e).kind {
                ExprKind::Range { lo, hi } => Some((lo, hi, false)),
                _ => None,
            },
            _ => None,
        }
    }

    /// The least value of the loop variable `v` in its body.
    fn lower(&self, v: Var, tag: Tag) -> Option<Lin> {
        let (lo, hi, closed) = self.walked(v)?;
        // `a + min(room, k)` with `room >= past - a`: `p >= a + k`.
        if let ExprKind::Binary(BinOp::Add, a, cut) = &self.resolved(lo).kind {
            let past = self
                .value(hi, tag, 0)
                .and_then(|h| h.plus(&Lin::k(i64::from(closed)), 1));
            if let Some(k) = past.and_then(|past| self.clamped(cut, a, &past, 1, tag)) {
                return self.value(a, tag, 0)?.plus(&k, 1);
            }
        }
        self.value(lo, tag, 0)
    }

    /// The greatest value of the loop variable `v` in its body.
    fn upper(&self, v: Var, tag: Tag) -> Option<Lin> {
        let (lo, hi, closed) = self.walked(v)?;
        let last = i64::from(closed) - 1;
        // `b - min(room, k)` with `room >= b - first`: `p < b - k`.
        if let ExprKind::Binary(BinOp::Sub, b, cut) = &self.resolved(hi).kind {
            let first = self.value(lo, tag, 0);
            if let Some(k) = first.and_then(|first| self.clamped(cut, b, &first, -1, tag)) {
                return self.value(b, tag, 0)?.plus(&k, -1)?.plus(&Lin::k(last), 1);
            }
        }
        self.value(hi, tag, 0)?.plus(&Lin::k(last), 1)
    }

    /// `k` of a cut `min(room, k)` from `from` toward `to` (`sign` is the
    /// way), when `room` covers the whole way, so the clamp leaves nothing.
    fn clamped(&self, cut: &Expr, from: &Expr, to: &Lin, sign: i64, tag: Tag) -> Option<Lin> {
        let ExprKind::Builtin(Builtin::Min, args) = &self.resolved(cut).kind else {
            return None;
        };
        let [room, k] = args.as_slice() else {
            return None;
        };
        let way = to.clone().plus(&self.value(from, tag, 0)?, -1)?;
        let way = Lin::k(0).plus(&way, sign)?;
        let spare = self.room(room, tag)?.plus(&way, -1)?.constant()?;
        (spare >= 0).then(|| self.value(k, tag, 0)).flatten()
    }

    /// A lower bound of `x`, a range's room: `max(hi - lo, 0)`, or the
    /// saturating length `if lo >= hi: 0 else ...` the lowering writes.
    fn room(&self, x: &Expr, tag: Tag) -> Option<Lin> {
        let x = self.resolved(x);
        match &x.kind {
            ExprKind::Builtin(Builtin::Max, args) if args.len() == 2 => self
                .value(&args[0], tag, 0)
                .or_else(|| self.value(&args[1], tag, 0)),
            ExprKind::If(c, t, f) => match (&c.kind, &self.resolved(t).kind) {
                // `0` where `l >= h`, which is at least `h - l` there.
                (ExprKind::Cmp(CmpOp::Ge, l, h), ExprKind::Int(0)) => {
                    let gap = self.value(h, tag, 0)?.plus(&self.value(l, tag, 0)?, -1)?;
                    (self.room(f, tag)? == gap).then_some(gap)
                }
                // A wrapped difference is replaced by the most an `Int` holds.
                (ExprKind::Cmp(CmpOp::Lt, w, z), ExprKind::Int(i64::MAX))
                    if matches!(z.kind, ExprKind::Int(0)) && w == f =>
                {
                    match &self.resolved(f).kind {
                        ExprKind::Builtin(Builtin::WrapSub, args) if args.len() == 2 => self
                            .value(&args[0], tag, 0)?
                            .plus(&self.value(&args[1], tag, 0)?, -1),
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => self.value(x, tag, 0),
        }
    }
}
