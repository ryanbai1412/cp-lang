use super::*;
use cpl_syntax::ast::{self, ExprKind};

impl Checker<'_> {
    /// The least and greatest value an integer expression can take, where
    /// that follows from literals and range types alone.
    pub(crate) fn bounds(&mut self, expr: &ast::Expr) -> (Option<i128>, Option<i128>) {
        match &expr.kind {
            ExprKind::Int(v) => (Some(*v), Some(*v)),
            ExprKind::Paren(inner) => self.bounds(inner),
            ExprKind::Name(name) => {
                let ty = match self.local(name) {
                    Some(local) => Some(local.ty.clone()),
                    None => match self.global(name).as_deref() {
                        Some(Def::Value { value: Some(v), .. }) => return (Some(*v), Some(*v)),
                        Some(Def::Value { ty, .. }) => Some(ty.clone()),
                        _ => None,
                    },
                };
                match ty {
                    Some(Ty::Range { lo, hi, .. }) => (lo, hi),
                    _ => (None, None),
                }
            }
            ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
            } => {
                let (lo, hi) = self.bounds(expr);
                (hi.map(|v| -v), lo.map(|v| -v))
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let (alo, ahi) = self.bounds(lhs);
                let (blo, bhi) = self.bounds(rhs);
                let both =
                    |a: Option<i128>, b: Option<i128>, f: fn(i128, i128) -> Option<i128>| f(a?, b?);
                match op {
                    ast::BinOp::Add => (
                        both(alo, blo, i128::checked_add),
                        both(ahi, bhi, i128::checked_add),
                    ),
                    ast::BinOp::Sub => (
                        both(alo, bhi, i128::checked_sub),
                        both(ahi, blo, i128::checked_sub),
                    ),
                    ast::BinOp::Mul => match (alo, ahi, blo, bhi) {
                        (Some(a), Some(b), Some(c), Some(d)) => {
                            let products = [
                                a.checked_mul(c),
                                a.checked_mul(d),
                                b.checked_mul(c),
                                b.checked_mul(d),
                            ];
                            if products.iter().all(Option::is_some) {
                                let values = products.map(Option::unwrap);
                                (values.iter().min().copied(), values.iter().max().copied())
                            } else {
                                (None, None)
                            }
                        }
                        _ => (None, None),
                    },
                    // `/` and `%` are Euclidean (decision 9): the
                    // remainder is never negative, whatever the signs,
                    // and the quotient is the one that goes with it.
                    ast::BinOp::Div | ast::BinOp::Rem => match (blo, bhi) {
                        (Some(d), Some(same)) if d == same && d != 0 => {
                            if *op == ast::BinOp::Rem {
                                let exact = match (alo, ahi) {
                                    (Some(a), Some(same)) if a == same => Some(a.rem_euclid(d)),
                                    _ => None,
                                };
                                (exact.or(Some(0)), exact.or(Some(d.abs() - 1)))
                            } else {
                                // Dividing by a positive number keeps the
                                // order; by a negative one reverses it.
                                let (lo, hi) = if d > 0 { (alo, ahi) } else { (ahi, alo) };
                                (
                                    lo.and_then(|a| a.checked_div_euclid(d)),
                                    hi.and_then(|a| a.checked_div_euclid(d)),
                                )
                            }
                        }
                        _ => (None, None),
                    },
                    ast::BinOp::Pow => match (alo, ahi, blo, bhi) {
                        (Some(a), Some(b), Some(e), Some(f)) if a == b && e == f => {
                            let v = u32::try_from(e).ok().and_then(|e| a.checked_pow(e));
                            (v, v)
                        }
                        _ => (None, None),
                    },
                    _ => (None, None),
                }
            }
            ExprKind::Call { .. } => match self.infer_quietly(expr) {
                Ty::Range { lo, hi, .. } => (lo, hi),
                _ => (None, None),
            },
            _ => (None, None),
        }
    }
}
