//! The least and greatest value an expression or a range type can hold,
//! and how many positions an index type has.

use super::*;

impl<'a> Lowerer<'a> {
    /// The least and greatest value of `e`: a constant, a top-level `let`
    /// of a range with constant ends, or sums and products of them.
    pub(in crate::lower) fn ast_bounds(&self, e: &ast::Expr) -> Option<(i64, i64)> {
        if let Some(v) = self.eval_const_int(e) {
            return Some((v, v));
        }
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.ast_bounds(inner),
            ast::ExprKind::Name(n) => match self.get_sym(n) {
                Some(Sym::Global(g)) => self.global_bounds.get(&g).copied(),
                _ => None,
            },
            ast::ExprKind::Binary { op, lhs, rhs, .. } => {
                interval(*op, self.ast_bounds(lhs)?, self.ast_bounds(rhs)?)
            }
            ast::ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => {
                let (least, most) = self.ast_bounds(expr)?;
                Some((most.checked_neg()?, least.checked_neg()?))
            }
            ast::ExprKind::Unwrap(call) => self.fold_bounds(call, true),
            ast::ExprKind::Call { .. } => self.fold_bounds(e, false),
            _ => None,
        }
    }

    /// The bounds of `xs.min()!`, `xs.max()!` and `xs.sum()` over a
    /// top-level array whose elements have a range type.
    pub(in crate::lower) fn fold_bounds(
        &self,
        call: &ast::Expr,
        unwrapped: bool,
    ) -> Option<(i64, i64)> {
        let ast::ExprKind::Call { callee, args } = &call.kind else {
            return None;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let ast::ExprKind::Name(xs) = &base.kind else {
            return None;
        };
        let Some(Sym::Global(g)) = self.get_sym(xs) else {
            return None;
        };
        let ast::TypeKind::Array { index, elem } = &self.global_specs.get(&g)?.kind else {
            return None;
        };
        let each = self.type_bounds(elem)?;
        match (name.text.as_str(), unwrapped, args.is_empty()) {
            ("min" | "max", true, true) => Some(each),
            ("sum", false, true) => {
                let len = index
                    .iter()
                    .try_fold((1, 1), |(least, most): (i64, i64), d| {
                        let (l, m) = self.type_len(d)?;
                        Some((least.checked_mul(l)?, most.checked_mul(m)?))
                    })?;
                interval(BinOp::Mul, len, each)
            }
            _ => None,
        }
    }

    /// The values a range type such as `1..=400` holds.
    pub(in crate::lower) fn type_bounds(&self, ty: &ast::Type) -> Option<(i64, i64)> {
        let ast::TypeKind::Range {
            lo,
            hi: Some(hi),
            inclusive,
        } = &ty.kind
        else {
            return None;
        };
        let least = match lo {
            Some(lo) => self.ast_bounds(lo)?.0,
            None => 0,
        };
        Some((
            least,
            self.ast_bounds(hi)?.1.checked_sub(i64::from(!*inclusive))?,
        ))
    }

    /// How many positions an index type such as `0..n` or a domain has.
    pub(in crate::lower) fn type_len(&self, ty: &ast::Type) -> Option<(i64, i64)> {
        match &ty.kind {
            ast::TypeKind::Range {
                lo,
                hi: Some(hi),
                inclusive,
            } => {
                let lo = match lo {
                    Some(lo) => self.ast_bounds(lo)?,
                    None => (0, 0),
                };
                let (hi, extra) = (self.ast_bounds(hi)?, i64::from(*inclusive));
                let least = hi.0.checked_add(extra)?.checked_sub(lo.1)?.max(0);
                Some((least, hi.1.checked_add(extra)?.checked_sub(lo.0)?.max(0)))
            }
            ast::TypeKind::Named { name, .. } => match self.get_sym(&name.text) {
                Some(Sym::Domain(d)) => self.domain_lens.get(&d).copied(),
                _ => None,
            },
            _ => None,
        }
    }

    /// [`Self::ast_bounds`] of a lowered expression.
    pub(in crate::lower) fn ir_bounds(&self, e: &Expr) -> Option<(i64, i64)> {
        match &e.kind {
            ExprKind::Int(v) => Some((*v, *v)),
            ExprKind::Global(g) => self.global_bounds.get(g).copied(),
            ExprKind::DomainLen(d) => self.domain_lens.get(d).copied(),
            ExprKind::Binary(op, a, b) => interval(*op, self.ir_bounds(a)?, self.ir_bounds(b)?),
            _ => None,
        }
    }

    /// Conservative `(min, max)` bounds for an AST range bound expression,
    /// including compile-time constants and domain counts (`V.count` / `V.K` <= `i32::MAX`).
    pub(in crate::lower) fn range_bound_limits(&self, e: &ast::Expr) -> Option<(i64, i64)> {
        if let Some(v) = self.eval_const_int(e) {
            return Some((v, v));
        }
        match &e.kind {
            ast::ExprKind::Field { base, name }
                if matches!(name.text.as_str(), "count" | "len" | "size") =>
            {
                if let ast::ExprKind::Name(dname) = &base.kind {
                    let is_dom = matches!(self.gen_env.get(dname), Some(GenArg::Domain(_)))
                        || matches!(self.mod_syms[self.cur_mod].get(dname), Some(Sym::Domain(_)));
                    if is_dom {
                        return Some((0, i64::from(i32::MAX)));
                    }
                }
                None
            }
            ast::ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
                ..
            } => {
                let (lo, hi) = self.range_bound_limits(expr)?;
                Some((hi.checked_neg()?, lo.checked_neg()?))
            }
            ast::ExprKind::Binary {
                op: ast::BinOp::Add,
                lhs,
                rhs,
                ..
            } => {
                let (alo, ahi) = self.range_bound_limits(lhs)?;
                let (blo, bhi) = self.range_bound_limits(rhs)?;
                Some((alo.checked_add(blo)?, ahi.checked_add(bhi)?))
            }
            ast::ExprKind::Binary {
                op: ast::BinOp::Sub,
                lhs,
                rhs,
                ..
            } => {
                let (alo, ahi) = self.range_bound_limits(lhs)?;
                let (blo, bhi) = self.range_bound_limits(rhs)?;
                Some((alo.checked_sub(bhi)?, ahi.checked_sub(blo)?))
            }
            _ => None,
        }
    }
}
