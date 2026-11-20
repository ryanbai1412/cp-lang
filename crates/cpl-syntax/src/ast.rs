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
    pub name: Name,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub decorators: Vec<Decorator>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StmtKind {
    /// `let pat: ty = init` or `var ...`.
    Let {
        mutable: bool,
        pat: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
    },
    Const {
        name: Name,
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
        body: Block,
    },
    Impl {
        trait_name: Option<Type>,
        target: Type,
        body: Block,
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
    },
    For {
        label: Option<Name>,
        pat: Pattern,
        iter: Expr,
        body: Block,
    },
    Return(Option<Expr>),
    Break(Option<Name>),
    Continue(Option<Name>),
    Yield(Expr),
    Assert {
        cond: Expr,
        message: Option<Expr>,
    },
    Unchecked(Block),
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
    pub params: Vec<Param>,
    pub ret: Option<Type>,
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
    /// After `;` in the parameter list: can only be passed by name.
    pub named_only: bool,
    pub name: Name,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldDecl {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
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
    /// `if let v = find():`.
    Let { pat: Pattern, value: Expr },
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
    Paren(Box<Expr>),
    Tuple(Vec<Expr>),
    Record(Vec<(Name, Expr)>),
    List(Vec<Expr>),
    /// `[body for pat in iter if cond]`, or the same without brackets as
    /// the only argument of a call: `max(dist[v] for v in V)`.
    Comprehension {
        body: Box<Expr>,
        pat: Pattern,
        iter: Box<Expr>,
        cond: Option<Box<Expr>>,
    },
    /// `[V] u => deg[u] * 2`.
    Table {
        domains: Vec<Type>,
        params: Vec<Pattern>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<Pattern>,
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
    /// Postfix `?`.
    Try(Box<Expr>),
    /// Postfix `!`: the value of an optional that is known to be there.
    Unwrap(Box<Expr>),
    /// An expression was expected and nothing was there. Already reported.
    Missing,
    /// Tokens the parser could not read as an expression. Already reported.
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StrPiece {
    Text(String),
    Expr(Expr),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Arm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Block,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnOp {
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
    Tuple(Vec<Pattern>),
    /// `Add(l, r)` or a bare variant such as `L`.
    Variant { path: Vec<Name>, args: Vec<Pattern> },
    Literal(Box<Expr>),
    Or(Vec<Pattern>),
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
    Named { name: Name, args: Vec<Type> },
    /// `[V]Int`, `[Row, Col]Char`.
    Array { index: Vec<Type>, elem: Box<Type> },
    /// `[domain S]Char`: an array that brings its own domain.
    OwnDomain { name: Name, elem: Box<Type> },
    Optional(Box<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(Name, Type)>),
    /// `0..=1e9`, `1..=n`.
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
        inclusive: bool,
    },
    Fn { params: Vec<Type>, ret: Box<Type> },
    Error,
}
