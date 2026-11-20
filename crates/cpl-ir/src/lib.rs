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
pub use lower::{interactor, list_tests, lower, lower_judge, lower_test};
pub use show::{sep_of, Sep};

use cpl_diag::Span;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The value `INF` stands for, as in the prototype: big enough to be
/// "unreachable", small enough that two of them still add without overflow.
pub const INF: i64 = 4_000_000_000_000_000_000;
/// `Int.MIN`: the smallest 64-bit signed integer (`-(1 << 63)`).
pub const INT_MIN: i64 = i64::MIN;

pub type DomainId = usize;

/// The most values a domain can have for `Subset<D>` to hold its subsets.
pub const MAX_SUBSET_BITS: i64 = 25;
pub type StructId = usize;
pub type EnumId = usize;
pub type FnId = usize;
pub type ClosureId = usize;
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
    /// A half-open range value `lo..hi` over `Int` or a domain `Pos(d)`.
    Range(Box<Ty>),
    /// A runtime domain handle `(lo, len)` returned by `-> (domain C, ...)`.
    DomainVal,
    Array {
        dims: Vec<Dim>,
        elem: Box<Ty>,
    },
    Opt(Box<Ty>),
    Tuple(Vec<Ty>),
    Record(Vec<(String, Ty)>),
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    HashMap(Box<Ty>, Box<Ty>),
    Set(Box<Ty>),
    HashSet(Box<Ty>),
    /// A first-class function or closure `(args...) -> ret`.
    Fn(Vec<Ty>, Box<Ty>),
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
                | Ty::DomainVal
                | Ty::Error
        )
    }

    /// True when values of this type can be compared with `==` and `<` in C++.
    pub fn is_comparable(&self, program: &Program) -> bool {
        match self {
            Ty::Fn(..) | Ty::DomainVal | Ty::HashMap(..) | Ty::HashSet(_) => false,
            Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::Range(t) => t.is_comparable(program),
            Ty::Array { elem, .. } => elem.is_comparable(program),
            Ty::Map(k, v) => k.is_comparable(program) && v.is_comparable(program),
            Ty::Tuple(ts) => ts.iter().all(|t| t.is_comparable(program)),
            Ty::Record(fs) => fs.iter().all(|(_, t)| t.is_comparable(program)),
            Ty::Struct(id) => program.structs[*id]
                .fields
                .iter()
                .all(|(_, t)| t.is_comparable(program)),
            Ty::Enum(id) => program.enums[*id]
                .variants
                .iter()
                .all(|(_, fs)| fs.iter().all(|t| t.is_comparable(program))),
            _ => true,
        }
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
    /// Calls a first-class function or closure value.
    CallExpr(Box<Expr>, Vec<Expr>),
    /// Creates a closure from [`Program::closures`], capturing variables from the enclosing scope.
    Closure(ClosureId),
    /// A half-open range value `lo..hi`.
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
    },
    /// A runtime domain handle `(lo, len)` for `-> (domain C, ...)`.
    DomainVal(DomainId),
    Builtin(Builtin, Vec<Expr>),
    /// A builtin that changes its receiver: `xs.push(x)`.
    Mutate(Mutation, Box<Place>, Vec<Expr>),
    /// Sort in place, naturally, by key expression, or by fn/Order value.
    Sort(Box<Place>, SortBy),
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

/// How a `sort` orders elements.
#[derive(Clone, PartialEq, Debug)]
pub enum SortBy {
    Natural,
    Key(Slot, Box<Expr>),
    /// A fn value `T -> K` (`is_order = false`) or comparator `(T, T) -> Bool` (`is_order = true`).
    Fn {
        func: Box<Expr>,
        is_order: bool,
    },
}

