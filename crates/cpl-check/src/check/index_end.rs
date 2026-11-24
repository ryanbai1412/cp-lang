//! Indices that read two ways: a bare `Int` into a cut table (R07.1.6, decision 507).

use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    /// The type of a one-index read handled before the general path: an index that reads two
    /// ways is an error, then a struct's `at` (R12.4.13).
    pub(super) fn index_one_way(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> Option<Ty> {
        if self.index_reads_two_ways(expr, base, index) {
            self.infer(index);
            return Some(Ty::Error);
        }
        self.index_by_at(expr, base, index)
    }

    fn index_reads_two_ways(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> bool {
        if matches!(index.kind, ExprKind::Range { .. })
            || !matches!(self.quietly(|c| c.infer(index)), Ty::Int | Ty::Range { .. })
        {
            return false;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        self.bare_int_cut(expr, base, index, &base_ty)
    }

    fn bare_int_cut(&mut self, expr: &Expr, base: &Expr, index: &Expr, base_ty: &Ty) -> bool {
        let Ty::Named { name, args } = base_ty else {
            return false;
        };
        if !matches!(name.as_str(), "Prefix" | "FillPrefix") {
            return false;
        }
        if let Some(Ty::Pos(d)) = args.first() {
            if matches!(self.type_def(d).as_deref(), Some(Def::Domain(_))) {
                return false;
            }
        }
        let (pre, k) = (self.quote(base.span), self.quote(index.span));
        let message =
            format!("`{pre}[{k}]`: an `Int` index into a cut table can't say position or cut");
        if let Some(diag) = self.error(codes::INT_INDEX_ON_CUT_TABLE, index.span, message) {
            let next = match index.kind {
                ExprKind::Int(n) => (n + 1).to_string(),
                _ => format!("{k} + 1"),
            };
            diag.help = Some(format!(
                "`{pre}.cut({k})` is the fold of the first `{k}`; through position `{k}` is `{pre}.cut({next})` (R07.1.6)"
            ));
            diag.fix(
                format!("write `{pre}.cut({k})`"),
                expr.span,
                format!("{pre}.cut({k})"),
            );
        }
        true
    }
}
