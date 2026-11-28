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
            T::Var if self.nth(1) == T::Ident => {
                self.start(NodeKind::PathPat);
                self.bump();
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
            T::SelfKw => {
                self.start(NodeKind::PathPat);
                self.bump();
                self.finish();
            }
            T::LParen => {
                let mut record = false;
                let (mut elems, mut comma) = (0, false);
                self.list(NodeKind::TuplePat, T::RParen, "the tuple pattern", |p| {
                    if p.at(T::DotDot) && matches!(p.nth(1), T::Comma | T::RParen) {
                        record = true;
                        p.start(NodeKind::RecordPatRest);
                        p.bump();
                        p.finish();
                    } else if p.at(T::Ident) && p.nth(1) == T::Colon {
                        record = true;
                        p.start(NodeKind::RecordPatField);
                        p.bump();
                        p.bump();
                        if !matches!(p.peek(), T::Comma | T::RParen) {
                            p.pattern();
                        }
                        p.finish();
                    } else {
                        p.pattern();
                    }
                    elems += 1;
                    comma |= p.at(T::Comma);
                });
                if record {
                    self.b.relabel_last(NodeKind::RecordPat);
                } else if elems == 1 && !comma {
                    self.b.relabel_last(NodeKind::ParenPat);
                }
            }
            k if begins_pattern(k) => {
                self.start(NodeKind::LiteralPat);
                let minus = self.at(T::Minus);
                if minus {
                    self.start(NodeKind::PrefixExpr);
                    self.bump();
                }
                if matches!(
                    self.peek(),
                    T::Int | T::Float | T::Str | T::RawStr | T::Char | T::True | T::False
                ) {
                    self.start(NodeKind::Literal);
                    self.bump();
                    self.finish();
                } else {
                    self.recover(codes::EXPECTED_TOKEN, "a number after `-`");
                }
                if minus {
                    self.finish();
                }
                self.finish();
            }
            _ => self.recover(codes::EXPECTED_TOKEN, "a pattern"),
        }
    }
}
