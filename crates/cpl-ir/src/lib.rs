//! The typed program that `cpl run` and `cpl build` share.
//!
//! [`lower`] turns the front end's [`cpl_check::Analysis`] into a
//! [`Program`]: every name is resolved to a slot, every expression has its
//! type, and the sugar (patterns, `match`, comprehensions, `if let`) is
//! already spelled out as loops and tests. The interpreter walks this tree
//! and the C++ back end prints it, so the two can't read a program
//! differently: there is one place where the meaning of the source is
//! decided.
//!
//! Strings are bytes. `Char` is one byte, `Str` is a flat byte array, and a
//! literal with a character outside ASCII is an error here; nothing after
//! this point knows about UTF-8.

mod lower;
mod show;

pub use cpl_syntax::ast::{BinOp, CmpOp, UnOp};
pub use lower::lower;
pub use show::{sep_of, Sep};

use cpl_diag::Span;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The value `INF` stands for, as in the prototype: big enough to be
/// "unreachable", small enough that two of them still add without overflow.
pub const INF: i64 = 4_000_000_000_000_000_000;
/// `Int.MIN` is one above the 64-bit minimum, so negating any `Int` is safe.
pub const INT_MIN: i64 = -i64::MAX;

pub type DomainId = usize;
pub type StructId = usize;
pub type EnumId = usize;
pub type FnId = usize;
pub type GlobalId = usize;
pub type Slot = usize;
pub type LoopId = u32;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Ty {
    Unit,
    Int,
    Float,
    Bool,
    Char,
    Str,
    /// A position of a domain, held as its 0-based ordinal.
    Pos(DomainId),
    /// A gap between positions of a domain: `0..=len`.
    Bound(DomainId),
    Array {
        dims: Vec<Dim>,
        elem: Box<Ty>,
    },
    Opt(Box<Ty>),
    Tuple(Vec<Ty>),
    Record(Vec<(String, Ty)>),
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    Set(Box<Ty>),
    Struct(StructId),
    Enum(EnumId),
    /// Already reported. Fits everything, so one mistake is one message.
    Error,
}

/// What one dimension of an array is indexed by.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Dim {
    Pos(DomainId),
    Bound(DomainId),
    /// A plain `Int` from 0, checked against the length.
    Int,
}

impl Ty {
    pub fn opt(inner: Ty) -> Ty {
        Ty::Opt(Box::new(inner))
    }

    pub fn list(elem: Ty) -> Ty {
        Ty::List(Box::new(elem))
    }

    /// Passed and stored by value without a thought: fits in a register.
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Ty::Unit
                | Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Char
                | Ty::Pos(_)
                | Ty::Bound(_)
                | Ty::Error
        )
    }

    /// The fields of a tuple or record, in order.
    pub fn fields(&self) -> Option<Vec<Ty>> {
        match self {
            Ty::Tuple(items) => Some(items.clone()),
            Ty::Record(fields) => Some(fields.iter().map(|(_, t)| t.clone()).collect()),
            _ => None,
        }
    }
}

