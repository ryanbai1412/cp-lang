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
/// How many decimals a `Float` is written with when none are asked for:
/// enough for a judge that compares to within 1e-9.
pub const FLOAT_DIGITS: u32 = 10;

pub mod pushed;
mod show;
pub mod sites;
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
        for more in results
            .into_iter()
            .map(|r| r.unwrap_or_else(|errors| errors))
        {
            for diagnostic in more {
                if !diagnostics.contains(&diagnostic) {
                    diagnostics.push(diagnostic);
                }
            }
        }
    }
    diagnostics
}

/// What `cpl check` and the editor report: exactly what `cpl run` and
/// `cpl build` would reject, in every edition.
pub fn reported(analysis: &cpl_check::Analysis) -> Vec<cpl_diag::Diagnostic> {
    let cites_spec = |d: &cpl_diag::Diagnostic| {
        d.code == cpl_diag::codes::NOT_IMPLEMENTED && d.message.contains("(see design/")
    };
    // A library module is lowered only through the programs that use it,
    // where its generic code has its types.
    let runs = analysis
        .items
        .iter()
        .flat_map(|item| &item.syntax.parsed.stmts)
        .any(|stmt| {
            !matches!(
                stmt.kind,
                cpl_syntax::ast::StmtKind::Fn(_)
                    | cpl_syntax::ast::StmtKind::Struct { .. }
                    | cpl_syntax::ast::StmtKind::Enum { .. }
                    | cpl_syntax::ast::StmtKind::Domain { .. }
                    | cpl_syntax::ast::StmtKind::TypeAlias { .. }
                    | cpl_syntax::ast::StmtKind::Trait { .. }
                    | cpl_syntax::ast::StmtKind::Impl { .. }
                    | cpl_syntax::ast::StmtKind::Law { .. }
                    | cpl_syntax::ast::StmtKind::Use { .. }
                    | cpl_syntax::ast::StmtKind::Const { .. }
                    | cpl_syntax::ast::StmtKind::Context { .. }
            )
        });
    let mut out = if runs {
        check(analysis)
    } else {
        analysis.diagnostics.clone()
    };
    out.retain(|d| !cites_spec(d));

    out
}

pub use lower::{
    brute, checker, generator, has_cached_fields, interactor, list_tests, lower, lower_brute,
    lower_checker, lower_cold_cache, lower_gen, lower_gen_read, lower_judge, lower_sampling_laws,
    lower_test,
};
pub use show::{sep_of, Sep};

use cpl_diag::Span;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The value `INF` stands for, as in the prototype: big enough to be
/// "unreachable", small enough that two of them still add without overflow.
pub const INF: i64 = 4_000_000_000_000_000_000;
/// `Int.MIN`: `-(2^63 - 1)`. The one 64-bit pattern below it is not an
/// `Int`; it is `None` in a `?Int`.
pub const INT_MIN: i64 = -i64::MAX;

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
    /// O(1) read-only view into a `Str` (slice or reverse).
    StrView,
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
    /// O(1) read-only view into a `List<T>` or 1D `[D]T` (slice or
    /// reverse), keyed like its source (R04b.1.2).
    ListView(Box<Ty>, Dim),
    /// An owned table keyed like a view: `z.rekey(s.keys)` on an owned `z`
    /// (R04b.1.7). Every use reads it as a `ListView`.
    Keyed(Box<Ty>, Dim),
    /// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T`.
    MutView(Box<Ty>),
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
    /// Symmetric unordered pairs `{u, v}` (`u <= v`) of a domain: packed
    /// triangular storage of length `n * (n + 1) / 2`.
    Sym(DomainId),
    /// Indexed by every variant of a plain (fieldless) enum.
    Enum(EnumId),
    /// A plain `Int` from 0, checked against the length.
    Int,
    /// A plain `Int` from this first index: `[1..=9]T` starts at 1. The
    /// cells are stored from 0; indexing takes the first index off.
    IntFrom(i64),
}

impl Ty {
    /// True if some part is a view into another value, which must outlive it.
    pub fn holds_view(&self) -> bool {
        match self {
            Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(_) | Ty::StrView => true,
            Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Mut(t) => {
                t.holds_view()
            }
            Ty::Array { elem, .. } => elem.holds_view(),
            Ty::Tuple(ts) => ts.iter().any(Ty::holds_view),
            Ty::Record(fs) => fs.iter().any(|(_, t)| t.holds_view()),
            Ty::Map(k, v) | Ty::HashMap(k, v) => k.holds_view() || v.holds_view(),
            _ => false,
        }
    }

