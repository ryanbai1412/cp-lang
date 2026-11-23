//! Operators whose declared operands can make them trap: a `/` or `%` by a
//! divisor that can be 0, a `<<` that can leave `Int`.

use super::{bounds, Proven};
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
