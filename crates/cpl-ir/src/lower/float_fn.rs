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

/// `n` from the text `Int(sqrt(n))`, `Int(sqrt(F64(n)))` or `Int(n.sqrt)`,
/// when the text is exactly one of those.
pub(super) fn sqrt_operand(text: &str) -> Option<String> {
    let inner = text.strip_prefix("Int(")?.strip_suffix(')')?.trim();
    let arg = match inner.strip_prefix("sqrt(") {
        Some(rest) => rest.strip_suffix(')')?,
        None => inner.strip_suffix(".sqrt")?,
    };
    let arg = arg
        .strip_prefix("F64(")
        .and_then(|a| a.strip_suffix(')'))
        .unwrap_or(arg)
        .trim();
    let mut depth = 0i32;
    for c in arg.chars() {
        depth += match c {
            '(' | '[' => 1,
            ')' | ']' => -1,
            _ => 0,
        };
        if depth < 0 {
            return None;
        }
    }
    (depth == 0 && !arg.is_empty()).then(|| arg.to_string())
}
