//! Contract 4 and 5: one IR, used generic (`T = Ty`) and per instance (`T = CTy`).
//! After lowering, anything that can trap or has an effect is its own statement:
//! expressions are pure. That fixes evaluation order for every backend.
use crate::ty::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FuncKey {
    Main,
    Decl(FnId),
    Lambda(LambdaId),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intr {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Not,
    RawEmpty,
    RawNew,
    RawGet,
    RawSet,
    ReadInt,
}
impl Intr {
    /// The check this operation makes, if any. A checked operation is a statement of its own.
    pub fn check(self) -> Option<&'static str> {
        match self {
            Intr::Add | Intr::Sub | Intr::Mul | Intr::Neg => Some("overflow"),
            Intr::Div | Intr::Rem => Some("division by zero"),
            Intr::RawGet | Intr::RawSet | Intr::RawNew => Some("storage bounds"),
            Intr::ReadInt => Some("input"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    Local(usize),
    Global(usize),
    Field(Box<Place>, usize),
    /// The payload of member `i` of a function-value set. Canonical IR only.
    Variant(Box<Place>, usize),
    /// The variable a captured reference points at.
    Deref(Box<Place>),
}

#[derive(Clone, Debug)]
pub struct Expr<T> {
    pub kind: EK<T>,
    pub ty: T,
}
#[derive(Clone, Debug)]
pub enum EK<T> {
    Int(i64),
    Bool(bool),
    Unit,
    Copy(Place),
    /// A reference to a `var`, for a closure to capture. The closure is then lent.
    AddrOf(Place),
    MakeStruct(Vec<Expr<T>>),
    /// A closure value: its captures, in order. The closure is named by the type.
    MakeClosure(Vec<Expr<T>>),
    Inject(Box<Expr<T>>),
    Widen(Box<Expr<T>>),
    /// Canonical IR only: member `i` of a set, and the tag of a set value.
    MakeUnion(usize, Box<Expr<T>>),
    Tag(Place),
    /// The size of a domain type. Resolved by the link step to a variable or a hidden parameter.
    DomLen(T),
    Intr(Intr, Vec<Arg<T>>),
    Call(Target<T>, Vec<Arg<T>>),
    CallValue(Box<Expr<T>>, Vec<Expr<T>>),
}
#[derive(Clone, Debug)]
pub enum Arg<T> {
    Value(Expr<T>),
    Lend(Place),
}
#[derive(Clone, Debug)]
pub enum Target<T> {
    /// Generic IR: a declared fn with its type and set arguments.
    Fn(FnRef),
    /// Generic IR: a method of a bound, resolved per instance.
    Trait {
        tr: TraitId,
        method: String,
        self_ty: T,
    },
    /// Instance IR: the instance to call.
    Inst(usize),
}
#[derive(Clone, Debug)]
pub enum PrintPiece<T> {
    Text(String),
    Value(Expr<T>),
}
#[derive(Clone, Debug)]
pub enum Stmt<T> {
    Let(usize, Expr<T>),
    Assign(Place, Expr<T>),
    Expr(Expr<T>),
    If(Expr<T>, Vec<Stmt<T>>, Vec<Stmt<T>>),
    /// `Loop(body, step)`: `continue` jumps to `step`.
    Loop(Vec<Stmt<T>>, Vec<Stmt<T>>),
    Switch(Expr<T>, Vec<Vec<Stmt<T>>>),
    Break,
    Continue,
    Return(Option<Expr<T>>),
    Check {
        name: &'static str,
        cond: Expr<T>,
        msg: String,
        line: u32,
    },
    Print(Vec<PrintPiece<T>>),
    /// A call to a generator or a lender. `body` is the caller's code the callee runs
    /// at its `Yield` or `LendOut`, in the caller's frame, with `var` bound.
    /// Leaving `body` early: a generator stops there; a lender runs the rest of itself first.
    Region {
        target: Target<T>,
        args: Vec<Arg<T>>,
        var: usize,
        lend: bool,
        body: Vec<Stmt<T>>,
    },
    Yield(Expr<T>),
    LendOut(Place),
    /// A point a trace reports: a label and the values to show. Placed on instance IR, on request only.
    Event(String, Vec<Expr<T>>),
}

#[derive(Clone, Debug)]
pub struct Func<T> {
    pub key: FuncKey,
    pub name: String,
    /// Local index and whether it is lent (`mut`) by the caller.
    pub params: Vec<(usize, bool)>,
    /// For a lambda: how many leading params are captures.
    pub n_captures: usize,
    pub locals: Vec<T>,
    pub body: Vec<Stmt<T>>,
    pub ret: T,
    /// Set for a generator (`false`) or a lender (`true`): what it hands to the caller's code.
    pub region: Option<(bool, T)>,
    pub line: u32,
}
