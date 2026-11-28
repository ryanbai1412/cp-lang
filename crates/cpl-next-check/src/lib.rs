//! Contract 3: declarations -> typed tree. Decides every type, target and
//! conversion. Nothing after this infers anything.
use crate::diag::Msg;
use crate::ir::CheckKind;
use crate::ty::*;
use cpl_diag::Span;
use cpl_next_decls::decls::{
    self, Decls, ImplDecl, Impls, ModId, Named, Owner, SelfMode, TraitRef,
};
use cpl_next_decls::sym::{Sym, Syms};
use cpl_next_decls::views::{self as views, Items, Scope, Shapes, Signatures, TypeTables};
use cpl_next_decls::{diag, kind_name, modules, records};
use cpl_next_ir::build::{Build, CheckName, Checks};
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_ir::{ir, ops, ty};
use cpl_next_syntax::ast::{self as next, AstNode};
use cpl_next_syntax::fall_through;
use cpl_next_syntax::ops::{BinOp, CmpOp, UnOp};
use std::collections::HashMap;
use std::collections::HashSet;

mod assign;
mod call;
mod domain;
mod expr;
mod free;
pub use free::{free_vars, group_free};
mod from_op;
mod index;
mod labels;
mod lambda;
mod list;
mod loops;
mod member;
mod memo;
mod oblige;
mod pattern;
mod place;
mod question;
mod range;
mod recipe;
mod stmt;
mod targs;
mod types;
pub use types::{ItemTypes, TyId, TypeFacts, TypeView};
mod unify;
pub use unify::show_ty;
mod variants;
mod zonk;

/// Why a type must meet a trait, which names the verdict when it does not.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Origin {
    /// A bound a fn or impl states.
    Bound,
    /// `T(x)`: `x` is converted by its impl of the `@role(convert)` trait (R04.2.28).
    Convert,
    /// `T(x)` where `T`'s `new` takes nothing: without a conversion, `x` is one argument too
    /// many (R02.6.3, R07.8.1).
    Construct,
}

pub use cpl_next_ir::whole::VarRef;

#[derive(Clone, Debug)]
pub struct TExpr {
    pub kind: TK,
    pub ty: Ty,
    pub span: Span,
}
impl TExpr {
    /// Calls `f` on this expression and then on each expression inside it, in source order.
    pub fn each<'a>(&'a self, f: &mut dyn FnMut(&'a TExpr)) {
        f(self);
        let arg = |a: &'a TArg| match a {
            TArg::Value(e) | TArg::Mut(e) => e,
        };
        match &self.kind {
            TK::Int(_) | TK::Float(_) | TK::Bool(_) | TK::Unit | TK::Var(_) | TK::FnItem(_) => {}
            TK::Itself(_) | TK::ReadInt | TK::DomLen(_) | TK::DomCounted(_) => {}
            TK::HasBound(..) | TK::Text(_) | TK::Unset | TK::Error => {}
            TK::Call { recv, args, .. } | TK::CallTrait { recv, args, .. } => {
                (recv.iter().map(|r| arg(r)))
                    .chain(args.iter().map(arg))
                    .for_each(|e| e.each(f));
            }
            TK::CallValue { callee, args } => {
                callee.each(f);
                args.iter().for_each(|a| arg(a).each(f));
            }
            TK::Struct(es) => es.iter().for_each(|e| e.each(f)),
            TK::Lambda { body, .. } => body.each(f),
            TK::Field(e, _) | TK::Inject(e) | TK::Widen(e) | TK::Owned(e) | TK::Pack(e) => {
                e.each(f)
            }
            TK::Elem(e) | TK::Not(e) | TK::Neg(e) | TK::ToU64(e) | TK::ToI128(e) => e.each(f),
            TK::DomFrom(_, e, _) | TK::CutOrd(_, e) | TK::DomNth(_, e) | TK::DomHas(_, e) => {
                e.each(f)
            }
            TK::DomOrd(e) | TK::CharOf(e) | TK::CodeOf(e) | TK::Ordinal(e) => e.each(f),
            TK::Variant(_, e) | TK::Tag(e) | TK::Payload(e, _) => e.each(f),
            TK::Slot(a, b) | TK::Bin(_, a, b) | TK::Cmp(_, a, b) => {
                a.each(f);
                b.each(f);
            }
            TK::If(c, t, e) => [c, t, e].iter().for_each(|x| x.each(f)),
            TK::Block(stmts, tail) => {
                stmts.iter().for_each(|s| s.each(f));
                tail.iter().for_each(|e| e.each(f));
            }
        }
    }
}

