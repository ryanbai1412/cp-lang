//! Assignments to a part: `a[l..r] = ..` between parts, `a max= b` on a
//! row, `m[k] f= x` on a map, and the size of a literal domain.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// True for `g[r, ..]`: an index with a range among several.
    pub(super) fn part_of_grid(e: &ast::Expr) -> bool {
        match &e.kind {
            ExprKind::Index { indices, .. } => {
                indices.len() > 1
                    && indices
                        .iter()
                        .any(|i| matches!(i.kind, ExprKind::Range { .. }))
            }
            _ => false,
        }
    }

    /// `a[l..r] = b[l2..r2] + x`, or `x + b[..]`: a part takes another
    /// part worked with one value, cell for cell. Gives the type of the
    /// part on the right, a row; `None` when the statement is not one.
    pub(super) fn part_map(
        &mut self,
        target: &ast::Expr,
        op: AssignOp,
        value: &ast::Expr,
    ) -> Option<Ty> {
        let is_part = |e: &ast::Expr| {
            matches!(&e.kind, ExprKind::Index { indices, .. }
                if indices.iter().filter(|i| matches!(i.kind, ExprKind::Range { step: None, .. })).count() == 1)
        };
        let ExprKind::Binary {
            op: bin, lhs, rhs, ..
        } = &value.kind
        else {
            return None;
        };
        let worked = matches!(
            bin,
            ast::BinOp::Add
                | ast::BinOp::Sub
                | ast::BinOp::Mul
                | ast::BinOp::BitAnd
                | ast::BinOp::BitOr
                | ast::BinOp::BitXor
        );
        if op != AssignOp::Set || !worked || !is_part(target) || is_part(lhs) == is_part(rhs) {
            return None;
        }
        let (part, one) = if is_part(lhs) { (lhs, rhs) } else { (rhs, lhs) };
        let (part_ty, one_ty) = (self.infer(part), self.infer(one));
        let cell = self.element_ty(&part_ty);
        let row = |ty: &Ty| {
            matches!(ty, Ty::Array { .. }) || matches!(ty, Ty::Named { name, .. } if name == "List")
        };
        if part_ty == Ty::Error || one_ty == Ty::Error {
            return Some(Ty::Error);
        }
        let number = cell.is_int() || cell == Ty::Float;
        if !row(&part_ty)
            || row(&one_ty)
            || !number
            || !(one_ty.is_vague() || self.fits(&one_ty, &cell))
        {
            if let Some(diag) = self.report(
                value.span,
                msgs::check::BAD_OPERANDS_WORKS_EACH_CELL_OF,
                &[
                    &self.quote(value.span),
                    &self.quote(part.span),
                    &self.quote(one.span),
                    &one_ty.with_article(),
                ],
            ) {
                diag.help = Some(
                    "the cells and the value are one kind of number: `a[l..r] = b[l2..r2] + x`"
                        .to_string(),
                );
            }
            return Some(Ty::Error);
        }
        Some(part_ty)
    }

    /// `a max= b` on a row (an array of one dimension, or a list) goes
    /// cell by cell, so `b` is a row of the same type.
    pub(crate) fn row_keep(
        &mut self,
        target: &ast::Expr,
        target_ty: &Ty,
        value: &ast::Expr,
        value_ty: &Ty,
    ) {
        // `a[l..r] max= x`, `g[r, ..] min= x`: one value against every cell
        // of the part.
        let part = matches!(&target.kind, ExprKind::Index { indices, .. } if indices.iter().any(|i| matches!(i.kind, ExprKind::Range { .. })));
        let cell = match target_ty {
            Ty::Array { elem, .. } => Some(&**elem),
            Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(&args[0]),
            _ => None,
        };
        if let (true, Some(cell)) = (part, cell) {
            let row = matches!(value_ty, Ty::Array { .. })
                || matches!(value_ty, Ty::Named { name, .. } if name == "List");
            if !row && !value_ty.is_vague() && self.fits(value_ty, cell) {
                return;
            }
        }
        // `g[r, ..] max= g[v, ..]`: a row of a grid against cells of its kind.
        if let (true, Ty::Array { elem: want, .. }) = (Self::part_of_grid(target), target_ty) {
            let got = match value_ty {
                Ty::Array { index, elem } if index.len() == 1 || Self::part_of_grid(value) => {
                    Some((**elem).clone())
                }
                Ty::Named { name, args } if name == "List" && args.len() == 1 => {
                    Some(args[0].clone())
                }
                other if other.is_vague() || *other == Ty::Error => return,
                _ => None,
            };
            if got
                .as_ref()
                .is_some_and(|got| got.is_vague() || self.fits(got, want))
            {
                return;
            }
            self.report(
                value.span,
                msgs::check::TYPE_MISMATCH_IS_A_ROW_OF,
                &[
                    &self.quote(target.span),
                    &want,
                    &self.quote(value.span),
                    &match got {
                        Some(got) => format!("a row of {got}"),
                        None => value_ty.with_article(),
                    },
                ],
            );
            return;
        }
        let row = match target_ty {
            Ty::Array { index, .. } => index.len() == 1,
            Ty::Named { name, args } => name == "List" && args.len() == 1,
            _ => false,
        };
        if !row {
            return;
        }
        self.expect_fits(value_ty, target_ty, value, |c, got, want| {
            format!(
                "`{}` is {}, so the other side is one too, cell for cell; `{}` is {}",
                c.quote(target.span),
                want.with_article(),
                c.quote(value.span),
                got.with_article()
            )
        });
    }

    /// `m[k] f= x` on a map folds `x` into the entry, and a key that is
    /// not there starts from the identity of `f` (decisions §13). `min`
    /// and `max` store `x`; an operation with no identity is an error.
    pub(super) fn map_fold(&mut self, target: &ast::Expr, op: AssignOp, value: &ast::Expr) {
        let ExprKind::Index { base, indices } = &target.kind else {
            return;
        };
        let fold = match (op, &value.kind) {
            (AssignOp::Binary(op), _) => {
                if matches!(
                    op,
                    ast::BinOp::Add
                        | ast::BinOp::Sub
                        | ast::BinOp::Mul
                        | ast::BinOp::BitOr
                        | ast::BinOp::BitXor
                        // `m[k] ??= init` is the entry itself: a missing
                        // key gets `init`.
                        | ast::BinOp::Coalesce
                ) {
                    return;
                }
                op.text().to_string()
            }
            // `m[k] f= x`, which the parser wrote as `m[k] = f(m[k], x)`.
            (AssignOp::Set, ExprKind::Call { callee, args })
                if args.first().is_some_and(|first| first.value == *target) =>
            {
                match &callee.kind {
                    ExprKind::Name(name) if !matches!(name.as_str(), "gcd" | "lcm") => name.clone(),
                    _ => return,
                }
            }
            _ => return,
        };
        let map = self.quietly(|c| c.infer(base));
        let Ty::Named { name, args } = &map else {
            return;
        };
        let plain = matches!(args.as_slice(), [_, value] if !matches!(value, Ty::Optional(_)) && !value.is_vague());
        if !matches!(name.as_str(), "Map" | "HashMap") || !plain || indices.is_empty() {
            return;
        }
        let (target_text, key) = (self.quote(target.span), self.quote(indices[0].span));
        let base = self.quote(base.span);
        let help = match op {
            AssignOp::Binary(_) => format!(
                "say what a missing key means: `{target_text} = ({base}.get({key}) ?? start) {fold} x`"
            ),
            _ => format!(
                "say what a missing key means: `{target_text} = {fold}({base}.get({key}) ?? start, x)`"
            ),
        };
        if let Some(diag) = self.report(
            target.span,
            msgs::check::BAD_OPERANDS_X_MAY_MEET_A,
            &[&target_text, &fold, &fold],
        ) {
            diag.help = Some(help);
        }
    }

    /// A domain written with two literal ends holds at most 2^31 - 1
    /// values: a position is an `i32` in the generated code.
    /// `In + Out` in a `domain` declaration: the disjoint union of domains.
    pub(super) fn domain_sum(&mut self, def: &ast::Expr) -> bool {
        let ExprKind::Binary {
            op: ast::BinOp::Add,
            lhs,
            rhs,
            ..
        } = &def.kind
        else {
            return false;
        };
        let part = |c: &mut Self, e: &ast::Expr| {
            c.domain_sum(e)
                || matches!(e.kind, ExprKind::Name(_)) && matches!(c.infer(e), Ty::Domain(_))
        };
        part(self, lhs) && part(self, rhs)
    }

    pub(super) fn domain_size(&mut self, def: &ast::Expr) {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            ..
        } = &def.kind
        else {
            return;
        };
        let (ExprKind::Int(lo), ExprKind::Int(hi)) = (&lo.kind, &hi.kind) else {
            return;
        };
        let len = *hi - *lo + i128::from(*inclusive);
        if len > i128::from(i32::MAX) {
            self.report(
                def.span,
                msgs::check::TYPE_MISMATCH_A_DOMAIN_HOLDS_AT,
                &[&len],
            );
        }
    }
}
