//! Contract 4 and 5: one IR, used generic (`T = Ty`) and per instance (`T = CTy`).
//! After lowering, anything that can trap or has an effect is its own statement:
//! expressions are pure. That fixes evaluation order for every backend.
use crate::ty::*;
use std::sync::Arc;

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
    /// The absolute value of an `Int` or `F64`; traps on `Int.MIN`.
    Abs,
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
    /// The input's next byte, or `-1` at its end (`ops::Input::byte`).
    ReadByte,
    /// The input's byte `k` past the next, taking nothing (`ops::Input::peek`).
    PeekByte,
    /// A check a token read makes: traps R0501 unless a token is there (R10.2.6).
    CheckToken,
    /// A check a read makes of its input: traps R0502 unless it holds (R10.2.2).
    CheckInput,
    /// Writes the first `n` characters of storage to the output.
    Out,
    /// The `F64` nearest the number the first `n` characters of storage spell, a token
    /// `Read for F64` has checked (`ops::f64_token`, R10.2.6).
    ParseF64,
    /// Hands the output written so far to the host now, not at exit.
    Flush,
    /// The next word of the program's one random stream (`ops::rand_word`).
    RandWord,
    /// The character with a code; traps R0201 outside `0..=127`.
    CharOf,
    /// A character's code.
    CodeOf,
    /// An `Int`'s two's-complement bits as a `U64`.
    ToU64,
    /// A `U64` as an `Int`; traps past `Int`.
    FromU64,
    /// An `Int` as an `I128`.
    ToI128,
    /// An `I128` as an `Int`; traps past `Int`.
    FromI128,
    /// The `F64` nearest an `I128`.
    F64OfI128,
    /// A `U64` as an `I128`.
    I128OfU64,
    /// An `I128` as a `U64`; traps below 0 or past `2^64 - 1`.
    U64OfI128,
    /// A 64-bit word's 1 bits, an `Int` as its two's complement.
    Popcount,
    /// A word's 0 bits below its lowest 1 bit; 64 for 0.
    Ctz,
    /// A word's 0 bits above its highest 1 bit; 64 for 0.
    Clz,
    /// The bits needed to write a word, `64 - clz`.
    BitWidth,
    /// The smallest power of two at least an `Int`, 1 at or below 1; past `2^62` it traps.
    BitCeil,
    /// The `F64` nearest an `Int`.
    ToF64,
    /// An `F64`'s bits, as a `U64`: what std's `Show` writes it from.
    F64Bits,
    /// An `F64` fn of one or two `F64`s: `sqrt`, `sin`, `atan2`, ...
    Float(FloatFn),
    /// An `F64` rounded to an `Int`; traps outside `Int`.
    ToInt(Rounding),
    /// A slot with no value yet, of the type it is given. Never read before it is stored to.
    Unset,
    /// `Int` arithmetic where overflow checks are off: it wraps to 64 bits (R09.4.2).
    AddWrap,
    SubWrap,
    MulWrap,
    PowWrap,
    ShlWrap,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FloatFn {
    Sqrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan2,
    Hypot,
    Exp,
    Ln,
}
impl FloatFn {
    /// The C++ runtime's row for the fn (`ops`), which every engine runs.
    pub fn cpp(self) -> &'static str {
        match self {
            FloatFn::Sqrt => "cpl_sqrt",
            FloatFn::Sin => "cpl_sin",
            FloatFn::Cos => "cpl_cos",
            FloatFn::Tan => "cpl_tan",
            FloatFn::Asin => "cpl_asin",
            FloatFn::Acos => "cpl_acos",
            FloatFn::Atan2 => "cpl_atan2",
            FloatFn::Hypot => "cpl_hypot",
            FloatFn::Exp => "cpl_exp",
            FloatFn::Ln => "cpl_ln",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rounding {
    Floor,
    Ceil,
    /// Halves away from zero.
    Round,
    Trunc,
}
impl Rounding {
    pub fn cpp(self) -> &'static str {
        match self {
            Rounding::Floor => "cpl_floor",
            Rounding::Ceil => "cpl_ceil",
            Rounding::Round => "cpl_round",
            Rounding::Trunc => "cpl_trunc",
        }
    }
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
    /// The operation a std fn declared `= @intrinsic(name)` is (R12.14.1): its row of
    /// [`crate::ops::INTRINSICS`].
    pub fn named(name: &str) -> Option<Intr> {
        crate::ops::Intrinsic::named(name).and_then(crate::ops::Intrinsic::op)
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
    /// Whether it reads or writes state outside its operands, so it is never dropped or moved.
    pub fn effectful(self) -> bool {
        matches!(
            self,
            Intr::RawSet
                | Intr::RawTake
                | Intr::ReadInt
                | Intr::ReadByte
                | Intr::PeekByte
                | Intr::Out
                | Intr::Flush
                | Intr::RandWord
        )
    }
    pub fn checked(self) -> bool {
        use Intr::*;
        matches!(
            self,
            Add | Sub
                | Mul
                | Neg
                | Abs
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
                | CheckToken
                | CheckInput
                | RawNew
                | ReadInt
                | FromU64
                | FromI128
                | F64OfI128
                | U64OfI128
                | CharOf
                | BitCeil
                | ToInt(_)
        )
    }
}

/// A node of the user's file that an operation that can trap, a call or a region stands for:
/// an index into its fn's site tables (`Func::sites`, `Func::origins`), so a node is its fn and
/// a local index. Site 0 is code with no place in the user's file (std, derived impls). A trap is reported at the
/// innermost site with a place that is running, so a check std makes fails at the line that
/// called std. Every instance of a generic fn keeps its sites.
pub type Site = u32;

/// What an event point reports (contract 8's `Semantic`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// Its fn's body starts; the values are its shown parameters.
    Enter,
    /// Its fn returns; the value is its shown result, if any.
    Exit,
    /// Top-level binding `global` of the user's file was written; the value is what it holds.
    /// `shown` is only what a trace calls it.
    Write { global: u32, shown: Arc<str> },
}

impl EventKind {
    /// What a trace line calls it, inside fn `f`.
    pub fn label(&self, f: &str) -> String {
        match self {
            EventKind::Enter => format!("enter {f}"),
            EventKind::Exit => format!("exit {f}"),
            EventKind::Write { shown, .. } => format!("set {shown}"),
        }
    }
}

/// An event point: what it reports, where, and the values it shows. `func` and `site` name
/// the statement it belongs to, `func`'s `sites[site]`, wherever the statement is moved to.
#[derive(Clone, Debug)]
pub struct EventPoint<T> {
    pub kind: EventKind,
    pub func: u32,
    pub site: Site,
    pub vals: Vec<Expr<T>>,
}

impl<T> EventPoint<T> {
    /// What a trace line calls it.
    pub fn label<U>(&self, funcs: &[Func<U>]) -> String {
        self.kind.label(&funcs[self.func as usize].name)
    }
}

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
    /// An `Int` key becoming a position of the domain an array is keyed by (R07.2.17).
    Key,
    /// What the checker could not type: it fails wherever it runs.
    Untyped,
}

