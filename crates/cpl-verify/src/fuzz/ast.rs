//! The fuzzer's program model: the slice of the language the generator
//! writes, the reference evaluator runs and the printer turns into `.cp`.
//!
//! It grows with the generator. Everything here must have one meaning
//! that `plan/` pins down, because the evaluator in `eval.rs` is the
//! oracle the real backends are compared against.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Int,
    Bool,
    /// `List<Int>`.
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Min,
    Max,
    Abs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Var(String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// `if c: a else: b` as an expression.
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Builtin(Builtin, Vec<Expr>),
    /// `xs[i]`.
    Index(String, Box<Expr>),
    /// `xs.len`.
    Len(String),
    ListLit(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
}

impl AssignOp {
    pub fn symbol(self) -> &'static str {
        match self {
            AssignOp::Set => "=",
            AssignOp::Add => "+=",
            AssignOp::Sub => "-=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// `let name: ty = init`, or `var` when `mutable`.
    Let {
        name: String,
        ty: Ty,
        mutable: bool,
        init: Expr,
    },
    Assign {
        name: String,
        op: AssignOp,
        value: Expr,
    },
    IndexAssign {
        list: String,
        index: Expr,
        op: AssignOp,
        value: Expr,
    },
    Push {
        list: String,
        value: Expr,
    },
    /// `if` / `elif` arms and an optional `else`.
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        els: Option<Vec<Stmt>>,
    },
    ForRange {
        var: String,
        lo: Expr,
        hi: Expr,
        inclusive: bool,
        body: Vec<Stmt>,
    },
    ForIn {
        var: String,
        list: String,
        body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Print(Vec<Expr>),
    Return(Expr),
    /// The last expression of a fn body, returned without `return`.
    Tail(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    pub funcs: Vec<Func>,
    pub body: Vec<Stmt>,
}