impl TStmt {
    /// Calls `f` on each expression in this statement, in source order, as [`TExpr::each`].
    pub fn each<'a>(&'a self, f: &mut dyn FnMut(&'a TExpr)) {
        match self {
            TStmt::Let(_, e) | TStmt::Expr(e) | TStmt::Yield(e) | TStmt::LendOut(e) => e.each(f),
            TStmt::Assert(e, ..) => e.each(f),
            TStmt::Assign(a, b) | TStmt::While(a, b) => {
                a.each(f);
                b.each(f);
            }
            TStmt::For(_, a, b, c) => [a, b, c].iter().for_each(|e| e.each(f)),
            TStmt::Return(e) => e.iter().for_each(|e| e.each(f)),
            TStmt::Break | TStmt::Continue | TStmt::Item(_) | TStmt::Checks(_) => {}
            TStmt::Cases(e, stmts) => {
                e.iter().for_each(|e| e.each(f));
                stmts.iter().for_each(|s| s.each(f));
            }
            TStmt::Region {
                recv, args, body, ..
            } => {
                for a in recv.iter().map(|r| &**r).chain(args) {
                    let (TArg::Value(e) | TArg::Mut(e)) = a;
                    e.each(f);
                }
                body.each(f);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TK {
    Int(i64),
    /// An `F64` literal, by its bits.
    Float(u64),
    Bool(bool),
    Unit,
    Var(VarRef),
    Field(Box<TExpr>, usize),
    Call {
        f: FnRef,
        recv: Option<Box<TArg>>,
        args: Vec<TArg>,
    },
    /// A fn of a bound, chosen per instance by `self_ty`: the receiver's type, or `T` in
    /// `T.zero`, a fn without `self` (`recv: None`).
    CallTrait {
        tr: TraitRef,
        method: String,
        recv: Option<Box<TArg>>,
        self_ty: Box<Ty>,
        /// The fn's own type arguments, after `Self` and the trait's (`S` of `fn mask<S>`).
        targs: Box<[Ty]>,
        args: Vec<TArg>,
        /// The trait's fn's context copy the call makes, as `FnRef::ctx` (R09.1.3).
        ctx: u32,
    },
    CallValue {
        callee: Box<TExpr>,
        args: Vec<TArg>,
    },
    Struct(Vec<TExpr>),
    Lambda {
        id: LambdaId,
        params: Vec<LocalId>,
        body: Box<TExpr>,
    },
    FnItem(FnRef),
    /// A lambda's own value, named in its body by the `let` it is bound to (R06.2.3).
    Itself(LambdaId),
    /// A closure flows into a function-value slot.
    Inject(Box<TExpr>),
    /// A function value flows into a slot whose set is larger.
    Widen(Box<TExpr>),
    /// A value read into a temporary of its own: a heap place is copied, not lent (R01.6.4).
    Owned(Box<TExpr>),
    /// A variadic `..xs` argument's tuple or record: like any argument, each place it packs
    /// is lent for the call, not moved in.
    Pack(Box<TExpr>),
    /// `s[i]`: the element its `Indexed.at` call reads, in place in `s` (R05.3.5).
    Elem(Box<TExpr>),
    /// `buf[i]` on core's raw storage: cell `i`, a place a container's `IndexedMut` lends (R05.3.5).
    Slot(Box<TExpr>, Box<TExpr>),
    Bin(BinOp, Box<TExpr>, Box<TExpr>),
    Cmp(CmpOp, Box<TExpr>, Box<TExpr>),
    Not(Box<TExpr>),
    Neg(Box<TExpr>),
    /// `U64(x)`: an `Int`'s two's-complement bits.
    ToU64(Box<TExpr>),
    /// An `Int` beside an `I128`, widened to one.
    ToI128(Box<TExpr>),
    If(Box<TExpr>, Box<TExpr>, Box<TExpr>),
    Block(Vec<TStmt>, Option<Box<TExpr>>),
    ReadInt,
    /// The size of a domain type: `D.count`.
    DomLen(Ty),
    /// Whether a domain type is declared by a count alone: `D.counted` (R10.2.2).
    DomCounted(Ty),
    /// `D(i)`: an Int becomes a value of the domain, checked.
    DomFrom(Ty, Box<TExpr>, CheckKind),
    /// An Int as a cut of the domain: how many positions come before it, checked (R04.4.25).
    CutOrd(Ty, Box<TExpr>),
    /// `D.nth(k)`: the position `k` steps into `D`, checked; of a type that is not a domain, `k`.
    DomNth(Ty, Box<TExpr>),
    /// `x in D`: whether the integer `x` is a position of `D`.
    DomHas(Ty, Box<TExpr>),
    /// `if T: Tr<A>:`: whether `T` meets the bound, decided per instance (R08.2.9).
    HasBound(Ty, TraitRef),
    /// `Int(v)`: the integer of a position, 1 for the first of `1..=n`.
    DomOrd(Box<TExpr>),
    /// The characters of a string literal, as `Raw<Char>`.
    Text(String),
    /// `Char(i)` once `i` is known to be in `0..=127`.
    CharOf(Box<TExpr>),
    /// `Int(c)`: a character's code.
    CodeOf(Box<TExpr>),
    /// A slot with no value yet: the payload of `None`.
    Unset,
    /// `v.ord`: how many positions come before this one (R04.2.22).
    Ordinal(Box<TExpr>),
    /// Variant `i` of an enum, with its payload struct.
    Variant(usize, Box<TExpr>),
    /// Which variant an enum value is.
    Tag(Box<TExpr>),
    /// The payload of variant `i` of an enum place, which must be that variant.
    Payload(Box<TExpr>, usize),
    Error,
}
#[derive(Clone, Debug)]
pub enum TArg {
    Value(TExpr),
    /// A place lent for the call.
    Mut(TExpr),
}
/// What a `Region` calls: a fn, or a method of a type parameter's bound whose impl each
/// instance picks.
#[derive(Clone, Debug)]
pub enum Callee {
    Fn(FnRef),
    Trait(TraitRef, String),
    /// A generator value's closure (R07.6.2).
    Value(Box<TExpr>),
}

#[derive(Clone, Debug)]
pub enum TStmt {
    Let(VarRef, TExpr),
    Assign(TExpr, TExpr),
    Expr(TExpr),
    While(TExpr, TExpr),
    For(VarRef, TExpr, TExpr, Box<TExpr>),
    Return(Option<TExpr>),
    Assert(TExpr, String, Span, CheckKind),
    Break,
    Continue,
    /// `@!multitest`: the code below it runs once per test case, `count` times, or while the
    /// input has a token left when there is no count. A `return` in it ends the case.
    Cases(Option<TExpr>, Vec<TStmt>),
    /// Top-level code only: the statements after this one, to the end of the block, are from
    /// the item that starts at this offset in the source. Their spans are relative to it.
    Item(ItemId),
    /// The checks kept from here to the end of the block (R09.4): `with checks(..)` and
    /// `unchecked:` set them, and lowering leaves out each check they do not keep.
    Checks(Checks),
    /// In a generator: hand a value to the caller's loop body.
    Yield(TExpr),
    /// In a lender: hand a place to the caller's code; the rest of the fn runs after it.
    LendOut(TExpr),
    /// A call that takes the code in `body` and runs it at its `yield` or `lend`, with `var` bound.
    Region {
        callee: Callee,
        recv: Option<Box<TArg>>,
        args: Vec<TArg>,
        var: VarRef,
        lend: bool,
        body: TExpr,
    },
}

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
    pub ty: Ty,
    /// A top-level binding of the file, as opposed to one inside a block.
    pub top: bool,
    /// Declared with `var`: a closure that uses it shares it.
    pub mutable: bool,
    /// Bound by `let`: it cannot be assigned.
    pub fixed: bool,
    /// The `let` that bound it, which a fix writes `var` over.
    pub let_at: Option<cpl_next_ir::ids::SourceId>,
    /// Made by the checker, not named by the program: no suggestion or fix names it.
    pub synthetic: bool,
}
/// The global a top-level value's `let` binds, before its type is known.
fn top_global(d: &Decls, t: &cpl_next_decls::decls::TopValue) -> Local {
    Local {
        name: d.syms.text(t.name).to_string(),
        ty: Ty::Error,
        top: true,
        mutable: t.mutable,
        fixed: false,
        let_at: None,
        synthetic: false,
    }
}
pub struct TypedFn {
    pub self_local: Option<LocalId>,
    pub params: Vec<LocalId>,
    pub locals: Vec<Local>,
    pub body: TExpr,
    pub ret: Ty,
    /// What it yields or lends, with the sets its body gives a fn-typed result.
    pub region: Option<(bool, Ty)>,
}
#[derive(Clone)]
pub struct LambdaInfo {
    pub params: Vec<Ty>,
    pub ret: Ty,
    /// The fn it sits in; `None` for a top-level statement.
    pub owner: Option<FnId>,
    /// A nested `fn`'s run of sibling `fn` statements, itself included; empty for a lambda.
    pub group: Vec<LambdaId>,
    /// What it yields: the closure a generator value walks (R07.6.2).
    pub yields: Option<Ty>,
    /// A nested `fn` with a jagged parameter: each parameter's name, and its written range when
    /// jagged; empty otherwise.
    pub written: Vec<(String, Option<next::Type>)>,
}
/// A range with run-time ends: the variables that hold them, each inclusive; `None` is an open end.
#[derive(Clone)]
struct RangeInfo {
    lo: Option<VarRef>,
    hi: Option<VarRef>,
}
#[derive(Clone)]
pub struct DomInfo {
    pub name: String,
    /// The variable that holds its size.
    pub len: VarRef,
    /// The variable that holds the integer of its first position.
    pub lo: VarRef,
    /// The least integer its first position can be and the greatest its last can be, when
    /// the types of its ends say: an integer literal outside these is never a position.
    pub within: (Option<i64>, Option<i64>),
    pub owner: Option<FnId>,
    /// How many positions it has, when both its ends are constants.
    pub count: Option<i64>,
    /// Its ends are characters, its positions their codes: `L(c)` takes a `Char` and `Char(l)`
    /// gives one back (R04.2.24).
    pub chars: bool,
    /// It is declared by a count alone (`domain E = [m]`), so `io.base` shifts it (R10.2.2).
    pub counted: bool,
    /// An array axis's or a list's range, which no declaration names: its positions are plain
    /// `Int`s (R04.3.4).
    pub anonymous: bool,
}
/// A block's `@memo` fn, which lower builds from: `f` answers from `table` when it holds the
/// key and else runs `body` and keeps its result; `clear` empties the table (R06.4.1).
#[derive(Clone, Debug)]
pub struct Memo {
    pub f: VarRef,
    pub body: VarRef,
    pub table: VarRef,
    pub clear: VarRef,
    pub key: Vec<Ty>,
    pub value: Ty,
    pub word: Sym,
}

pub struct Typed {
    /// The checks the build keeps where no `with checks(..)` says otherwise (contract 0).
    pub checks: Checks,
    pub domains: Vec<DomInfo>,
    /// The `Domain` and `Ordered` impls of each domain the check declared (decls' own aside).
    pub dom_rows: Vec<ImplDecl>,
    pub fns: HashMap<FnId, TypedFn>,
    /// The copies of `uses` fns checked under context values a call gave them: `FnRef::ctx`
    /// `k` is the `k`-th (R09.1.4).
    pub ctx_fns: Vec<(FnId, TypedFn)>,
    /// The copy of an impl's fn that a trait's fn's copy runs when the impl is picked.
    pub ctx_impls: HashMap<(u32, FnId), u32>,
    pub globals: Vec<Local>,
    pub main: Vec<TStmt>,
    pub lambdas: Vec<LambdaInfo>,
    /// The closures a fn's body adds to its i-th set parameter, a callee's adds included.
    pub param_adds: HashMap<(FnId, usize), SetTy>,
    /// What the program puts in each global's set, `SetTy::Global(g, k)`.
    pub global_adds: HashMap<(usize, usize), SetTy>,
    /// Each call passing a set argument, and that argument's slot (`SetTy::AddedAt`).
    pub call_adds: Vec<(FnRef, usize)>,
    /// Whether each of `call_adds`' callees may add to that slot.
    pub call_live: Vec<bool>,
    pub errors: Vec<Msg>,
    /// The errors of each library fn reached only through a trait, said if an instance runs it.
    pub deferred: HashMap<FnId, Vec<Msg>>,
    /// The library fns reached only through a trait: what is wrong in one is said if it runs.
    pub quiet: HashSet<FnId>,
    pub ids: Ids,
    /// Each block's `@memo` fn, in the order they were checked.
    pub memos: Vec<Memo>,
}

enum SetVar {
    Link(usize),
    Root(Vec<Member>),
    Bound(SetTy),
}

/// One context a copy of a fn is made for: the context, its field, and the value's text.
type CtxSetting = (usize, String, String);

/// The tables a body adds to, whose ids it names in its types.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Table {
    Lambda,
    Domain,
    Range,
    /// What a call's callee adds to one of its set arguments (`SetTy::AddedAt`).
    Added,
}

/// `t` with `x` at `i`, the id `Cx::id` gave it. An id an earlier check gave and this one
/// does not make holds a copy of `x`, which nothing names.
fn put<T: Clone>(t: &mut Vec<T>, i: usize, x: T) {
    if t.len() <= i {
        t.resize(i + 1, x.clone());
    }
    t[i] = x;
}

/// The ids a check gave what bodies make, kept for the check of an edited program
/// (`check`'s `prior`): a body the edit leaves alone names its lambdas, domains, ranges, set calls
/// and context copies as before.
#[derive(Clone, Default)]
pub struct Ids {
    keyed: HashMap<(Option<FnId>, u32, Table, u32), usize>,
    next: [usize; 4],
    ctx: HashMap<(FnId, Vec<CtxSetting>), u32>,
}

struct Cx<'a> {
    d: &'a Decls,
    /// The program's views, each read by key (incremental.md): `d` and `src` go once every
    /// read is on them.
    items: &'a dyn Items,
    names: &'a dyn views::Names,
    facts: &'a dyn views::FnFacts,
    bodies: &'a dyn views::Bodies,
    syms: &'a Syms,
    /// Each block's `@memo` fn checked so far.
    memos: Vec<Memo>,
    /// The `Domain` and `Ordered` impls of each domain this check declares (R04.2).
    dom_rows: Vec<ImplDecl>,
    /// What a constant expression's value is, worked out by the driver's evaluator.
    values: &'a dyn Consts,
    /// The loaded modules the spans index, offset by `item`.
    src: &'a crate::modules::Loaded,
    tys: Vec<Option<Ty>>,
    sets: Vec<SetVar>,
    /// Set variables being written out, to catch a set that contains itself.
    zonking: std::cell::RefCell<Vec<usize>>,
    cyclic: std::cell::Cell<bool>,
    /// A top-level domain a top-level statement named before the line declaring it.
    early: std::cell::Cell<Option<usize>>,
    /// The keys of a `let w = s[lo..hi]` with literal ends, which prove a literal `w[k]` (R07.4.5).
    view_keys: HashMap<VarRef, (i128, i128)>,
    /// The judge the build is for, for its memory limit (R03.1.9).
    judge: Option<cpl_next_ir::build::Judge>,
    errors: Vec<Msg>,
    lambdas: Vec<LambdaInfo>,
    /// Each lambda's, local domain's and range's id, by the copy of the fn it sits in and its
    /// place among that copy's: a body checked alone names them as the whole program's does.
    ids: Ids,
    /// Per table, the next id and how many the fn copy being checked has made.
    made: [u32; 4],
    /// The context copy of the fn being checked; 0 is the fn itself.
    copy: u32,
    globals: Vec<Local>,
    global_names: HashMap<String, LocalId>,
    /// The global each top-level value (`Decls::top_values`, by `TopValueId`) binds, made before
    /// any body is checked so a fn's range type may end at it wherever the fn is (R02.10.3).
    top_slots: Vec<LocalId>,
    param_adds: HashMap<(FnId, usize), Vec<Member>>,
    /// What the top level and each body put in a global's set (`SetTy::Global`).
    global_adds: HashMap<(usize, usize), Vec<Member>>,
    /// `SetTy::AddedAt(i)`'s call and the slot of the set it passes there.
    added: Vec<(FnRef, usize)>,
    domains: Vec<DomInfo>,
    /// A body domain and the result's `domain C` it was returned as, which it then stands for;
    /// or a result's `domain C` and the domain its caller bound it to (R02.9.3).
    fresh_eq: Vec<(usize, usize)>,
    /// The `let`s bound to the `None` value: comparing one with a `?T` tests presence (R02.4.5).
    none_lets: HashSet<VarRef>,
    ranges: Vec<RangeInfo>,
    /// The results whose `_`s an earlier check of their bodies fixed (R02.9.3), and the ones
    /// this check fixes.
    found_ret: HashMap<FnId, Ty>,
    /// The fns whose `_` result is being fixed by checking their bodies, innermost last.
    ret_busy: Vec<FnId>,
    /// Each fn's checked body, by the driver or by a caller needing its `_` result.
    fns: HashMap<FnId, TypedFn>,
    /// Fns a reference reached, to be checked; and the library places already reported.
    reached: Vec<FnId>,
    reported: HashSet<(u8, usize)>,
    /// Library fns reached only as a trait's impl, or from one: what they use that the slice
    /// does not build refuses only a program whose instances run them.
    through_trait: Vec<FnId>,
    /// Each library `const`'s global, once its initializer is checked.
    consts: Vec<Option<LocalId>>,
    const_busy: Vec<bool>,
    /// The code that sets them, first reads first, above the program's.
    const_init: Vec<TStmt>,
    /// The checks kept where the code being checked is (R09.4).
    checks_kept: Checks,
    /// The checks the build keeps, where nothing in the code says otherwise.
    checks: Checks,
    /// Each copy of a `uses` fn made so far, by the fn and the context values it is checked
    /// under; and those still to check.
    /// The context copies this check made, by `FnRef::ctx`.
    ctx_made: HashSet<u32>,
    ctx_pending: Vec<(FnId, u32, Vec<CtxFrame>)>,
    ctx_impls: HashMap<(u32, FnId), u32>,
    /// What checking the body at hand keeps: a body checked inside another one's check (a
    /// `_` result its caller needs, R02.9.3) gets its own.
    b: Body,
}

struct Body {
    doms: HashMap<Sym, Ty>,
    /// The domains declared in the blocks being checked, innermost last (R02.9.2).
    block_doms: Vec<usize>,
    /// The domains a binding owns, by the binding and the path to each in its value: `t`'s
    /// `Pre`, or `r`'s `tree.Pre` (R02.9.1).
    owned: HashMap<(VarRef, Vec<String>), usize>,
    /// A block's `@memo` fn's `f.clear`, by the fn's binding (R06.4.1).
    memo_clears: HashMap<VarRef, VarRef>,
    decl: Option<FnId>,
    generics: Vec<Sym>,
    n_sets: usize,
    locals: Vec<Local>,
    /// The names this body binds that no declaration gave.
    syms: BodySyms,
    scopes: Vec<HashMap<String, LocalId>>,
    self_local: Option<LocalId>,
    /// Bounds calls must meet, with where: a type, the trait it must implement, and why.
    obligations: Vec<(Ty, decls::TraitRef, Span, Origin)>,
    /// Temporaries holding an input value whose parts are held to their types one by one.
    input_parts: std::collections::HashSet<VarRef>,
    /// The range each `let` keeps from its initializer, where it is known (R02.10.2).
    let_ranges: std::collections::HashMap<VarRef, (i64, i64)>,
    ret: Ty,
    /// What the fn being checked yields or lends.
    region: Option<(bool, Ty)>,
    /// How many loops enclose the statement being checked, and a marker for `For`'s two-step entry.
    loops: u32,
    in_for: bool,
    /// The loops around this point that have a label or an `else:`, innermost last.
    loop_frames: Vec<labels::Frame>,
    /// The loop statement whose label and `else:` are being handled, by its start.
    framed: Option<u32>,
    /// Set while checking the one call that may be a generator or a lender.
    allow_region: bool,
    /// Set while checking `a[i]` of `a[i][j]`: the whole index's span and `j`'s (R01.8.5).
    indexed_again: Option<(Span, Span)>,
    /// Where a `min=` / `max=` checked as a statement puts its `Bool`, when it is a condition.
    kept: Option<VarRef>,
    /// The name a `let` binds the lambda being checked to; and the lambdas whose bodies see
    /// their own name, with the scope their parameters live in.
    recur: Option<String>,
    selves: Vec<(String, LambdaId, usize)>,
    /// The nested `fn`s of the blocks being checked, by the offset of their statement; and the
    /// one whose lambda is being made.
    group: Vec<(u32, Option<String>, LambdaId)>,
    /// Each use of a nested `fn` above its statement, with the names set at that point: every
    /// capture of the fn must be among them (R06.1.7).
    ahead: Vec<(LambdaId, String, Span, std::collections::BTreeSet<VarRef>)>,
    nested: Option<LambdaId>,
    /// Each list literal of `n` elements made a `[D]T` whose `D` nothing has fixed yet: unless
    /// its statement fixes it, `D` is the list's positions `0..n` (R04.4.30).
    list_doms: Vec<(Ty, usize, Span)>,
    /// The names of items the checker refused: a use of one is downstream of that refusal.
    refused_names: std::collections::HashSet<String>,
    /// Set while converting the branches of an `if` into their common type: sets are joined, never merged.
    joining: bool,
    item: ItemId,
    /// The module whose names the code being checked sees.
    module: ModId,
    /// The `?` extents being checked, innermost last, and the statement right-hand side that is one.
    tries: Vec<question::Extent>,
    /// The bounds an `if T: Tr:` branch around this point has tested, of `T`'s parameter.
    assumed: Vec<(usize, TraitRef)>,
    rhs_extent: Option<Span>,
    /// The fields `with` lines set around the code being checked, innermost last (R09.1.2).
    frames: Vec<CtxFrame>,
}

impl Body {
    fn new(d: &Decls) -> Body {
        Body {
            tries: Vec::new(),
            assumed: Vec::new(),
            rhs_extent: None,
            frames: Vec::new(),
            doms: d.top_doms.clone(),
            block_doms: Vec::new(),
            owned: HashMap::new(),
            memo_clears: HashMap::new(),
            decl: None,
            generics: Vec::new(),
            n_sets: 0,
            locals: Vec::new(),
            syms: BodySyms::default(),
            scopes: vec![HashMap::new()],
            self_local: None,
            obligations: Vec::new(),
            input_parts: Default::default(),
            let_ranges: Default::default(),
            ret: Ty::Unit,
            region: None,
            loops: 0,
            in_for: false,
            loop_frames: Vec::new(),
            framed: None,
            allow_region: false,
            indexed_again: None,
            kept: None,
            recur: None,
            selves: Vec::new(),
            group: Vec::new(),
            ahead: Vec::new(),
            list_doms: Vec::new(),
            refused_names: Default::default(),
            nested: None,
            joining: false,
            item: ItemId::file(
                cpl_next_ir::ids::FileId::new(crate::modules::PROGRAM),
                crate::modules::PROGRAM.into(),
            ),
            module: d.program,
        }
    }
}

/// One field a `with` line set, read as `ctx.field` in the rest of its block.
#[derive(Clone)]
struct CtxFrame {
    ctx: decls::ContextId,
    field: String,
    value: next::Expr,
}

/// The values of constant expressions: the `const_value` query (`handbook/design/next/
/// incremental.md`). The driver works each out by running it on the engine, the one evaluator,
/// so the checker holds no evaluator of its own and does not depend on the engine.
pub trait Consts {
    /// The value of the expression at `at`, when nothing it reads is decided at run time:
    /// `None` for a value that is not a scalar.
    fn value(&self, at: cpl_next_ir::ids::SourceId) -> Result<Option<ops::Scalar>, ConstError>;
}

/// Why an expression has no value at compile time.
#[derive(Clone, Debug)]
pub enum ConstError {
    /// It reads something decided at run time.
    Open,
    /// Working it out trapped, ran past its step budget or read itself: the error, said where
    /// the expression is.
    Failed(Box<Msg>),
}

/// The items a check covers.
#[derive(Clone, Copy, Default)]
pub struct Program<'a> {
    /// The program's own items, in order.
    pub main: &'a [ItemId],
    /// Values bound after them, each a top-level `let` of its name.
    pub values: &'a [(ItemId, String, next::Expr)],
    /// Library modules whose every fn is checked as the program's are, reached or not.
    pub roots: &'a [ModId],
}