    /// True when some part is [`Ty::Error`]: already reported, so a
    /// mismatch against it is that report's cascade.
    pub fn mentions_error(&self) -> bool {
        match self {
            Ty::Error => true,
            Ty::Range(t)
            | Ty::Opt(t)
            | Ty::List(t)
            | Ty::ListView(t, _)
            | Ty::Keyed(t, _)
            | Ty::MutView(t)
            | Ty::Set(t)
            | Ty::HashSet(t)
            | Ty::Mut(t) => t.mentions_error(),
            Ty::Array { elem, .. } => elem.mentions_error(),
            Ty::Tuple(ts) => ts.iter().any(Ty::mentions_error),
            Ty::Record(fs) => fs.iter().any(|(_, t)| t.mentions_error()),
            Ty::Map(k, v) | Ty::HashMap(k, v) => k.mentions_error() || v.mentions_error(),
            Ty::Fn(ps, r) => ps.iter().any(Ty::mentions_error) || r.mentions_error(),
            _ => false,
        }
    }

    /// True when some part is a fn value.
    pub fn has_fn(&self) -> bool {
        match self {
            Ty::Fn(..) => true,
            Ty::Range(t)
            | Ty::Opt(t)
            | Ty::List(t)
            | Ty::ListView(t, _)
            | Ty::Keyed(t, _)
            | Ty::MutView(t)
            | Ty::Set(t)
            | Ty::HashSet(t)
            | Ty::Mut(t) => t.has_fn(),
            Ty::Array { elem, .. } => elem.has_fn(),
            Ty::Tuple(ts) => ts.iter().any(Ty::has_fn),
            Ty::Record(fs) => fs.iter().any(|(_, t)| t.has_fn()),
            Ty::Map(k, v) | Ty::HashMap(k, v) => k.has_fn() || v.has_fn(),
            _ => false,
        }
    }

