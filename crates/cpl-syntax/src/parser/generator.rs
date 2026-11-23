//! Generators outside a call: one error, with the free fold as its fix.

use super::*;

impl<'a> Parser<'a> {
    /// `(x for x in xs)`: a generator is only ever a call's argument, so
    /// `(x for x in xs).max()` is fixed to `max(x for x in xs)`.
    pub(super) fn paren_generator(&mut self, start: u32) -> Expr {
        self.clauses();
        let close = self.span();
        self.expect(T::RParen, "`)` to close the parenthesis");
        let abs = |at: u32| (at as i64 - self.delta) as usize;
        let inner = self.src[abs(start) + 1..abs(close.start)]
            .trim()
            .to_string();
        let fold = self.at(T::Dot)
            && self.peek_n(1) == T::Ident
            && self.peek_n(2) == T::LParen
            && self.peek_n(3) == T::RParen;
        let message = "a generator is written only as a call's argument";
        let mut diag;
        if fold {
            let name = self.tokens[self.pos + 1].span.text(self.src).to_string();
            let tail = Span::new(self.span().start, self.span_n(3).end);
            for _ in 0..4 {
                self.bump();
            }
            let ours = format!("{name}({inner})");
            diag = Diagnostic::error(codes::EXPECTED_TOKEN, self.span_from(start), message)
                .with_help(format!("write `{ours}`"));
            diag.fixes.push(cpl_diag::Fix {
                title: format!("write `{ours}`"),
                edits: vec![
                    (Span::new(start, start + 1), format!("{name}(")),
                    (tail, String::new()),
                ],
            });
        } else {
            diag = Diagnostic::error(codes::EXPECTED_TOKEN, self.span_from(start), message)
                .with_help(format!("pass it to the fold, as in `max({inner})`"));
        }
        self.error(diag);
        Expr {
            kind: ExprKind::Error,
            span: self.span_from(start),
        }
    }
}
