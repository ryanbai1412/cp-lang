//! Operators whose declared operands can make them trap: a `/` or `%` by a
//! divisor that can be 0, a `<<` that can leave `Int`, a `for` range that
//! starts above its end and so never runs, a `!` on a slice that can be
//! empty.

use super::{bounds, Proven};
use crate::walk::unparen;
use cpl_diag::Span;
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

pub(super) fn hazards(
    base: u32,
    op: BinOp,
    op_span: Span,
    lhs: &Expr,
    rhs: &Expr,
    bounds: &bounds::Bounds,
    out: &mut Proven,
) {
    if op == BinOp::Shl {
        let declared = match &crate::walk::unparen(rhs).kind {
            ExprKind::Name(n) => bounds.is_input(n),
            _ => false,
        };
        if let (true, Some(x), Some(k)) = (declared, bounds.expr(lhs), bounds.expr(rhs)) {
            // The most `k` with `x.hi << k` still in `Int`.
            let safe = (0..=62i128)
                .take_while(|i| x.hi << i <= i64::MAX as i128)
                .last();
            if let Some(safe) =
                safe.filter(|safe| x.lo >= 0 && x.hi > 0 && k.hi > *safe && k.lo <= *safe)
            {
                out.shifts.push((
                    (base + op_span.start, base + op_span.end),
                    (base + rhs.span.start, base + rhs.span.end),
                    k.hi,
                    safe,
                ));
            }
        }
    }
    if matches!(op, BinOp::Div | BinOp::Rem) {
        // Only a divisor read as declared: a value derived from one
        // (`a.len`, `k - 1`) is usually nonzero for reasons the
        // bounds can't see.
        let declared = match &crate::walk::unparen(rhs).kind {
            ExprKind::Name(n) => bounds.is_input(n),
            _ => false,
        };
        if let Some(d) = bounds
            .expr(rhs)
            .filter(|d| declared && d.lo <= 0 && 0 <= d.hi && d.lo != d.hi)
        {
            out.divisors.push((
                (base + op_span.start, base + op_span.end),
                (base + rhs.span.start, base + rhs.span.end),
                d.lo,
                d.hi,
            ));
        }
    }
}

/// `for i in n..0` where every declared `n` is above the end: the range only
/// counts up, so the loop never runs.
pub(super) fn empty_range(base: u32, iter: &Expr, bounds: &bounds::Bounds) -> Option<(u32, u32)> {
    let ExprKind::Range {
        lo: Some(lo),
        hi: Some(hi),
        step: None,
        ..
    } = &iter.kind
    else {
        return None;
    };
    let (a, b) = (bounds.expr(lo)?, bounds.expr(hi)?);
    (a.lo > b.hi).then_some((base + iter.span.start, base + iter.span.end))
}

/// A `!` on a member of a slice whose declared ends let it be empty: the
/// whole unwrap, and the slice's range.
pub(crate) type EmptySlice = ((u32, u32), (u32, u32));

pub(crate) fn proven_empty_slices(file: &crate::File) -> Vec<EmptySlice> {
    super::proven(file).empty_slices
}

/// `a[l..r].first!` or `a[l..r].max()!` where the declared `l` and `r`
/// allow an empty slice: every member read with no argument is `None` on
/// an empty one. Then on into the operand.
pub(super) fn unwrap(
    base: u32,
    expr: &Expr,
    inner: &Expr,
    bounds: &bounds::Bounds,
    out: &mut Proven,
) {
    let recv = match &unparen(inner).kind {
        ExprKind::Field { base, .. } => Some(base),
        ExprKind::Call { callee, args } if args.is_empty() => match &unparen(callee).kind {
            ExprKind::Field { base, .. } => Some(base),
            _ => None,
        },
        _ => None,
    };
    if let Some(ExprKind::Index { indices, .. }) = recv.map(|r| &unparen(r).kind) {
        if let [range] = indices.as_slice() {
            if let ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } = &range.kind
            {
                if can_be_empty(lo, hi, *inclusive, bounds).is_some() {
                    out.empty_slices.push((
                        (base + expr.span.start, base + expr.span.end),
                        (base + range.span.start, base + range.span.end),
                    ));
                }
            }
        }
    }
    super::arith(base, inner, bounds, out);
}