/// `program.roots`: library modules whose every fn is checked as the program's are, reached or not.
/// A result's `_` that is no domain is fixed by its fn's body (R02.9.3): a call checked before
/// that body is checked again, with each such result as the bodies fixed it.
/// What bodies make is named as `prior` named it where they make it again: an edit leaves
/// the bodies it does not touch typed the same (`Ids::default()` for a first check).
pub fn check(
    d: &Decls,
    program: Program,
    src: &crate::modules::Loaded,
    consts: &dyn Consts,
    build: Build,
    prior: &Ids,
) -> Typed {
    let Program {
        main,
        values,
        roots,
    } = program;
    let Build { judge, checks } = build;
    let mut cx = Cx {
        memos: Vec::new(),
        values: consts,
        checks_kept: checks,
        checks,
        ctx_made: HashSet::new(),
        ctx_pending: Vec::new(),
        ctx_impls: HashMap::new(),
        d,
        items: d,
        names: d,
        facts: d,
        bodies: src,
        syms: &d.syms,
        dom_rows: Vec::new(),
        src,
        tys: Vec::new(),
        sets: Vec::new(),
        zonking: Default::default(),
        cyclic: Default::default(),
        early: Default::default(),
        view_keys: HashMap::new(),
        judge,
        errors: Vec::new(),
        lambdas: Vec::new(),
        ids: prior.clone(),
        made: [0; 4],
        copy: 0,
        globals: d.top_values.iter().map(|t| top_global(d, t)).collect(),
        global_names: HashMap::new(),
        top_slots: (0..d.top_values.len()).collect(),
        param_adds: HashMap::new(),
        global_adds: HashMap::new(),
        added: Vec::new(),
        // A top-level domain's place is kept for it until its statement runs.
        domains: (0..d.dom_ids.len())
            .map(|_| DomInfo {
                name: String::new(),
                len: VarRef::Global(usize::MAX),
                lo: VarRef::Global(usize::MAX),
                within: (None, None),
                owner: None,
                count: None,
                chars: false,
                counted: false,
                anonymous: false,
            })
            .collect(),
        fresh_eq: Vec::new(),
        none_lets: HashSet::new(),
        ranges: Vec::new(),
        found_ret: HashMap::new(),
        ret_busy: Vec::new(),
        fns: HashMap::new(),
        reached: Vec::new(),
        through_trait: Vec::new(),
        reported: HashSet::new(),
        consts: vec![None; d.consts.len()],
        const_busy: vec![false; d.consts.len()],
        const_init: Vec::new(),
        b: Body::new(d),
    };
    // A result's `domain C` keeps its ends in one static slot per declaration site (R02.9.4).
    let slot = |cx: &mut Cx, role: &str| {
        cx.globals.push(Local {
            name: role.to_string(),
            ty: Ty::Int,
            top: true,
            mutable: true,
            fixed: false,
            let_at: None,
            synthetic: true,
        });
        VarRef::Global(cx.globals.len() - 1)
    };
    for (i, name) in &d.fresh_doms {
        let (lo, len) = (slot(&mut cx, "$lo"), slot(&mut cx, "$len"));
        cx.domains[*i] = DomInfo {
            name: name.clone(),
            len,
            lo,
            within: (None, None),
            owner: None,
            count: None,
            chars: false,
            counted: false,
            anonymous: false,
        };
    }
    for (at, code, msg) in &d.misdeclared {
        cx.errors
            .push(Msg::at(at.clone(), Msg::verdict(*code, msg.clone())));
    }
    for (i, s) in (d.structs.iter().enumerate()).filter(|(_, s)| d.seen(s.module)) {
        if let Some(f) = decls::field_holding_itself(d, StructId(i)) {
            cx.errors.push(Msg::at(
                s.field_at[f].clone(),
                Msg::verdict(
                    cpl_diag::codes::RECURSIVE_FIELD,
                    format!(
                        "a `{}` would hold another `{}` inline, so it has no size; \
                 keep the others in a list and hold an index",
                        s.name, s.name
                    ),
                ),
            ));
        }
    }
    for (i, a) in d.aliases.iter().enumerate() {
        if d.of_program(a.module) && decls::alias_cycle(d, decls::AliasId(i)) {
            cx.errors.push(Msg::at(
                a.at.clone(),
                Msg::verdict(
                    cpl_diag::codes::ALIAS_CYCLE,
                    format!(
                        "type alias `{}` is defined by itself, so it names no type",
                        a.name
                    ),
                ),
            ));
        }
    }
    let mut main_out = Vec::new();
    let mut cases: Option<(Option<TExpr>, Vec<TStmt>)> = None;
    // A `const` with a scalar value, and a domain whose ends have values, hoist; one read at
    // run time runs where it is written (R11.1.1).
    let main = program_items(src, main);
    let lifted: Vec<Option<Option<ops::Scalar>>> = (main.iter())
        .map(|(off, s)| {
            cx.b.item = off.clone();
            cx.hoisted(s)
        })
        .collect();
    for ((off, s), known) in main.iter().zip(&lifted) {
        if let Some(known) = known {
            cx.b.item = off.clone();
            main_out.push(TStmt::Item(off.clone()));
            cx.top_item(s, *known, &mut main_out);
        }
    }
    // A bare `with` line comes before the program's first statement (R09.1.1).
    let mut begun = false;
    // Statements a `let p = mut a[i]` above holds ran inside its lend.
    let mut placed = 0;
    for (k, ((off, s), hoisted)) in main.iter().zip(&lifted).enumerate() {
        if hoisted.is_none() && placed > 0 {
            placed -= 1;
            continue;
        }
        cx.b.item = off.clone();
        let span = cx.at(s);
        // `@!multitest`: the count is read once and everything below runs once per case.
        let counted = s.directive().filter(multitest);
        if let (None, Some(m)) = (&cases, &counted) {
            main_out.push(TStmt::Item(off.clone()));
            let reads = case_count(m).unwrap_or_else(|| {
                cx.err(span, Unbuilt::MultitestArg);
                true
            });
            let count = reads.then_some(TExpr {
                kind: TK::ReadInt,
                ty: Ty::Int,
                span,
            });
            cases = Some((count, Vec::new()));
        }
        let unbuilt: Vec<String> = (s.directive().filter(|x| !multitest(x)))
            .and_then(|x| x.name())
            .into_iter()
            .chain(s.decorators().iter().filter_map(next::Decorator::name))
            .map(|t| t.text().to_string())
            .collect();
        for name in unbuilt {
            cx.err(span, Unbuilt::Decorator { name });
        }
        // A hoisted item ran above; `@!multitest` on it still divides the file here.
        if hoisted.is_some() {
            continue;
        }
        let into = match &mut cases {
            Some((_, body)) => body,
            None => &mut main_out,
        };
        let bare_with = matches!(s.stmt(), Some(next::Stmt::With(w)) if w.body().is_none());
        if bare_with && begun {
            let code = cpl_diag::codes::WITH_NOT_FIRST;
            cx.err(
                span,
                Msg::verdict(code, "a bare `with` line comes first in its block"),
            );
            continue;
        }
        begun |= !bare_with;
        into.push(TStmt::Item(off.clone()));
        if let Some((p, _)) = s.stmt().as_ref().and_then(place::place_binding) {
            let items: Vec<(ItemId, next::Item)> = (main[k..].iter().zip(&lifted[k..]))
                .filter(|(_, l)| l.is_none())
                .map(|(x, _)| x.clone())
                .collect();
            let rest: Vec<next::Item> = items[1..].iter().map(|(_, s)| s.clone()).collect();
            let held = place::place_held(&p, &rest);
            placed = held;
            into.pop();
            cx.items_placed(&items[..=held], into);
            continue;
        }
        cx.top_item(s, None, into);
        if let Some(i) = cx.early.take() {
            let name = (d.top_doms.iter())
                .find(|(_, t)| matches!(t, Ty::Dom(k) if *k == i))
                .map_or("", |(n, _)| d.text(*n));
            cx.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::USED_BEFORE_SET,
                    format!("domain `{name}` is used above the line that declares it (R11.1.1)"),
                ),
            );
        }
    }
    // Each value a constant's run works out is a top-level `let` after the program's items.
    for (item, name, e) in values {
        cx.b.item = item.clone();
        main_out.push(TStmt::Item(item.clone()));
        let span = cx.at(e);
        cx.let_name(name, e, span, &mut main_out);
    }
    cx.solve(true);
    if let Some((count, body)) = cases {
        main_out.push(TStmt::Cases(count, body));
    }
    cx.own_global_sets();
    // The program's fns are all checked; a library's, core's too, only once a reference reaches
    // them, so what a module declares and the program never uses refuses nothing.
    // A record's impls are made, not written: one is checked once a reference reaches it.
    let mut todo: Vec<FnId> = (0..d.fns.len())
        .map(FnId)
        .filter(|id| {
            let f = d.f(*id);
            let later = |im: &decls::ImplDecl| {
                !d.live(im) || matches!(im.for_, Ty::Struct(s, ..) if d.s(s).record)
            };
            (roots.contains(&f.module) || !d.is_library(f.module))
                && !matches!(f.owner, Some(Owner::Impl(i)) if later(d.imp(i)))
        })
        .collect();
    // The trace role's fns are reached by events, which a build adds after checking (contract 8).
    todo.extend(d.roles.trace.iter().copied());
    // In the order the collector made them, so a program over std checks as one collection would.
    todo.sort_by_key(|id| std::cmp::Reverse((d.f(*id).made, *id)));
    let mut deferred: HashMap<FnId, Vec<Msg>> = HashMap::new();
    let mut quiet_fns = HashSet::new();
    let mut ctx_fns: Vec<(u32, FnId, TypedFn)> = Vec::new();
    loop {
        todo.append(&mut cx.reached);
        if let Some((id, k, frames)) = cx.ctx_pending.pop() {
            cx.copy = k;
            let tf = cx.check_fn(id, frames);
            cx.copy = 0;
            ctx_fns.push((k, id, tf));
            continue;
        }
        let (id, quiet) = match todo.pop() {
            Some(id) => (id, false),
            None => match cx.through_trait.pop() {
                Some(id) => (id, true),
                None => break,
            },
        };
        let f = d.f(id);
        // A trait's fn whose body is an intrinsic is checked into that operation on `Self`.
        let of_trait = matches!(f.owner, Some(Owner::Trait(_)));
        let lowered_intrinsic = f.intrinsic.is_some() && !of_trait;
        let undeclared = matches!(f.body, decls::Body::None | decls::Body::Synth(_)) && !of_trait;
        if cx.fns.contains_key(&id) || lowered_intrinsic || undeclared {
            continue;
        }
        if !quiet {
            let tf = cx.check_fn(id, Vec::new());
            cx.fns.insert(id, tf);
            continue;
        }
        quiet_fns.insert(id);
        let (n, reported) = (cx.errors.len(), cx.reported.clone());
        cx.reach(id);
        let tf = cx.check_fn(id, Vec::new());
        cx.fns.insert(id, tf);
        let errors = cx.errors.split_off(n);
        cx.reported = reported;
        let more = std::mem::take(&mut cx.reached);
        cx.through_trait.extend(more);
        if !errors.is_empty() {
            // A library body that does not check is the pipeline's to fix, said where it is used.
            let errors = match d.is_library(d.f(id).module) {
                true => (errors.into_iter())
                    .map(|m| Msg {
                        refusal: Some(Unbuilt::LibraryBody {
                            func: d.f(id).name.clone(),
                            why: m.text.clone(),
                        }),
                        text: String::new(),
                        code: cpl_diag::codes::INTERNAL,
                        ..m
                    })
                    .collect(),
                false => errors,
            };
            deferred.insert(id, errors);
        }
    }
    // Every top value's global is filled by its `let`; one left unfilled with no refusal to
    // explain it is a checker ordering bug, reported rather than read as `Error`.
    if cx.errors.is_empty() {
        for (t, &g) in d.top_values.iter().zip(&cx.top_slots) {
            if cx.globals[g].ty == Ty::Error {
                let what = format!(
                    "`{}`'s global was never filled by its `let`",
                    d.syms.text(t.name)
                );
                cx.errors
                    .push(Msg::at(t.item.at(Span::default()), Msg::internal(what)));
            }
        }
    }
    let call_live = cx.adds_live();
    let call_adds = std::mem::take(&mut cx.added);
    // Zonk: every variable is replaced by what it was solved to.
    let mut main_z = std::mem::take(&mut cx.const_init);
    main_z.extend(main_out);
    for s in &mut main_z {
        cx.zonk_stmt(s);
    }
    ctx_fns.sort_by_key(|c| c.0);
    let mut globals = std::mem::take(&mut cx.globals);
    for g in &mut globals {
        g.ty = cx.zonk(&g.ty);
    }
    let mut lambdas = std::mem::take(&mut cx.lambdas);
    for l in &mut lambdas {
        l.params = l.params.iter().map(|t| cx.zonk(t)).collect();
        l.ret = cx.zonk(&l.ret);
        l.yields = l.yields.as_ref().map(|t| cx.zonk(t));
    }
    let call_adds = (call_adds.into_iter())
        .map(|(f, k)| {
            let f = FnRef {
                targs: f.targs.iter().map(|t| cx.zonk(t)).collect(),
                sets: f.sets.iter().map(|s| cx.zonk_set(s)).collect(),
                ..f
            };
            (f, k)
        })
        .collect();
    // Each through a variable, so its closures' own types are written out as any set's are.
    let param_adds = (cx.param_adds.clone().into_iter())
        .map(|(k, ms)| {
            let v = cx.new_set();
            if let SetTy::Var(i) = v {
                cx.sets[i] = SetVar::Root(ms);
            }
            (k, cx.zonk_set(&v))
        })
        .collect();
    let global_adds = (cx.global_adds.clone().into_iter())
        .map(|(k, ms)| {
            let v = cx.new_set();
            if let SetTy::Var(i) = v {
                cx.sets[i] = SetVar::Root(ms);
            }
            (k, cx.zonk_set(&v))
        })
        .collect();
    // R08.2.10: each call passes its caller's type parameters to the callee's, as themselves or
    // inside a larger type. A cycle of calls through a larger one makes a new instance on every
    // round, so its first such call is E0319.
    let mut callers: Vec<FnId> = cx.fns.keys().copied().collect();
    callers.sort();
    let mut edges = Vec::new();
    for caller in callers {
        cx.fns[&caller].body.each(&mut |e| {
            let TK::Call { f, .. } = &e.kind else { return };
            for (j, t) in f.targs.iter().enumerate() {
                let mut ps = std::collections::BTreeSet::new();
                cpl_next_ir::ty::params_in(t, &mut ps);
                for i in ps {
                    let grows = *t != Ty::Param(i);
                    edges.push(((caller, i), (f.decl, j), grows, e.span, t.clone()));
                }
            }
        });
    }
    let mut grown = HashSet::new();
    for (from, to, grows, span, t) in &edges {
        if !grows || grown.contains(&from.0) {
            continue;
        }
        let mut seen = HashSet::from([*to]);
        let mut stack = vec![*to];
        while let Some(n) = stack.pop() {
            for (a, b, ..) in &edges {
                if *a == n && seen.insert(*b) {
                    stack.push(*b);
                }
            }
        }
        if !seen.contains(from) {
            continue;
        }
        grown.insert(from.0);
        let (caller, callee) = (d.f(from.0), d.f(to.0));
        cx.b.item = caller.item.clone();
        cx.b.generics = caller.generics.clone();
        let (outer, inner) = (cx.show(&Ty::Param(from.1)), cx.show(t));
        let name = &callee.name;
        let text = cpl_diag::msgs::ir::GROWING_INSTANTIATION_CALLS_ITSELF_AT_A
            .say(&[name, name, &outer, name, &inner]);
        let help = "a generic fn is compiled once per type: recurse at the same type, or keep the growing part in a value of one fixed type (R08.2.10)";
        let code = cpl_diag::codes::GROWING_INSTANTIATION;
        cx.err_help(*span, Msg::verdict(code, text), Some(help.to_string()));
    }
    if cx.domains.iter().any(|d| d.name.is_empty()) {
        cx.errors.push(Unbuilt::UndeclaredTopDomain.into());
    }
    if cx.cyclic.get() {
        cx.errors.push(Msg::internal("a variable holding a function is assigned a function built from its own old value, so the set of functions it can hold contains itself; bind the new function to a new name"));
    }
    Typed {
        checks,
        domains: std::mem::take(&mut cx.domains),
        dom_rows: std::mem::take(&mut cx.dom_rows),
        fns: std::mem::take(&mut cx.fns),
        ctx_fns: ctx_fns.into_iter().map(|(_, id, tf)| (id, tf)).collect(),
        ctx_impls: std::mem::take(&mut cx.ctx_impls),
        globals,
        main: main_z,
        lambdas,
        param_adds,
        global_adds,
        call_adds,
        call_live,
        errors: cx.errors,
        deferred,
        quiet: quiet_fns,
        ids: cx.ids,
        memos: cx.memos,
    }
}

