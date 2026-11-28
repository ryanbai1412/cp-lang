//! Types (spec §1.10 `type`).

use super::*;
use crate::ops::{self, Level};

impl Parser<'_> {
    pub(super) fn ty(&mut self) {
        match self.peek() {
            T::Question => {
                self.start(NodeKind::OptionalType);
                self.bump();
                self.ty();
                self.finish();
            }
            T::LBracket => {
                self.start(NodeKind::ArrayType);
                self.list(NodeKind::Dims, T::RBracket, "the dimensions", |p| p.ty());
                self.ty();
                self.finish();
            }
            T::LParen => self.paren_type(),
            T::Ident if ops::RANGE.contains(&self.nth(1)) => self.range_type(),
            T::Ident | T::SelfKw => self.path_type(),
            T::Int | T::Float | T::Minus | T::DotDot | T::DotDotEq => self.range_type(),
            T::Str | T::Char => {
                self.start(NodeKind::LiteralType);
                self.bump();
                while self.eat(T::Pipe) {
                    if matches!(self.peek(), T::Str | T::Char | T::Int) {
                        self.bump();
                    } else {
                        self.recover(codes::EXPECTED_TOKEN, "a literal after `|`");
                    }
                }
                self.finish();
            }
            _ => self.recover(codes::EXPECTED_TOKEN, "a type"),
        }
    }

    /// `0..n`, `..=9`, `-1e9..=1e9`, and a literal union `1 | 2`.
    fn range_type(&mut self) {
        if self.at(T::Int) && !ops::RANGE.contains(&self.nth(1)) {
            self.start(NodeKind::LiteralType);
            self.bump();
            while self.eat(T::Pipe) {
                self.want(T::Int, "a literal after `|`");
            }
            self.finish();
            return;
        }
        self.start(NodeKind::RangeType);
        self.expr_at(Level::Range);
        self.finish();
    }

    fn path_type(&mut self) {
        self.start(NodeKind::PathType);
        self.bump();
        while matches!(self.peek(), T::Dot | T::ColonColon) && self.nth(1) == T::Ident {
            self.bump();
            self.bump();
        }
        if self.at(T::Lt) {
            self.generic_args();
        }
        self.finish();
    }

    /// `<T, 3, ?U>`. A `>>` closes two lists: the inner one takes the token,
    /// and the outer one closes on it too.
    fn generic_args(&mut self) {
        self.start(NodeKind::GenericArgs);
        self.bump();
        loop {
            match self.peek() {
                T::Int | T::Float | T::Minus => self.expr_at(Level::BitOr),
                _ => self.ty(),
            }
            if !self.eat(T::Comma) || matches!(self.peek(), T::Gt | T::Shr) {
                break;
            }
        }
        if self.half_gt {
            self.half_gt = false;
        } else if self.at(T::Shr) {
            self.bump();
            self.half_gt = true;
        } else {
            self.want(T::Gt, "`>` to close the type arguments");
        }
        self.finish();
    }

    /// `()`, `(A, B)`, `(A) -> B`, or a record type `(lo: Int, ..R)`.
    fn paren_type(&mut self) {
        let record = self.nth(1) == T::Ident && self.nth(2) == T::Colon || self.nth(1) == T::DotDot;
        if record {
            return self.list(NodeKind::RecordType, T::RParen, "the record type", |p| {
                p.start(NodeKind::RecordTypeField);
                if p.eat(T::DotDot) {
                    p.ty();
                } else {
                    p.want(T::Ident, "a field name");
                    p.want(T::Colon, "`:` and the field's type");
                    p.ty();
                }
                p.finish();
            });
        }
        let at = self.checkpoint();
        self.list(NodeKind::TupleType, T::RParen, "the tuple type", |p| p.ty());
        if self.at(T::Arrow) {
            self.start_at(at, NodeKind::FnType);
            self.bump();
            self.ty();
            self.finish();
        }
    }
}
