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

    fn pattern_atom(&mut self) -> Pattern {
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
            T::Ident if self.peek_n(1) == T::LParen => {
                self.bump();
                self.bump();
                let mut args = self.comma_list(T::RParen, "the pattern", Self::pattern);
                let span = self.span_from(start);
                self.planned(span, &crate::planned::LOWER_VARIANT_PATTERN);
                if args.len() == 1 {
                    args.pop().unwrap().kind
                } else {
                    PatternKind::Tuple(args)
                }
            }
            T::Ident => PatternKind::Name(self.name().text),
            T::Var => {
                let span = self.span();
                self.bump();
                self.planned(span, &crate::planned::VAR_PATTERN);
                let inner = self.pattern();
                if self.speculating == 0 {
                    self.sink.poison(self.span_from(start));
                }
                return inner;
            }
            T::SelfKw => {
                self.bump();
                PatternKind::Name("self".into())
            }
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
                    .with_fix("remove `mut`", Span::new(span.start, self.span_n(1).start), ""),
                );
                self.bump();
                return self.pattern_atom();
            }
            _ => {
                let after = self.prev_kind().map_or("here", |k| k.describe());
                self.error_expected(
                    codes::EXPECTED_PATTERN,
                    &format!("a name or pattern after {after}"),
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
}
