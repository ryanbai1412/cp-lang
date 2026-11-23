//! The surface language of the prover: terms, sorts and values.
//!
//! A [`Term`] is an expression as it is written in a `.cp` file: the body
//! of a `law`, an `invariant`, an index, an `assert`. The checker builds
//! terms directly; tests and tools parse them from text ([`crate::parse`]).

use std::fmt;

/// `Int.MAX`: the largest `Int`.
pub const INT_MAX: i128 = i64::MAX as i128;
/// `Int.MIN`: `-(2^63 - 1)`. The one value below it is the niche of `?Int`,
/// so `-x` and `abs(x)` can never overflow (decision 1.1).
pub const INT_MIN: i128 = -INT_MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    Neg,
    Not,
    /// `x!`: the value inside an optional.
    Unwrap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    /// Euclidean quotient (decision 1.9).
    Div,
    /// Euclidean remainder: never negative.
    Mod,
    Pow,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    Xor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    /// `x ?? d`: `x`'s value, or `d` when `x` is `none`.
    Coalesce,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::Xor => "^",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::Coalesce => "??",
        }
    }

    /// Binding strength; a higher number binds tighter.
    fn level(self) -> u8 {
        match self {
            BinOp::Coalesce => 1,
            BinOp::Or => 2,
            BinOp::And => 3,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 5,
            BinOp::BitOr => 6,
            BinOp::Xor => 7,
            BinOp::BitAnd => 8,
            BinOp::Shl | BinOp::Shr => 9,
            BinOp::Add | BinOp::Sub => 10,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 11,
            BinOp::Pow => 13,
        }
    }
}

/// An expression.
#[derive(Clone, PartialEq, Debug)]
pub enum Term {
    Int(i128),
    Bool(bool),
    None,
    Var(String),
    Unary(UnOp, Box<Term>),
    Binary(BinOp, Box<Term>, Box<Term>),
    /// `if c: a else: b`.
    If(Box<Term>, Box<Term>, Box<Term>),
    /// `f(x, y)`: a lambda, a builtin (`min`, `max`, `abs`, `gcd`, `some`)
    /// or a declared function the prover knows nothing about.
    Call(Box<Term>, Vec<Term>),
    /// `a[i]`, `g[r, c]`: an element of a declared array.
    Index(Box<Term>, Vec<Term>),
    /// `(x, y)` or `(sum: x, pre: y)`.
    Tuple(Vec<(Option<String>, Term)>),
    /// `x.0`, `x.sum`.
    Field(Box<Term>, String),
    /// `(a, b) => a + b`.
    Lambda(Vec<String>, Box<Term>),
    /// `x in lo..hi` or, when the flag is set, `x in lo..=hi`.
    InRange(Box<Term>, Box<Term>, Box<Term>, bool),
}

impl Term {
    pub fn var(name: &str) -> Term {
        Term::Var(name.to_string())
    }

    pub fn binary(op: BinOp, lhs: Term, rhs: Term) -> Term {
        Term::Binary(op, Box::new(lhs), Box::new(rhs))
    }

    pub fn negate(self) -> Term {
        Term::Unary(UnOp::Not, Box::new(self))
    }

    pub fn and(self, other: Term) -> Term {
        Term::binary(BinOp::And, self, other)
    }

    pub fn call(name: &str, args: Vec<Term>) -> Term {
        Term::Call(Box::new(Term::var(name)), args)
    }

    fn level(&self) -> u8 {
        match self {
            Term::Lambda(..) | Term::If(..) => 0,
            Term::Binary(op, ..) => op.level(),
            Term::Unary(UnOp::Not, _) => 4,
            Term::InRange(..) => 5,
            Term::Unary(UnOp::Neg, _) => 12,
            Term::Int(v) if *v < 0 => 12,
            _ => 14,
        }
    }

    fn fmt_at(&self, f: &mut fmt::Formatter<'_>, min: u8) -> fmt::Result {
        if self.level() < min {
            write!(f, "({self})")
        } else {
            write!(f, "{self}")
        }
    }
}

