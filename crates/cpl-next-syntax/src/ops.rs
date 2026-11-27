//! The operator table: every operator's precedence and associativity, as
//! data the expression parser reads (spec/core/01-lexical-grammar.md §1.5).
//! A level is the spec's line, tightest first, so a higher level binds
//! tighter here: `Level::Pow` is the spec's line 2.

use crate::token::TokenKind::{self, *};

/// The spec's §1.5 lines, loosest first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    /// The bottom of an expression: anything parses.
    Expr,
    Coalesce,
    Or,
    And,
    Not,
    Compare,
    Range,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Add,
    Mul,
    Unary,
    Pow,
    Postfix,
}

impl Level {
    /// The next level up: the left operand of a left-associative operator.
    pub fn tighter(self) -> Level {
        LEVELS
            .get(self as usize + 1)
            .copied()
            .unwrap_or(Level::Postfix)
    }
}

const LEVELS: [Level; 16] = [
    Level::Expr,
    Level::Coalesce,
    Level::Or,
    Level::And,
    Level::Not,
    Level::Compare,
    Level::Range,
    Level::BitOr,
    Level::BitXor,
    Level::BitAnd,
    Level::Shift,
    Level::Add,
    Level::Mul,
    Level::Unary,
    Level::Pow,
    Level::Postfix,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    Left,
    Right,
    /// Comparisons chain: `a < b <= c` is one node with three operands.
    Chain,
}

/// A binary operator: its token, its level, how it groups, and the level
/// its right operand is read at.
#[derive(Clone, Copy, Debug)]
pub struct Binary {
    pub token: TokenKind,
    pub level: Level,
    pub assoc: Assoc,
}

const fn bin(token: TokenKind, level: Level, assoc: Assoc) -> Binary {
    Binary {
        token,
        level,
        assoc,
    }
}

/// Every binary operator. `not in` is `Not` then `In` and is found by
/// [`binary`]; ranges (`..`, `..=`) have their own node and are listed in
/// [`RANGE`].
pub const BINARY: &[Binary] = &[
    bin(QuestionQuestion, Level::Coalesce, Assoc::Right),
    bin(Or, Level::Or, Assoc::Left),
    bin(And, Level::And, Assoc::Left),
    bin(EqEq, Level::Compare, Assoc::Chain),
    bin(NotEq, Level::Compare, Assoc::Chain),
    bin(Lt, Level::Compare, Assoc::Chain),
    bin(LtEq, Level::Compare, Assoc::Chain),
    bin(Gt, Level::Compare, Assoc::Chain),
    bin(GtEq, Level::Compare, Assoc::Chain),
    bin(In, Level::Compare, Assoc::Chain),
    bin(Spaceship, Level::Compare, Assoc::Left),
    bin(Pipe, Level::BitOr, Assoc::Left),
    bin(Caret, Level::BitXor, Assoc::Left),
    bin(Amp, Level::BitAnd, Assoc::Left),
    bin(Shl, Level::Shift, Assoc::Left),
    bin(Shr, Level::Shift, Assoc::Left),
    bin(Plus, Level::Add, Assoc::Left),
    bin(Minus, Level::Add, Assoc::Left),
    bin(Star, Level::Mul, Assoc::Left),
    bin(Slash, Level::Mul, Assoc::Left),
    bin(Percent, Level::Mul, Assoc::Left),
    bin(StarStar, Level::Pow, Assoc::Right),
];

/// The range operators, at [`Level::Range`].
pub const RANGE: &[TokenKind] = &[DotDot, DotDotEq];

/// Prefix operators and the level of their operand.
pub const PREFIX: &[(TokenKind, Level)] = &[
    (Not, Level::Not),
    (Minus, Level::Unary),
    (Plus, Level::Unary),
    (Tilde, Level::Unary),
];

/// Postfix operators that take no operand (`x?`, `x!`).
pub const POSTFIX: &[TokenKind] = &[Question, Bang];

/// The level of `token` as a binary operator, if it is one.
pub fn binary(token: TokenKind) -> Option<Binary> {
    BINARY.iter().find(|b| b.token == token).copied()
}

/// The level of `token` as a prefix operator, if it is one.
pub fn prefix(token: TokenKind) -> Option<Level> {
    PREFIX.iter().find(|(t, _)| *t == token).map(|(_, l)| *l)
}

/// Every level `token` is an operator at: `-` is both `Add` and `Unary`.
/// The test that the table and the spec agree reads these.
pub fn levels(token: TokenKind) -> Vec<Level> {
    let mut out: Vec<Level> = binary(token).map(|b| b.level).into_iter().collect();
    out.extend(prefix(token));
    if RANGE.contains(&token) {
        out.push(Level::Range);
    }
    if POSTFIX.contains(&token) {
        out.push(Level::Postfix);
    }
    out
}
