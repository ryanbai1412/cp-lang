//! Contract 3: declarations -> typed tree. Decides every type, target and
//! conversion. Nothing after this infers anything.
use crate::diag::Msg;
use crate::ir::CheckKind;
use crate::ty::*;
use cpl_diag::Span;
use cpl_next_decls::decls::{self, Decls, ModId, Named, Owner, SelfMode, TraitRef};
use cpl_next_decls::{diag, kind_name, modules, plain, records};
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_ir::{ir, ops, ty};
use cpl_syntax::ast::{
    self, AssignOp, BinOp, CmpOp, Cond, ExprKind, FnBody, PatternKind, StmtKind, StrPiece,
    TypeKind, UnOp,
};
use std::collections::HashMap;
use std::collections::HashSet;

mod call;
mod domain;
mod expr;
mod free;
pub use free::{free_vars, group_free};
mod from_op;
mod index;
mod keep;
mod labels;
mod lambda;
mod list;
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
mod unify;
mod variants;
mod zonk;

/// Why a type must meet a trait, which names the verdict when it does not.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Origin {
    /// A bound a fn or impl states.
    Bound,
    /// `T(x)`: `x` is converted by its `@convert` impl (R04.2.28).
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
    Bin(BinOp, Box<TExpr>, Box<TExpr>, Overflow),
    Cmp(CmpOp, Box<TExpr>, Box<TExpr>),
    Not(Box<TExpr>),
    Neg(Box<TExpr>),
    /// `U64(x)`: an `Int`'s two's-complement bits.
    ToU64(Box<TExpr>),
    If(Box<TExpr>, Box<TExpr>, Box<TExpr>),
    Block(Vec<TStmt>, Option<Box<TExpr>>),
    ReadInt,
    /// The size of a domain type: `D.count`.
    DomLen(Ty),
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

/// What `Int` arithmetic does past `Int`: trap, or wrap where `with checks(overflow: off)`
/// covers it (R09.4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    Traps,
    Wraps,
}

#[derive(Clone, Debug)]
pub enum TStmt {
    Let(VarRef, TExpr),
    Assign(TExpr, TExpr),
    Expr(TExpr),
    While(TExpr, TExpr),
    For(VarRef, TExpr, TExpr, TExpr),
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
    pub written: Vec<(String, Option<ast::Type>)>,
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
}
pub struct Typed {
    pub domains: Vec<DomInfo>,
    pub fns: HashMap<FnId, TypedFn>,
    /// The copies of `uses` fns checked under context values a call gave them: `FnRef::ctx`
    /// `k` is the `k`-th (R09.1.4).
    pub ctx_fns: Vec<(FnId, TypedFn)>,
    pub globals: Vec<Local>,
    pub main: Vec<TStmt>,
    pub lambdas: Vec<LambdaInfo>,
    /// The closures a fn's body adds to its i-th set parameter, a callee's adds included.
    pub param_adds: HashMap<(FnId, usize), SetTy>,
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
    param_adds: HashMap<(FnId, usize), Vec<Member>>,
    /// `SetTy::AddedAt(i)`'s call and the slot of the set it passes there.
    added: Vec<(FnRef, usize)>,
    domains: Vec<DomInfo>,
    /// A body domain and the result's `domain C` it was returned as, which it then stands for;
    /// or a result's `domain C` and the domain its caller bound it to (R02.9.3).
    fresh_eq: Vec<(usize, usize)>,
    doms: HashMap<String, Ty>,
    /// The domains a binding owns, by the binding and the path to each in its value: `t`'s
    /// `Pre`, or `r`'s `tree.Pre` (R02.9.1).
    owned: HashMap<(VarRef, Vec<String>), usize>,
    /// A block's `@memo` fn's `f.clear`, by the fn's binding (R06.4.1).
    memo_clears: HashMap<VarRef, VarRef>,
    /// The `let`s bound to the `None` value: comparing one with a `?T` tests presence (R02.4.5).
    none_lets: HashSet<VarRef>,
    ranges: Vec<RangeInfo>,
    // per fn
    decl: Option<FnId>,
    generics: Vec<String>,
    n_sets: usize,
    locals: Vec<Local>,
    scopes: Vec<HashMap<String, LocalId>>,
    self_local: Option<LocalId>,
    /// Bounds calls must meet, with where: a type, the trait it must implement, and why.
    obligations: Vec<(Ty, decls::TraitRef, Span, Origin)>,
    /// Temporaries holding an input value whose parts are held to their types one by one.
    input_parts: std::collections::HashSet<VarRef>,
    ret: Ty,
    /// The results whose `_`s an earlier check of their bodies fixed (R02.9.3), and the ones
    /// this check fixes.
    solved_ret: HashMap<FnId, Ty>,
    found_ret: HashMap<FnId, Ty>,
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
    /// Where a `min=` / `max=` checked as a statement puts its `Bool`, when it is a condition.
    kept: Option<VarRef>,
    /// The name a `let` binds the lambda being checked to; and the lambdas whose bodies see
    /// their own name, with the scope their parameters live in.
    recur: Option<String>,
    selves: Vec<(String, LambdaId, usize)>,
    /// The nested `fn`s of the blocks being checked, by the offset of their statement; and the
    /// one whose lambda is being made.
    group: Vec<(u32, String, LambdaId)>,
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
    /// Inside `with checks(overflow: off)`: `Int` arithmetic wraps.
    wraps: bool,
    /// The `?` extents being checked, innermost last, and the statement right-hand side that is one.
    tries: Vec<question::Extent>,
    /// The bounds an `if T: Tr:` branch around this point has tested, of `T`'s parameter.
    assumed: Vec<(usize, TraitRef)>,
    rhs_extent: Option<Span>,
    /// The fields `with` lines set around the code being checked, innermost last (R09.1.2).
    frames: Vec<CtxFrame>,
    /// The lists comprehensions being checked fill, innermost last.
    filling: Vec<list::Filling>,
    /// Each copy of a `uses` fn made so far, by the fn and the context values it is checked
    /// under; and those still to check.
    /// The context copies this check made, by `FnRef::ctx`.
    ctx_made: HashSet<u32>,
    ctx_pending: Vec<(FnId, u32, Vec<CtxFrame>)>,
}