impl CheckKind {
    /// The check's name in `with checks(..)` (R09.3); `None` for one no build turns off.
    pub fn name(self) -> Option<crate::build::CheckName> {
        use crate::build::CheckName as N;
        match self {
            CheckKind::Assert => Some(N::Asserts),
            CheckKind::Domain | CheckKind::Key => Some(N::Bounds),
            CheckKind::RangeStore => Some(N::Ranges),
            CheckKind::ReadRange => Some(N::Reads),
            CheckKind::Unwrap => Some(N::Unwrap),
            CheckKind::Todo | CheckKind::Untyped => None,
        }
    }
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
            CheckKind::Key => codes::INDEX_OUT_OF_RANGE,
            CheckKind::Untyped => codes::UNTYPED_RAN,
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
    /// The node it was lowered from, 0 for one the compiler made.
    pub site: Site,
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
    /// How far a position of a domain type is held from its ordinal: a range's or `Int`'s are
    /// held as their integers, so its first one's; a declared domain's as ordinals, 0. Resolved
    /// by the link step to a constant.
    DomHeld(T),
    /// Whether a domain type is declared by a count alone (R10.2.2). Resolved by the link step
    /// to a constant.
    DomCounted(T),
    /// Generic IR only: whether a type meets a trait at its arguments (`if T: Tr<A>:`). The
    /// link step makes it a `Bool` and keeps only the branch it takes (R08.2.9).
    HasBound(T, TraitId, Vec<T>),
    Intr(Intr, Vec<Arg<T>>, Site),
    Call(Target<T>, Vec<Arg<T>>, Site),
    CallValue(Box<Expr<T>>, Vec<Arg<T>>, Site),
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
        /// The fn's own type arguments, after `Self` and the trait's.
        targs: Vec<T>,
        /// The call's result type, whose function types carry the sets it passes the impl.
        ret: T,
        /// The trait's fn's context copy, as `FnRef::ctx`: the impl's copy under the same values.
        ctx: u32,
    },
    /// Instance IR: the instance to call.
    Inst(usize),
    /// Generic IR: the closure a generator value holds (R07.6.2); its set picks the instance.
    Value(Box<Expr<T>>),
}
#[derive(Clone, Debug)]
pub enum Stmt<T> {
    Let(usize, Expr<T>),
    Assign(Place, Expr<T>),
    Expr(Expr<T>),
    If(Expr<T>, Vec<Stmt<T>>, Vec<Stmt<T>>),
    /// `Loop(body, step, site)`: `continue` jumps to `step`; `site` is the source loop's body.
    Loop(Vec<Stmt<T>>, Vec<Stmt<T>>, Site),
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
        /// The node of the source loop's body, where a message on the whole walk is placed.
        body_site: Site,
    },
    Yield(Expr<T>),
    LendOut(Place),
    /// Ownership's release of the value a place holds: at an overwrite, or of a discarded
    /// value's temporary. The place holds no value after it until it is set again.
    Drop(Place),
    /// A point a trace reports. Placed on instance IR, on request only.
    Event(EventPoint<T>),
    /// A condition that holds here, which lowering states for the facts (a `for` variable at
    /// or after its start). No backend runs it; the reference interpreter checks it.
    Fact(Expr<T>),
}

