//! Expressions: precedence climbing over `ops`'s table.

use super::*;
use crate::grammar::{contextual, Contextual};
use crate::ops::{self, Assoc, Level};

/// Tokens that can begin an operand.
fn begins_operand(kind: TokenKind) -> bool {
    matches!(
        kind,
        T::Ident
            | T::Int
            | T::Float
            | T::Str
            | T::RawStr
            | T::Char
            | T::True
            | T::False
            | T::SelfKw
            | T::LParen
            | T::LBracket
    ) || ops::prefix(kind).is_some()
        || ops::RANGE.contains(&kind)
}

impl Parser<'_> {
    pub(super) fn expr(&mut self) {
        match self.peek() {
            T::If => self.if_clauses(NodeKind::IfExpr, |p| {
                p.want(T::Colon, "`:` and the value");
                p.expr();
            }),
            T::Match => self.match_expr(),
            _ if self.lambda_ahead() => self.lambda(),
            _ => self.expr_at(Level::Expr),
        }
    }

    /// `x =>` or `(...) =>` / `(...) ->`.
    fn lambda_ahead(&self) -> bool {
        match self.peek() {
            T::Ident => self.nth(1) == T::FatArrow,
            T::LParen => {
                let mut depth = 0usize;
                for n in 0.. {
                    match self.nth(n) {
                        T::LParen | T::LBracket | T::LBrace => depth += 1,
                        T::RParen | T::RBracket | T::RBrace => {
                            depth -= 1;
                            if depth == 0 {
                                return matches!(self.nth(n + 1), T::FatArrow | T::Arrow);
                            }
                        }
                        T::Eof | T::Newline => return false,
                        _ => {}
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn lambda(&mut self) {
        self.start(NodeKind::LambdaExpr);
        if self.at(T::LParen) {
            self.list(
                NodeKind::ParamList,
                T::RParen,
                "the lambda's parameters",
                |p| {
                    p.eat(T::Mut);
                    p.pattern();
                    if p.eat(T::Colon) {
                        p.expr_at(Level::Range);
                    }
                },
            );
            if self.eat(T::Arrow) {
                self.expr_at(Level::Range);
            }
        } else {
            self.start(NodeKind::ParamList);
            self.pattern();
            self.finish();
        }
        if self.want(T::FatArrow, "`=>` before the lambda's body") {
            self.arm_body();
        }
        self.finish();
    }

    /// An expression whose operators all bind at least as tight as `min`.
    pub(super) fn expr_at(&mut self, min: Level) {
        let at = self.checkpoint();
        self.operand(min);
        loop {
            let kind = self.peek();
            if ops::RANGE.contains(&kind) && min <= Level::Range {
                self.start_at(at, NodeKind::RangeExpr);
                self.range_rest();
                self.finish();
                continue;
            }
            if let Some(word) = self.contextual() {
                if matches!(word, Contextual::Max | Contextual::Min)
                    && min <= Level::Compare
                    && self.nth(1) == T::Eq
                    && self.glued(1)
                {
                    self.start_at(at, NodeKind::KeepExpr);
                    self.bump();
                    self.bump();
                    self.expr_at(Level::Range);
                    self.finish();
                    continue;
                }
                if word == Contextual::By && min <= Level::Range {
                    self.start_at(at, NodeKind::RangeExpr);
                    self.bump();
                    self.expr_at(Level::BitOr);
                    self.finish();
                    continue;
                }
            }
            let Some(op) = self.binary_here() else { break };
            if op.level < min {
                break;
            }
            match op.assoc {
                Assoc::Chain => {
                    self.start_at(at, NodeKind::CompareExpr);
                    while self.binary_here().is_some_and(|b| b.assoc == Assoc::Chain) {
                        self.bump_compare_op();
                        self.expr_at(Level::Range);
                    }
                    self.finish();
                }
                Assoc::Left | Assoc::Right => {
                    self.start_at(at, NodeKind::BinExpr);
                    self.bump();
                    let rhs = match (op.assoc, op.level) {
                        (_, Level::Pow) => Level::Unary,
                        (Assoc::Right, l) => l,
                        (_, l) => l.tighter(),
                    };
                    self.expr_at(rhs);
                    self.finish();
                }
            }
        }
    }

    /// The binary operator here: `ops::BINARY`'s, or `not in`.
    fn binary_here(&self) -> Option<ops::Binary> {
        if self.at(T::Not) && self.nth(1) == T::In {
            return ops::binary(T::In);
        }
        // `a || b` and `a && b` are not two operators: the doubled forms
        // are left to the foreign-syntax diagnostics.
        if matches!(self.peek(), T::Pipe | T::Amp) && self.nth(1) == self.peek() && self.glued(1) {
            return None;
        }
        ops::binary(self.peek())
    }

    fn bump_compare_op(&mut self) {
        if self.at(T::Not) {
            self.bump();
        }
        self.bump();
    }

    fn contextual(&self) -> Option<Contextual> {
        if self.at(T::Ident) {
            contextual(self.text())
        } else {
            None
        }
    }

    /// After a range's low end: `..` or `..=`, the high end if one is
    /// written, and `by step`.
    fn range_rest(&mut self) {
        let inclusive = self.at(T::DotDotEq);
        self.bump();
        if begins_operand(self.peek()) {
            self.expr_at(Level::BitOr);
        } else if inclusive {
            self.recover(codes::EXPECTED_EXPR, "the end of the range after `..=`");
        }
        if self.contextual() == Some(Contextual::By) {
            self.bump();
            self.expr_at(Level::BitOr);
        }
    }

    /// A prefix operator and its operand, a range with no low end, or a
    /// postfix expression.
    fn operand(&mut self, min: Level) {
        if let Some(level) =
            ops::prefix(self.peek()).filter(|_| !(self.at(T::Not) && self.nth(1) == T::In))
        {
            self.start(NodeKind::PrefixExpr);
            self.bump();
            self.expr_at(level.max(min.min(Level::Unary)));
            self.finish();
            return;
        }
        if ops::RANGE.contains(&self.peek()) && min <= Level::Range {
            self.start(NodeKind::RangeExpr);
            self.range_rest();
            self.finish();
            return;
        }
        self.postfix();
    }

    fn postfix(&mut self) {
        let at = self.checkpoint();
        self.primary();
        loop {
            let node = match self.peek() {
                T::LParen => {
                    self.start_at(at, NodeKind::CallExpr);
                    self.list(NodeKind::ArgList, T::RParen, "the argument list", Self::arg);
                    self.finish();
                    continue;
                }
                T::LBracket => {
                    self.start_at(at, NodeKind::IndexExpr);
                    self.list(NodeKind::IndexList, T::RBracket, "the index", |p| p.expr());
                    self.finish();
                    continue;
                }
                T::Dot => NodeKind::FieldExpr,
                T::Question => NodeKind::TryExpr,
                // `x !in xs` is a foreign `not in`, not an unwrap.
                T::Bang if !(self.nth(1) == T::In && self.glued(1)) => NodeKind::UnwrapExpr,
                T::Ident if self.contextual() == Some(Contextual::Is) => {
                    self.start_at(at, NodeKind::IsExpr);
                    self.bump();
                    self.pattern();
                    self.finish();
                    continue;
                }
                _ => break,
            };
            self.start_at(at, node);
            self.bump();
            if node == NodeKind::FieldExpr {
                if matches!(
                    self.peek(),
                    T::Ident | T::Int | T::In | T::Type | T::Domain | T::Use
                ) {
                    self.bump();
                } else {
                    self.recover(codes::EXPECTED_TOKEN, "a field name after `.`");
                }
            }
            self.finish();
        }
    }

    fn arg(&mut self) {
        self.start(NodeKind::Arg);
        if self.nth(1) == T::Colon && (self.at(T::Ident) || self.peek().spelling().is_some()) {
            self.bump();
            self.bump();
        }
        self.eat(T::Mut);
        self.expr();
        self.finish();
    }

    fn primary(&mut self) {
        match self.peek() {
            T::Int | T::Float | T::Str | T::RawStr | T::Char | T::True | T::False => {
                self.start(NodeKind::Literal);
                self.bump();
                self.finish();
            }
            T::Ident | T::SelfKw => {
                self.start(NodeKind::NameRef);
                self.bump();
                self.finish();
            }
            T::LParen => self.paren(),
            T::LBracket => self.list(NodeKind::ListExpr, T::RBracket, "the list", |p| p.expr()),
            _ => self.recover(codes::EXPECTED_EXPR, "an expression"),
        }
    }

    /// `(e)`, or a tuple: `()`, `(a,)`, `(a, b)`.
    fn paren(&mut self) {
        let mut elems = 0;
        let mut comma = false;
        self.list(NodeKind::TupleExpr, T::RParen, "the parentheses", |p| {
            elems += 1;
            p.expr();
            comma |= p.at(T::Comma);
        });
        if elems == 1 && !comma {
            // Re-label: a single element with no comma is a parenthesized
            // expression. The tuple node is replaced in place.
            self.b.relabel_last(NodeKind::ParenExpr);
        }
    }
}

pub(super) fn begins_pattern(kind: TokenKind) -> bool {
    matches!(
        kind,
        T::Ident
            | T::LParen
            | T::Int
            | T::Float
            | T::Str
            | T::RawStr
            | T::Char
            | T::True
            | T::False
            | T::Minus
    )
}
