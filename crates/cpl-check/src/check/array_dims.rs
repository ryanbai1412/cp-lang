//! Array dimensions written as anonymous ranges.

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, ExprKind, TypeKind};

impl Checker<'_> {
    /// `[lo..hi]T` counts its cells from a number: `[1..=n]`, `[-5..5]`. A
    /// dimension starting anywhere else is a domain of its own.
    pub(super) fn dim_starts_at_a_number(&mut self, dim: &cpl_syntax::ast::Type) {
        let TypeKind::Range {
            lo: Some(lo),
            hi: Some(_),
            ..
        } = &dim.kind
        else {
            return;
        };
        let number = match &lo.kind {
            ExprKind::Int(_) => true,
            ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
            } => matches!(expr.kind, ExprKind::Int(_)),
            _ => false,
        };
        if !number {
            let start = self.quote(lo.span);
            if let Some(diag) =
                self.report(lo.span, msgs::check::TYPE_MISMATCH_DIM_START, &[&start])
            {
                diag.help = Some("name the range: `domain D = lo..=hi`, then `[D]T`".into());
            }
        }
    }
}
