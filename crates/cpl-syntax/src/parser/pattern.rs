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
        let kind = match self.peek() {
            T::Ident if self.text() == "_" => {
                self.bump();
                PatternKind::Wildcard
            }
            T::Ident if self.text().starts_with(|c: char| c.is_ascii_uppercase()) => {
                let mut path = vec![self.name()];
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
            T::Ident => PatternKind::Name(self.name().text),
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