/// A type as it was written, keeping what the type alone forgets: the
/// range of an integer and the length of a count-indexed dimension. This
/// is what `read()` follows and what `var a: [n]Int = 0` allocates from.
#[derive(Clone, PartialEq, Debug)]
pub enum Spec {
    /// Any type with nothing to add.
    Plain(Ty),
    Int {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        /// The range as written, for the message when input is outside it.
        text: String,
    },
    /// A tuple, a record, or a struct, field by field.
    Fields(Ty, Vec<Spec>),
    Array {
        dims: Vec<DimSpec>,
        elem: Box<Spec>,
    },
    Opt(Box<Spec>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DimSpec {
    Pos(DomainId),
    Bound(DomainId),
    /// A count-indexed dimension and its length.
    Int(Box<Expr>),
}

impl DimSpec {
    pub fn dim(&self) -> Dim {
        match self {
            DimSpec::Pos(d) => Dim::Pos(*d),
            DimSpec::Bound(d) => Dim::Bound(*d),
            DimSpec::Int(_) => Dim::Int,
        }
    }
}

impl Spec {
    pub fn ty(&self) -> Ty {
        match self {
            Spec::Plain(ty) | Spec::Fields(ty, _) => ty.clone(),
            Spec::Int { .. } => Ty::Int,
            Spec::Array { dims, elem } => Ty::Array {
                dims: dims.iter().map(DimSpec::dim).collect(),
                elem: Box::new(elem.ty()),
            },
            Spec::Opt(inner) => Ty::opt(inner.ty()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExprKind {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(u8),
    Str(Vec<u8>),
    None,
    Some(Box<Expr>),
    Local(Slot),
    Global(GlobalId),
    Unary(UnOp, Box<Expr>),
    /// Both operands have the same type, which picks the operation:
    /// `+` on `Int`, `Float` or `Str`. `and`, `or` and `??` evaluate the
    /// right side only when needed.
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// Operands of one type; never `in` (see [`Builtin::Contains`]).
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
    Cast(Cast, Box<Expr>),
    /// `a[i]`, `g[r, c]`, `xs[k]`, `s[k]`, `m[key]`: the base's type says
    /// which. Positions index unchecked; `Int`s and keys are checked.
    Index(Box<Expr>, Vec<Expr>),
    /// Field of a tuple, record or struct, by position.
    Field(Box<Expr>, usize),
    /// A tuple, record or struct value: its fields in order.
    Tuple(Vec<Expr>),
    /// An enum value: the variant and its payload.
    Variant(usize, Vec<Expr>),
    IsVariant(Box<Expr>, usize),
    /// Payload field of a value known to be that variant.
    Payload(Box<Expr>, usize, usize),
    IsSome(Box<Expr>),
    /// The value inside an optional. `checked` is the user's `!`, which
    /// fails on `none`; unchecked follows a test.
    Unwrap(Box<Expr>, bool),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    /// A block with a value: runs until a [`Stmt::Give`].
    Do(Block),
    Call(FnId, Vec<Arg>),
    Builtin(Builtin, Vec<Expr>),
    /// A builtin that changes its receiver: `xs.push(x)`.
    Mutate(Mutation, Box<Place>, Vec<Expr>),
    /// Sort in place, by a key of each element if one is given.
    Sort(Box<Place>, Option<(Slot, Box<Expr>)>),
    Read(Box<Spec>),
    /// `print(a, b)`: the values space-separated, then a newline.
    Print(Vec<FmtArg>),
    /// An interpolated string.
    Format(Vec<FmtPiece>),
    /// An array of the given shape with every cell `value`, or the
    /// type's zero when there is none.
    Fill(Vec<DimSpec>, Option<Box<Expr>>),
    /// `V.len`, and the label of the first position.
    DomainLen(DomainId),
    DomainLo(DomainId),
    /// Already reported.
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FmtArg {
    pub value: Expr,
    /// Digits after the point for floats inside.
    pub digits: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FmtPiece {
    Text(Vec<u8>),
    Value(FmtArg),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cast {
    IntToFloat,
    /// Toward zero. Fails when the float is not an `Int`'s size.
    FloatToInt,
    CharToInt,
    /// Fails outside `0..=255`.
    IntToChar,
    /// A position or bound as its 0-based ordinal.
    Ord,
    /// A position as the label it was read with: ordinal plus the
    /// domain's first label.
    Label(DomainId),
    /// `V(x)`: a label to a position; fails outside the domain.
    FromLabel(DomainId),
    /// `V.at(i)`: an ordinal to a position; fails outside the domain.
    FromOrd(DomainId),
    /// An ordinal known to be in range, to a position or bound.
    Trust,
}

/// Fns the language provides. Arguments are plain values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Builtin {
    /// Smallest and largest of two or more values of one type.
    Min,
    Max,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    /// Length of an array, list, string, map or set.
    Len,
    /// `xs.first`, `xs.last`: optional.
    First,
    Last,
    /// `x in xs`, for a list, array, set, map (keys) or string.
    Contains,
    /// `m.get(k)`: optional.
    MapGet,
    /// Smallest and largest element of a set: optional.
    SetMin,
    SetMax,
    Sorted,
    Reversed,
    /// `v.next`, `v.prev`: optional position.
    PosNext,
    PosPrev,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mutation {
    Push,
    /// Removes and gives the last element: optional.
    Pop,
    /// Set insert; gives whether the value was new.
    Insert,
    /// Set or map remove; gives whether it was there.
    Remove,
    Clear,
    Reverse,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Arg {
    Value(Expr),
    /// A `mut` argument: the callee works on the caller's variable.
    Mut(Place),
}

/// Something that can be assigned to.
#[derive(Clone, PartialEq, Debug)]
pub struct Place {
    pub kind: PlaceKind,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PlaceKind {
    Local(Slot),
    Global(GlobalId),
    Index(Box<Place>, Vec<Expr>),
    Field(Box<Place>, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignOp {
    Set,
    Binary(BinOp),
    Max,
    Min,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Stmt {
    /// Declares a local; without `init` it starts as its type's zero.
    Let(Slot, Option<Expr>),
    Assign(Place, AssignOp, Expr),
    Expr(Expr),
    If(Expr, Block, Block),
    While(LoopId, Expr, Block),
    For(LoopId, Slot, Iter, Block),
    Return(Option<Expr>),
    /// `None` is the innermost loop.
    Break(Option<LoopId>),
    Continue(Option<LoopId>),
    /// The value of the enclosing [`ExprKind::Do`].
    Give(Expr),
    Assert {
        cond: Expr,
        message: Option<Expr>,
        /// The condition as written, and its line, for the report.
        text: String,
        line: u32,
        span: Span,
    },
    /// `domain V = lo..=hi`: fixes the first label and the size.
    DefDomain {
        id: DomainId,
        lo: Expr,
        len: Expr,
    },
    /// Control can't get here in a correct program.
    Trap(String, Span),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Iter {
    /// Integers, or the ordinals of a domain. `hi` is exclusive.
    Range {
        lo: Expr,
        hi: Expr,
        step: Option<Expr>,
        rev: bool,
    },
    /// The elements of an array, list, set or string, or the `(key,
    /// value)` pairs of a map. `rev` walks backward.
    Seq(Expr, bool),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Local {
    pub name: String,
    pub ty: Ty,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Function {
    pub name: String,
    /// The first `params.len()` slots are the parameters, in order.
    pub params: Vec<ParamDef>,
    pub ret: Ty,
    pub locals: Vec<Local>,
    pub body: Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParamDef {
    pub mutable: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<(String, Vec<Ty>)>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Global {
    pub name: String,
    pub ty: Ty,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Program {
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    /// Domain names. Sizes are set as the program runs.
    pub domains: Vec<String>,
    pub globals: Vec<Global>,
    pub fns: Vec<Function>,
    /// The file's top-level code and its locals.
    pub main: Block,
    pub main_locals: Vec<Local>,
    /// `@multitest`: the input starts with a count and the file runs once
    /// per test.
    pub multitest: bool,
}

impl Program {
    /// A type as the user would write it.
    pub fn show_ty(&self, ty: &Ty) -> String {
        show::ty_name(ty, &self.domains, &self.structs, &self.enums)
    }
}