/// One field a `with` line set, read as `ctx.field` in the rest of its block.
#[derive(Clone)]
struct CtxFrame {
    ctx: decls::ContextId,
    field: String,
    value: ast::Expr,
}

/// The values of constant expressions: the `const_value` query (`handbook/design/next/
/// incremental.md`). The driver works each out by running it on the engine, the one evaluator,
/// so the checker holds no evaluator of its own and does not depend on the engine.
pub trait Consts {
    /// The value of `e`, written in program item `item`, when nothing it reads is decided at
    /// run time: `None` for a value that is not a scalar.
    fn value(&self, item: &ItemId, e: &ast::Expr) -> Result<Option<ops::Scalar>, ConstError>;
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

/// `roots`: library modules whose every fn is checked as the program's are, reached or not.
/// A result's `_` that is no domain is fixed by its fn's body (R02.9.3): a call checked before
/// that body is checked again, with each such result as the bodies fixed it.
/// What bodies make is named as `prior` named it where they make it again: an edit leaves
/// the bodies it does not touch typed the same (`Ids::default()` for a first check).
pub fn check(
    d: &Decls,
    main: &[(ItemId, ast::Stmt)],
    src: &crate::modules::Loaded,
    consts: &dyn Consts,
    roots: &[ModId],
    judge: Option<cpl_next_ir::build::Judge>,
    prior: &Ids,
) -> Typed {
    let mut solved = HashMap::new();
    loop {
        let (t, found) = check_with(
            d,
            main,
            src,
            consts,
            roots,
            judge.clone(),
            Earlier {
                ret: solved.clone(),
                ids: prior,
            },
        );
        if found.keys().all(|f| solved.contains_key(f)) {
            return t;
        }
        solved.extend(found);
    }
}

/// What earlier checks settled: each `_` result a body fixed (R02.9.3), and the ids of an
/// earlier revision (`check`'s `prior`).
struct Earlier<'p> {
    ret: HashMap<FnId, Ty>,
    ids: &'p Ids,
}