/// Whether `lo..hi` (`lo..=hi`) with ends read as declared, or literals,
/// can be empty: then the least value `hi` takes.
pub(crate) fn can_be_empty(
    lo: &Expr,
    hi: &Expr,
    inclusive: bool,
    bounds: &bounds::Bounds,
) -> Option<i128> {
    let declared = |e: &Expr| match &unparen(e).kind {
        ExprKind::Int(_) => bounds.expr(e),
        ExprKind::Name(name) if bounds.is_input(name) => bounds.expr(e),
        _ => None,
    };
    let (a, b) = (declared(lo)?, declared(hi)?);
    if !matches!(unparen(hi).kind, ExprKind::Name(_))
        && !matches!(unparen(lo).kind, ExprKind::Name(_))
    {
        return None;
    }
    // The most `lo - hi` can be: from a relation when the two ends have
    // one, else from their declared ranges.
    let same = bounds.atom(lo).is_some() && bounds.atom(lo) == bounds.atom(hi);
    let gap = if same {
        0
    } else {
        bounds.diff(lo, hi).map_or(a.hi - b.lo, |d| d.hi)
    };
    let empty = if inclusive { gap > 0 } else { gap >= 0 };
    empty.then_some(b.lo)
}

/// `name` is read as `ty`, so it lies in `range`, and an end of `ty`
/// written from another name (`y: x + 1..=n`) relates the two.
pub(crate) fn declared(
    name: &str,
    range: bounds::Interval,
    ty: &cpl_syntax::ast::Type,
    bounds: &mut bounds::Bounds,
) {
    bounds.input(name, range);
    let cpl_syntax::ast::TypeKind::Range { lo, hi, inclusive } = &ty.kind else {
        return;
    };
    let mut rel: Vec<(String, bounds::Interval)> = Vec::new();
    let mut add = |y: String, d: bounds::Interval| match rel.iter_mut().find(|(n, _)| *n == y) {
        Some((_, e)) => {
            *e = bounds::Interval {
                lo: e.lo.max(d.lo),
                hi: e.hi.min(d.hi),
            }
        }
        None => rel.push((y, d)),
    };
    let shift = |e: &Expr| super::shift_of(e, bounds);
    // `name - y` from each end `y + c`, with the end's range for `y`'s.
    if let Some(lo) = lo {
        if let (Some((y, c)), Some(end)) = (shift(lo), bounds.expr(lo)) {
            add(
                y,
                bounds::Interval {
                    lo: c,
                    hi: range.hi - (end.lo - c),
                },
            );
        }
    }
    if let Some(hi) = hi {
        if let (Some((y, c)), Some(end)) = (shift(hi), bounds.expr(hi)) {
            let top = if *inclusive { c } else { c - 1 };
            add(
                y,
                bounds::Interval {
                    lo: range.lo - (end.hi - c),
                    hi: top,
                },
            );
        }
    }
    for (y, d) in rel {
        if y != name && d.lo <= d.hi {
            bounds.relate(name, &y, d);
        }
    }
}

/// A comparison the declared bounds settle: its span, and whether it always
/// holds (else it never does).
pub(crate) type Settled = ((u32, u32), bool);

pub(crate) fn proven_settled(file: &crate::File) -> Vec<Settled> {
    super::proven(file).settled
}

/// `x >= 0` with `x: 0..=n`, or `n == 0` with `n: 1..=10`: one comparison
/// of declared inputs and literals, at least one an input, that the
/// bounds here (narrowed by the tests around it) settle either way.
pub(super) fn settle(base: u32, expr: &Expr, bounds: &bounds::Bounds, out: &mut Proven) {
    use cpl_syntax::ast::CmpOp;
    let ExprKind::Compare { first, rest } = &expr.kind else {
        return;
    };
    let [(op, rhs)] = rest.as_slice() else {
        return;
    };
    let lhs = first.as_ref();
    let input = |e: &Expr| matches!(&unparen(e).kind, ExprKind::Name(n) if bounds.is_input(n));
    let literal = |e: &Expr| matches!(unparen(e).kind, ExprKind::Int(_));
    if !(input(lhs) || input(rhs)) || !(input(lhs) || literal(lhs)) || !(input(rhs) || literal(rhs))
    {
        return;
    }
    let (Some(a), Some(b)) = (bounds.expr(lhs), bounds.expr(rhs)) else {
        return;
    };
    // What `lhs - rhs` can be.
    let mut d = bounds::Interval {
        lo: a.lo - b.hi,
        hi: a.hi - b.lo,
    };
    if let Some(r) = bounds.diff(lhs, rhs) {
        d = bounds::Interval {
            lo: d.lo.max(r.lo),
            hi: d.hi.min(r.hi),
        };
    }
    if d.lo > d.hi {
        return;
    }
    let (always, never) = match op {
        CmpOp::Lt => (d.hi < 0, d.lo >= 0),
        CmpOp::Le => (d.hi <= 0, d.lo > 0),
        CmpOp::Gt => (d.lo > 0, d.hi <= 0),
        CmpOp::Ge => (d.lo >= 0, d.hi < 0),
        CmpOp::Eq => (d.lo == 0 && d.hi == 0, d.lo > 0 || d.hi < 0),
        CmpOp::Ne => (d.lo > 0 || d.hi < 0, d.lo == 0 && d.hi == 0),
        _ => return,
    };
    if always || never {
        out.settled
            .push(((base + expr.span.start, base + expr.span.end), always));
    }
}
