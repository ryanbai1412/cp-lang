//! The syntax tree.
//!
//! Every span in a tree is relative to the start of the top-level item it
//! belongs to. An edit above an item moves it without changing its text, so
//! its tree, and everything computed from it, is reused as is.
//!
//! The tree is always complete. Where the source is broken the parser puts
//! `Missing` or `Error` nodes, which the checker gives the error type and
//! never reports on again.

use cpl_diag::Span;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

impl Name {
    /// True for names that are types or domains by the naming rule
    /// (`PascalCase`), which the parser relies on to read `Name<...>`.
    pub fn is_type_like(&self) -> bool {
        self.text.starts_with(|c: char| c.is_ascii_uppercase())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Decorator {
    /// `@!name`: a file-level directive, in force from its line down. A
    /// plain `@name` decorates the one item below it.
    pub directive: bool,
    pub name: Name,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub decorators: Vec<Decorator>,
    /// Declared with `pub`: visible to modules that `use` this one.
    pub public: bool,
}

/// `T: Ord + Bounded`, from a type parameter list or a `where` clause.
#[derive(Clone, PartialEq, Debug)]
pub struct Bound {
    pub name: Name,
    pub traits: Vec<Type>,
}

/// `lends T`, `lends mut T`, or `lends(a) T` in a projection's signature.
#[derive(Clone, PartialEq, Debug)]
pub struct Lends {
    pub sources: Vec<Name>,
    pub mutable: bool,
    pub ty: Type,
    /// The `lends` keyword, when written without the `->` before it.
    pub bare: Option<Span>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StmtKind {
    /// `let pat: ty = init` or `var ...`.
    Let {
        mutable: bool,
        pat: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
        /// `let v = u else: return`: runs when the value is `none`.
        otherwise: Option<Block>,
        /// `let k: 0..n = read() where k < n`: a refinement of the input.
        guard: Option<Expr>,
    },
    Const {
        name: Name,
        /// `const Sum<T> = ... where T: Add`: one value per choice of `T`.
        generics: Vec<Name>,
        bounds: Vec<Bound>,
        ty: Option<Type>,
        /// Absent only in a trait, where the implementor supplies it.
        init: Option<Expr>,
    },
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Domain {
        name: Name,
        def: Expr,
    },
    TypeAlias {
        name: Name,
        generics: Vec<Name>,
        ty: Type,
    },
    Trait {
        name: Name,
        generics: Vec<Name>,
        /// `trait Seq: Index`.
        supers: Vec<Type>,
        body: Block,
        /// `trait Both = Sh + Nm`: a name for its bounds, never implemented.
        alias: bool,
    },
    Impl {
        trait_name: Option<Type>,
        target: Type,
        bounds: Vec<Bound>,
        body: Block,
    },
    /// `law assoc(a: T, b: T) = op(a, b) == op(b, a)` in a struct, or
    /// `law disjoint(i, j) = i != j` in a projection.
    Law {
        name: Name,
        params: Vec<Param>,
        body: Expr,
    },
    /// `lend place` or `lend mut place`, the last statement of a projection.
    Lend {
        mutable: bool,
        place: Expr,
    },
    Use {
        path: Vec<Name>,
        /// `use a.b.{X, Y}`.
        names: Vec<Name>,
        glob: bool,
        alias: Option<Name>,
    },
    Expr(Expr),
    Assign {
        target: Expr,
        /// `=`, `+=`, `max=`, ...
        op: AssignOp,
        value: Expr,
    },
    While {
        cond: Cond,
        body: Block,
        else_body: Option<Block>,
    },
    For {
        label: Option<Name>,
        pat: Pattern,
        iter: Expr,
        body: Block,
        else_body: Option<Block>,
    },
    Return(Option<Expr>),
    Break(Option<Name>),
    Continue(Option<Name>),
    /// `yield x`, or `yield from xs` to yield each value of `xs`.
    Yield {
        value: Expr,
        from: bool,
    },
    Assert {
        cond: Expr,
        message: Option<Expr>,
    },
    Unchecked(Block),
    /// `with io(sep: ",")`: a context for `body`, or for the rest of the
    /// enclosing block when written bare. (R09.1.1)
    With {
        ctx: Name,
        args: Vec<Arg>,
        body: Option<Block>,
    },
    /// `context opt:\n    k: Int = 2`: user-defined context declaration.
    Context {
        name: Name,
        fields: Vec<FieldDecl>,
    },
    /// A statement the parser could not read. Already reported.
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssignOp {
    Set,
    Binary(BinOp),
    Max,
    Min,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub bounds: Vec<Bound>,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    /// `fn at(self, i: V) lends mut Int`: a projection.
    pub lends: Option<Lends>,
    /// `fn pick() -> Int uses opt`: contexts used by this fn.
    pub uses: Vec<Name>,
    pub body: FnBody,
    /// From `fn` to the end of the signature.
    pub sig_span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FnBody {
    Block(Block),
    /// `fn sq(x: Int) -> Int = x * x`.
    Expr(Expr),
    /// A declaration without a body, as in a trait.
    None,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Param {
    pub mutable: bool,
    pub variadic: bool,
    /// `lazy f: () -> T`: a caller passes an expression of type `T`, and it
    /// becomes `() => expr`.
    pub lazy: bool,
    /// After `;` in the parameter list: can only be passed by name.
    pub named_only: bool,
    /// Optional external call-site label (`while ok: (K) -> Bool`), so any
    /// function can declare a keyword-labelled parameter without shadowing.
    pub label: Option<Name>,
    pub name: Name,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub span: Span,
}

impl Param {
    /// Name callers write when passing this parameter by name.
    pub fn label_text(&self) -> &str {
        self.label
            .as_ref()
            .map_or(self.name.text.as_str(), |l| l.text.as_str())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub generic_defaults: Vec<Option<Type>>,
    pub bounds: Vec<Bound>,
    /// Header fields (`struct Monoid<T>(id: T, op: (T, T) -> T)`) first,
    /// then the fields of the body.
    pub fields: Vec<FieldDecl>,
    /// Everything in the body that is not a field: fns, laws, owned
    /// domains, constants.
    pub methods: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldDecl {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
    /// `var top = 0`: assignable after construction.
    pub mutable: bool,
    /// `@cache parent: ...`: interior-mutable cache field, skipped by `==`.
    pub cached: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub variants: Vec<Variant>,
    pub methods: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variant {
    pub name: Name,
    pub fields: Vec<(Option<Name>, Type)>,
    pub value: Option<Expr>,
    pub tag: Option<Expr>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Arg {
    pub name: Option<Name>,
    pub mutable: bool,
    pub value: Expr,
}

/// The condition of an `if` or `while`.
#[derive(Clone, PartialEq, Debug)]
pub enum Cond {
    Expr(Expr),
    /// `if let v = find():`, `if let b = before[c] and b < d:`,
    /// `if let a = i - 1 and let b = j - 1:`.
    Let {
        pat: Pattern,
        value: Expr,
        /// What must also hold, with the pattern's names in scope: a
        /// condition, or another `let`.
        and: Option<Box<Cond>>,
    },
    /// `if a == 1, let b = p:`: a test, then after the `,` what must
    /// also hold.
    Guard {
        test: Expr,
        and: Box<Cond>,
    },
    /// `if const D.max_len <= 64:`.
    Const(Expr),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExprKind {
    Int(i128),
    Float(String),
    Str(Vec<StrPiece>),
    Char(char),
    Bool(bool),
    None,
    Name(String),
    SelfValue,
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `lo <= x < hi`: operands and the operators between them.
    Compare {
        first: Box<Expr>,
        rest: Vec<(CmpOp, Expr)>,
    },
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        inclusive: bool,
        step: Option<Box<Expr>>,
    },
    /// `(lo..hi).rev by step`: a walk taken every `step`-th value, from
    /// where it starts.
    By {
        walk: Box<Expr>,
        step: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
    },
    Index {
        base: Box<Expr>,
        indices: Vec<Expr>,
    },
    /// `base.name`. The name is empty while the user is still typing it.
    Field {
        base: Box<Expr>,
        name: Name,
    },
    /// `Name<T, U>` or `name::<T>` in an expression.
    Generic {
        base: Box<Expr>,
        args: Vec<Type>,
    },
    /// `best min= d` where a value is wanted: a `Bool`, true when the
    /// place changed. As a statement it is a [`StmtKind::Assign`].
    Keep {
        target: Box<Expr>,
        max: bool,
        value: Box<Expr>,
    },
    Paren(Box<Expr>),
    Tuple(Vec<Expr>),
    Record(Vec<(Name, Expr)>),
    List(Vec<Expr>),
    /// `[3: 4, 5: 6]`.
    Map(Vec<(Expr, Expr)>),
    /// `[body for pat in iter if cond]`, or the same without brackets as
    /// the only argument of a call: `max(dist[v] for v in V)`.
    Comprehension {
        body: Box<Expr>,
        /// One per `for`, outermost first.
        clauses: Vec<Clause>,
    },
    /// `[V] u => deg[u] * 2`.
    Table {
        domains: Vec<Type>,
        params: Vec<Pattern>,
        body: Box<Expr>,
    },
    /// `[w.keys]0` or `[0..n]0`: table over a key space filled with a constant value.
    TableFill {
        domain: Box<Expr>,
        value: Box<Expr>,
    },
    Lambda {
        params: Vec<Pattern>,
        param_tys: Vec<Option<Type>>,
        /// `(mut d, e) => ..`: empty unless some parameter is `mut`.
        param_mut: Vec<bool>,
        ret_ty: Option<Type>,
        body: Box<Expr>,
    },
    /// `{ let k = key(a); (k.x, -k.y) }`.
    Block(Block),
    If {
        branches: Vec<(Cond, Block)>,
        otherwise: Option<Block>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
    },
    /// `e with w: 7`.
    With {
        base: Box<Expr>,
        updates: Vec<(Name, Expr)>,
    },
    /// `(to: r.v, ..e without (u, v))` or `(..e without (u, v))`.
    Without {
        prefix: Vec<(Name, Expr)>,
        base: Box<Expr>,
        removed: Vec<Name>,
    },
    /// `if T: Trait:` compile-time trait bound test.
    HasBound {
        param: Name,
        trait_name: Name,
    },
    /// Postfix `?`.
    Try(Box<Expr>),
    /// Postfix `!`: the value of an optional that is known to be there.
    Unwrap(Box<Expr>),
    /// `mut place` in `let st = mut a[i]`: a named mutable alias to a place.
    Mut(Box<Expr>),
    /// An expression was expected and nothing was there. Already reported.
    Missing,
    /// Tokens the parser could not read as an expression. Already reported.
    Error,
}

/// `for pat in iter [if cond]` in a comprehension.
#[derive(Clone, PartialEq, Debug)]
pub struct Clause {
    pub pat: Pattern,
    pub iter: Expr,
    /// Each `if cond` or `if let pat = value` after the iterator.
    pub conds: Vec<Cond>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StrPiece {
    Text(String),
    Expr(Expr),
    /// `{x:.2f}`, `{n:05}`, `{n:x}`: a number in the format after the `:`.
    Spec(Expr, FmtSpec),
}

/// A number format: `W` (least width), `0W` (zero-filled), then `.D` or
/// `.Df` (digits after the point), `x` (hex) or `b` (binary).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FmtSpec {
    pub zero: bool,
    pub width: u32,
    pub kind: FmtKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FmtKind {
    Dec,
    Hex,
    Bin,
    Fixed(u32),
}

impl std::fmt::Display for FmtSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.zero { ":0" } else { ":" })?;
        if self.width > 0 {
            write!(f, "{}", self.width)?;
        }
        match self.kind {
            FmtKind::Dec => Ok(()),
            FmtKind::Hex => f.write_str("x"),
            FmtKind::Bin => f.write_str("b"),
            FmtKind::Fixed(places) => write!(f, ".{places}f"),
        }
    }
}

impl FmtSpec {
    /// The format `text` spells (what follows the `:`), if it is one.
    pub fn parse(text: &str) -> Option<FmtSpec> {
        let digits = |s: &str| s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let zero = text.starts_with('0') && text.len() > 1 && text.as_bytes()[1].is_ascii_digit();
        let rest = if zero { &text[1..] } else { text };
        let (width, rest) = rest.split_at(digits(rest));
        let kind = match rest {
            "" if !width.is_empty() => FmtKind::Dec,
            "x" => FmtKind::Hex,
            "b" => FmtKind::Bin,
            _ => {
                let places = rest.strip_prefix('.')?;
                let places = places.strip_suffix('f').unwrap_or(places);
                if places.is_empty() || digits(places) != places.len() {
                    return None;
                }
                FmtKind::Fixed(places.parse().ok().filter(|d| *d <= 300)?)
            }
        };
        let width = if width.is_empty() {
            0
        } else {
            width.parse().ok().filter(|w| *w <= 4096)?
        };
        Some(FmtSpec { zero, width, kind })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Arm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Block,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
    BitNot,
}

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
}

impl BinOp {
    pub fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Pow => "**",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::Coalesce => "??",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
    /// `a <=> b`: an `Ordering`.
    Order,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PatternKind {
    Wildcard,
    Name(String),
    /// `domain P` in a binding pattern: `let (domain P, part) = f(..)`.
    Domain(Name),
    Tuple(Vec<Pattern>),
    /// `Add(l, r)` or a bare variant such as `L`.
    Variant {
        path: Vec<Name>,
        args: Vec<Pattern>,
    },
    Literal(Box<Expr>),
    Or(Vec<Pattern>),
    /// `var x` inside a destructuring pattern: `for (var left, v) in ...:`.
    Var(Box<Pattern>),
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TypeKind {
    /// `Int`, `List<Int>`, `V`.
    Named {
        name: Name,
        args: Vec<Type>,
    },
    /// `[V]Int`, `[Row, Col]Char`.
    Array {
        index: Vec<Type>,
        elem: Box<Type>,
    },
    /// `[domain S]Char`: an array that brings its own domain.
    OwnDomain {
        name: Name,
        elem: Box<Type>,
    },
    Optional(Box<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(Name, Type)>),
    /// `0..=1e9`, `1..=n`, `0..`, `..=8`.
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        inclusive: bool,
    },
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    /// `yields T`: the return type of a generator.
    Yields(Box<Type>),
    /// A value in a type argument list: the `7` of `Mod<7>`.
    Value(Box<Expr>),
    /// `domain C` in a returned record, `-> (domain C, comp: [V]C)`: the
    /// fn makes a fresh domain and names it for the caller.
    NewDomain,
    /// `domain P` in a returned tuple, `-> (domain P, [P]Int)`: an
    /// existential fresh domain bound for later elements of the tuple.
    DomainBinder(Name),
    /// `(lo: Int, hi: Int, ..)`: open row bound on a record type parameter.
    Row(Vec<(Name, Type)>),
    /// `(to: V, ..R without (u, v))` or `(..R without (u, v))`: record type with named fields added and/or removed.
    RowWithout {
        prefix: Vec<(Name, Type)>,
        base: Box<Type>,
        removed: Vec<Name>,
    },
    /// `Str of "01"`: string whose characters belong to the given alphabet.
    ConstrainedStr(String),
    /// `'+' | '-'`: finite union of literal values.
    LiteralUnion(Vec<String>),
    /// A field written `total = 0`: its type is its default's.
    Infer,
    Error,
}
