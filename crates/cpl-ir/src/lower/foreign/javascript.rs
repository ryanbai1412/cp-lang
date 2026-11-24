//! JavaScript habits lowering recovers: `x.toFixed(2)` and `xs.slice(a, b)`.

use super::*;

impl<'a> Lowerer<'a> {
    /// JavaScript's `x.toFixed(2)` and `x.to_string()` on a float: its
    /// text is an interpolation, `"{x:.2}"` or `"{x}"` (R10).
    pub(in crate::lower) fn float_text_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        if !matches!(base.kind, ast::ExprKind::Name(_)) {
            return;
        }
        let x = self.source(base.span);
        let new = match (ty, method, args) {
            (Ty::Float, "toFixed", [d]) if matches!(d.value.kind, ast::ExprKind::Int(_)) => {
                format!("\"{{{x}:.{}}}\"", self.source(d.value.span))
            }
            (Ty::Float, "to_string" | "toString", []) => format!("\"{{{x}}}\""),
            _ => return,
        };
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("a float's text is an interpolation: `{new}`"));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// JavaScript's `xs.slice(a, b)` or `s.substring(a, b)`: a slice indexes a range, `xs[a..b]`.
    pub(in crate::lower) fn slice_call_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        self.float_text_fix(ty, base, method, args, span);
        let (Ty::Str | Ty::List(_), "slice" | "substring", [a, b]) = (ty, method, args) else {
            return;
        };
        if !matches!(
            base.kind,
            ast::ExprKind::Name(_) | ast::ExprKind::Field { .. } | ast::ExprKind::Call { .. }
        ) {
            return;
        }
        let new = format!(
            "{}[{}..{}]",
            self.source(base.span),
            self.source(a.value.span),
            self.source(b.value.span)
        );
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("a slice indexes a range of positions: `{new}`"));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }
}
