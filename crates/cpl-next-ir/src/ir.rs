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
            "rand_word" => Intr::RandWord,
            "read_byte" => Intr::ReadByte,
            "read_int" => Intr::ReadInt,
            "input_peek" => Intr::PeekByte,
            "token_check" => Intr::CheckToken,
            "input_check" => Intr::CheckInput,
            "input_left" => Intr::InputLeft,
            "code_of" => Intr::CodeOf,
            "u64_of" => Intr::ToU64,
            "int_of_u64" => Intr::FromU64,
            "i128_of" => Intr::ToI128,
            "int_of_i128" => Intr::FromI128,
            "f64_of_i128" => Intr::F64OfI128,
            "i128_of_u64" => Intr::I128OfU64,
            "u64_of_i128" => Intr::U64OfI128,
            "f64_of" => Intr::ToF64,
            "f64_bits" => Intr::F64Bits,
            "out" => Intr::Out,
            _ => return cpl_prim::Prim::from_id(name).map(Intr::of_prim),
        })
    }
    /// The operation of a primitive of `cpl-prim`'s one table.
    fn of_prim(p: cpl_prim::Prim) -> Intr {
        use cpl_prim::Prim as P;
        match p {
            P::Popcount => Intr::Popcount,
            P::Abs => Intr::Abs,
            P::Ctz => Intr::Ctz,
            P::Clz => Intr::Clz,
            P::BitWidth => Intr::BitWidth,
            P::BitCeil => Intr::BitCeil,
            P::Sqrt => Intr::Float(FloatFn::Sqrt),
            P::Sin => Intr::Float(FloatFn::Sin),
            P::Cos => Intr::Float(FloatFn::Cos),
            P::Tan => Intr::Float(FloatFn::Tan),
            P::Asin => Intr::Float(FloatFn::Asin),
            P::Acos => Intr::Float(FloatFn::Acos),
            P::Atan2 => Intr::Float(FloatFn::Atan2),
            P::Hypot => Intr::Float(FloatFn::Hypot),
            P::Exp => Intr::Float(FloatFn::Exp),
            P::Ln => Intr::Float(FloatFn::Ln),
            P::Floor => Intr::ToInt(Rounding::Floor),
            P::Ceil => Intr::ToInt(Rounding::Ceil),
            P::Round => Intr::ToInt(Rounding::Round),
            P::Trunc => Intr::ToInt(Rounding::Trunc),
        }
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
    /// An `Int` key becoming a position of the domain an array is keyed by (R07.2.17).
    Key,
    /// What the checker could not type: it fails wherever it runs.
    Untyped,
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
    /// For a lambda: the fn it is written in, `None` at top level.
    pub maker: Option<FnId>,
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
    /// Where each of its sites (`Site`) is, and the line of the user's file it is on (0 for
    /// site 0 and inside std).
    pub sites: Vec<crate::ids::SourceId>,
    pub lines: Vec<u32>,
    /// What a trap's call stack calls it: `f`, `Box.at`, `top level`.
    pub shown: String,
    /// The operation the engines and C++ run in place of a call (`@fast(op)` on a fn with
    /// a body); the reference interpreter runs the body. The body is what the fn means: the
    /// op only computes it faster, and `fast_ops_agree_with_their_bodies` checks the two
    /// equal on every runner.
    pub fast: Option<Intr>,
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
            Stmt::Event(_, xs) => xs.iter_mut().for_each(|x| expr(x, f)),
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
        Stmt::Event(_, vs) => vs.iter().any(|v| touches_in(v, l, all)),
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
