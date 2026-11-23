//! The `F64` intrinsics: `sqrt`, `atan2`, `hypot`, `exp`, `ln` and the trigonometric fns.

use super::*;

impl Lowerer<'_> {
    pub(super) fn float_fn(&mut self, name: &str, args: &[ast::Arg], span: Span) -> Expr {
        if let Some(op) = match name {
            "atan2" => Some(Builtin::Atan2),
            "hypot" => Some(Builtin::Hypot),
            _ => None,
        } {
            let Some(exprs) = self.positional_any(name, args, 2, span) else {
                return self.err_expr(span);
            };
            let a = self.expect(exprs[0], &Ty::Float);
            let b = self.expect(exprs[1], &Ty::Float);
            return self.mk(ExprKind::Builtin(op, vec![a, b]), Ty::Float, span);
        }
        let op = match name {
            "sqrt" => Builtin::Sqrt,
            "sin" => Builtin::Sin,
            "cos" => Builtin::Cos,
            "tan" => Builtin::Tan,
            "asin" => Builtin::Asin,
            "exp" => Builtin::Exp,
            "ln" => Builtin::Ln,
            _ => Builtin::Acos,
        };
        match self.one_arg(name, args, &Ty::Float, span) {
            Some(value) => self.mk(ExprKind::Builtin(op, vec![value]), Ty::Float, span),
            None => self.err_expr(span),
        }
    }
}