impl<'a> Cx<'a> {
    /// What an expression that was diagnosed or refused becomes. An operation on it is the
    /// error node again and says nothing more, so one fault is one diagnostic.
    fn error_node(span: Span) -> TExpr {
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    fn is_error(&self, t: &Ty) -> bool {
        self.shallow(t) == Ty::Error
    }
    fn err(&mut self, span: Span, msg: impl Into<Msg>) {
        self.err_help(span, msg, None);
    }
    fn err_help(&mut self, span: Span, msg: impl Into<Msg>, help: Option<String>) {
        let mut msg = Msg::at(self.b.item.at(span), msg);
        if help.is_some() {
            msg.help = help;
        }
        self.errors.push(msg);
    }
    /// The source text `span` covers.
    fn text(&self, span: Span) -> &'a str {
        self.bodies.text(&self.b.item.at(span))
    }
    /// A reference reached fn `id`: it is checked, and what its library declaration uses that
    /// the slice does not build is said once. A trait's method reaches every impl's.
    /// Which copy of `id` a call here makes: 0 when no context it `uses` is set around the
    /// call, else the one checked under the values set (R09.1.3, R09.1.4).
    /// The copy a call of trait `tr`'s fn `method` makes.
    pub(crate) fn trait_ctx(&mut self, tr: TraitId, method: &str) -> u32 {
        match self.d.member(&self.items.t(tr).methods, method) {
            Some(m) => self.ctx_copy(m),
            None => 0,
        }
    }
    /// The impl lookup: decls' impls and the rows of the domains this check declares.
    pub(crate) fn impls(&self) -> Impls<'_> {
        Impls {
            items: self.d,
            rows: &self.dom_rows,
        }
    }
    /// A trait's fn's copy has each impl's under the same values, which link picks (`ctx_impls`).
    pub(crate) fn ctx_copy(&mut self, id: FnId) -> u32 {
        let around: Vec<CtxFrame> = self.b.frames.iter().rev().cloned().collect();
        let k = self.copy_under(id, &around);
        let d = self.d;
        let f = d.f(id);
        if let (Some(Owner::Trait(t)), true) = (f.owner, k != 0) {
            for (_, im) in d.trait_impls(t) {
                if let Some(m) = d.member(&im.methods, &f.name) {
                    let km = self.copy_under(m, &around);
                    if km != 0 {
                        self.ctx_impls.insert((k, m), km);
                    }
                }
            }
        }
        k
    }
    /// The copy of `id` checked under the innermost of `around` that it `uses`, or 0 for none.
    fn copy_under(&mut self, id: FnId, around: &[CtxFrame]) -> u32 {
        let d = self.d;
        let f = d.f(id);
        let used: Vec<decls::ContextId> = (f.uses.iter())
            .filter_map(|n| match d.named(f.module, n) {
                Some(decls::Named::Context(c)) => Some(c),
                _ => None,
            })
            .collect();
        let mut seen = HashSet::new();
        let mut frames: Vec<CtxFrame> = (around.iter())
            .filter(|x| used.contains(&x.ctx) && seen.insert((x.ctx, x.field.clone())))
            .cloned()
            .collect();
        if frames.is_empty() {
            return 0;
        }
        frames.sort_by(|a, b| (a.ctx, &a.field).cmp(&(b.ctx, &b.field)));
        let key: Vec<_> = (frames.iter())
            .map(|x| (x.ctx.0, x.field.clone(), konst(&x.value)))
            .collect();
        let next = self.ids.ctx.len() as u32 + 1;
        let k = *self.ids.ctx.entry((id, key)).or_insert(next);
        if self.ctx_made.insert(k) {
            self.ctx_pending.push((id, k, frames));
        }
        k
    }

    /// A call reaches a fn that adds closures to a set argument, so the set it passes gains
    /// them: a set the caller owns takes them as a member, and a set parameter of the caller
    /// passes them on to its own callers. Run to a fixpoint once every body is checked.
    /// Which calls' callees may add to the set they are passed there, read from the summaries
    /// alone (`param_adds`): no body is checked again.
    fn adds_live(&self) -> Vec<bool> {
        let callee_adds = |live: &[bool], (f, slot): &(FnRef, usize)| {
            let fd = self.items.f(f.decl);
            let by_impl = matches!(fd.owner, Some(Owner::Trait(_))) && *slot >= fd.n_sets;
            by_impl
                || (self.param_adds.get(&(f.decl, *slot))).is_some_and(|ms| {
                    ms.iter().any(|m| match m {
                        Member::Sub(SetTy::AddedAt(c)) => live[*c],
                        _ => true,
                    })
                })
        };
        let mut live = vec![false; self.added.len()];
        loop {
            let next: Vec<bool> = self.added.iter().map(|c| callee_adds(&live, c)).collect();
            if next == live {
                return live;
            }
            live = next;
        }
    }
    pub(crate) fn reach(&mut self, id: FnId) {
        let d = self.d;
        let f = d.f(id);
        self.reached.push(id);
        if let Some(Owner::Trait(t)) = f.owner {
            for (_, im) in d.trait_impls(t) {
                if let Some(m) = d.member(&im.methods, &f.name) {
                    let made = matches!(im.for_, Ty::Struct(s, ..) if d.s(s).record);
                    match d.is_library(im.module) || made {
                        true => self.through_trait.push(m),
                        false => self.reached.push(m),
                    }
                }
            }
        }
        if !d.is_library(f.module) {
            return;
        }
        let module = &d.module_names[f.module.0];
        let mut said = Vec::new();
        if self.reported.insert((0, id.0)) {
            said.extend(f.errors.iter().cloned());
        }
        if let Some(Owner::Struct(s)) = f.owner {
            if self.reported.insert((1, s.0)) {
                said.extend(d.s(s).errors.iter().cloned());
            }
        }
        let tr = match f.owner {
            Some(Owner::Trait(t)) => Some(t),
            Some(Owner::Impl(i)) => Some(d.imp(i).trait_.tr),
            _ => None,
        };
        if let Some(t) = tr.filter(|t| self.reported.insert((3, t.0))) {
            said.extend(d.t(t).errors.iter().cloned());
        }
        if self.reported.insert((2, f.module.0)) {
            said.extend(d.module_errors[f.module.0].iter().cloned());
        }
        self.errors
            .extend(said.into_iter().map(|e| e.in_module(module)));
    }
    /// E0409: a fn that gives a value gives one, or leaves, on every path through its body;
    /// `end` is where its body can end without one. Whether it was reported.
    pub(crate) fn falls_through(&mut self, name: &str, end: fall_through::End, ret: &Ty) -> bool {
        let Some((at, how)) = end else {
            return false;
        };
        if matches!(self.shallow(ret), Ty::Unit | Ty::Error) {
            return false;
        }
        let value = cpl_diag::with_article(&self.show(ret));
        let text =
            cpl_diag::msgs::check::FN_FALLS_THROUGH_AND_THEN_ENDS_WITHOUT.say(&[&how, &name]);
        let help = fall_through::help(name, &value, how);
        let msg = Msg::verdict(cpl_diag::codes::FN_FALLS_THROUGH, text);
        self.err_help(at, msg, Some(help));
        true
    }

    /// `tf` with every variable replaced by what it was solved to.
    fn zonk_fn(&mut self, tf: &mut TypedFn) {
        self.zonk_expr(&mut tf.body);
        for l in &mut tf.locals {
            l.ty = self.zonk(&l.ty);
        }
        tf.ret = self.zonk(&tf.ret);
        tf.region = (tf.region.take()).map(|(l, t)| (l, self.zonk(&t)));
    }
    fn check_fn(&mut self, id: FnId, frames: Vec<CtxFrame>) -> TypedFn {
        let lends_last = |b: &next::Block| matches!(b.stmts().last(), Some(next::Stmt::Lend(_)));
        let d = self.d;
        let f = d.f(id);
        self.b.decl = Some(id);
        self.made = [0; 4];
        self.b.module = f.module;
        let top_doms = self.b.doms.clone();
        let top_owned = self.b.owned.clone();
        let top_clears = self.b.memo_clears.clone();
        self.b.generics = f.generics.clone();
        // A fn's result slots are set parameters after its own: each call passes its sets.
        self.b.n_sets = f.n_sets + f.n_ret_sets;
        self.b.locals = Vec::new();
        self.b.syms = BodySyms::default();
        self.b.scopes = vec![HashMap::new()];
        self.b.item = f.item.clone();
        self.b.self_local = None;
        // A top-level fn reads the contexts it was written in, and a `uses` fn its caller's
        // (R09.1.3): `frames` are the values this copy of it is checked under.
        let frames = std::mem::replace(&mut self.b.frames, frames);
        if let Some(ty) = f.self_ty.clone().filter(|_| f.self_mode != SelfMode::None) {
            self.b.self_local = Some(self.add_local(&f.self_name, ty));
        }
        let mut params = Vec::new();
        for p in &f.params {
            let ty = self.fresh_sets(&p.ty);
            params.push(self.add_local(&p.name, ty));
        }
        let n = f.n_sets;
        self.b.region =
            (f.region.clone()).map(|(l, t)| (l, map_slots(&t, &|k| SetTy::Param(n + k))));
        let ret = (self.found_ret.get(&id).cloned()).unwrap_or_else(|| d.f(id).ret.clone());
        let holes = d.ret_holes.contains(&id) && !unify::ground(&ret);
        let ret = if holes { self.open_holes(&ret) } else { ret };
        self.b.ret = map_slots(&ret, &|k| SetTy::Param(n + k));
        let ret = self.b.ret.clone();
        // A fn refused as declared (a decorator the slice lacks) is that refusal, and one rejected
        // as declared is that error: its body is not checked.
        let refused =
            (f.errors.iter()).any(|e| e.refusal.is_some()) || d.rejected_bodies.contains(&id);
        // A body that can end without its value is that error, not a mismatch at its end.
        let syntax = match f.body {
            decls::Body::Written | decls::Body::Value => {
                (self.bodies.item(&f.item)).and_then(|it| decls::body_of(d, id, &it))
            }
            decls::Body::Synth(_) | decls::Body::None => None,
        };
        let end = (syntax.as_ref()).and_then(|b| b.falls_through(&|n| self.traps(n)));
        let falls = !refused
            && f.region.is_none()
            && f.intrinsic.is_none()
            && self.falls_through(&f.name, end, &ret);
        let body = match &syntax {
            _ if refused => Self::error_node(Span::new(0, 0)),
            Some(next::FnBody::Block(b)) if falls => self.block(b, Some(&Ty::Unit)),
            Some(next::FnBody::Expr(e)) if falls => self.tail_block(&[], Some(e), Some(&Ty::Unit)),
            // A trait's fn whose body is an operation on `Self`, as `Domain`'s are.
            _ if f.intrinsic.is_some() => {
                let span = Span::new(0, 0);
                let arg = match self.b.self_local {
                    Some(l) => Some((l, Ty::Param(0))),
                    None => params.first().map(|&l| (l, Ty::Int)),
                };
                let arg = arg.map(|(l, ty)| TExpr {
                    kind: TK::Var(VarRef::Local(l)),
                    ty,
                    span,
                });
                let op = f.intrinsic.map_or("", |i| i.id());
                let made = match f.intrinsic {
                    Some(cpl_next_ir::ops::Intrinsic::Domain(dop)) => {
                        self.domain_op(Ty::Param(0), dop, arg, span)
                    }
                    _ => None,
                };
                made.unwrap_or_else(|| {
                    self.err(span, Unbuilt::TraitIntrinsic { op: op.to_string() });
                    TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    }
                })
            }
            // A block that reaches its end with no value gives `()`, not the result; a
            // generator's or lender's result is what it yields or lends.
            Some(next::FnBody::Block(b)) => match self.block(b, Some(&ret)) {
                v @ TExpr {
                    kind: TK::Block(_, None),
                    ..
                } if f.region.is_none() && !b.never_ends(&|n| self.traps(n)) && !lends_last(b) => {
                    self.coerce(v, &ret)
                }
                v => v,
            },
            // A generator's one expression is the sequence it yields, or, when it yields itself
            // (`= match k: 0: yield 1 ...`), its body (R07.2.10).
            Some(next::FnBody::Expr(e)) if matches!(f.region, Some((false, _))) => {
                if yields_in(e) {
                    self.tail_block(&[], Some(e), Some(&ret))
                } else {
                    let span = self.at(e);
                    self.scoped_block(span, |cx, out| {
                        cx.yield_all(e, span, out);
                        None
                    })
                }
            }
            // An expression that never gives a value is the body, not its result (R01.6.7).
            Some(next::FnBody::Expr(e)) if e.leaves(&|n| self.traps(n)) => {
                self.tail_block(&[], Some(e), Some(&ret))
            }
            // A `lends mut` fn's one expression is the place it lends (R05.6.3).
            Some(next::FnBody::Expr(e)) if matches!(f.region, Some((true, _))) => {
                let span = self.at(e);
                self.scoped_block(span, |cx, out| {
                    cx.lend_stmt(e, span, out);
                    None
                })
            }
            // A `()` fn's one expression is a statement: its value is dropped, as it would be
            // last in a block (R06.1.1, R06.4.8).
            Some(next::FnBody::Expr(e))
                if e.bare_try().is_none() && self.shallow(&ret) == Ty::Unit =>
            {
                self.tail_block(&[], Some(e), Some(&ret))
            }
            Some(next::FnBody::Expr(e)) => {
                let v = match e.bare_try() {
                    Some(_) => self.extent(e, Some(&ret)),
                    None => self.expr(e, Some(&ret)),
                };
                self.coerce(v, &ret)
            }
            None => match f.owner {
                // A trait's fn as a value: the impl's fn of `Self`'s type.
                Some(Owner::Trait(tr)) => self.trait_dispatch(id, tr, &params),
                _ => {
                    let span = Span::new(0, 0);
                    self.err(span, Unbuilt::FnWithoutBody);
                    TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    }
                }
            },
        };
        self.solve(true);
        if holes {
            let fixed = self.zonk(&ret);
            if unify::ground(&fixed) {
                self.found_ret.insert(id, fixed);
            }
        }
        self.b.frames = frames;
        let mut out = TypedFn {
            self_local: self.b.self_local,
            params,
            locals: std::mem::take(&mut self.b.locals),
            body,
            ret,
            region: self.b.region.take(),
        };
        // Typed once checked: no later body changes this one's types (`SetTy::AddedAt`, `SetTy::Global`).
        self.zonk_fn(&mut out);
        self.b.decl = None;
        self.b.module = d.program;
        self.b.region = None;
        self.b.doms = top_doms;
        self.b.owned = top_owned;
        self.b.memo_clears = top_clears;
        self.b.generics = Vec::new();
        self.b.n_sets = 0;
        self.b.self_local = None;
        self.b.scopes = vec![HashMap::new()];
        out
    }
}