fn check_with(
    d: &Decls,
    main: &[(ItemId, ast::Stmt)],
    src: &crate::modules::Loaded,
    consts: &dyn Consts,
    roots: &[ModId],
    judge: Option<cpl_next_ir::build::Judge>,
    earlier: Earlier,
) -> (Typed, HashMap<FnId, Ty>) {
    let mut cx = Cx {
        values: consts,
        wraps: false,
        tries: Vec::new(),
        assumed: Vec::new(),
        rhs_extent: None,
        frames: Vec::new(),
        filling: Vec::new(),
        ctx_made: HashSet::new(),
        ctx_pending: Vec::new(),
        d,
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
        ids: earlier.ids.clone(),
        made: [0; 4],
        copy: 0,
        globals: Vec::new(),
        global_names: HashMap::new(),
        param_adds: HashMap::new(),
        added: Vec::new(),
        // A top-level domain's place is kept for it until its statement runs.
        domains: (d.top_doms.values())
            .filter(|t| matches!(t, Ty::Dom(_)))
            .map(|_| DomInfo {
                name: String::new(),
                len: VarRef::Global(usize::MAX),
                lo: VarRef::Global(usize::MAX),
                within: (None, None),
                owner: None,
                count: None,
                chars: false,
            })
            .collect(),
        fresh_eq: Vec::new(),
        doms: d.top_doms.clone(),
        owned: HashMap::new(),
        memo_clears: HashMap::new(),
        none_lets: HashSet::new(),
        ranges: Vec::new(),
        decl: None,
        generics: Vec::new(),
        n_sets: 0,
        locals: Vec::new(),
        scopes: vec![HashMap::new()],
        self_local: None,
        obligations: Vec::new(),
        input_parts: Default::default(),
        ret: Ty::Unit,
        solved_ret: earlier.ret,
        found_ret: HashMap::new(),
        region: None,
        loops: 0,
        in_for: false,
        loop_frames: Vec::new(),
        framed: None,
        allow_region: false,
        kept: None,
        recur: None,
        selves: Vec::new(),
        group: Vec::new(),
        ahead: Vec::new(),
        list_doms: Vec::new(),
        refused_names: Default::default(),
        nested: None,
        joining: false,
        item: ItemId::file(crate::modules::PROGRAM.into()),
        module: d.program,
        reached: Vec::new(),
        through_trait: Vec::new(),
        reported: HashSet::new(),
        consts: vec![None; d.consts.len()],
        const_busy: vec![false; d.consts.len()],
        const_init: Vec::new(),
    };
    // A result's `domain C` keeps its ends in one static slot per declaration site (R02.9.4).
    let slot = |cx: &mut Cx, role: &str| {
        cx.globals.push(Local {
            name: role.to_string(),
            ty: Ty::Int,
            top: true,
            mutable: true,
            fixed: false,
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
    let lifted: Vec<Option<ast::Stmt>> = (main.iter())
        .map(|(off, s)| {
            cx.item = off.clone();
            cx.hoisted(s)
        })
        .collect();
    for ((off, _), s) in main.iter().zip(&lifted) {
        if let Some(s) = s {
            cx.item = off.clone();
            main_out.push(TStmt::Item(off.clone()));
            cx.stmt(s, &mut main_out);
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
        cx.item = off.clone();
        // `@!multitest`: the count is read once and everything below runs once per case.
        if cases.is_none()
            && s.decorators
                .iter()
                .any(|d| d.directive && d.name.text == "multitest")
        {
            main_out.push(TStmt::Item(off.clone()));
            let count = cx.case_count(s).then_some(TExpr {
                kind: TK::ReadInt,
                ty: Ty::Int,
                span: s.span,
            });
            cx.scopes.push(HashMap::new());
            cases = Some((count, Vec::new()));
        }
        for x in &s.decorators {
            if !(x.directive && x.name.text == "multitest") {
                cx.err(
                    s.span,
                    Unbuilt::Decorator {
                        name: x.name.text.clone(),
                    },
                );
            }
        }
        // A hoisted item ran above; `@!multitest` on it still divides the file here.
        if hoisted.is_some() {
            continue;
        }
        let into = match &mut cases {
            Some((_, body)) => body,
            None => &mut main_out,
        };
        let bare_with = matches!(&s.kind, ast::StmtKind::With { body: None, .. });
        if bare_with && begun {
            let code = cpl_diag::codes::WITH_NOT_FIRST;
            cx.err(
                s.span,
                Msg::verdict(code, "a bare `with` line comes first in its block"),
            );
            continue;
        }
        begun |= !bare_with;
        into.push(TStmt::Item(off.clone()));
        if let Some((p, _)) = place::place_binding(s) {
            let items: Vec<(ItemId, ast::Stmt)> = (main[k..].iter().zip(&lifted[k..]))
                .filter(|(_, l)| l.is_none())
                .map(|((off, s), _)| (off.clone(), s.clone()))
                .collect();
            let stmts: Vec<ast::Stmt> = items[1..].iter().map(|(_, s)| s.clone()).collect();
            let held = place::place_held(&p, &stmts);
            placed = held;
            into.pop();
            cx.items_placed(&items[..=held], into);
            continue;
        }
        cx.stmt(s, into);
        if let Some(i) = cx.early.take() {
            let name = (d.top_doms.iter())
                .find(|(_, t)| matches!(t, Ty::Dom(k) if *k == i))
                .map(|(n, _)| n.clone())
                .unwrap_or_default();
            cx.err(
                s.span,
                Msg::verdict(
                    cpl_diag::codes::USED_BEFORE_SET,
                    format!("domain `{name}` is used above the line that declares it (R11.1.1)"),
                ),
            );
        }
    }
    cx.solve(true);
    if let Some((count, body)) = cases {
        cx.scopes.pop();
        main_out.push(TStmt::Cases(count, body));
    }
    // The program's fns are all checked; a library's, core's too, only once a reference reaches
    // them, so what a module declares and the program never uses refuses nothing.
    // A record's impls are made, not written: one is checked once a reference reaches it.
    let mut fns = HashMap::new();
    let mut todo: Vec<FnId> = (0..d.fns.len())
        .map(FnId)
        .filter(|id| {
            let f = d.f(*id);
            let later = |im: &decls::ImplDecl| {
                !d.live(im) || matches!(im.for_, Ty::Struct(s, ..) if d.s(s).record)
            };
            (roots.contains(&f.module) || !d.is_library(f.module) && f.module != decls::CORE)
                && !matches!(f.owner, Some(Owner::Impl(i)) if later(&d.impls[i]))
        })
        .collect();
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
        let undeclared = matches!(f.body, FnBody::None) && !of_trait;
        if fns.contains_key(&id) || lowered_intrinsic || undeclared {
            continue;
        }
        if !quiet {
            let tf = cx.check_fn(id, Vec::new());
            fns.insert(id, tf);
            continue;
        }
        quiet_fns.insert(id);
        let (n, reported) = (cx.errors.len(), cx.reported.clone());
        cx.reach(id);
        let tf = cx.check_fn(id, Vec::new());
        fns.insert(id, tf);
        let errors = cx.errors.split_off(n);
        cx.reported = reported;
        let more = std::mem::take(&mut cx.reached);
        cx.through_trait.extend(more);
        if !errors.is_empty() {
            deferred.insert(id, errors);
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
    for tf in fns.values_mut().chain(ctx_fns.iter_mut().map(|c| &mut c.2)) {
        cx.zonk_expr(&mut tf.body);
        for l in &mut tf.locals {
            l.ty = cx.zonk(&l.ty);
        }
        tf.ret = cx.zonk(&tf.ret);
        tf.region = (tf.region.take()).map(|(l, t)| (l, cx.zonk(&t)));
    }
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
    if cx.domains.iter().any(|d| d.name.is_empty()) {
        cx.errors.push(Unbuilt::UndeclaredTopDomain.into());
    }
    if cx.cyclic.get() {
        cx.errors.push(Msg::internal("a variable holding a function is assigned a function built from its own old value, so the set of functions it can hold contains itself; bind the new function to a new name"));
    }
    let typed = Typed {
        domains: std::mem::take(&mut cx.domains),
        fns,
        ctx_fns: ctx_fns.into_iter().map(|(_, id, tf)| (id, tf)).collect(),
        globals,
        main: main_z,
        lambdas,
        param_adds,
        call_adds,
        call_live,
        errors: cx.errors,
        deferred,
        quiet: quiet_fns,
        ids: cx.ids,
    };
    (typed, cx.found_ret)
}

impl<'a> Cx<'a> {
    /// Whether the `@!multitest` on `s` reads a count of cases: `(until_eof)` does not.
    fn case_count(&mut self, s: &ast::Stmt) -> bool {
        let args = s
            .decorators
            .iter()
            .find(|d| d.directive && d.name.text == "multitest")
            .map_or(&[][..], |d| &d.args[..]);
        match args {
            [] => true,
            [a] if a.name.is_none()
                && !a.mutable
                && matches!(&a.value.kind, ast::ExprKind::Name(n) if n == "until_eof") =>
            {
                false
            }
            _ => {
                self.err(s.span, Unbuilt::MultitestArg);
                true
            }
        }
    }
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
        let at = self.item.at(span);
        self.errors.push(Msg {
            help,
            ..Msg::at(at, msg)
        });
    }
    /// The source text `span` covers.
    fn text(&self, span: Span) -> &'a str {
        (self.src).text(&self.item.at(span))
    }
    /// A reference reached fn `id`: it is checked, and what its library declaration uses that
    /// the slice does not build is said once. A trait's method reaches every impl's.
    /// Which copy of `id` a call here makes: 0 when no context it `uses` is set around the
    /// call, else the one checked under the values set (R09.1.3, R09.1.4).
    pub(crate) fn ctx_copy(&mut self, id: FnId) -> u32 {
        let d = self.d;
        let f = d.f(id);
        let used: Vec<decls::ContextId> = (f.uses.iter())
            .filter_map(|n| match d.named(f.module, n) {
                Some(decls::Named::Context(c)) => Some(c),
                _ => None,
            })
            .collect();
        let mut seen = HashSet::new();
        let mut frames: Vec<CtxFrame> = (self.frames.iter().rev())
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
            let fd = self.d.f(f.decl);
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
                if let Some(&m) = im.methods.get(&f.name) {
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
            Some(Owner::Impl(i)) => Some(d.impls[i].trait_.tr),
            _ => None,
        };
        if let Some(t) = tr.filter(|t| self.reported.insert((3, t.0))) {
            said.extend(d.traits[t.0].errors.iter().cloned());
        }
        if self.reported.insert((2, f.module.0)) {
            said.extend(d.module_errors[f.module.0].iter().cloned());
        }
        self.errors
            .extend(said.into_iter().map(|e| e.in_module(module)));
    }
    fn check_fn(&mut self, id: FnId, frames: Vec<CtxFrame>) -> TypedFn {
        let d = self.d;
        let f = d.f(id);
        self.decl = Some(id);
        self.made = [0; 4];
        self.module = f.module;
        let top_doms = self.doms.clone();
        let top_owned = self.owned.clone();
        let top_clears = self.memo_clears.clone();
        self.generics = f.generics.clone();
        // A fn's result slots are set parameters after its own: each call passes its sets.
        self.n_sets = f.n_sets + f.n_ret_sets;
        self.locals = Vec::new();
        self.scopes = vec![HashMap::new()];
        self.item = f.item.clone();
        self.self_local = None;
        // A top-level fn reads the contexts it was written in, and a `uses` fn its caller's
        // (R09.1.3): `frames` are the values this copy of it is checked under.
        let frames = std::mem::replace(&mut self.frames, frames);
        if let Some(ty) = f.self_ty.clone().filter(|_| f.self_mode != SelfMode::None) {
            self.self_local = Some(self.add_local(&f.self_name, ty));
        }
        let mut params = Vec::new();
        for p in &f.params {
            let ty = self.fresh_sets(&p.ty);
            params.push(self.add_local(&p.name, ty));
        }
        let n = f.n_sets;
        self.region = (f.region.clone()).map(|(l, t)| (l, map_slots(&t, &|k| SetTy::Param(n + k))));
        let ret = self.decl_ret(id);
        let holes = d.ret_holes.contains(&id) && !unify::ground(&ret);
        let ret = if holes { self.open_holes(&ret) } else { ret };
        self.ret = map_slots(&ret, &|k| SetTy::Param(n + k));
        let ret = self.ret.clone();
        // A fn refused as declared (a decorator the slice lacks) is that refusal: its body,
        // written for the meaning the slice does not build, is not checked.
        let refused = (f.errors.iter()).any(|e| e.refusal.is_some());
        let body = match &f.body {
            _ if refused => Self::error_node(Span::new(0, 0)),
            // A trait's fn whose body is an operation on `Self`, as `Domain`'s are.
            _ if f.intrinsic.is_some() => {
                let span = Span::new(0, 0);
                let arg = match self.self_local {
                    Some(l) => Some((l, Ty::Param(0))),
                    None => params.first().map(|&l| (l, Ty::Int)),
                };
                let arg = arg.map(|(l, ty)| TExpr {
                    kind: TK::Var(VarRef::Local(l)),
                    ty,
                    span,
                });
                let op = f.intrinsic.clone().unwrap_or_default();
                self.domain_op(Ty::Param(0), &op, arg, span)
                    .unwrap_or_else(|| {
                        self.err(span, Unbuilt::TraitIntrinsic { op: op.to_string() });
                        TExpr {
                            kind: TK::Error,
                            ty: Ty::Error,
                            span,
                        }
                    })
            }
            FnBody::Block(b) => self.block(b, Some(&ret)),
            // A generator's one expression is the sequence it yields, or, when it yields itself
            // (`= match k: 0: yield 1 ...`), its body (R07.2.10).
            FnBody::Expr(e) if matches!(f.region, Some((false, _))) => {
                let body = if yields_in(e) {
                    one_stmt(e)
                } else {
                    yield_each(e)
                };
                self.block(&body, Some(&ret))
            }
            // An expression that never gives a value is the body, not its result (R01.6.7).
            FnBody::Expr(e) if e.leaves() => self.block(&one_stmt(e), Some(&ret)),
            // A `lends mut` fn's one expression is the place it lends (R05.6.3).
            FnBody::Expr(e) if matches!(f.region, Some((true, _))) => {
                let mut body = one_stmt(e);
                body.stmts[0].kind = StmtKind::Lend {
                    mutable: true,
                    place: e.clone(),
                };
                self.block(&body, Some(&ret))
            }
            // A `()` fn's one expression is a statement: its value is dropped, as it would be
            // last in a block (R06.1.1, R06.4.8).
            FnBody::Expr(e) if e.bare_try().is_none() && self.shallow(&ret) == Ty::Unit => {
                self.block(&one_stmt(e), Some(&ret))
            }
            FnBody::Expr(e) => {
                let v = match e.bare_try() {
                    Some(_) => self.extent(e, Some(&ret)),
                    None => self.expr(e, Some(&ret)),
                };
                self.coerce(v, &ret)
            }
            FnBody::None => match f.owner {
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
        self.frames = frames;
        let out = TypedFn {
            self_local: self.self_local,
            params,
            locals: std::mem::take(&mut self.locals),
            body,
            ret,
            region: self.region.take(),
        };
        self.decl = None;
        self.module = d.program;
        self.region = None;
        self.doms = top_doms;
        self.owned = top_owned;
        self.memo_clears = top_clears;
        self.generics = Vec::new();
        self.n_sets = 0;
        self.self_local = None;
        self.scopes = vec![HashMap::new()];
        out
    }
}

/// `for x in e: yield x`, the body of a generator written as one expression.
/// The block of the one statement `e`.
fn one_stmt(e: &ast::Expr) -> ast::Block {
    ast::Block {
        stmts: vec![ast::Stmt {
            kind: StmtKind::Expr(e.clone()),
            span: e.span,
            decorators: Vec::new(),
            public: false,
        }],
        span: e.span,
    }
}
/// Whether `e` holds a `yield` of its own.
fn yields_in(e: &ast::Expr) -> bool {
    struct Yields(bool);
    impl cpl_syntax::visit::Visitor for Yields {
        fn stmt(&mut self, s: &ast::Stmt) {
            self.0 |= matches!(s.kind, StmtKind::Yield { .. });
        }
    }
    let mut v = Yields(false);
    cpl_syntax::visit::walk_expr(e, &mut v);
    v.0
}
fn yield_each(e: &ast::Expr) -> ast::Block {
    let stmt = |kind| ast::Stmt {
        kind,
        span: e.span,
        decorators: Vec::new(),
        public: false,
    };
    let item = "$item".to_string();
    let each = stmt(StmtKind::Yield {
        value: ast::Expr {
            kind: ExprKind::Name(item.clone()),
            span: e.span,
        },
        from: false,
    });
    let walk = stmt(StmtKind::For {
        label: None,
        pat: ast::Pattern {
            kind: PatternKind::Name(item),
            span: e.span,
        },
        iter: e.clone(),
        body: ast::Block {
            stmts: vec![each],
            span: e.span,
        },
        else_body: None,
    });
    ast::Block {
        stmts: vec![walk],
        span: e.span,
    }
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
    fn const_value(&mut self, e: &ast::Expr) -> Option<Option<ops::Scalar>> {
        match self.values.value(&self.item, e) {
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
    /// `s` as it runs above the program, if it hoists: a `const` with a scalar value, an `Int`
    /// written in as its literal, or a domain whose ends have values.
    fn hoisted(&mut self, s: &ast::Stmt) -> Option<ast::Stmt> {
        match &s.kind {
            ast::StmtKind::Const {
                generics,
                init: Some(init),
                ..
            } if generics.is_empty() => {
                let v = self.const_value(init)??;
                let mut s = s.clone();
                if let (ops::Scalar::Int(i), ast::StmtKind::Const { init: Some(e), .. }) =
                    (v, &mut s.kind)
                {
                    e.kind = ast::ExprKind::Int(i128::from(i));
                }
                Some(s)
            }
            ast::StmtKind::Domain { def, .. } => {
                let ends: Vec<&ast::Expr> = match &def.kind {
                    ast::ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        step: None,
                        ..
                    } => vec![lo, hi],
                    ast::ExprKind::List(xs) if xs.len() == 1 => vec![&xs[0]],
                    _ => return None,
                };
                for e in ends {
                    self.const_value(e)??;
                }
                Some(s.clone())
            }
            _ => None,
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
            self.decl.take(),
            std::mem::take(&mut self.generics),
            std::mem::take(&mut self.n_sets),
            std::mem::take(&mut self.locals),
            std::mem::replace(&mut self.scopes, vec![HashMap::new(), HashMap::new()]),
            self.self_local.take(),
            std::mem::replace(&mut self.ret, Ty::Unit),
            self.region.take(),
            (
                std::mem::take(&mut self.loops),
                std::mem::take(&mut self.in_for),
            ),
            (std::mem::take(&mut self.allow_region), self.recur.take()),
            (
                std::mem::take(&mut self.selves),
                std::mem::take(&mut self.group),
            ),
            (std::mem::take(&mut self.ahead), self.nested.take()),
            std::mem::take(&mut self.joining),
            std::mem::replace(&mut self.item, k.item.clone()),
            std::mem::replace(&mut self.module, k.module),
        );
        let mut out = vec![TStmt::Item(k.item.clone())];
        let want = (k.ty.as_ref()).map(|t| self.body_ty(t, k.init.span, &mut out));
        let mut e = self.expr(&k.init, want.as_ref());
        if let Some(t) = &want {
            e = self.coerce(e, t);
        }
        (
            self.decl,
            self.generics,
            self.n_sets,
            self.locals,
            self.scopes,
        ) = (body.0, body.1, body.2, body.3, body.4);
        (self.self_local, self.ret, self.region) = (body.5, body.6, body.7);
        ((self.loops, self.in_for), (self.allow_region, self.recur)) = (body.8, body.9);
        ((self.selves, self.group), (self.ahead, self.nested)) = (body.10, body.11);
        (self.joining, self.item, self.module) = (body.12, body.13, body.14);
        self.globals.push(Local {
            name: k.name.clone(),
            ty: e.ty.clone(),
            top: true,
            mutable: false,
            fixed: true,
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
fn konst(e: &ast::Expr) -> String {
    match &e.kind {
        ast::ExprKind::Tuple(xs) => {
            format!("({})", xs.iter().map(konst).collect::<Vec<_>>().join(","))
        }
        ast::ExprKind::Unary { op, expr } => format!("{op:?}{}", konst(expr)),
        ast::ExprKind::Str(ps) => {
            let t: String = (ps.iter())
                .map(|p| match p {
                    ast::StrPiece::Text(t) => t.as_str(),
                    _ => "",
                })
                .collect();
            format!("{t:?}")
        }
        k => format!("{k:?}"),
    }
}