/// A bound test `if T: Trait<args>:` and whether the code it guards is its taken branch.
pub type Guard = (Ty, TraitId, Vec<Ty>, bool);

#[derive(Clone, Debug)]
pub struct Func<T> {
    pub key: FuncKey,
    pub name: String,
    /// Local index and whether it is lent (`mut`) by the caller.
    pub params: Vec<(usize, bool)>,
    /// For a lambda: how many leading params are captures.
    pub n_captures: usize,
    /// For a lambda: the copy of the fn it is written in, or the top level or case.
    pub maker: Option<FuncKey>,
    pub locals: Vec<T>,
    /// What a message calls each local: its name in the source, empty for a temp.
    pub names: Vec<String>,
    /// Which locals are `var`s: a `var` made from a place is its own copy (R05.3.1).
    pub vars: Vec<bool>,
    /// Per param, set by ownership: nothing writes it and it has a heap part, so the callee
    /// reads it where the caller has it instead of taking a copy (R1).
    pub views: Vec<bool>,
    /// Set by ownership: the locals the frame releases when the fn leaves, every one that
    /// may hold storage but the lent parameters.
    pub drops: Vec<usize>,
    pub body: Vec<Stmt<T>>,
    pub ret: T,
    /// Set for a generator (`false`) or a lender (`true`): what it hands to the caller's code.
    pub region: Option<(bool, T)>,
    /// Where it is written: its line in the user's file, 0 inside std (`Site`).
    pub line: u32,
    /// Where each of its sites (`Site`) is.
    pub sites: Vec<crate::ids::SourceId>,
    /// Per site, what a trap there names: its number in `Program::origins`, 0 for none (site 0,
    /// std, a derived impl). Lowering marks a site of the user's file 1, and link numbers them.
    pub origins: Vec<u32>,
    /// What a trap's call stack calls it: `f`, `Box.at`, `top level`.
    pub shown: String,
    /// The operation the engines and C++ run in place of a call (`@fast(op)` on a fn with
    /// a body); the reference interpreter runs the body. The body is what the fn means: the
    /// op only computes it faster, and `fast_ops_agree_with_their_bodies` checks the two
    /// equal on every runner.
    pub fast: Option<Intr>,
    /// For a lambda: the bound tests around where it is written, each with the branch it is
    /// in. An instance where one decides the other way never makes it (R08.2.9).
    pub guards: Vec<Guard>,
    /// Set by link on an instance: per local, what its type is in the generic fn.
    pub declared: Vec<Declared>,
}

