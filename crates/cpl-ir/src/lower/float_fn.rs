//! The `F64` intrinsics: `sqrt`, `atan2` and the trigonometric fns.

use super::*;

impl Lowerer<'_> {
    pub(super) fn float_fn(&mut self, name: &str, args: &[ast::Arg], span: Span) -> Expr {
        if name == "atan2" {
            let Some(exprs) = self.positional_any(name, args, 2, span) else {
                return self.err_expr(span);
            };
            let y = self.expect(exprs[0], &Ty::Float);
            let x = self.expect(exprs[1], &Ty::Float);
            return self.mk(
                ExprKind::Builtin(Builtin::Atan2, vec![y, x]),
                Ty::Float,
                span,
            );
        }
        let op = match name {
            "sqrt" => Builtin::Sqrt,
            "sin" => Builtin::Sin,
            "cos" => Builtin::Cos,
            "tan" => Builtin::Tan,
            "asin" => Builtin::Asin,
            _ => Builtin::Acos,
        };
        match self.one_arg(name, args, &Ty::Float, span) {
            Some(value) => self.mk(ExprKind::Builtin(op, vec![value]), Ty::Float, span),
            None => self.err_expr(span),
        }
    }
}
