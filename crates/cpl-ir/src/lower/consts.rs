//! Compile-time values in the lowerer: integers through the evaluator the
//! checker shares (`cpl_syntax::const_int`), and the tests built on them.

use super::*;

/// The operators and operands after the first of a comparison chain.
pub(super) type Chain = [(CmpOp, ast::Expr)];

impl Lowerer<'_> {
    /// Evaluates a compile-time integer expression (for `Mod<998244353>`, `Mod<P>`, etc.).
    pub(super) fn eval_const_int(&self, e: &ast::Expr) -> Option<i64> {
        cpl_syntax::const_int::eval(e, &|n| self.const_name(n))
    }

    pub(super) fn const_name(&self, n: &str) -> Option<i64> {
        if let Some(GenArg::Int(v)) = self.gen_env.get(n) {
            return Some(*v);
        }
        if let Some(Sym::ModConst(cid)) = self.get_sym(n) {
            return self.eval_const_int(self.mod_consts[cid].init);
        }
        self.const_ints.get(n).copied()
    }

    /// Evaluates the test of an `if const`: comparisons of compile-time
    /// integers, joined by `and`, `or` and `not`.
    pub(super) fn eval_const_bool(&self, e: &ast::Expr) -> Option<bool> {
        match &e.kind {
            ast::ExprKind::Bool(b) => Some(*b),
            // `const FAST = true`, then `if const FAST:`.
            ast::ExprKind::Name(name) => match self.get_sym(name) {
                Some(Sym::ModConst(cid)) => self.eval_const_bool(self.mod_consts[cid].init),
                _ => self.const_bools.get(name).copied(),
            },
            ast::ExprKind::Paren(inner) => self.eval_const_bool(inner),
            ast::ExprKind::Unary {
                op: UnOp::Not,
                expr,
            } => Some(!self.eval_const_bool(expr)?),
            ast::ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => Some(self.eval_const_bool(lhs)? && self.eval_const_bool(rhs)?),
            ast::ExprKind::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
                ..
            } => Some(self.eval_const_bool(lhs)? || self.eval_const_bool(rhs)?),
            ast::ExprKind::Compare { first, rest } => self.eval_const_compare(first, rest),
            _ => None,
        }
    }

    /// A chain of comparisons of compile-time integers: `P != Q`,
    /// `P - 1 < 1 << 30`.
    pub(super) fn eval_const_compare(
        &self,
        first: &ast::Expr,
        rest: &[(ast::CmpOp, ast::Expr)],
    ) -> Option<bool> {
        let mut left = self.eval_const_int(first)?;
        for (op, next) in rest {
            let right = self.eval_const_int(next)?;
            let holds = match op {
                ast::CmpOp::Eq => left == right,
                ast::CmpOp::Ne => left != right,
                ast::CmpOp::Lt => left < right,
                ast::CmpOp::Le => left <= right,
                ast::CmpOp::Gt => left > right,
                ast::CmpOp::Ge => left >= right,
                _ => return None,
            };
            if !holds {
                return Some(false);
            }
            left = right;
        }
        Some(true)
    }

    /// A comparison the shared evaluator settles (`P != Q` once `P` is
    /// known) is that bool, so the branch it guards folds in both backends.
    pub(super) fn settled_compare(&mut self, first: &ast::Expr, rest: &Chain, span: Span) -> Expr {
        let mut sides = std::iter::once(first).chain(rest.iter().map(|(_, e)| e));
        let constant = sides.all(|e| self.names_no_local(e));
        match constant
            .then(|| self.eval_const_compare(first, rest))
            .flatten()
        {
            Some(holds) => self.mk(ExprKind::Bool(holds), Ty::Bool, span),
            None => self.compare(first, rest, span),
        }
    }

    /// Whether no name in the arithmetic `e` is a local (or a name standing
    /// for one), so a `const` it names is that `const`.
    fn names_no_local(&self, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Name(n) => {
                self.lookup(n).is_none()
                    && !self.cx.mut_aliases.contains_key(n)
                    && self.recipe(n).is_none()
            }
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                self.names_no_local(inner)
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                self.names_no_local(lhs) && self.names_no_local(rhs)
            }
            _ => true,
        }
    }
}
