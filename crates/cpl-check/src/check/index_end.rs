//! Indices that read two ways: a bare `Int` into a cut table (R07.1.6, decision 507), and a
//! negative literal key, which never wraps (R07.2.17, decision 508).

use super::*;
use cpl_syntax::ast::{Expr, ExprKind, UnOp};

impl Checker<'_> {
    /// The type of a one-index read handled before the general path: an index that reads two
    /// ways is an error, then a struct's `at` (R12.4.13).
    pub(super) fn index_one_way(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> Option<Ty> {
        if self.index_reads_two_ways(expr, base, index)
            || self.negative_literal_key(expr, base, index)
        {
            self.infer(index);
            return Some(Ty::Error);
        }
        self.index_by_at(expr, base, index)
    }

    fn index_reads_two_ways(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> bool {
        if matches!(index.kind, ExprKind::Range { .. })
            || Self::says_cut(index)
            || !matches!(self.quietly(|c| c.infer(index)), Ty::Int | Ty::Range { .. })
        {
            return false;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        self.bare_int_cut(expr, base, index, &base_ty)
    }

    /// `a[-1]`: a key never wraps to the end, so a negative literal key into a sequence on
    /// zero-based `Int` keys is out of range before it runs (R07.2.17).
    fn negative_literal_key(&mut self, expr: &Expr, base: &Expr, index: &Expr) -> bool {
        let k = match &index.kind {
            ExprKind::Int(n) if *n < 0 => n.unsigned_abs(),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr: inner,
            } => match inner.kind {
                ExprKind::Int(n) if n > 0 => n.unsigned_abs(),
                _ => return false,
            },
            _ => return false,
        };
        let zero_based = match self.quietly(|c| c.infer(base)) {
            Ty::Str => true,
            Ty::Named { name, .. } => name == "List",
            Ty::Array { index, .. } => {
                matches!(
                    index.as_slice(),
                    [Ty::Const(_)] | [Ty::Range { lo: Some(0), .. }]
                )
            }
            _ => false,
        };
        if !zero_based {
            return false;
        }
        let a = self.quote(base.span);
        let (fix, help) = if k == 1 {
            (
                format!("{a}.last!"),
                format!("keys never wrap: the last element is `{a}.last!`"),
            )
        } else {
            let fix = format!("{a}[{a}.len - {k}]");
            let help = format!("keys never wrap: `{k}` from the end is `{fix}`");
            (fix, help)
        };
        let message = format!("`{a}[-{k}]`: keys never wrap, so a negative key is out of range");
        if let Some(diag) = self.error(codes::NEGATIVE_LITERAL_INDEX, index.span, message) {
            diag.help = Some(format!("{help} (R07.2.17)"));
            diag.fix(format!("write `{fix}`"), expr.span, fix);
        }
        true
    }

    /// `s[a.cut_before(i)]`: an index made by a cut-producing call already says "cut".
    fn says_cut(index: &Expr) -> bool {
        let ExprKind::Call { callee, .. } = &index.kind else {
            return false;
        };
        matches!(&callee.kind, ExprKind::Field { name, .. }
            if matches!(name.text.as_str(), "cut" | "cut_before" | "cut_after"))
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