fn fmt_list(f: &mut fmt::Formatter<'_>, items: &[Term]) -> fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Int(INT_MAX) => write!(f, "Int.MAX"),
            Term::Int(INT_MIN) => write!(f, "Int.MIN"),
            Term::Int(v) => write!(f, "{v}"),
            Term::Bool(b) => write!(f, "{b}"),
            Term::None => write!(f, "none"),
            Term::Var(name) => write!(f, "{name}"),
            Term::Unary(UnOp::Neg, x) => {
                write!(f, "-")?;
                x.fmt_at(f, 12)
            }
            Term::Unary(UnOp::Not, x) => {
                write!(f, "not ")?;
                x.fmt_at(f, 4)
            }
            Term::Unary(UnOp::Unwrap, x) => {
                x.fmt_at(f, 14)?;
                write!(f, "!")
            }
            Term::Binary(op, lhs, rhs) => {
                let level = op.level();
                // Comparisons don't chain and `-`, `/`, `%` group to the left.
                let comparison = level == 5;
                lhs.fmt_at(f, if comparison { level + 1 } else { level })?;
                write!(f, " {} ", op.symbol())?;
                rhs.fmt_at(f, level + 1)
            }
            Term::If(c, a, b) => write!(f, "if {c}: {a} else: {b}"),
            Term::Call(callee, args) => {
                callee.fmt_at(f, 14)?;
                write!(f, "(")?;
                fmt_list(f, args)?;
                write!(f, ")")
            }
            Term::Index(base, args) => {
                base.fmt_at(f, 14)?;
                write!(f, "[")?;
                fmt_list(f, args)?;
                write!(f, "]")
            }
            Term::Tuple(items) => {
                write!(f, "(")?;
                for (i, (name, item)) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Some(name) = name {
                        write!(f, "{name}: ")?;
                    }
                    write!(f, "{item}")?;
                }
                if items.len() == 1 && items[0].0.is_none() {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Term::Field(base, name) => {
                base.fmt_at(f, 14)?;
                write!(f, ".{name}")
            }
            Term::Lambda(params, body) => {
                if let [one] = params.as_slice() {
                    write!(f, "{one} => {body}")
                } else {
                    write!(f, "({}) => {body}", params.join(", "))
                }
            }
            Term::InRange(x, lo, hi, inclusive) => {
                x.fmt_at(f, 6)?;
                write!(f, " in ")?;
                lo.fmt_at(f, 10)?;
                write!(f, "{}", if *inclusive { "..=" } else { ".." })?;
                hi.fmt_at(f, 10)
            }
        }
    }
}

/// The type of a variable, as far as the prover cares.
#[derive(Clone, PartialEq, Debug)]
pub enum Sort {
    /// Any `Int`: `Int.MIN..=Int.MAX`.
    Int,
    Bool,
    /// A range type: `0..n`, `1..=n`, `0..=1e9`. The ends may name variables.
    Range {
        lo: Box<Term>,
        hi: Box<Term>,
        inclusive: bool,
    },
    Tuple(Vec<(Option<String>, Sort)>),
    /// `?T`.
    Opt(Box<Sort>),
    /// A type parameter (`T`, `F`), bound when a law is checked.
    Named(String),
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sort::Int => write!(f, "Int"),
            Sort::Bool => write!(f, "Bool"),
            Sort::Range { lo, hi, inclusive } => {
                lo.fmt_at(f, 10)?;
                write!(f, "{}", if *inclusive { "..=" } else { ".." })?;
                hi.fmt_at(f, 10)
            }
            Sort::Tuple(items) => {
                write!(f, "(")?;
                for (i, (name, sort)) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Some(name) = name {
                        write!(f, "{name}: ")?;
                    }
                    write!(f, "{sort}")?;
                }
                write!(f, ")")
            }
            Sort::Opt(inner) => write!(f, "?{inner}"),
            Sort::Named(name) => write!(f, "{name}"),
        }
    }
}

/// A concrete value: what a counterexample assigns to a variable.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value {
    Int(i128),
    Bool(bool),
    Tuple(Vec<(Option<String>, Value)>),
    /// `none`. A present optional is just its value.
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(INT_MAX) => write!(f, "Int.MAX"),
            Value::Int(INT_MIN) => write!(f, "Int.MIN"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, (name, value)) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Some(name) = name {
                        write!(f, "{name}: ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Value::None => write!(f, "none"),
        }
    }
}

/// A term or a declaration that the prover can't read.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Error {
        Error {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}
