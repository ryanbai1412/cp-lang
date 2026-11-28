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

/// What a binary operator computes; a comparison is a [`CmpOp`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    And,
    Or,
    Coalesce,
    /// `<=>`.
    Spaceship,
}

/// A link of a comparison chain: `a < b <= c` has two.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    /// `not in`, two tokens read as one.
    NotIn,
}

/// What a prefix operator computes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
    BitNot,
}

/// What a binary row computes: a [`Assoc::Chain`] row is a comparison.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Bin(BinOp),
    Cmp(CmpOp),
}

/// A binary operator: its token and how it is written, its level, how it groups, and what
/// it computes.
#[derive(Clone, Copy, Debug)]
pub struct Binary {
    pub token: TokenKind,
    pub spelling: &'static str,
    pub level: Level,
    pub assoc: Assoc,
    pub kind: Kind,
}

const fn bin(
    token: TokenKind,
    spelling: &'static str,
    level: Level,
    assoc: Assoc,
    kind: Kind,
) -> Binary {
    Binary {
        token,
        spelling,
        level,
        assoc,
        kind,
    }
}

/// Every binary operator. `not in` is `Not` then `In` and is found by
/// [`binary`]; ranges (`..`, `..=`) have their own node and are listed in
/// [`RANGE`].
pub const BINARY: &[Binary] = &[
    bin(
        QuestionQuestion,
        "??",
        Level::Coalesce,
        Assoc::Right,
        Kind::Bin(BinOp::Coalesce),
    ),
    bin(Or, "or", Level::Or, Assoc::Left, Kind::Bin(BinOp::Or)),
    bin(And, "and", Level::And, Assoc::Left, Kind::Bin(BinOp::And)),
    bin(
        EqEq,
        "==",
        Level::Compare,
        Assoc::Chain,
        Kind::Cmp(CmpOp::Eq),
    ),
    bin(
        NotEq,
        "!=",
        Level::Compare,
        Assoc::Chain,
        Kind::Cmp(CmpOp::Ne),
    ),
    bin(Lt, "<", Level::Compare, Assoc::Chain, Kind::Cmp(CmpOp::Lt)),
    bin(
        LtEq,
        "<=",
        Level::Compare,
        Assoc::Chain,
        Kind::Cmp(CmpOp::Le),
    ),
    bin(Gt, ">", Level::Compare, Assoc::Chain, Kind::Cmp(CmpOp::Gt)),
    bin(
        GtEq,
        ">=",
        Level::Compare,
        Assoc::Chain,
        Kind::Cmp(CmpOp::Ge),
    ),
    bin(In, "in", Level::Compare, Assoc::Chain, Kind::Cmp(CmpOp::In)),
    bin(
        Spaceship,
        "<=>",
        Level::Compare,
        Assoc::Left,
        Kind::Bin(BinOp::Spaceship),
    ),
    bin(
        Pipe,
        "|",
        Level::BitOr,
        Assoc::Left,
        Kind::Bin(BinOp::BitOr),
    ),
    bin(
        Caret,
        "^",
        Level::BitXor,
        Assoc::Left,
        Kind::Bin(BinOp::BitXor),
    ),
    bin(
        Amp,
        "&",
        Level::BitAnd,
        Assoc::Left,
        Kind::Bin(BinOp::BitAnd),
    ),
    bin(Shl, "<<", Level::Shift, Assoc::Left, Kind::Bin(BinOp::Shl)),
    bin(Shr, ">>", Level::Shift, Assoc::Left, Kind::Bin(BinOp::Shr)),
    bin(Plus, "+", Level::Add, Assoc::Left, Kind::Bin(BinOp::Add)),
    bin(Minus, "-", Level::Add, Assoc::Left, Kind::Bin(BinOp::Sub)),
    bin(Star, "*", Level::Mul, Assoc::Left, Kind::Bin(BinOp::Mul)),
    bin(Slash, "/", Level::Mul, Assoc::Left, Kind::Bin(BinOp::Div)),
    bin(Percent, "%", Level::Mul, Assoc::Left, Kind::Bin(BinOp::Rem)),
    bin(
        StarStar,
        "**",
        Level::Pow,
        Assoc::Right,
        Kind::Bin(BinOp::Pow),
    ),
];

/// The range operators, at [`Level::Range`].
pub const RANGE: &[TokenKind] = &[DotDot, DotDotEq];

/// Prefix operators, how they are written, the level of their operand, and what they
/// compute.
pub const PREFIX: &[(TokenKind, &str, Level, UnOp)] = &[
    (Not, "not", Level::Not, UnOp::Not),
    (Minus, "-", Level::Unary, UnOp::Neg),
    (Plus, "+", Level::Unary, UnOp::Pos),
    (Tilde, "~", Level::Unary, UnOp::BitNot),
];

/// Postfix operators that take no operand (`x?`, `x!`).
pub const POSTFIX: &[TokenKind] = &[Question, Bang];

/// The level of `token` as a binary operator, if it is one.
pub fn binary(token: TokenKind) -> Option<Binary> {
    BINARY.iter().find(|b| b.token == token).copied()
}

/// The level of `token` as a prefix operator, if it is one.
pub fn prefix(token: TokenKind) -> Option<Level> {
    PREFIX
        .iter()
        .find(|(t, ..)| *t == token)
        .map(|(_, _, l, _)| *l)
}

/// What `token` computes as a prefix operator, if it is one.
pub fn prefix_op(token: TokenKind) -> Option<UnOp> {
    PREFIX
        .iter()
        .find(|(t, ..)| *t == token)
        .map(|(.., op)| *op)
}

impl BinOp {
    /// The operator written `text`: `@operator("+")` names [`BinOp::Add`].
    pub fn from_spelling(text: &str) -> Option<BinOp> {
        BINARY.iter().find_map(|b| match b.kind {
            Kind::Bin(op) if b.spelling == text => Some(op),
            _ => None,
        })
    }
}

impl CmpOp {
    /// The comparison written `text`; `not in` is the prefix `not` before `in`.
    pub fn from_spelling(text: &str) -> Option<CmpOp> {
        if let Some(rest) = text.strip_prefix(UnOp::Not.spelling()) {
            return match CmpOp::from_spelling(rest.strip_prefix(' ')?)? {
                CmpOp::In => Some(CmpOp::NotIn),
                _ => None,
            };
        }
        BINARY.iter().find_map(|b| match b.kind {
            Kind::Cmp(op) if b.spelling == text => Some(op),
            _ => None,
        })
    }
}

impl UnOp {
    /// The prefix operator written `text`.
    pub fn from_spelling(text: &str) -> Option<UnOp> {
        PREFIX
            .iter()
            .find(|(_, spelling, ..)| *spelling == text)
            .map(|(.., op)| *op)
    }

    fn spelling(self) -> &'static str {
        PREFIX
            .iter()
            .find(|(.., op)| *op == self)
            .map_or("", |(_, spelling, ..)| spelling)
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row = BINARY.iter().find(|b| b.kind == Kind::Bin(*self));
        f.write_str(row.map_or("", |b| b.spelling))
    }
}

impl std::fmt::Display for CmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == CmpOp::NotIn {
            return write!(f, "{} {}", UnOp::Not, CmpOp::In);
        }
        let row = BINARY.iter().find(|b| b.kind == Kind::Cmp(*self));
        f.write_str(row.map_or("", |b| b.spelling))
    }
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.spelling())
    }
}

/// The binary operator a compound assignment's `OpEq` token applies: `+` in `+=`.
pub fn compound(op_eq: &str) -> Option<BinOp> {
    BinOp::from_spelling(op_eq.strip_suffix('=')?)
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
