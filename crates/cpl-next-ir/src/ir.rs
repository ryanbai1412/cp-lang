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
    /// The `k`-th copy of a `uses` fn, checked under one set of context values (R09.1.4).
    Ctx(FnId, u32),
    Lambda(LambdaId),
}
impl FuncKey {
    /// The declared fn it is a copy of.
    pub fn decl(&self) -> Option<FnId> {
        match self {
            FuncKey::Decl(f) | FuncKey::Ctx(f, _) => Some(*f),
            _ => None,
        }
    }
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
    /// Moves cell `i` out of storage, leaving it unset until a `RawSet` fills it again.
    RawTake,
    /// Storage of `n` unset cells, each filled by a `RawSet` before it is read.
    RawCap,
    /// A key check std makes before it reads: traps R0101 unless its operand is true (R07.2.17).
    CheckKey,
    /// A range check std makes before it cuts a range: traps R0001 unless its operand is true.
    CheckRange,
    /// A check an enum conversion makes: traps R0102 unless its operand is true (R02.7).
    CheckVariant,
    ReadInt,
    /// Whether the input has another token.
    InputLeft,
    /// The character with a code; traps R0201 outside `0..=127`.
    CharOf,
    /// A character's code.
    CodeOf,
    /// An `Int`'s two's-complement bits as a `U64`.
    ToU64,
    /// A `U64` as an `Int`; traps past `Int`.
    FromU64,
    /// A 64-bit word's 1 bits, an `Int` as its two's complement.
    Popcount,
    /// A word's 0 bits below its lowest 1 bit; 64 for 0.
    Ctz,
    /// A word's 0 bits above its highest 1 bit; 64 for 0.
    Clz,
    /// The bits needed to write a word, `64 - clz`.
    BitWidth,
    /// The `F64` nearest an `Int`.
    ToF64,
    /// A slot with no value yet, of the type it is given. Never read before it is stored to.
    Unset,
    /// `Int` arithmetic where overflow checks are off: it wraps to 64 bits (R09.4.2).
    AddWrap,
    SubWrap,
    MulWrap,
    PowWrap,
    ShlWrap,
}
impl Intr {
    /// The operation that wraps where this one traps on overflow; itself if it cannot overflow.
    pub fn wrapping(self) -> Intr {
        match self {
            Intr::Add => Intr::AddWrap,
            Intr::Sub => Intr::SubWrap,
            Intr::Mul => Intr::MulWrap,
            Intr::Pow => Intr::PowWrap,
            Intr::Shl => Intr::ShlWrap,
            other => other,
        }
    }
    /// The storage operation a std fn declared `= @intrinsic(name)` is (R12.14.1).
    pub fn named(name: &str) -> Option<Intr> {
        Some(match name {
            "raw_empty" => Intr::RawEmpty,
            "raw_new" => Intr::RawNew,
            "raw_get" => Intr::RawGet,
            "raw_set" => Intr::RawSet,
            "move_out" => Intr::RawTake,
            "unset_cells" => Intr::RawCap,
            "key_check" => Intr::CheckKey,
            "range_check" => Intr::CheckRange,
            "value_check" => Intr::CheckVariant,
            "char_of" => Intr::CharOf,
            "code_of" => Intr::CodeOf,
            "u64_of" => Intr::ToU64,
            "int_of_u64" => Intr::FromU64,
            "popcount" => Intr::Popcount,
            "ctz" => Intr::Ctz,
            "clz" => Intr::Clz,
            "bit_width" => Intr::BitWidth,
            "f64_of" => Intr::ToF64,
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
    /// The arguments its result holds, so the result lends them (R05.5.3).
    pub fn lends(self) -> &'static [usize] {
        match self {
            Intr::RawGet => &[0],
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
                | RawTake
                | RawCap
                | CheckKey
                | CheckRange
                | CheckVariant
                | RawNew
                | ReadInt
                | FromU64
                | CharOf
        )
    }
}

/// A node of the user's file that an operation that can trap, a call or a region stands for:
/// an index into its fn's site tables (`Func::sites`, `Func::lines`), so a node is its fn and
/// a local index. Site 0 is code with no place in the user's file (std, derived impls). A trap is reported at the
/// innermost site with a place that is running, so a check std makes fails at the line that
/// called std. Every instance of a generic fn keeps its sites.
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
    /// `todo` reached (R01.6.7).
    Todo,
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
            CheckKind::Todo => codes::TODO_REACHED,
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
    /// The value a lent slot or closure capture (`Ty::Lent`) points at.
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
    /// An `F64` constant, by its bits.
    F64(u64),
    Bool(bool),
    Unit,
    Copy(Place),
    /// Instance IR before ownership only: a heap place passed to a call (R4, R11). Ownership
    /// makes it a copy taken here, or else a view from the call's start where `true`, or a read.
    Pass(Place, bool, Site),
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
        /// The trait's arguments, which pick among a type's impls of it (`Into<Char>`).
        args: Vec<T>,
        method: String,
        self_ty: T,
    },
    /// Instance IR: the instance to call.
    Inst(usize),
    /// Generic IR: the closure a generator value holds (R07.6.2); its set picks the instance.
    Value(Box<Expr<T>>),
}
#[derive(Clone, Debug)]
pub enum PrintPiece<T> {
    Text(String),
    Value(Expr<T>, crate::ops::Fmt),
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
    /// A condition that holds here, which lowering states for the facts (a `for` variable at
    /// or after its start). No backend runs it; the reference interpreter checks it.
    Fact(Expr<T>),
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
    /// Where it is written: its line in the user's file, 0 inside std (`Site`).
    pub line: u32,
    /// Where each of its sites (`Site`) is, and the line of the user's file it is on (0 for
    /// site 0 and inside std).
    pub sites: Vec<crate::ids::SourceId>,
    pub lines: Vec<u32>,
    /// What a trap's call stack calls it: `f`, `Box.at`, `top level`.
    pub shown: String,
}

