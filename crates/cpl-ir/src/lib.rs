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
pub mod pushed;
mod show;
pub mod walk;

pub use cpl_syntax::ast::{BinOp, CmpOp, UnOp};
/// Every diagnostic the front end has for a file: the checker's, and when
/// it has no error, what lowering rejects. `cpl check`, `cpl run`,
/// `cpl build` and the playground all answer with this, so a file one of
/// them takes is a file all of them take.
pub fn check(analysis: &cpl_check::Analysis) -> Vec<cpl_diag::Diagnostic> {
    let mut diagnostics = analysis.diagnostics.clone();
    if !analysis.has_errors() {
        // The judge's side and the brute are programs of their own, and
        // an error in one is an error in the file.
        let mut results = vec![lower(analysis).map(|p| p.warnings)];
        if let Some(name) = interactor(analysis) {
            results.push(lower_judge(analysis, &name).map(|p| p.warnings));
        }
        if let Some((name, _)) = brute(analysis) {
            results.push(lower_brute(analysis, &name).map(|p| p.warnings));
        }
        if let Some((name, _)) = generator(analysis) {
            results.push(lower_gen(analysis, &name, 1).map(|p| p.warnings));
        }
        if let Some((name, _)) = checker(analysis) {
            results.push(lower_checker(analysis, &name).map(|p| p.warnings));
        }
        for more in results.into_iter().map(|r| r.unwrap_or_else(|errors| errors)) {
            for diagnostic in more {
                if !diagnostics.contains(&diagnostic) {
                    diagnostics.push(diagnostic);
                }
            }
        }
    }
    diagnostics
}

pub use lower::{
    brute, checker, generator, interactor, list_tests, lower, lower_brute, lower_checker, lower_gen,
    lower_judge,
    lower_test,
};
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
    /// `I128`: 128 bits, signed. An `Int` becomes one where one is wanted.
    I128,
    /// `U64`: 64 bits, unsigned. `+`, `-`, `*` and `<<` wrap.
    U64,
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
    /// A `mut` parameter of a fn value: only ever a parameter of a
    /// [`Ty::Fn`]. The call works on the caller's variable.
    Mut(Box<Ty>),
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
    /// A plain `Int` from this first index: `[1..=9]T` starts at 1. The
    /// cells are stored from 0; indexing takes the first index off.
    IntFrom(i64),
}