/// What a local's type is in the generic fn an instance comes from, which instancing
/// substitutes away: link records it, so ownership reads the instance alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Declared {
    /// Its type holds a type parameter: each instance copies such a value itself (R05.2.2).
    pub of_param: bool,
    /// It is written as a fn value, not as a type parameter's type an instance makes one.
    pub fn_value: bool,
    /// For a struct of type parameters: its fields of a type parameter's type.
    pub param_fields: Vec<usize>,
}
impl Declared {
    /// What `t` declares, given each struct's declared field types.
    pub fn of(t: &Ty, fields: &dyn Fn(StructId) -> Vec<Ty>) -> Declared {
        fn has(t: &Ty) -> bool {
            match t {
                Ty::Param(_) => true,
                Ty::Struct(_, ts, _) => ts.iter().any(has),
                Ty::Raw(t) | Ty::Ref(t) | Ty::Mut(t) | Ty::RangeTo(t, ..) => has(t),
                _ => false,
            }
        }
        let param_fields = match t {
            Ty::Struct(s, ts, _)
                if !ts.is_empty() && ts.iter().all(|t| matches!(t, Ty::Param(_))) =>
            {
                (fields(*s).iter().enumerate())
                    .filter(|(_, f)| matches!(f, Ty::Param(_)))
                    .map(|(i, _)| i)
                    .collect()
            }
            _ => Vec::new(),
        };
        Declared {
            of_param: has(t),
            fn_value: matches!(t, Ty::Fn(..) | Ty::Clo(_)),
            param_fields,
        }
    }
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
                args(xs, f);
            }
            EK::Int(_)
            | EK::F64(_)
            | EK::Bool(_)
            | EK::Unit
            | EK::Text(_)
            | EK::DomLen(_)
            | EK::DomLo(_)
            | EK::DomHeld(_)
            | EK::DomCounted(_)
            | EK::HasBound(..) => {}
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
            Stmt::Loop(x, y, _) => {
                visit_places(x, f);
                visit_places(y, f);
            }
            Stmt::Switch(c, arms) => {
                expr(c, f);
                arms.iter_mut().for_each(|a| visit_places(a, f));
            }
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
            Stmt::LendOut(p) | Stmt::Drop(p) => place(p, f),
            Stmt::Event(EventPoint { vals: xs, .. }) => xs.iter_mut().for_each(|x| expr(x, f)),
            Stmt::Break | Stmt::Continue => {}
        }
    }
}

