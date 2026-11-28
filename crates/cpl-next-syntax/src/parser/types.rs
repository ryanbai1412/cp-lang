//! Types (spec §1.10 `type`).

use super::*;
use crate::grammar::Contextual;
use crate::ops::{self, Level};

impl Parser<'_> {
    /// A type; `A -> B` after any type is a fn type, and `Str of "ab"` a
    /// string over an alphabet.
    pub(super) fn ty(&mut self) {
        let at = self.checkpoint();
        self.ty_atom();
        if self.word() == Some(Contextual::Of) && self.nth(1) == T::Str {
            self.start_at(at, NodeKind::AlphabetType);
            self.bump();
            self.bump();
            self.finish();
        }
        if self.at(T::Arrow) {
            self.start_at(at, NodeKind::FnType);
            self.bump();
            self.ty();
            self.finish();
        }
    }

    fn ty_atom(&mut self) {
        match self.peek() {
            T::Question => {
                self.start(NodeKind::OptionalType);
                self.bump();
                self.ty();
                self.finish();
            }
            T::LBracket => {
                self.start(NodeKind::ArrayType);
                self.list(NodeKind::Dims, T::RBracket, "the dimensions", |p| p.targ());
                self.ty();
                self.finish();
            }
            T::LParen => self.paren_type(),
            T::Domain => {
                self.start(NodeKind::DomainBinder);
                self.bump();
                self.name("the domain's name");
                self.finish();
            }
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
        // `Mod(p)`: a type taking values. After type arguments, a `(`
        // is what follows the type, as in `fn f<S: Seq<B>>(xs: S)`.
        if self.at(T::Lt) {
            self.generic_args();
        } else if self.at(T::LParen) {
            self.list(
                NodeKind::ArgList,
                T::RParen,
                "the type's arguments",
                Self::arg,
            );
        }
        self.finish();
    }

    /// A type argument or a dimension: a type, a value such as `n - 1`
    /// or `1e9 + 7`, or `K: Ord` naming a parameter in an impl's head.
    pub(super) fn targ(&mut self) {
        if self.at(T::Ident) && self.nth(1) == T::Colon {
            return self.generic_param();
        }
        if self.value_ahead() {
            return self.range_type();
        }
        self.ty();
    }

    /// An arithmetic operator before the argument ends.
    fn value_ahead(&self) -> bool {
        let mut depth = 0usize;
        for n in 0.. {
            match self.nth(n) {
                T::LParen | T::LBracket | T::Lt => depth += 1,
                T::RParen | T::RBracket | T::Gt | T::Shr | T::Comma if depth == 0 => return false,
                T::RParen | T::RBracket | T::Gt => depth -= 1,
                T::Shr => depth = depth.saturating_sub(2),
                T::Newline | T::Eof | T::Colon | T::Eq => return false,
                T::Plus
                | T::Minus
                | T::Star
                | T::Slash
                | T::Percent
                | T::DotDot
                | T::DotDotEq
                | T::Float
                    if depth == 0 =>
                {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    /// `<T, 3, ?U>`. A `>>` closes two lists: the inner one takes the token,
    /// and the outer one closes on it too.
    pub(super) fn generic_args(&mut self) {
        self.start(NodeKind::GenericArgs);
        self.bump();
        let outer = std::mem::replace(&mut self.no_gt, true);
        loop {
            self.targ();
            if self.half_gt || !self.eat(T::Comma) || matches!(self.peek(), T::Gt | T::Shr) {
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
        self.no_gt = outer;
        self.finish();
    }

    /// `()`, `(A, B)`, `(A) -> B`, or a record type `(lo: Int, ..R)`.
    fn paren_type(&mut self) {
        let record = self.nth(1) == T::Ident && self.nth(2) == T::Colon || self.nth(1) == T::DotDot;
        if record {
            return self.list(NodeKind::RecordType, T::RParen, "the record type", |p| {
                p.start(NodeKind::RecordTypeField);
                if p.eat(T::DotDot) {
                    if !matches!(p.peek(), T::RParen | T::Comma) {
                        p.ty();
                    }
                } else {
                    p.want(T::Ident, "a field name");
                    p.want(T::Colon, "`:` and the field's type");
                    p.ty();
                }
                p.finish();
            });
        }
        self.list(NodeKind::TupleType, T::RParen, "the tuple type", |p| {
            p.eat(T::Mut);
            p.ty();
        });
    }
}