impl Ty {
    /// True for a collection: with nothing in it, `print` shows nothing,
    /// and puts no space around it (`print(1, [], 2)` is `1 2`).
    pub fn may_show_nothing(&self) -> bool {
        matches!(
            self,
            Ty::List(_) | Ty::Array { .. } | Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_) | Ty::HashSet(_)
        )
    }

    /// True for a type that crosses to the judge's grader: numbers, text,
    /// and lists and tuples of them.
    pub fn crosses_to_judge(&self) -> bool {
        match self {
            Ty::Unit | Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str => true,
            Ty::List(elem) => **elem != Ty::Unit && elem.crosses_to_judge(),
            Ty::Tuple(items) => items.iter().all(|t| *t != Ty::Unit && t.crosses_to_judge()),
            _ => false,
        }
    }

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
                | Ty::I128
                | Ty::U64
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
    /// `?T`, and the number that stands for `none` in the input, from
    /// `read(none: 0)`.
    Opt(Box<Spec>, Option<Box<Expr>>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DimSpec {
    Pos(DomainId),
    Bound(DomainId),
    /// A count-indexed dimension and its length.
    Int(Box<Expr>),
    /// The same, indexed from a first index other than 0.
    IntFrom(i64, Box<Expr>),
}

impl DimSpec {
    pub fn dim(&self) -> Dim {
        match self {
            DimSpec::Pos(d) => Dim::Pos(*d),
            DimSpec::Bound(d) => Dim::Bound(*d),
            DimSpec::Int(_) => Dim::Int,
            DimSpec::IntFrom(lo, _) => Dim::IntFrom(*lo),
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
            Spec::Opt(inner, _) => Ty::opt(inner.ty()),
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
    CallExpr(Box<Expr>, Vec<Arg>),
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
    /// The last part is `desc:`: when true the order is turned around,
    /// equal elements staying as they were.
    Sort(Box<Place>, SortBy, Option<Box<Expr>>),
    Read(Box<Spec>),
    /// `print(a, b)`: the values space-separated, then a newline, or the
    /// text of `end: s` in its place.
    Print(Vec<FmtArg>, Option<Box<Expr>>),
    /// `debug(a, b)`: shown on a local run and recorded as a checkpoint of
    /// its trace. The C++ for a judge has none of it.
    /// With its 1-based line.
    Debug(u32, Vec<DebugArg>),
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

/// One argument of `debug(...)`.
#[derive(Clone, PartialEq, Debug)]
pub struct DebugArg {
    /// The argument as written: `dp[i]`.
    pub text: String,
    pub value: Expr,
    /// A string literal: it names the checkpoint instead of being a value.
    pub label: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FmtPiece {
    Text(Vec<u8>),
    Value(FmtArg),
    /// `{n:05}`, `{x:.3f}`: an `Int` or an `F64` in a number format.
    Number(Expr, NumFmt),
}

/// How `{v:...}` writes a number. A sign comes before the zeros, and a
/// value that rounds to zero has none: never `-0.000`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NumFmt {
    /// Fill to `width` with zeros after the sign, not spaces before it.
    pub zero: bool,
    pub width: u32,
    pub kind: NumKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NumKind {
    /// An `Int` in base 10, 16 or 2; the sign is written apart.
    Dec,
    Hex,
    Bin,
    /// An `F64` with that many digits after the point.
    Fixed(u32),
}

impl NumFmt {
    /// `digits`, the number without its width, padded out to it.
    pub fn pad(&self, digits: &str) -> String {
        let width = self.width as usize;
        match digits.strip_prefix('-') {
            Some(rest) if self.zero => format!("-{rest:0>w$}", w = width.saturating_sub(1)),
            None if self.zero => format!("{digits:0>width$}"),
            _ => format!("{digits:>width$}"),
        }
    }

    pub fn int(&self, v: i64) -> String {
        let sign = if v < 0 { "-" } else { "" };
        let abs = v.unsigned_abs();
        self.pad(&match self.kind {
            NumKind::Hex => format!("{sign}{abs:x}"),
            NumKind::Bin => format!("{sign}{abs:b}"),
            NumKind::Dec | NumKind::Fixed(_) => v.to_string(),
        })
    }

    pub fn float(&self, v: f64) -> String {
        let NumKind::Fixed(places) = self.kind else {
            return String::new();
        };
        let text = if v.is_nan() { "nan".to_string() } else { format!("{v:.*}", places as usize) };
        let zero = text.bytes().all(|b| matches!(b, b'-' | b'0' | b'.'));
        self.pad(if zero { text.trim_start_matches('-') } else { &text })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cast {
    IntToFloat,
    /// An `Int` as an `I128`.
    IntToI128,
    /// Fails when the `I128` is not an `Int`'s size.
    I128ToInt,
    /// An `Int` as a `U64` with the same 64 bits: `-1` is `2**64 - 1`.
    IntToU64,
    /// Fails when the `U64` is `2**63` or more.
    U64ToInt,
    /// Fails when the `I128` is negative or `2**64` or more.
    I128ToU64,
    U64ToI128,
    I128ToFloat,
    U64ToFloat,
    /// Toward zero. Fails when the float is not an `Int`'s size.
    FloatToInt,
    CharToInt,
    /// Fails outside `0..=255`.
    IntToChar,
    /// The result of `Char + Int`. Fails outside ASCII, `0..=127`.
    StepToChar,
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
    /// An `Int` that indexes a list inside `unchecked:`: the writer
    /// vouches that it is inside. `cpl run` still checks; the C++ does not.
    Raw,
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
    /// `flush()`: what was printed so far is written out now.
    Flush,
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
    /// The length of one axis of an array of two or more: the array, then
    /// the axis as a literal.
    AxisLen,
    /// `rand(lo, hi)`: inclusive integer in `lo..=hi`.
    Rand,
    /// `copy(x)`: explicit deep copy of a value.
    Copy,
    /// `List.filled(n, val)`: list of `n` copies of `val`.
    ListFilled,
    /// `Str(x)`: format any value as a string.
    ShowStr,
    /// `sep.join(xs)`: the items of a list or 1-D array as `print` shows
    /// them, with `sep` between. A third argument is what a `none` item
    /// is written as.
    Join,
    /// `xs.get(i)`: the item at 0-based `i`, or `none` outside the list.
    SeqGet,
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
    /// `s.first_ge(k)`, `s.first_gt(k)`, `s.last_le(k)`, `s.last_lt(k)` on
    /// an ordered `Set` (a key) or `Map` (a `(key, value)`): optional,
    /// O(log n).
    FirstGe,
    FirstGt,
    LastLe,
    LastLt,
    Sorted,
    Reversed,
    /// `v.next`, `v.prev`: optional position.
    PosNext,
    PosPrev,
    /// `V.from_idx(ord)` -> `?V`.
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
    /// `s.pop_first()`, `s.pop_last()` on a `Set` or `Map`: takes out the
    /// smallest or largest and gives it, or `none` from an empty one.
    PopFirst,
    PopLast,
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
    /// All subsets of a `Subset<D>` bitmask, from the mask down to `0` inclusive.
    Subsets(Expr),
    /// The members of a `Subset<D>` bitmask, lowest first.
    Members(Expr),
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
    /// The parameters declared `mut`: a call works on the caller's variable.
    pub muts: Vec<Slot>,
    pub captures: Vec<Capture>,
    pub locals: Vec<Local>,
    pub ret: Ty,
    pub body: Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    /// The fields that are a row of a length written as a number, `kid:
    /// [0..2]?Int`, by position: every value of the struct holds exactly
    /// that many cells there.
    pub fixed: std::collections::BTreeMap<usize, u32>,
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
    /// Where the file declares it; empty for a variable the compiler made
    /// or one of an imported module.
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Program {
    /// What the lowering found that is allowed but almost surely a
    /// mistake, for whoever shows diagnostics.
    pub warnings: Vec<cpl_diag::Diagnostic>,
    /// Key types whose `Map`s (and `Set`s) the program never walks in
    /// order: the C++ may keep them in a hash table. Filled in by the
    /// code generator.
    pub unordered_maps: Vec<Ty>,
    pub unordered_sets: Vec<Ty>,
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
    /// What is above `@!multitest`: run once, before the first test case.
    /// Its temporaries are in `main_locals`. Empty without a divider.
    pub setup: Block,
    /// `@!multitest`: `main` runs once per test case, after `setup`.
    pub multitest: bool,
    /// `@!multitest(until_eof)`: there is no count; cases run until the
    /// input ends.
    pub until_eof: bool,
    /// `Subset<D>`: each `(subset, base)` is a domain of the `2^K` subsets
    /// of `base`, a subset held as its bitmask. It is sized when `base` is.
    pub subsets: Vec<(DomainId, DomainId)>,
    /// The program calls `ask(...)`: every `print` goes out at once, so
    /// the judge sees a query before the answer is read.
    pub interactive: bool,
    /// An `@interactor` under `@!multitest`: it reads the number of test
    /// cases from the hidden test and sends it to the solution first.
    pub announce_tests: bool,
    /// `@grader fn name(...)` without a body: the judge's grader supplies
    /// these fns. Each has its signature and an empty body.
    pub judge_fns: Vec<FnId>,
    /// `@grader fn` with a body: the judge's grader calls these. A program
    /// with one has no `main` of its own in the emitted C++.
    pub entry_fns: Vec<FnId>,
    /// Lists whose every index the lowering has already tested (the flat
    /// tables of `@memo`): a back end may index them unchecked.
    pub raw_lists: Vec<GlobalId>,
}

impl Program {
    /// A type as the user would write it.
    pub fn show_ty(&self, ty: &Ty) -> String {
        show::ty_name(ty, &self.domains, &self.structs, &self.enums)
    }
}
