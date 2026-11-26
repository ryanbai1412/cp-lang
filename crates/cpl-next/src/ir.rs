//! Contract 4 and 5: one IR, used generic (`T = Ty`) and per instance (`T = CTy`).
//! After lowering, anything that can trap or has an effect is its own statement:
//! expressions are pure. That fixes evaluation order for every backend.
use crate::ty::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FuncKey {
    Main,
    /// The code below `@!multitest`: one test case. A `return` in it ends the case.
    Case,
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
    Pow,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// `0 <= a < b`: whether `a` is an ordinal of a domain of `b` positions.
    Below,
    Not,
    RawEmpty,
    RawNew,
    RawGet,
    RawSet,
    /// A key check std makes before it reads: traps R0101 unless its operand is true (R07.2.17).
    CheckKey,
    ReadInt,
    /// Whether the input has another token.
    InputLeft,
    /// The character with a code the checker has shown is in `0..=127`.
    CharOf,
    /// A character's code.
    CodeOf,
    /// An `Int`'s two's-complement bits as a `U64`.
    ToU64,
    /// A `U64` as an `Int`; traps past `Int`.
    FromU64,
    /// A slot with no value yet, of the type it is given. Never read before it is stored to.
    Unset,
}
impl Intr {
    /// The storage operation a std fn declared `= @intrinsic(name)` is (R12.14.1).
    pub fn named(name: &str) -> Option<Intr> {
        Some(match name {
            "raw_empty" => Intr::RawEmpty,
            "raw_new" => Intr::RawNew,
            "raw_get" => Intr::RawGet,
            "raw_set" => Intr::RawSet,
            "key_check" => Intr::CheckKey,
            _ => return None,
        })
    }
    /// The arguments it stores, so a caller's owned heap value passed there moves in (R05.2.2).
    pub fn keeps(self) -> &'static [usize] {
        match self {
            Intr::RawSet => &[2],
            _ => &[],
        }
    }
    /// Whether this operation can trap. One that can is a statement of its own.
    pub fn checked(self) -> bool {
        use Intr::*;
        matches!(
            self,
            Add | Sub
                | Mul
                | Neg
                | Div
                | Rem
                | Pow
                | Shl
                | Shr
                | BitAnd
                | BitXor
                | RawGet
                | RawSet
                | CheckKey
                | RawNew
                | ReadInt
                | FromU64
        )
    }
}

/// Where an operation that can trap, or a call, is written: its line in the user's
/// file, or 0 inside std. A trap is reported at the innermost such line that is
/// running, so a check std makes fails at the line that called std.
pub type Site = u32;

/// What a `Check` statement checks, which decides the code its trap carries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckKind {
    Assert,
    /// A value becoming a member of a domain.
    Domain,
    /// A value stored into a slot of a range type.
    RangeStore,
    /// A value read from the input into a range type.
    ReadRange,
    /// `x!` on an optional.
    Unwrap,
}

impl CheckKind {
    /// The trap a failed check of this kind is, in the diagnostic registry.
    pub fn code(self) -> cpl_diag::Code {
        use cpl_diag::codes;
        match self {
            CheckKind::Assert => codes::ASSERTION_FAILED,
            CheckKind::Domain => codes::DOMAIN_CAST,
            CheckKind::RangeStore => codes::RANGE_STORE,
            CheckKind::ReadRange => codes::INPUT_OUT_OF_CONTRACT,
            CheckKind::Unwrap => codes::UNWRAP_NONE,
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
    /// Cell `i` of the raw storage at the place, `i` the value of that local.
    Slot(Box<Place>, usize),
    /// The value a lent slot (`CTy::Lent`) points at; made by instantiation only.
    Through(Box<Place>),
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
    /// A heap parameter stored in a struct the fn returns, which lends it (R05.5.3): the slot
    /// points at the caller's value instead of copying it. A plain value is copied.
    Lent(Place),
    /// The characters of a string literal, as storage.
    Text(String),
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
    /// The integer of a domain type's first position. Resolved like `DomLen`.
    DomLo(T),
    Intr(Intr, Vec<Arg<T>>, Site),
    Call(Target<T>, Vec<Arg<T>>, Site),
    CallValue(Box<Expr<T>>, Vec<Expr<T>>, Site),
}
#[derive(Clone, Debug)]
pub enum Arg<T> {
    Value(Expr<T>),
    /// A place the callee reads where it is. Nothing writes it while the call runs.
    View(Place),
    /// A place the callee may write.
    Lend(Place),
}
#[derive(Clone, Debug)]
pub enum Target<T> {
    /// Generic IR: a declared fn with its type and set arguments.
    Fn(FnRef),
    /// Generic IR: the code of one test case.
    Case,
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
        kind: CheckKind,
        cond: Expr<T>,
        msg: String,
        site: Site,
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
        site: Site,
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