/// Whether `b` writes local `l` or lets anything else write it: assigns it or a part, lends
/// it, takes its address or lends it out.
pub fn writes<T>(b: &[Stmt<T>], l: usize) -> bool {
    touches(b, l, Touch::Any)
}
/// Whether `b` itself writes local `l`: assigns it or a part, lends it or lends it out. Taking
/// its address does not count.
pub fn sets<T>(b: &[Stmt<T>], l: usize) -> bool {
    touches(b, l, Touch::Set)
}
/// Whether `b` takes the address of local `l`, after which any call may write it.
pub fn addressed<T>(b: &[Stmt<T>], l: usize) -> bool {
    touches(b, l, Touch::Addr)
}
/// Which touches of a local count: writes, address-takings, or both.
#[derive(Clone, Copy, PartialEq)]
enum Touch {
    Set,
    Addr,
    Any,
}
fn touches<T>(b: &[Stmt<T>], l: usize, all: Touch) -> bool {
    let at = |p: &Place| all != Touch::Addr && p.local() == Some(l);
    b.iter().any(|s| match s {
        Stmt::Let(x, e) => (all != Touch::Addr && *x == l) || touches_in(e, l, all),
        Stmt::Assign(p, e) => at(p) || touches_in(e, l, all),
        Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) => touches_in(e, l, all),
        Stmt::Return(e) => e.as_ref().is_some_and(|e| touches_in(e, l, all)),
        Stmt::If(c, a, b) => touches_in(c, l, all) || touches(a, l, all) || touches(b, l, all),
        Stmt::Loop(a, b, _) => touches(a, l, all) || touches(b, l, all),
        Stmt::Switch(c, arms) => touches_in(c, l, all) || arms.iter().any(|a| touches(a, l, all)),
        Stmt::Check { cond, .. } => touches_in(cond, l, all),
        Stmt::Event(EventPoint { vals: vs, .. }) => vs.iter().any(|v| touches_in(v, l, all)),
        Stmt::Region { args, body, .. } => touches_args(args, l, all) || touches(body, l, all),
        Stmt::LendOut(p) | Stmt::Drop(p) => at(p),
        Stmt::Break | Stmt::Continue => false,
    })
}
fn touches_in<T>(e: &Expr<T>, l: usize, all: Touch) -> bool {
    match &e.kind {
        EK::AddrOf(p) | EK::Lent(p) => all != Touch::Set && p.local() == Some(l),
        EK::Intr(_, args, _) | EK::Call(_, args, _) => touches_args(args, l, all),
        EK::CallValue(f, xs, _) => touches_in(f, l, all) || touches_args(xs, l, all),
        EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().any(|x| touches_in(x, l, all)),
        EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => touches_in(x, l, all),
        _ => false,
    }
}
pub fn writes_args<T>(args: &[Arg<T>], l: usize) -> bool {
    touches_args(args, l, Touch::Any)
}
fn touches_args<T>(args: &[Arg<T>], l: usize, all: Touch) -> bool {
    args.iter().any(|a| match a {
        Arg::Value(v) => touches_in(v, l, all),
        Arg::Lend(p) => all != Touch::Addr && p.local() == Some(l),
        Arg::View(_) => false,
    })
}

impl Place {
    /// The local a place is part of.
    pub fn local(&self) -> Option<usize> {
        match self {
            Place::Local(l) => Some(*l),
            Place::Global(_) => None,
            Place::Field(b, _)
            | Place::Variant(b, _)
            | Place::Deref(b)
            | Place::Through(b)
            | Place::Slot(b, _) => b.local(),
        }
    }
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

/// Whether evaluating `e` has no effect and cannot trap, so it may run earlier or later
/// than written: lowering's sequencing and ownership's temporaries both order by it.
pub fn pure<T>(e: &Expr<T>) -> bool {
    match &e.kind {
        EK::Int(_)
        | EK::F64(_)
        | EK::Bool(_)
        | EK::Text(_)
        | EK::Unit
        | EK::Copy(_)
        | EK::Pass(..)
        | EK::AddrOf(_)
        | EK::Lent(_)
        | EK::Tag(_)
        | EK::DomLen(_)
        | EK::DomLo(_)
        | EK::DomHeld(_)
        | EK::DomCounted(_)
        | EK::HasBound(..) => true,
        EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().all(pure),
        EK::Inject(x) | EK::MakeUnion(_, x) => pure(x),
        // Widening is a branch once sets are known, so it is a statement of its own.
        EK::Widen(_) => false,
        EK::Intr(i, args, _) => {
            !i.checked()
                && !i.effectful()
                && args.iter().all(|a| matches!(a, Arg::Value(v) if pure(v)))
        }
        EK::Call(..) | EK::CallValue(..) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Taking a local's address lets a call write it, but is no write of the local itself.
    #[test]
    fn an_address_is_a_write_only_through_a_call() {
        let lent = |l| Expr {
            kind: EK::<()>::Lent(Place::Local(l)),
            ty: (),
            site: Site::default(),
        };
        let b = vec![Stmt::Expr(lent(0)), Stmt::Assign(Place::Local(1), lent(2))];
        assert!(writes(&b, 0) && addressed(&b, 0) && !sets(&b, 0));
        assert!(writes(&b, 1) && !addressed(&b, 1) && sets(&b, 1));
        assert!(!writes(&b, 3) && !sets(&b, 3));
    }
}