/// Calls `f` on every place a block names, outermost first: read, written, lent, viewed or
/// addressed. A `let`'s local and a region's variable come as `Place::Local`, and a local
/// `f` leaves them is their new one.
pub fn visit_places<T>(b: &mut [Stmt<T>], f: &mut impl FnMut(&mut Place)) {
    fn local(l: &mut usize, f: &mut impl FnMut(&mut Place)) {
        let mut p = Place::Local(*l);
        f(&mut p);
        if let Place::Local(x) = p {
            *l = x;
        }
    }
    /// A place, then the locals its slots index by.
    fn place(p: &mut Place, f: &mut impl FnMut(&mut Place)) {
        f(p);
        let mut at = p;
        loop {
            at = match at {
                Place::Slot(b, i) => {
                    local(i, f);
                    b
                }
                Place::Field(b, _) | Place::Variant(b, _) | Place::Deref(b) | Place::Through(b) => {
                    b
                }
                Place::Local(_) | Place::Global(_) => break,
            };
        }
    }
    fn args<T>(xs: &mut [Arg<T>], f: &mut impl FnMut(&mut Place)) {
        for a in xs {
            match a {
                Arg::Value(v) => expr(v, f),
                Arg::View(p) | Arg::Lend(p) => place(p, f),
            }
        }
    }
    fn expr<T>(e: &mut Expr<T>, f: &mut impl FnMut(&mut Place)) {
        match &mut e.kind {
            EK::Copy(p) | EK::Pass(p, ..) | EK::AddrOf(p) | EK::Lent(p) | EK::Tag(p) => place(p, f),
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter_mut().for_each(|x| expr(x, f)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => expr(x, f),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => args(xs, f),
            EK::CallValue(g, xs, _) => {
                expr(g, f);
                xs.iter_mut().for_each(|x| expr(x, f));
            }
            EK::Int(_)
            | EK::F64(_)
            | EK::Bool(_)
            | EK::Unit
            | EK::Text(_)
            | EK::DomLen(_)
            | EK::DomLo(_) => {}
        }
    }
    for s in b {
        match s {
            Stmt::Let(l, e) => {
                local(l, f);
                expr(e, f);
            }
            Stmt::Assign(p, e) => {
                place(p, f);
                expr(e, f);
            }
            Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) | Stmt::Check { cond: e, .. } => {
                expr(e, f)
            }
            Stmt::Return(e) => e.iter_mut().for_each(|e| expr(e, f)),
            Stmt::If(c, x, y) => {
                expr(c, f);
                visit_places(x, f);
                visit_places(y, f);
            }
            Stmt::Loop(x, y) => {
                visit_places(x, f);
                visit_places(y, f);
            }
            Stmt::Switch(c, arms) => {
                expr(c, f);
                arms.iter_mut().for_each(|a| visit_places(a, f));
            }
            Stmt::Print(ps) => ps.iter_mut().for_each(|p| {
                if let PrintPiece::Value(v, _) = p {
                    expr(v, f)
                }
            }),
            Stmt::Region {
                args: xs,
                var,
                body,
                ..
            } => {
                args(xs, f);
                local(var, f);
                visit_places(body, f);
            }
            Stmt::LendOut(p) => place(p, f),
            Stmt::Event(_, xs) => xs.iter_mut().for_each(|x| expr(x, f)),
            Stmt::Break | Stmt::Continue => {}
        }
    }
}

impl Place {
    /// The variable a place is part of.
    pub fn root_mut(&mut self) -> &mut Place {
        match self {
            Place::Field(b, _)
            | Place::Variant(b, _)
            | Place::Deref(b)
            | Place::Slot(b, _)
            | Place::Through(b) => b.root_mut(),
            p => p,
        }
    }
}
