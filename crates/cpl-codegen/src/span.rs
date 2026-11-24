//! Bounds from below, and where a value lies against a count it is taken
//! modulo: one rule decides how a Euclidean `a % n` is reduced (R09.3.4).
//!
//! A bound relative to a positive count `n` is `k * n + c`. `Rem.wrap(r.ord
//! + 1)` has `r.ord` in `[0, n - 1]`, so the sum is in `[1, n]`: below
//! `2 * n`, one conditional subtract is the whole remainder.

use crate::ranges::{Facts, Sign};
use cpl_ir::{BinOp, Cast, Expr, ExprKind, Spec, Ty};

/// `k * n + c` for the count `n`.
type Lin = (i64, i64);

/// How `a % n` reduces once `a`'s bounds against `n` are known.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Reduce {
    /// `a` is in `[0, n)`: `a` itself.
    Keep,
    /// `a` is in `[n, 2n)`: `a - n`.
    SubOnce,
    /// `a` is in `[0, 2n)`: `a - n` when `a >= n`.
    SubIf,
    /// `a` is in `[-n, n)`: `a + n` when `a < 0`.
    AddIf,
}

/// Whether `a` and `b` read the same count.
pub(crate) fn same_count(a: &Expr, b: &Expr) -> bool {
    crate::stmt::same_path(a, b)
}

impl Facts<'_> {
    /// A constant `x` with `x <= e` in every run, mirroring `upper_bound`.
    pub(crate) fn lower_bound(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> Option<i64> {
        if depth > 8 || e.ty != Ty::Int {
            return None;
        }
        let at = |x: &Expr| self.lower_bound(fn_id, x, depth + 1);
        let once = |v: crate::ranges::Var| {
            if self.stores(v) != 1 {
                return None;
            }
            let from = self.loop_lo(v).or_else(|| self.init(v))?;
            self.lower_bound(v.0, from, depth + 1)
        };
        let known = match &e.kind {
            ExprKind::Int(n) => Some(*n),
            ExprKind::Read(spec) => match spec.as_ref() {
                Spec::Int { lo: Some(lo), .. } => at(lo),
                _ => None,
            },
            ExprKind::Local(slot) => once(self.local(fn_id, *slot)),
            ExprKind::Global(id) => once((None, true, *id)),
            ExprKind::Binary(BinOp::Add, l, r) => at(l)?.checked_add(at(r)?),
            ExprKind::Binary(BinOp::Sub, l, r) => {
                at(l)?.checked_sub(self.upper_bound(fn_id, r, depth + 1)?)
            }
            ExprKind::If(_, a, b) => Some(at(a)?.min(at(b)?)),
            ExprKind::Cast(Cast::Label(d), p) => {
                let ord = match &p.kind {
                    ExprKind::Cast(Cast::Trust, k) => match k.kind {
                        ExprKind::Int(k) => k,
                        _ => 0,
                    },
                    _ => 0,
                };
                self.const_domain_lo(*d)?.checked_add(ord)
            }
            _ => None,
        };
        let sign = match self.sign_at(fn_id, e, u32::from(depth) * 2 + 1) {
            Sign::Positive => Some(1),
            Sign::NonNeg => Some(0),
            Sign::Unknown => None,
        };
        match (known, sign) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    /// Whether `l op r` stays inside `Int` for every pair of values the
    /// operands' constant bounds allow, whatever their signs.
    pub(crate) fn fits_bounds(&self, fn_id: Option<usize>, op: BinOp, l: &Expr, r: &Expr) -> bool {
        let bounds = |e: &Expr| {
            Some((
                i128::from(self.lower_bound(fn_id, e, 1)?),
                i128::from(self.upper_bound(fn_id, e, 1)?),
            ))
        };
        let (Some((a, b)), Some((c, d))) = (bounds(l), bounds(r)) else {
            return false;
        };
        let corners = match op {
            BinOp::Add => [a + c, b + d, a + c, b + d],
            BinOp::Sub => [a - d, b - c, a - d, b - c],
            BinOp::Mul => [a * c, a * d, b * c, b * d],
            _ => return false,
        };
        let int = i128::from(i64::MIN) + 1..=i128::from(i64::MAX);
        corners.iter().all(|x| int.contains(x))
    }

    /// `e`'s bounds as `k * n + c`, for a count `n` taken to be positive.
    fn span_in(&self, fn_id: Option<usize>, e: &Expr, n: &Expr, depth: u8) -> Option<(Lin, Lin)> {
        if depth > 8 {
            return None;
        }
        if same_count(e, n) {
            return Some(((1, 0), (1, 0)));
        }
        let span = |x: &Expr| self.span_in(fn_id, x, n, depth + 1);
        let add = |x: Lin, y: Lin| Some((x.0.checked_add(y.0)?, x.1.checked_add(y.1)?));
        let sub = |x: Lin, y: Lin| Some((x.0.checked_sub(y.0)?, x.1.checked_sub(y.1)?));
        let rel = match &e.kind {
            ExprKind::Binary(BinOp::Rem, _, m) if same_count(m, n) => Some(((0, 0), (1, -1))),
            // A position's ordinal is below its domain's count.
            ExprKind::Cast(Cast::Ord, p) => match (&p.ty, &n.kind) {
                (Ty::Pos(d), ExprKind::DomainLen(m)) if d == m => Some(((0, 0), (1, -1))),
                _ => None,
            },
            ExprKind::Binary(BinOp::Add, a, b) => {
                let (x, y) = (span(a)?, span(b)?);
                Some((add(x.0, y.0)?, add(x.1, y.1)?))
            }
            ExprKind::Binary(BinOp::Sub, a, b) => {
                let (x, y) = (span(a)?, span(b)?);
                Some((sub(x.0, y.1)?, sub(x.1, y.0)?))
            }
            _ => None,
        };
        rel.or_else(|| {
            let lo = self.lower_bound(fn_id, e, depth)?;
            let hi = self.upper_bound(fn_id, e, depth)?;
            Some(((0, lo), (0, hi)))
        })
    }

    /// How the Euclidean `a % n` reduces, when `n > 0` and `a`'s bounds
    /// against `n` leave less than a whole extra lap: `None` divides.
    pub(crate) fn reduce(&self, fn_id: Option<usize>, a: &Expr, n: &Expr) -> Option<Reduce> {
        if self.sign(fn_id, n) != Sign::Positive {
            return None;
        }
        let n_lo = i128::from(self.lower_bound(fn_id, n, 0).unwrap_or(1).max(1));
        let n_hi = self.upper_bound(fn_id, n, 0).map(i128::from);
        let (lo, hi) = self.span_in(fn_id, a, n, 0)?;
        // `x(n) >= y(n)` for every count, both linear in `n`.
        let ge = |x: Lin, y: (i128, i128)| {
            let k = i128::from(x.0) - y.0;
            let c = i128::from(x.1) - y.1;
            k * n_lo + c >= 0 && n_hi.map_or(k >= 0, |hi| k * hi + c >= 0)
        };
        let le = |x: Lin, y: (i128, i128)| ge((-x.0, -x.1), (-y.0, -y.1));
        let within = |from: (i128, i128), below: (i128, i128)| ge(lo, from) && le(hi, below);
        [
            (Reduce::Keep, (0, 0), (1, -1)),
            (Reduce::SubOnce, (1, 0), (2, -1)),
            (Reduce::SubIf, (0, 0), (2, -1)),
            (Reduce::AddIf, (-1, 0), (1, -1)),
        ]
        .into_iter()
        .find(|&(_, from, below)| within(from, below))
        .map(|(r, ..)| r)
    }
}