/// Fns the language provides. Arguments are plain values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Builtin {
    /// Smallest and largest of two or more values of one type.
    Min,
    Max,
    Abs,
    Sqrt,
    Isqrt,
    Atan2,
    Floor,
    Ceil,
    Round,
    /// `ask(.., limit: K)`: counts the query and fails past `K` of them.
    CountQuery,
    /// Switches `read()` between the hidden test and what the solution
    /// prints, around each `sol.recv()` of an `@interactor`.
    SwapInput,
    Popcount,
    Ctz,
    Clz,
    BitWidth,
    BitCeil,
    /// `rand(lo, hi)`: inclusive integer in `lo..=hi`.
    Rand,
    /// `copy(x)`: explicit deep copy of a value.
    Copy,
    /// `List.filled(n, val)`: list of `n` copies of `val`.
    ListFilled,
    /// `Str(x)`: format any value as a string.
    ShowStr,
    /// `Int(s)`: parse an integer from a string.
    ParseInt,
    /// `Str` <-> `List<Char>` and `[D]T` -> `List<T>` conversions.
    StrToList,
    ListToStr,
    ArrayToList,
    /// Length of an array, list, string, map, set or range.
    Len,
    /// `xs.first`, `xs.last`: optional (for `List` or `Range`).
    First,
    Last,
    /// `xs.first`, `xs.last`: non-optional element of a non-empty `List` (traps if empty).
    FirstVal,
    LastVal,
    /// `x in xs`, for a list, array, set, map (keys), string or range.
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
    /// `V.try(label)` / `V.try_at(ord)` -> `?V`.
    PosTryLabel(DomainId),
    PosTryOrd(DomainId),
    /// `v.offset(k)` / `v + k` -> `?V`, and `v.cyc(k)` -> `V`.
    PosOffset(DomainId),
    PosCyc(DomainId),
    /// `v.l(k)` / `v.r(k)` -> `Bound(d)` (checked in `0..=K`).
    BoundStepL(DomainId),
    BoundStepR(DomainId),
    /// `b.left` / `b.right` -> `?Pos(d)`.
    BoundLeft(DomainId),
    BoundRight(DomainId),
    /// Range properties: `.lo`, `.hi`, `.rev`.
    RangeLo,
    RangeHi,
    RangeRev,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mutation {
    Push,
    /// Removes and gives the last element: optional.
    Pop,
    /// Removes and gives the last element: value (traps if empty).
    PopVal,
    /// Set insert; gives whether the value was new.
    Insert,
    /// Set or map remove; gives whether it was there.
    Remove,
    Clear,
    Reverse,
    /// `xs.swap(i, j)` on a list or 1-D array.
    Swap,
    /// `xs.resize(n, val)` on a list.
    Resize,
    /// `xs.truncate(n)` on a list.
    Truncate,
    /// `xs.extend(other)` on a list.
    Extend,
    /// `shuffle(mut xs)` on a list.
    Shuffle,
    /// `a.fill(val)` on an array.
    Fill,
    /// `xs.take()`: gives the value and leaves an empty one, in O(1).
    Take,
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
    /// The value inside an optional that was just filled: the place of
    /// `(o ??= [])`.
    Some(Box<Place>),
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
    /// Copies `(lo, len)` from a [`Ty::DomainVal`] into domain `id`.
    DefDomainFromVal {
        id: DomainId,
        val: Expr,
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
    /// A first-class `Range<T>` value.
    RangeVal(Expr, bool),
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
pub struct Capture {
    /// Local slot in the enclosing fn/closure/main (or `GlobalId` if `is_global`).
    pub outer: Slot,
    /// Local slot inside this closure.
    pub inner: Slot,
    /// True if `outer` refers to a `GlobalId` instead of a local `Slot`.
    pub is_global: bool,
    /// True if captured by reference (for mutable `var`s).
    pub by_ref: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ClosureDef {
    /// Parameter slots inside `locals` (always `0..params.len()`).
    pub params: Vec<Slot>,
    pub captures: Vec<Capture>,
    pub locals: Vec<Local>,
    pub ret: Ty,
    pub body: Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    /// From `impl Show for S`: `fn show(a: S) -> Str`.
    pub show_fn: Option<FnId>,
    /// From `impl Eq for S`: `fn eq(a: S, b: S) -> Bool`.
    pub eq_fn: Option<FnId>,
    /// From `impl Ord for S`: `fn lt(a: S, b: S) -> Bool`.
    pub lt_fn: Option<FnId>,
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
    pub closures: Vec<ClosureDef>,
    /// The file's top-level code and its locals.
    pub main: Block,
    pub main_locals: Vec<Local>,
    /// `@multitest`: the input starts with a count and the file runs once
    /// per test.
    pub multitest: bool,
    /// `Subset<D>`: each `(subset, base)` is a domain of the `2^K` subsets
    /// of `base`, a subset held as its bitmask. It is sized when `base` is.
    pub subsets: Vec<(DomainId, DomainId)>,
    /// The program calls `ask(...)`: every `print` goes out at once, so
    /// the judge sees a query before the answer is read.
    pub interactive: bool,
}

impl Program {
    /// A type as the user would write it.
    pub fn show_ty(&self, ty: &Ty) -> String {
        show::ty_name(ty, &self.domains, &self.structs, &self.enums)
    }
}