/// The syntax of each of the program's top-level items `main`, in order.
pub fn program_items(src: &crate::modules::Loaded, main: &[ItemId]) -> Vec<(ItemId, next::Item)> {
    let syntax: HashMap<&ItemId, &next::Item> = (src.modules.iter())
        .flat_map(|m| m.items.iter())
        .map(|i| (&i.id, &i.item))
        .collect();
    (main.iter())
        .filter_map(|id| Some((id.clone(), (*syntax.get(id)?).clone())))
        .collect()
}

/// The one name pattern `p` binds when it is a bare name, as `x` in `let x = ..`.
pub fn plain_name(p: &next::Pattern) -> Option<String> {
    let next::Pattern::Path(p) = p else {
        return None;
    };
    let mut seg = p.segments();
    let (Some(name), None, None) = (seg.next(), seg.next(), p.args()) else {
        return None;
    };
    Some(name.text().to_string())
}

/// Whether the `@!multitest` `m` reads a count of cases: `(until_eof)` does not. `None` for
/// arguments it does not take.
fn case_count(m: &next::Directive) -> Option<bool> {
    let args: Vec<next::Arg> = m.args().map_or_else(Vec::new, |a| a.args().collect());
    match &args[..] {
        [] => Some(true),
        [a] if a.label().is_none()
            && !a.is_mut()
            && matches!(a.value(), Some(next::Expr::Name(n))
                if n.ident().is_some_and(|t| t.text() == "until_eof")) =>
        {
            Some(false)
        }
        _ => None,
    }
}

