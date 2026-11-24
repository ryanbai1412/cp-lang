//! Patterns.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn pattern(&mut self) -> Pattern {
        let start = self.start();
        let first = self.pattern_atom();
        if !self.at(T::Pipe) {
            return first;
        }
        let mut alts = vec![first];
        while self.eat(T::Pipe) {
            alts.push(self.pattern_atom());
        }
        Pattern {
            kind: PatternKind::Or(alts),
            span: self.span_from(start),
        }
    }

    pub(super) fn pattern_atom(&mut self) -> Pattern {
        let start = self.start();
        if self.at_ident("some") && self.peek_n(1) == T::Ident {
            let span = self.span();
            self.planned(span, &crate::planned::SOME_PATTERN);
            self.bump();
            return self.pattern_atom();
        }
        let kind = match self.peek() {
            T::Ident if self.text() == "_" => {
                self.bump();
                PatternKind::Wildcard
            }
            T::Domain => {
                self.bump();
                let name = self.expect_name("a domain name after `domain`");
                PatternKind::Domain(name)
            }
            T::Ident if self.text() == "None" && self.peek_n(1) != T::Dot => {
                self.bump();
                PatternKind::Literal(Box::new(Expr {
                    kind: ExprKind::None,
                    span: self.span_from(start),
                }))
            }
            T::Ident if self.text().starts_with(|c: char| c.is_ascii_uppercase()) => {
                let mut path = vec![self.name()];
                if path[0].text == "Some" && self.at(T::LParen) {
                    path[0].text = "some".into();
                }
                while self.at(T::Dot) && self.peek_n(1) == T::Ident {
                    self.bump();
                    path.push(self.name());
                }
                let args = if self.eat(T::LParen) {
                    self.comma_list(T::RParen, "the pattern", Self::pattern)
                } else {
                    Vec::new()
                };
                PatternKind::Variant { path, args }
            }
            T::Ident if self.text() == "some" && self.peek_n(1) == T::LParen => {
                let path = vec![self.name()];
                self.bump();
                let args = self.comma_list(T::RParen, "the pattern", Self::pattern);
                PatternKind::Variant { path, args }
            }
            // Read as the variant it names, so it matches only that variant.
            T::Ident if self.peek_n(1) == T::LParen => {
                let path = vec![self.name()];
                self.bump();
                let args = self.comma_list(T::RParen, "the pattern", Self::pattern);
                let span = self.span_from(start);
                self.planned(span, &crate::planned::LOWER_VARIANT_PATTERN);
                PatternKind::Variant { path, args }
            }
            T::Ident => PatternKind::Name(self.name().text),
            // `none` is reserved only up to v2.2; v0.3 code may bind it as a name.
            T::None if matches!(self.prev_kind(), Some(T::Let | T::Var)) => {
                self.bump();
                PatternKind::Name("none".into())
            }
            T::Var => {
                self.bump();
                PatternKind::Var(Box::new(self.pattern()))
            }
            T::SelfKw => {
                self.bump();
                PatternKind::Name("self".into())
            }
            T::LParen if self.at_record_pattern() => self.record_pattern(),
            T::LParen => {
                self.bump();
                let mut items = self.comma_list(T::RParen, "the pattern", Self::pattern);
                if items.len() == 1 {
                    return items.pop().unwrap();
                }
                PatternKind::Tuple(items)
            }
            T::Int
            | T::Float
            | T::Str
            | T::RawStr
            | T::Char
            | T::True
            | T::False
            | T::None
            | T::Minus => PatternKind::Literal(Box::new(self.unary())),
            T::Mut if self.prev_kind() == Some(T::Let) => {
                let span = self.span();
                self.error(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        span,
                        "a binding in a `let` condition can't be `mut`",
                    )
                    .with_help("copy it where it changes: `var x = x` at the top of the block")
                    .with_fix(
                        "remove `mut`",
                        Span::new(span.start, self.span_n(1).start),
                        "",
                    ),
                );
                self.bump();
                return self.pattern_atom();
            }
            T::LBracket => self.list_pattern(),
            T::FatArrow => {
                let at = Span::point(self.span().start);
                self.error(
                    Diagnostic::error(
                        codes::EXPECTED_PATTERN,
                        at,
                        "a match arm needs a pattern before `=>`",
                    )
                    .with_help("write `_ =>` for the arm that takes everything else")
                    .with_fix("insert `_`", at, "_ "),
                );
                PatternKind::Error
            }
            _ => {
                let after = match self.prev_kind() {
                    None | Some(T::Indent | T::Newline | T::Dedent) => {
                        "at the start of the line".to_string()
                    }
                    Some(k) => format!("after {}", k.describe()),
                };
                self.error_expected(
                    codes::EXPECTED_PATTERN,
                    &format!("a name or pattern {after}"),
                    None,
                );
                PatternKind::Error
            }
        };
        Pattern {
            kind,
            span: self.span_from(start),
        }
    }

    /// True at the `(` of a record pattern: `(..`, or `(name:` followed by a
    /// pattern rather than a type, as in `(total:,`, `(to: u` or
    /// `(e: Some(w)`. `(a: Int, b: Str)` stays a typed tuple pattern.
    pub(super) fn at_record_pattern(&self) -> bool {
        if self.peek_n(1) == T::DotDot {
            return true;
        }
        if self.peek_n(1) != T::Ident || self.peek_n(2) != T::Colon {
            return false;
        }
        match self.tokens.get(self.pos + 3) {
            Some(t) if t.kind == T::Ident => {
                let text = t.span.text(self.src);
                !text.starts_with(|c: char| c.is_ascii_uppercase())
                    || text == "None"
                    || self.peek_n(4) == T::LParen
            }
            // `(to: 7, ..)` is a literal; `(a: 1..=100, b: Str)` a range type.
            Some(t) if matches!(t.kind, T::Int | T::Minus) => {
                let after = if t.kind == T::Minus { 5 } else { 4 };
                !matches!(self.peek_n(after), T::DotDot | T::DotDotEq)
            }
            Some(t) => matches!(
                t.kind,
                T::Comma
                    | T::RParen
                    | T::LParen
                    | T::DotDot
                    | T::Var
                    | T::Mut
                    | T::Str
                    | T::Char
                    | T::True
                    | T::False
            ),
            None => false,
        }
    }

    /// `(to: u, w: c)`, `(total:, take:)` (each field bound to its own name)
    /// and `(to: u, ..)` (the rest skipped): R02.7.6.
    fn record_pattern(&mut self) -> PatternKind {
        self.bump();
        let mut fields = Vec::new();
        let mut rest = false;
        while !self.at(T::RParen) && !self.at(T::Eof) {
            if self.eat(T::DotDot) {
                rest = true;
            } else {
                let name = self.expect_name("a field name in the record pattern");
                self.expect(T::Colon, "`:` after the field name");
                let pat = if matches!(self.peek(), T::Comma | T::RParen) {
                    Pattern {
                        kind: PatternKind::Name(name.text.clone()),
                        span: name.span,
                    }
                } else {
                    self.pattern()
                };
                fields.push((name, pat));
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.expect(T::RParen, "`)` to close the record pattern");
        PatternKind::Record { fields, rest }
    }
}
