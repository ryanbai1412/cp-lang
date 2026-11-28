//! Patterns. A broken pattern ends at `=>`, `:`, `,`, `|` or a closer, so
//! what follows it still parses.

use super::expr::begins_pattern;
use super::*;
use crate::grammar::{contextual, Contextual};

impl Parser<'_> {
    pub(super) fn pattern(&mut self) {
        let at = self.checkpoint();
        self.pattern_atom();
        if self.at(T::Pipe) {
            self.start_at(at, NodeKind::OrPat);
            while self.eat(T::Pipe) {
                self.pattern_atom();
            }
            self.finish();
        }
    }

    fn pattern_atom(&mut self) {
        match self.peek() {
            T::Ident if contextual(self.text()) == Some(Contextual::Wildcard) => {
                self.start(NodeKind::WildcardPat);
                self.bump();
                self.finish();
            }
            T::Ident => {
                self.start(NodeKind::PathPat);
                self.bump();
                while matches!(self.peek(), T::Dot | T::ColonColon) && self.nth(1) == T::Ident {
                    self.bump();
                    self.bump();
                }
                if self.at(T::LParen) {
                    self.list(NodeKind::PatArgs, T::RParen, "the variant's fields", |p| {
                        p.pattern()
                    });
                }
                self.finish();
            }
            T::LParen => {
                let mut record = false;
                self.list(NodeKind::TuplePat, T::RParen, "the tuple pattern", |p| {
                    if p.at(T::Ident) && p.nth(1) == T::Colon {
                        record = true;
                        p.start(NodeKind::RecordPatField);
                        p.bump();
                        p.bump();
                        p.pattern();
                        p.finish();
                    } else {
                        p.pattern();
                    }
                });
                if record {
                    self.b.relabel_last(NodeKind::RecordPat);
                }
            }
            k if begins_pattern(k) => {
                self.start(NodeKind::LiteralPat);
                self.eat(T::Minus);
                if matches!(
                    self.peek(),
                    T::Int | T::Float | T::Str | T::RawStr | T::Char | T::True | T::False
                ) {
                    self.bump();
                } else {
                    self.recover(codes::EXPECTED_TOKEN, "a number after `-`");
                }
                self.finish();
            }
            _ => self.recover(codes::EXPECTED_TOKEN, "a pattern"),
        }
    }
}