/// The names a body binds that no declaration gave, each a `Sym` past the declarations' last, so
/// a body's ids follow its walk and never leave its answer.
#[derive(Default)]
pub(crate) struct BodySyms {
    texts: Vec<String>,
    ids: HashMap<String, Sym>,
}

impl BodySyms {
    /// The id `text` already has, a declaration's or this body's, without making one.
    pub(crate) fn of(&self, syms: &Syms, text: &str) -> Option<Sym> {
        syms.get(text).or_else(|| self.ids.get(text).copied())
    }
    /// The id of `text`: a declaration's, or else one of this body's own.
    pub(crate) fn intern(&mut self, syms: &Syms, text: &str) -> Sym {
        if let Some(s) = self.of(syms, text) {
            return s;
        }
        let s = Sym((syms.len() + self.texts.len()) as u32);
        self.texts.push(text.to_string());
        self.ids.insert(text.to_string(), s);
        s
    }
    /// The text of `s`, a declaration's id or one of this body's.
    pub(crate) fn text<'s>(&'s self, syms: &'s Syms, s: Sym) -> &'s str {
        match (s.0 as usize).checked_sub(syms.len()) {
            Some(k) => &self.texts[k],
            None => syms.text(s),
        }
    }
}

/// Whether directive `m` is `@!multitest`.
fn multitest(m: &next::Directive) -> bool {
    let mark = m
        .name()
        .and_then(|n| cpl_next_syntax::grammar::Decorator::of(n.text()));
    mark == Some(cpl_next_syntax::grammar::Decorator::Multitest)
}

