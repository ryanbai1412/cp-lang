//! R04b.3.2: a built-in collection's constructor copies contents, never a size.

use super::*;

impl<'a> Lowerer<'a> {
    /// R04b.3.2: a built-in collection's one argument is the contents it
    /// copies, what a `for` walks, and a map's are `(key, value)` pairs;
    /// never a size, as C++'s `vector(5)` or a bucket count would read.
    pub(in crate::lower) fn not_contents(
        &mut self,
        map: bool,
        src: &ast::Expr,
        ty: &Ty,
        span: Span,
    ) -> Expr {
        if matches!(ty, Ty::Error) {
            return self.err_expr(span);
        }
        let text = self.source(src.span).trim().to_string();
        let call = self.source(span);
        let name = call
            .split('(')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        let shown = self.show(ty);
        let article = if shown.starts_with(['A', 'E', 'I', 'O', 'U']) {
            "an"
        } else {
            "a"
        };
        let (message, help) = match ty.seq_elem() {
            Some(elem) if map => (
                format!(
                    "`{name}({text})` copies `(key, value)` pairs, but `{text}` holds `{}`s",
                    self.show(&elem)
                ),
                format!("pair each with its value: `{name}((x, v) for x in {text})`"),
            ),
            _ if map => (
                format!("`{name}({text})` copies `(key, value)` pairs, not {article} `{shown}`"),
                format!("write `{name}()` for an empty map"),
            ),
            _ => (
                format!("`{name}({text})` copies a sequence, not {article} `{shown}`"),
                format!("write `[0 for _ in 0..{text}]` for {text} zeros, or `[{text}]`"),
            ),
        };
        self.error(codes::TYPE_MISMATCH, span, message);
        self.help(help);
        self.err_expr(span)
    }
}