    /// True for a collection: with nothing in it, `print` shows nothing,
    /// and puts no space around it (`print(1, [], 2)` is `1 2`).
    pub fn may_show_nothing(&self) -> bool {
        matches!(
            self,
            Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Array { .. }
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
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
            Ty::Opt(t)
            | Ty::List(t)
            | Ty::ListView(t, _)
            | Ty::Keyed(t, _)
            | Ty::MutView(t)
            | Ty::Set(t)
            | Ty::Range(t) => t.is_comparable(program),
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

    /// `[V]T == [a, b]`: an array and a list of the same cells compare as lists, cell by
    /// cell, also inside an optional, tuple or record.
    pub fn list_meet(&self, other: &Ty) -> Option<Ty> {
        match (self, other) {
            (Ty::Array { dims, elem }, Ty::List(e)) | (Ty::List(e), Ty::Array { dims, elem })
                if dims.len() == 1 && elem == e =>
            {
                Some(Ty::List(e.clone()))
            }
            (Ty::Opt(a), Ty::Opt(b)) => a.list_meet(b).map(|t| Ty::Opt(Box::new(t))),
            (Ty::Tuple(xs), Ty::Tuple(ys)) if self.same_shape(other) => {
                let met: Vec<_> = xs.iter().zip(ys).map(|(x, y)| x.list_meet(y)).collect();
                let pick = |(i, m): (usize, &Option<Ty>)| m.clone().unwrap_or(xs[i].clone());
                met.iter()
                    .any(Option::is_some)
                    .then(|| Ty::Tuple(met.iter().enumerate().map(pick).collect()))
            }
            (Ty::Record(xs), Ty::Record(ys)) if self.same_shape(other) => {
                let met: Vec<_> = xs
                    .iter()
                    .zip(ys)
                    .map(|(x, y)| x.1.list_meet(&y.1))
                    .collect();
                let pick = |(x, m): (&(String, Ty), &Option<Ty>)| {
                    (x.0.clone(), m.clone().unwrap_or(x.1.clone()))
                };
                met.iter()
                    .any(Option::is_some)
                    .then(|| Ty::Record(xs.iter().zip(&met).map(pick).collect()))
            }
            _ => None,
        }
    }

    /// Two tuples, or two records with the same field names, of one length and differing somewhere.
    pub fn same_shape(&self, other: &Ty) -> bool {
        match (self, other) {
            (Ty::Tuple(x), Ty::Tuple(y)) => x.len() == y.len() && x != y,
            (Ty::Record(x), Ty::Record(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(p, q)| p.0 == q.0) && x != y
            }
            _ => false,
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

    /// Element type of a built-in `Seq` / `Indexed` sequence (`Str`, `List<T>`,
    /// 1D `[D]T`, `Range<I>`, and O(1) slice/reverse views).
    pub fn seq_elem(&self) -> Option<Ty> {
        match self {
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem) => {
                Some((**elem).clone())
            }
            Ty::Str | Ty::StrView => Some(Ty::Char),
            Ty::Array { dims, elem } if dims.len() == 1 => Some((**elem).clone()),
            Ty::Range(inner) => Some((**inner).clone()),
            _ => None,
        }
    }

    /// The type of a sequence's keys: a domain's positions, else `Int`.
    pub fn key_ty(&self) -> Ty {
        match self.seq_key() {
            Some(Dim::Pos(d)) => Ty::Pos(d),
            _ => Ty::Int,
        }
    }

    /// What a sequence is indexed by: its keys (R04b.1).
    pub fn seq_key(&self) -> Option<Dim> {
        match self {
            Ty::List(_) | Ty::MutView(_) | Ty::Str | Ty::StrView => Some(Dim::Int),
            Ty::ListView(_, key) | Ty::Keyed(_, key) => Some(*key),
            Ty::Array { dims, .. } if dims.len() == 1 => Some(dims[0]),
            _ => None,
        }
    }

    /// True when this type implements the built-in `Seq` trait.
    pub fn is_seq(&self) -> bool {
        self.seq_elem().is_some()
    }

    /// True when this type implements the built-in `Indexed` trait (O(1) random access).
    pub fn is_indexed(&self) -> bool {
        self.seq_elem().is_some()
    }

    /// True when this type implements the built-in `IndexedMut` trait (O(1) random read/write).
    pub fn is_indexed_mut(&self) -> bool {
        match self {
            Ty::List(_) | Ty::MutView(_) | Ty::Str => true,
            Ty::Array { dims, .. } if dims.len() == 1 => true,
            _ => false,
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
    /// `Str(len: lo..=hi) of "01"`: a word whose length is in the range and
    /// whose characters are in the alphabet (R02.3.2).
    Str {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        alphabet: Option<String>,
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
    /// A type its own `impl Read` reads: the call to its `read()`.
    Call(Box<Expr>),
    /// `s.parse<T>()`: the inner spec read from the string `s` instead of
    /// the input, `none` on a bad or leftover token.
    Parse(Box<Expr>, Box<Spec>),
    /// `read_line()`: the rest of the current line (skipping a trailing `\r?\n` after a token).
    Line,
    /// `List<T>`: a count, then that many `T`s (R10.2.3). Lowering spells it
    /// out as a `Call` before a read reaches a backend.
    List(Box<Spec>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DimSpec {
    Pos(DomainId),
    Bound(DomainId),
    Sym(DomainId),
    Enum(EnumId),
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
            DimSpec::Sym(d) => Dim::Sym(*d),
            DimSpec::Enum(e) => Dim::Enum(*e),
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
            Spec::Call(call) => call.ty.clone(),
            Spec::Parse(_, inner) => Ty::opt(inner.ty()),
            Spec::Line | Spec::Str { .. } => Ty::Str,
            Spec::List(each) => Ty::list(each.ty()),
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
    /// Validates that `value` (an `Int`) lies inside the declared inclusive
    /// bounds `[lo, hi]` before storing into a range-typed place. Traps in
    /// the interpreter and under `--checked` (`-DLOCAL`) with `{v} is outside
    /// \`{text}\``; zero-cost in release.
    RangeCheck {
        value: Box<Expr>,
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        text: String,
    },
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
    /// O(1) mutable slice view `(place, lo, hi)` into a `List<T>`, 1D `[D]T`, or `MutView<T>`.
    MutSlice(Box<Place>, Box<Expr>, Box<Expr>),
    /// A builtin that changes its receiver: `xs.push(x)`.
    Mutate(Mutation, Box<Place>, Vec<Expr>),
    /// Sort in place, naturally, by key expression, or by fn/Order value.
    /// The last part is `desc:`: when true the order is turned around,
    /// equal elements staying as they were.
    Sort(Box<Place>, SortBy, Option<Box<Expr>>),
    Read(Box<Spec>),
    /// `s.parse<T>()`: parse a value of `Spec` from a string, returning `?T`.
    Parse(Box<Expr>, Box<Spec>),
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
        let text = if v.is_nan() {
            "nan".to_string()
        } else {
            format!("{v:.*}", places as usize)
        };
        let zero = text.bytes().all(|b| matches!(b, b'-' | b'0' | b'.'));
        self.pad(if zero {
            text.trim_start_matches('-')
        } else {
            &text
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cast {
    /// A plain enum variant to its 0-based tag ordinal.
    EnumOrd(EnumId),
    /// A plain enum variant to its `Int` value (explicit discriminant, or 0-based tag).
    EnumVal(EnumId),
    /// A 0-based tag ordinal to a plain enum variant.
    OrdToEnum(EnumId),
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
    /// `Int` `+`, `-` and `*` inside `unchecked:`: wrap at 64 bits.
    WrapAdd,
    WrapSub,
    WrapMul,
    Sqrt,
    Atan2,
    /// `sin`, `cos`, `tan`, `asin`, `acos` of an `F64`, in radians.
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    /// `exp`, `ln` of an `F64`, and `hypot(x, y)`, the length of `(x, y)`.
    Exp,
    Ln,
    Hypot,
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
    /// True when only blanks are left on stdin: what `read_all` stops at.
    AtEof,
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
    /// `Str` <-> `List<Char>` and `[D]T` <-> `List<T>` conversions.
    StrToList,
    ListToStr,
    ArrayToList,
    /// A list as an array over its positions, `0..len`.
    ListToArray,
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
    /// Two-pointer reach: `xs.reach(while: (a, b) => ..., cyclic: Bool)`.
    Reach,
    Reversed,
    /// O(1) slice view `(base, lo, hi)` on `Str`/`StrView` -> `StrView`.
    StrSlice,
    /// O(1) slice view `(base, lo, hi)` on `List`/`ListView` -> `ListView`.
    ListSlice,
    /// A view's smallest key: a slice keeps its source's keys (R04b.1).
    KeyLo,
    /// `(view, key)` -> the walk position holding `key`, `-1` if the view lacks it.
    KeyPos,
    /// `(view, pos)` -> the key at walk position `pos`.
    PosKey,
    /// `(view, lo, hi)` -> the view narrowed to the keys `lo..hi`, keeping them.
    KeySlice,
    /// `(seq, lo, hi)` -> the same cells keyed `lo..hi`; traps unless the lengths match.
    Rekey,
    /// Explicit O(n) copy of a `Str` or `StrView` into an owned `Str`.
    ToStr,
    /// Explicit O(n) copy of a `List` or `ListView` into an owned `List`.
    ToList,
    /// Direct O(n + m) concatenation of two `Str`/`StrView` values -> `Str`.
    StrConcat,
    /// Direct O(n + m) concatenation of two `List`/`ListView` values -> `List`.
    ListConcat,
    /// `v.next`, `v.prev`: optional position.
    PosNext,
    PosPrev,
    /// `V.try_at(ord)` -> `?V`.
    PosTryOrd(DomainId),
    /// Triangular packed index for `[Sym<V>]T`: `(a, b) -> hi * (hi + 1) / 2 + lo`.
    SymIdx,
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
    /// O(1) arithmetic-series sum of a `Range<Int>`, with optional positive step.
    RangeSum,
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
    /// `xs.reserve(n)` on a list.
    Reserve,
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
        /// Under `with checks(asserts: on)`: checked in every build. (R09.4.1)
        kept: bool,
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

impl Stmt {
    /// Where the statement is in the user's file: what its line is for a
    /// profile or a coverage report. `None` for one with nothing written
    /// (`break`, a bare `return`, a compiler-made zero) or made by the
    /// compiler.
    pub fn span(&self) -> Option<Span> {
        let span = match self {
            Stmt::Let(_, None) | Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) => {
                return None
            }
            Stmt::Let(_, Some(e)) | Stmt::Expr(e) | Stmt::Return(Some(e)) | Stmt::Give(e) => e.span,
            Stmt::Assign(place, _, value) => join(place.span, value.span),
            Stmt::If(cond, ..) | Stmt::While(_, cond, _) => cond.span,
            Stmt::For(_, _, iter, _) => match iter {
                Iter::Range { lo, hi, .. } => join(lo.span, hi.span),
                Iter::Walk { len: e, .. }
                | Iter::RangeVal(e, _)
                | Iter::Seq(e, _)
                | Iter::Subsets(e)
                | Iter::Members(e)
                | Iter::TreeRange { container: e, .. } => e.span,
            },
            Stmt::Assert { span, .. } | Stmt::Trap(_, span) => *span,
            Stmt::DefDomain { len, .. } => len.span,
            Stmt::DefDomainFromVal { val, .. } => val.span,
        };
        (span != Span::default()).then_some(span)
    }
}

/// The span over both, ignoring one the compiler made.
fn join(a: Span, b: Span) -> Span {
    match (a == Span::default(), b == Span::default()) {
        (true, _) => b,
        (_, true) => a,
        _ => a.to(b),
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Iter {
    /// Integers, or the ordinals of a domain. `hi` is exclusive, or the
    /// last value when `closed`: `lo..=hi` is walked without `hi + 1`,
    /// which `Int.MAX` does not have.
    Range {
        lo: Expr,
        hi: Expr,
        step: Option<Expr>,
        rev: bool,
        closed: bool,
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
    /// The values of a library domain in ordinal order, or backward when
    /// `rev`: `nth(k)` for each `k` in `0..len`, with `k` in slot `k` and
    /// `len` read once into slot `n`. Lowering expands it into an `Int`
    /// range loop before any backend sees it.
    Walk {
        k: Slot,
        n: Slot,
        len: Expr,
        nth: FnId,
        rev: bool,
    },
    /// Keys (for `Set`) or `(key, value)` pairs (for `Map`) in a bounded range,
    /// walking the underlying ordered tree directly in O(log n + k).
    TreeRange {
        container: Expr,
        lo: Option<(Expr, bool)>,
        hi: Option<(Expr, bool)>,
        rev: bool,
    },
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

/// Narrow storage width for a ranged `Int` field or ranged-`Int` array/list element.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum IntWidth {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
}

impl IntWidth {
    /// The narrowest width holding `lo..=hi` (R03.1.4), or `None` past 32 bits.
    pub fn of(lo: i64, hi: i64) -> Option<IntWidth> {
        let fits = |min: i64, max: i64| lo >= min && hi <= max;
        if lo > hi {
            None
        } else if fits(0, u8::MAX.into()) {
            Some(IntWidth::U8)
        } else if fits(i8::MIN.into(), i8::MAX.into()) {
            Some(IntWidth::I8)
        } else if fits(0, u16::MAX.into()) {
            Some(IntWidth::U16)
        } else if fits(i16::MIN.into(), i16::MAX.into()) {
            Some(IntWidth::I16)
        } else if fits(0, u32::MAX.into()) {
            Some(IntWidth::U32)
        } else if fits(i32::MIN.into(), i32::MAX.into()) {
            Some(IntWidth::I32)
        } else {
            None
        }
    }

    pub fn is_unsigned(self) -> bool {
        matches!(self, IntWidth::U8 | IntWidth::U16 | IntWidth::U32)
    }

    pub fn cpp(self) -> &'static str {
        match self {
            IntWidth::U8 => "uint8_t",
            IntWidth::I8 => "int8_t",
            IntWidth::U16 => "uint16_t",
            IntWidth::I16 => "int16_t",
            IntWidth::U32 => "uint32_t",
            IntWidth::I32 => "int32_t",
        }
    }
}

/// Narrowed element layout for a range-typed array, grid, or list.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum NarrowLayout {
    /// Leaf element is a narrowed 32-bit `Int` (`uint32_t` or `int32_t`).
    Int(IntWidth),
    /// Leaf element is a tuple or record whose fields are all narrowed 32-bit `Int`s.
    Tuple(Vec<IntWidth>),
    /// Element is itself a nested array/grid/list whose elements have the inner layout.
    Seq(Box<NarrowLayout>),
    /// Leaf element is a `?lo..hi` inside `int32_t` above `INT32_MIN`, which is `none`.
    OptI32,
}

impl NarrowLayout {
    /// C++ element type for a container whose IR element type is `elem_ty`.
    pub fn cpp_elem(&self, elem_ty: &Ty) -> Option<String> {
        match (self, elem_ty) {
            (NarrowLayout::Int(w), Ty::Int) => Some(w.cpp().to_string()),
            (NarrowLayout::OptI32, Ty::Opt(t)) if **t == Ty::Int => Some("cpl::OptI32".to_string()),
            (NarrowLayout::Tuple(ws), Ty::Tuple(ts)) if ws.len() == ts.len() => {
                let parts: Vec<&str> = ws.iter().map(|w| w.cpp()).collect();
                Some(format!("std::tuple<{}>", parts.join(", ")))
            }
            (NarrowLayout::Tuple(ws), Ty::Record(fs)) if ws.len() == fs.len() => {
                let parts: Vec<&str> = ws.iter().map(|w| w.cpp()).collect();
                Some(format!("std::tuple<{}>", parts.join(", ")))
            }
            (NarrowLayout::Seq(inner), Ty::Array { dims, elem }) => {
                let cell = inner.cpp_elem(elem)?;
                if dims.len() == 1 {
                    Some(format!("std::vector<{cell}>"))
                } else {
                    Some(format!("cpl::Grid<{cell}, {}>", dims.len()))
                }
            }
            (NarrowLayout::Seq(inner), Ty::List(elem)) => {
                let cell = inner.cpp_elem(elem)?;
                Some(format!("std::vector<{cell}>"))
            }
            (NarrowLayout::Seq(inner), Ty::ListView(elem, _) | Ty::Keyed(elem, _)) => {
                let cell = inner.cpp_elem(elem)?;
                Some(format!("cpl::ListView<{cell}>"))
            }
            _ => None,
        }
    }
}

/// Identifies a local-slot scope in a [`Program`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ScopeId {
    Main,
    Fn(FnId),
    Closure(ClosureId),
}

/// Storage location that may hold a 1D array/list/view with narrowed 32-bit `Int` elements.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ElemLoc {
    Global(GlobalId),
    Local(ScopeId, Slot),
    Field(StructId, usize),
    FnRet(FnId),
    ClosureRet(ClosureId),
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    /// The fields that are a row of a length written as a number, `kid:
    /// [0..2]?Int`, by position: every value of the struct holds exactly
    /// that many cells there.
    pub fixed: std::collections::BTreeMap<usize, u32>,
    /// Ranged `Int` fields whose static bounds fit in 32 bits, by position.
    pub narrow_ints: std::collections::BTreeMap<usize, IntWidth>,
    /// From `impl Show for S`: `fn show(a: S) -> Str`.
    pub show_fn: Option<FnId>,
    /// From `impl Eq for S`: `fn eq(a: S, b: S) -> Bool`.
    pub eq_fn: Option<FnId>,
    /// From `impl Ord for S`: `fn lt(a: S, b: S) -> Bool`.
    pub lt_fn: Option<FnId>,
    /// Field indices marked `@cache` (interior-mutable, skipped by `==`).
    pub cached_fields: std::collections::BTreeSet<usize>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<(String, Vec<Ty>)>,
    pub values: Vec<Option<i64>>,
    /// `@tag("Q") Ask`: the token a variant reads and prints as, when it
    /// is not the variant's name. (R02.7.2)
    pub tags: Vec<Option<String>>,
}

impl EnumDef {
    /// The token variant `tag` reads and prints as.
    pub fn token(&self, tag: usize) -> &str {
        match self.tags.get(tag) {
            Some(Some(token)) => token,
            _ => &self.variants[tag].0,
        }
    }
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
    /// Each struct's name as the file writes it, `MinHeap<(Int, V)>`.
    pub struct_shown: Vec<String>,
    pub enums: Vec<EnumDef>,
    /// Domain names. Sizes are set as the program runs.
    pub domains: Vec<String>,
    pub globals: Vec<Global>,
    pub fns: Vec<Function>,
    /// By [`FnId`]: whether the fn is an imported module's, not the file's.
    pub library_fns: Vec<bool>,
    /// Per fn, its name as the source writes it: `Heap.pop` for the method
    /// lowered as `Heap_Tup2_Int_P0_10.pop`. What traces and call stacks show.
    pub fn_display: Vec<String>,
    pub closures: Vec<ClosureDef>,
    /// Declared 32-bit element storage layouts for ranged-`Int` arrays, grids,
    /// nested arrays, lists, and all-ranged tuple elements.
    pub narrow_elems: std::collections::BTreeMap<ElemLoc, NarrowLayout>,
    /// The file's top-level code and its locals.
    pub main: Block,
    pub main_locals: Vec<Local>,
    /// What is above `@!multitest`: run once, before the first test case.
    /// Its temporaries are in `main_locals`. Empty without a divider.
    pub setup: Block,
    /// `@!multitest`: `main` runs once per test case, after `setup`.
    pub multitest: bool,
    /// Where `@!multitest` is written: a bad test count traps there.
    pub multitest_span: Span,
    /// `@!multitest(until_eof)`: there is no count; cases run until the
    /// input ends.
    pub until_eof: bool,
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
        if self.struct_shown.len() != self.structs.len() {
            return show::ty_name(ty, &self.domains, &self.structs, &self.enums);
        }
        let mut structs = self.structs.clone();
        for (def, shown) in structs.iter_mut().zip(&self.struct_shown) {
            def.name.clone_from(shown);
        }
        show::ty_name(ty, &self.domains, &structs, &self.enums)
    }

    /// Collects all `DomainId`s referenced by `ty`, traversing through struct
    /// and enum definitions. Domains referenced by a function's return type
    /// escape to the caller and must not be restored when that function exits.
    pub fn domains_in_ty(&self, ty: &Ty) -> std::collections::HashSet<DomainId> {
        let mut out = std::collections::HashSet::new();
        let mut seen_structs = std::collections::HashSet::new();
        let mut seen_enums = std::collections::HashSet::new();
        self.collect_domains_in_ty(ty, &mut out, &mut seen_structs, &mut seen_enums);
        out
    }

    fn collect_domains_in_ty(
        &self,
        ty: &Ty,
        out: &mut std::collections::HashSet<DomainId>,
        seen_structs: &mut std::collections::HashSet<StructId>,
        seen_enums: &mut std::collections::HashSet<EnumId>,
    ) {
        match ty {
            Ty::Pos(d) | Ty::Bound(d) => {
                out.insert(*d);
            }
            Ty::Range(inner)
            | Ty::Opt(inner)
            | Ty::List(inner)
            | Ty::ListView(inner, _)
            | Ty::Keyed(inner, _)
            | Ty::MutView(inner)
            | Ty::Set(inner)
            | Ty::HashSet(inner)
            | Ty::Mut(inner) => {
                self.collect_domains_in_ty(inner, out, seen_structs, seen_enums);
            }
            Ty::Array { dims, elem } => {
                for dim in dims {
                    if let Dim::Pos(d) | Dim::Bound(d) | Dim::Sym(d) = dim {
                        out.insert(*d);
                    }
                }
                self.collect_domains_in_ty(elem, out, seen_structs, seen_enums);
            }
            Ty::Tuple(ts) => {
                for t in ts {
                    self.collect_domains_in_ty(t, out, seen_structs, seen_enums);
                }
            }
            Ty::Record(fields) => {
                for (_, t) in fields {
                    self.collect_domains_in_ty(t, out, seen_structs, seen_enums);
                }
            }
            Ty::Map(k, v) | Ty::HashMap(k, v) => {
                self.collect_domains_in_ty(k, out, seen_structs, seen_enums);
                self.collect_domains_in_ty(v, out, seen_structs, seen_enums);
            }
            Ty::Fn(ps, r) => {
                for p in ps {
                    self.collect_domains_in_ty(p, out, seen_structs, seen_enums);
                }
                self.collect_domains_in_ty(r, out, seen_structs, seen_enums);
            }
            Ty::Struct(id) => {
                if seen_structs.insert(*id) {
                    for (_, ft) in &self.structs[*id].fields {
                        self.collect_domains_in_ty(ft, out, seen_structs, seen_enums);
                    }
                }
            }
            Ty::Enum(id) => {
                if seen_enums.insert(*id) {
                    for (_, fields) in &self.enums[*id].variants {
                        for ft in fields {
                            self.collect_domains_in_ty(ft, out, seen_structs, seen_enums);
                        }
                    }
                }
            }
            Ty::Unit
            | Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Bool
            | Ty::Char
            | Ty::Str
            | Ty::StrView
            | Ty::DomainVal
            | Ty::Error => {}
        }
    }

    /// Returns the local `DomainId`s defined inside `body` that do not escape through `ret`, in deterministic
    /// order. Both interpreter and C++ codegen save these on function entry and
    /// restore them on function exit so recursive re-entry preserves the caller.
    pub fn fn_guarded_domains(&self, ret: &Ty, body: &Block) -> Vec<DomainId> {
        let escaping = self.domains_in_ty(ret);
        let mut defined = Vec::new();
        let mut seen = std::collections::HashSet::new();
        self.collect_defined_domains_block(body, &escaping, &mut seen, &mut defined);
        defined
    }

    fn record_defined_domain(
        &self,
        id: DomainId,
        escaping: &std::collections::HashSet<DomainId>,
        seen: &mut std::collections::HashSet<DomainId>,
        out: &mut Vec<DomainId>,
    ) {
        if !escaping.contains(&id) && seen.insert(id) {
            out.push(id);
        }
    }

    fn collect_defined_domains_block(
        &self,
        block: &Block,
        escaping: &std::collections::HashSet<DomainId>,
        seen: &mut std::collections::HashSet<DomainId>,
        out: &mut Vec<DomainId>,
    ) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::DefDomain { id, lo, len } => {
                    self.collect_defined_domains_expr(lo, escaping, seen, out);
                    self.collect_defined_domains_expr(len, escaping, seen, out);
                    self.record_defined_domain(*id, escaping, seen, out);
                }
                Stmt::DefDomainFromVal { id, val } => {
                    self.collect_defined_domains_expr(val, escaping, seen, out);
                    self.record_defined_domain(*id, escaping, seen, out);
                }
                Stmt::Let(_, Some(e))
                | Stmt::Assign(_, _, e)
                | Stmt::Expr(e)
                | Stmt::Return(Some(e))
                | Stmt::Give(e) => {
                    self.collect_defined_domains_expr(e, escaping, seen, out);
                }
                Stmt::Assert { cond, message, .. } => {
                    self.collect_defined_domains_expr(cond, escaping, seen, out);
                    if let Some(m) = message {
                        self.collect_defined_domains_expr(m, escaping, seen, out);
                    }
                }
                Stmt::If(c, t, e) => {
                    self.collect_defined_domains_expr(c, escaping, seen, out);
                    self.collect_defined_domains_block(t, escaping, seen, out);
                    self.collect_defined_domains_block(e, escaping, seen, out);
                }
                Stmt::While(_, c, b) => {
                    self.collect_defined_domains_expr(c, escaping, seen, out);
                    self.collect_defined_domains_block(b, escaping, seen, out);
                }
                Stmt::For(_, _, iter, b) => {
                    match iter {
                        Iter::Range { lo, hi, step, .. } => {
                            self.collect_defined_domains_expr(lo, escaping, seen, out);
                            self.collect_defined_domains_expr(hi, escaping, seen, out);
                            if let Some(s) = step {
                                self.collect_defined_domains_expr(s, escaping, seen, out);
                            }
                        }
                        Iter::RangeVal(e, _)
                        | Iter::Seq(e, _)
                        | Iter::Subsets(e)
                        | Iter::Members(e)
                        | Iter::Walk { len: e, .. } => {
                            self.collect_defined_domains_expr(e, escaping, seen, out);
                        }
                        Iter::TreeRange {
                            container, lo, hi, ..
                        } => {
                            self.collect_defined_domains_expr(container, escaping, seen, out);
                            if let Some((e, _)) = lo {
                                self.collect_defined_domains_expr(e, escaping, seen, out);
                            }
                            if let Some((e, _)) = hi {
                                self.collect_defined_domains_expr(e, escaping, seen, out);
                            }
                        }
                    }
                    self.collect_defined_domains_block(b, escaping, seen, out);
                }
                Stmt::Let(_, None)
                | Stmt::Return(None)
                | Stmt::Break(_)
                | Stmt::Continue(_)
                | Stmt::Trap(..) => {}
            }
        }
    }

    fn collect_defined_domains_expr(
        &self,
        expr: &Expr,
        escaping: &std::collections::HashSet<DomainId>,
        seen: &mut std::collections::HashSet<DomainId>,
        out: &mut Vec<DomainId>,
    ) {
        match &expr.kind {
            ExprKind::Do(block) => {
                self.collect_defined_domains_block(block, escaping, seen, out);
            }
            ExprKind::If(c, t, e) => {
                self.collect_defined_domains_expr(c, escaping, seen, out);
                self.collect_defined_domains_expr(t, escaping, seen, out);
                self.collect_defined_domains_expr(e, escaping, seen, out);
            }
            ExprKind::Unary(_, x)
            | ExprKind::Some(x)
            | ExprKind::Unwrap(x, _)
            | ExprKind::IsSome(x)
            | ExprKind::Field(x, _)
            | ExprKind::Cast(_, x)
            | ExprKind::IsVariant(x, _)
            | ExprKind::Payload(x, _, _) => {
                self.collect_defined_domains_expr(x, escaping, seen, out);
            }
            ExprKind::Binary(_, a, b)
            | ExprKind::Cmp(_, a, b)
            | ExprKind::Range { lo: a, hi: b } => {
                self.collect_defined_domains_expr(a, escaping, seen, out);
                self.collect_defined_domains_expr(b, escaping, seen, out);
            }
            ExprKind::Index(base, idx) => {
                self.collect_defined_domains_expr(base, escaping, seen, out);
                for i in idx {
                    self.collect_defined_domains_expr(i, escaping, seen, out);
                }
            }
            ExprKind::Fill(_, Some(v)) => {
                self.collect_defined_domains_expr(v, escaping, seen, out);
            }
            ExprKind::RangeCheck { value, lo, hi, .. } => {
                self.collect_defined_domains_expr(value, escaping, seen, out);
                if let Some(lo) = lo {
                    self.collect_defined_domains_expr(lo, escaping, seen, out);
                }
                if let Some(hi) = hi {
                    self.collect_defined_domains_expr(hi, escaping, seen, out);
                }
            }
            ExprKind::Tuple(xs)
            | ExprKind::Variant(_, xs)
            | ExprKind::Builtin(_, xs)
            | ExprKind::Mutate(_, _, xs) => {
                for x in xs {
                    self.collect_defined_domains_expr(x, escaping, seen, out);
                }
            }
            ExprKind::Call(_, args) => {
                for a in args {
                    if let Arg::Value(e) = a {
                        self.collect_defined_domains_expr(e, escaping, seen, out);
                    }
                }
            }
            ExprKind::CallExpr(callee, args) => {
                self.collect_defined_domains_expr(callee, escaping, seen, out);
                for a in args {
                    if let Arg::Value(e) = a {
                        self.collect_defined_domains_expr(e, escaping, seen, out);
                    }
                }
            }
            _ => {}
        }
    }
}

impl Program {
    /// The source name of the fn lowered as `name`, or `name` itself.
    pub fn shown_fn<'a>(&'a self, name: &'a str) -> &'a str {
        let at = self.fns.iter().position(|f| f.name == name);
        at.and_then(|i| self.fn_display.get(i))
            .map_or(name, String::as_str)
    }
}