/// Whether `e` holds a `yield` of its own.
fn yields_in(e: &next::Expr) -> bool {
    (e.syntax().descendants()).any(|n| next::YieldStmt::cast(n).is_some())
}

impl Cx<'_> {
    /// A read of a library `const`, or of a struct's (`Dir4.up`).
    fn const_read(&mut self, c: decls::ConstId, span: Span) -> TExpr {
        match self.library_const(c, span) {
            Some((v, ty)) => TExpr {
                kind: TK::Var(v),
                ty,
                span,
            },
            None => Self::error_node(span),
        }
    }
    /// A library `const`'s global, checked once, when a read first reaches it: it is set at
    /// program start, above the program's code, after the consts its initializer reads (R01.6.1).
    /// `e`'s value at compile time, worked out by the driver (`Consts`): `None` when it reads
    /// something decided at run time or working it out failed, which is said; `Some(None)` for
    /// a value that is not a scalar.
    fn const_value(&mut self, e: &next::Expr) -> Option<Option<ops::Scalar>> {
        match self.values.value(self.b.item.at(e.syntax().span())) {
            Ok(v) => Some(v),
            Err(ConstError::Open) => None,
            Err(ConstError::Failed(m)) => {
                if !self.errors.iter().any(|x| x.at == m.at && x.code == m.code) {
                    self.errors.push(*m);
                }
                None
            }
        }
    }
    /// Whether item `s` runs above the program, and the value it is known to have: a `const`
    /// with a scalar value (an `Int` is written in as its literal), or a domain whose ends have
    /// values.
    fn hoisted(&mut self, s: &next::Item) -> Option<Option<ops::Scalar>> {
        match s.decl()? {
            next::Decl::Const(c) => {
                let v = self.const_value(&c.value()?)??;
                Some(matches!(v, ops::Scalar::Int(_)).then_some(v))
            }
            next::Decl::Domain(dom) => {
                let ends: Vec<next::Expr> = match dom.size()? {
                    next::Expr::Range(r) if r.step().is_none() => vec![r.lo()?, r.hi()?],
                    next::Expr::List(l) => {
                        let xs: Vec<next::Expr> = l.elems().collect();
                        if xs.len() != 1 {
                            return None;
                        }
                        xs
                    }
                    _ => return None,
                };
                for e in &ends {
                    self.const_value(e)??;
                }
                Some(None)
            }
            _ => None,
        }
    }
    /// One item of the program's top level, `known` the value a hoisted `const` has.
    fn top_item(&mut self, s: &next::Item, known: Option<ops::Scalar>, out: &mut Vec<TStmt>) {
        if let Some(st) = s.stmt() {
            return self.stmt(&st, out);
        }
        match s.decl() {
            Some(next::Decl::Const(c)) => self.const_stmt(&c, known, out),
            Some(next::Decl::Domain(dom)) => self.domain_stmt(&dom, out),
            _ => {}
        }
    }
    fn library_const(&mut self, c: decls::ConstId, span: Span) -> Option<(VarRef, Ty)> {
        if let Some(g) = self.consts[c.0] {
            return Some((VarRef::Global(g), self.globals[g].ty.clone()));
        }
        let d = self.d;
        let k = &d.consts[c.0];
        if self.const_busy[c.0] {
            let code = cpl_diag::codes::DEFINITION_CYCLE;
            let msg = Msg::verdict(
                code,
                format!("const `{}` is read by its own initializer", k.name),
            );
            self.err(span, msg);
            return None;
        }
        self.const_busy[c.0] = true;
        // Checked as top-level code of its module, whatever body first read it.
        let body = (
            self.b.decl.take(),
            std::mem::take(&mut self.b.generics),
            std::mem::take(&mut self.b.n_sets),
            std::mem::take(&mut self.b.locals),
            std::mem::replace(&mut self.b.scopes, vec![HashMap::new(), HashMap::new()]),
            self.b.self_local.take(),
            std::mem::replace(&mut self.b.ret, Ty::Unit),
            self.b.region.take(),
            (
                std::mem::take(&mut self.b.loops),
                std::mem::take(&mut self.b.in_for),
            ),
            (
                std::mem::take(&mut self.b.allow_region),
                self.b.recur.take(),
            ),
            (
                std::mem::take(&mut self.b.selves),
                std::mem::take(&mut self.b.group),
            ),
            (std::mem::take(&mut self.b.ahead), self.b.nested.take()),
            std::mem::take(&mut self.b.joining),
            std::mem::replace(&mut self.b.item, k.item.clone()),
            std::mem::replace(&mut self.b.module, k.module),
        );
        let mut out = vec![TStmt::Item(k.item.clone())];
        let want = (k.ty.as_ref()).map(|t| self.body_ty(t, &mut out));
        let mut e = self.expr(&k.init, want.as_ref());
        if let Some(t) = &want {
            e = self.coerce(e, t);
        }
        (
            self.b.decl,
            self.b.generics,
            self.b.n_sets,
            self.b.locals,
            self.b.scopes,
        ) = (body.0, body.1, body.2, body.3, body.4);
        (self.b.self_local, self.b.ret, self.b.region) = (body.5, body.6, body.7);
        (
            (self.b.loops, self.b.in_for),
            (self.b.allow_region, self.b.recur),
        ) = (body.8, body.9);
        ((self.b.selves, self.b.group), (self.b.ahead, self.b.nested)) = (body.10, body.11);
        (self.b.joining, self.b.item, self.b.module) = (body.12, body.13, body.14);
        self.globals.push(Local {
            name: k.name.clone(),
            ty: e.ty.clone(),
            top: true,
            mutable: false,
            fixed: true,
            let_at: None,
            synthetic: false,
        });
        let g = self.globals.len() - 1;
        out.push(TStmt::Let(VarRef::Global(g), e));
        self.const_init.extend(out);
        self.consts[c.0] = Some(g);
        self.const_busy[c.0] = false;
        Some((VarRef::Global(g), self.globals[g].ty.clone()))
    }
}

/// A context value written out, so two `with` lines that set the same value make one copy.
fn konst(e: &next::Expr) -> String {
    let words: Vec<String> = (e.syntax().tokens())
        .filter(|t| !t.kind().is_trivia())
        .map(|t| t.text().to_string())
        .collect();
    words.join(" ")
}
