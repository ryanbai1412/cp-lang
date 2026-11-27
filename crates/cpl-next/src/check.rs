//! Contract 3: declarations -> typed tree. Decides every type, target and
//! conversion. Nothing after this infers anything.
use crate::decls::{self, Decls, ModId, Named, Owner, SelfMode, TraitRef};
use crate::diag::Msg;
use crate::ir::CheckKind;
use crate::ty::*;
use cpl_diag::Span;
use cpl_syntax::ast::{
    self, AssignOp, BinOp, CmpOp, Cond, ExprKind, FnBody, PatternKind, StmtKind, StrPiece,
    TypeKind, UnOp,
};
use std::collections::HashMap;
use std::collections::HashSet;

mod expr;
mod index;
mod lambda;
mod member;
mod oblige;
mod pattern;
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
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum VarRef {
    Local(LocalId),
    Global(LocalId),
}

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
        args: Vec<TArg>,
    },
    CallValue {
        callee: Box<TExpr>,
        args: Vec<TExpr>,
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
    Print(Vec<Piece>),
    ReadInt,
    /// The size of a domain type: `D.count`.
    DomLen(Ty),
    /// `D(i)`: an Int becomes a value of the domain, checked.
    DomFrom(Ty, Box<TExpr>),
    /// `D.nth(k)`: the position `k` steps into `D`, checked; of a type that is not a domain, `k`.
    DomNth(Ty, Box<TExpr>),
    /// `x in D`: whether the integer `x` is a position of `D`.
    DomHas(Ty, Box<TExpr>),
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
pub enum Piece {
    Text(String),
    Value(TExpr, cpl_next_ir::ops::Fmt),
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
    Item(u32),
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
pub struct LambdaInfo {
    pub params: Vec<Ty>,
    pub ret: Ty,
    /// The fn it sits in; `None` for a top-level statement.
    pub owner: Option<FnId>,
    /// A nested `fn`'s run of sibling `fn` statements, itself included; empty for a lambda.
    pub group: Vec<LambdaId>,
    /// What it yields: the closure a generator value walks (R07.6.2).
    pub yields: Option<Ty>,
}
/// A range with run-time ends: the variables that hold them, each inclusive; `None` is an open end.
struct RangeInfo {
    lo: Option<VarRef>,
    hi: Option<VarRef>,
}
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
}
pub struct Typed {
    pub domains: Vec<DomInfo>,
    pub fns: HashMap<FnId, TypedFn>,
    pub globals: Vec<Local>,
    pub main: Vec<TStmt>,
    pub lambdas: Vec<LambdaInfo>,
    /// The set a fn returns, for fns whose result is a function value.
    pub ret_sets: HashMap<(FnId, usize), SetTy>,
    pub errors: Vec<Msg>,
    /// The errors of each library fn reached only through a trait, said if an instance runs it.
    pub deferred: HashMap<FnId, Vec<Msg>>,
    /// The library fns reached only through a trait: what is wrong in one is said if it runs.
    pub quiet: HashSet<FnId>,
    /// Members the program or a library asked for that nothing loaded declares.
    pub missing: Vec<String>,
}

enum SetVar {
    Link(usize),
    Root(Vec<Member>),
    Bound(SetTy),
}

struct Cx<'a> {
    d: &'a Decls,
    /// The loaded modules the spans index, offset by `item_off`.
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
    /// The judge the program names (`// judge:`, else `generic`), for its memory limit (R03.1.9).
    judge: Option<cpl_judge::Profile>,
    errors: Vec<Msg>,
    lambdas: Vec<LambdaInfo>,
    globals: Vec<Local>,
    global_names: HashMap<String, LocalId>,
    ret_sets: HashMap<(FnId, usize), SetTy>,
    domains: Vec<DomInfo>,
    doms: HashMap<String, Ty>,
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
    ret: Ty,
    /// What the fn being checked yields or lends.
    region: Option<(bool, Ty)>,
    /// How many loops enclose the statement being checked, and a marker for `For`'s two-step entry.
    loops: u32,
    in_for: bool,
    /// Set while checking the one call that may be a generator or a lender.
    allow_region: bool,
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
    /// The names of items the checker refused: a use of one is downstream of that refusal.
    refused_names: std::collections::HashSet<String>,
    /// Set while converting the branches of an `if` into their common type: sets are joined, never merged.
    joining: bool,
    item_off: u32,
    /// The module whose names the code being checked sees.
    module: ModId,
    /// Fns a reference reached, to be checked; and the library places already reported.
    reached: Vec<FnId>,
    reported: HashSet<(u8, usize)>,
    /// Library fns reached only as a trait's impl, or from one: what they use that the slice
    /// does not build refuses only a program whose instances run them.
    through_trait: Vec<FnId>,
    /// Members asked for that nothing loaded declares.
    missing: Vec<String>,
    /// Each library `const`'s global, once its initializer is checked.
    consts: Vec<Option<LocalId>>,
    const_busy: Vec<bool>,
    /// The code that sets them, first reads first, above the program's.
    const_init: Vec<TStmt>,
    /// Inside `with checks(overflow: off)`: `Int` arithmetic wraps.
    wraps: bool,
}

pub fn check(d: &Decls, main: &[(u32, ast::Stmt)], src: &crate::modules::Loaded) -> Typed {
    let mut cx = Cx {
        wraps: false,
        d,
        src,
        tys: Vec::new(),
        sets: Vec::new(),
        zonking: Default::default(),
        cyclic: Default::default(),
        early: Default::default(),
        view_keys: HashMap::new(),
        judge: cpl_judge::builtin(cpl_judge::named_in(src.program()).unwrap_or("generic")),
        errors: Vec::new(),
        lambdas: Vec::new(),
        globals: Vec::new(),
        global_names: HashMap::new(),
        ret_sets: HashMap::new(),
        // A top-level domain's place is kept for it until its statement runs.
        domains: (d.top_doms.iter())
            .map(|_| DomInfo {
                name: String::new(),
                len: VarRef::Global(usize::MAX),
                lo: VarRef::Global(usize::MAX),
                within: (None, None),
                owner: None,
                count: None,
            })
            .collect(),
        doms: d.top_doms.clone(),
        ranges: Vec::new(),
        decl: None,
        generics: Vec::new(),
        n_sets: 0,
        locals: Vec::new(),
        scopes: vec![HashMap::new()],
        self_local: None,
        obligations: Vec::new(),
        ret: Ty::Unit,
        region: None,
        loops: 0,
        in_for: false,
        allow_region: false,
        recur: None,
        selves: Vec::new(),
        group: Vec::new(),
        ahead: Vec::new(),
        refused_names: Default::default(),
        nested: None,
        joining: false,
        item_off: 0,
        module: d.program,
        reached: Vec::new(),
        through_trait: Vec::new(),
        missing: Vec::new(),
        reported: HashSet::new(),
        consts: vec![None; d.consts.len()],
        const_busy: vec![false; d.consts.len()],
        const_init: Vec::new(),
    };
    // The set each fn returns is one variable per declaration, filled as bodies are checked.
    for (id, f) in d.fns.iter().enumerate() {
        for k in 0..f.n_ret_sets {
            let v = cx.new_set();
            cx.ret_sets.insert((FnId(id), k), v);
        }
    }
    for (at, code, msg) in &d.misdeclared {
        let at = *at as u32;
        cx.errors
            .push(Msg::at(Span::new(at, at), format!("{code}: {msg}")));
    }
    for (i, s) in (d.structs.iter().enumerate()).filter(|(_, s)| d.seen(s.module)) {
        if let Some(f) = decls::field_holding_itself(d, StructId(i)) {
            let at = s.field_at[f] as u32;
            cx.errors.push(Msg::at(
                Span::new(at, at),
                format!(
                    "{}: a `{}` would hold another `{}` inline, so it has no size; \
                 keep the others in a list and hold an index",
                    cpl_diag::codes::RECURSIVE_FIELD.0,
                    s.name,
                    s.name
                ),
            ));
        }
    }
    for (i, a) in d.aliases.iter().enumerate() {
        if a.module == d.program && decls::alias_cycle(d, decls::AliasId(i)) {
            let at = a.at as u32;
            cx.errors.push(Msg::at(
                Span::new(at, at),
                format!(
                    "{}: type alias `{}` is defined by itself, so it names no type",
                    cpl_diag::codes::ALIAS_CYCLE.0,
                    a.name
                ),
            ));
        }
    }
    let mut main_out = Vec::new();
    let mut cases: Option<(Option<TExpr>, Vec<TStmt>)> = None;
    // A domain with constant ends hoists, with the `const`s they read; one read at run time runs
    // where it is written (R11.1.1).
    let mut consts = std::collections::HashSet::new();
    for (_, s) in main {
        if let cpl_syntax::ast::StmtKind::Const {
            name,
            generics,
            init: Some(init),
            ..
        } = &s.kind
        {
            if generics.is_empty() && constant(init, &consts) {
                consts.insert(name.text.clone());
            }
        }
    }
    let hoists = |s: &cpl_syntax::ast::Stmt| match &s.kind {
        cpl_syntax::ast::StmtKind::Domain { def, .. } => constant_domain(def, &consts),
        cpl_syntax::ast::StmtKind::Const { name, .. } => consts.contains(&name.text),
        _ => false,
    };
    for (off, s) in main.iter().filter(|(_, s)| hoists(s)) {
        cx.item_off = *off;
        main_out.push(TStmt::Item(*off));
        cx.stmt(s, &mut main_out);
    }
    for (off, s) in main {
        cx.item_off = *off;
        // `@!multitest`: the count is read once and everything below runs once per case.
        if cases.is_none()
            && s.decorators
                .iter()
                .any(|d| d.directive && d.name.text == "multitest")
        {
            main_out.push(TStmt::Item(*off));
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
                cx.err(s.span, format!("slice: decorator `@{}`", x.name.text));
            }
        }
        // A hoisted item ran above; `@!multitest` on it still divides the file here.
        if hoists(s) {
            continue;
        }
        let into = match &mut cases {
            Some((_, body)) => body,
            None => &mut main_out,
        };
        into.push(TStmt::Item(*off));
        cx.stmt(s, into);
        if let Some(i) = cx.early.take() {
            let name = (d.top_doms.iter())
                .find(|(_, t)| matches!(t, Ty::Dom(k) if *k == i))
                .map(|(n, _)| n.clone())
                .unwrap_or_default();
            cx.err(
                s.span,
                format!(
                    "{}: domain `{name}` is used above the line that declares it (R11.1.1)",
                    cpl_diag::codes::USED_BEFORE_SET.0
                ),
            );
        }
    }
    cx.solve(true);
    if let Some((count, body)) = cases {
        cx.scopes.pop();
        main_out.push(TStmt::Cases(count, body));
    }
    // The core's and the program's fns are all checked; a library's only once a reference
    // reaches them, so what a module declares and the program never uses refuses nothing.
    let mut fns = HashMap::new();
    let mut todo: Vec<FnId> = (0..d.fns.len())
        .map(FnId)
        .filter(|id| {
            let f = d.f(*id);
            !d.is_library(f.module)
                && !matches!(f.owner, Some(Owner::Impl(i)) if !d.live(&d.impls[i]))
        })
        .collect();
    // In the order the collector made them, so a program over std checks as one collection would.
    todo.sort_by_key(|id| std::cmp::Reverse((d.f(*id).made, *id)));
    let mut deferred: HashMap<FnId, Vec<Msg>> = HashMap::new();
    let mut quiet_fns = HashSet::new();
    loop {
        todo.append(&mut cx.reached);
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
        if fns.contains_key(&id) || lowered_intrinsic || matches!(f.body, FnBody::None) {
            continue;
        }
        if !quiet {
            let tf = cx.check_fn(id);
            fns.insert(id, tf);
            continue;
        }
        quiet_fns.insert(id);
        let (n, reported) = (cx.errors.len(), cx.reported.clone());
        cx.reach(id);
        let tf = cx.check_fn(id);
        fns.insert(id, tf);
        let errors = cx.errors.split_off(n);
        cx.reported = reported;
        let more = std::mem::take(&mut cx.reached);
        cx.through_trait.extend(more);
        if !errors.is_empty() {
            deferred.insert(id, errors);
        }
    }
    // Zonk: every variable is replaced by what it was solved to.
    let mut main_z = std::mem::take(&mut cx.const_init);
    main_z.extend(main_out);
    for s in &mut main_z {
        cx.zonk_stmt(s);
    }
    for tf in fns.values_mut() {
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
    let ret_sets = cx
        .ret_sets
        .clone()
        .into_iter()
        .map(|(k, v)| (k, cx.zonk_set(&v)))
        .collect();
    if cx.domains.iter().any(|d| d.name.is_empty()) {
        cx.errors
            .push("slice: a top-level domain that is not declared".into());
    }
    if cx.cyclic.get() {
        cx.errors.push("a variable holding a function is assigned a function built from its own old value, so the set of functions it can hold contains itself; bind the new function to a new name".into());
    }
    Typed {
        domains: std::mem::take(&mut cx.domains),
        fns,
        globals,
        main: main_z,
        lambdas,
        ret_sets,
        errors: cx.errors,
        missing: cx.missing,
        deferred,
        quiet: quiet_fns,
    }
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
                self.err(
                    s.span,
                    "slice: an argument of `@!multitest` other than `until_eof`",
                );
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
    fn err(&mut self, span: Span, msg: impl Into<String>) {
        self.err_help(span, msg, None);
    }
    fn err_help(&mut self, span: Span, msg: impl Into<String>, help: Option<String>) {
        let at = Span::new(self.item_off + span.start, self.item_off + span.end);
        self.errors.push(Msg {
            help,
            ..Msg::at(at, msg)
        });
    }
    /// The source text `span` covers.
    fn text(&self, span: Span) -> &'a str {
        (self.src).text(Span::new(
            self.item_off + span.start,
            self.item_off + span.end,
        ))
    }
    /// A reference reached fn `id`: it is checked, and what its library declaration uses that
    /// the slice does not build is said once. A trait's method reaches every impl's.
    pub(super) fn reach(&mut self, id: FnId) {
        let d = self.d;
        let f = d.f(id);
        self.reached.push(id);
        if let Some(Owner::Trait(t)) = f.owner {
            for (_, im) in d.trait_impls(t) {
                if let Some(&m) = im.methods.get(&f.name) {
                    match d.is_library(im.module) {
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
    fn check_fn(&mut self, id: FnId) -> TypedFn {
        let d = self.d;
        let f = d.f(id);
        self.decl = Some(id);
        self.module = f.module;
        let top_doms = self.doms.clone();
        self.generics = f.generics.clone();
        self.n_sets = f.n_sets;
        self.locals = Vec::new();
        self.scopes = vec![HashMap::new()];
        self.item_off = f.item;
        self.self_local = None;
        if let Some(ty) = f.self_ty.clone().filter(|_| f.self_mode != SelfMode::None) {
            self.self_local = Some(self.add_local(&f.self_name, ty));
        }
        let mut params = Vec::new();
        for p in &f.params {
            let ty = self.fresh_sets(&p.ty);
            params.push(self.add_local(&p.name, ty));
        }
        let slots = self.ret_sets.clone();
        self.region =
            (f.region.clone()).map(|(l, t)| (l, map_slots(&t, &|k| slots[&(id, k)].clone())));
        self.ret = map_slots(&f.ret, &|k| slots[&(id, k)].clone());
        let ret = self.ret.clone();
        let body = match &f.body {
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
                        self.err(span, format!("slice: intrinsic `{op}` of a trait"));
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
            FnBody::Expr(e) => {
                let v = self.expr(e, Some(&ret));
                self.coerce(v, &ret)
            }
            FnBody::None => {
                let span = Span::new(0, 0);
                self.err(span, "slice: a fn with no body");
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
        };
        self.solve(true);
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
        self.generics = Vec::new();
        self.n_sets = 0;
        self.self_local = None;
        self.scopes = vec![HashMap::new()];
        out
    }
}

/// `lo..hi` or `lo..=hi` with integer literal ends.
/// A literal, a `const` named in `consts`, or `+ - *` of them.
fn constant(e: &cpl_syntax::ast::Expr, consts: &std::collections::HashSet<String>) -> bool {
    use cpl_syntax::ast::{BinOp, ExprKind};
    match &e.kind {
        ExprKind::Int(_) => true,
        ExprKind::Name(n) => consts.contains(n),
        ExprKind::Paren(x) => constant(x, consts),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul)
                && constant(lhs, consts)
                && constant(rhs, consts)
        }
        _ => false,
    }
}

/// `lo..hi` or `[n]` with constant ends.
fn constant_domain(e: &cpl_syntax::ast::Expr, consts: &std::collections::HashSet<String>) -> bool {
    use cpl_syntax::ast::ExprKind;
    let end =
        |e: &Option<Box<cpl_syntax::ast::Expr>>| e.as_deref().is_some_and(|e| constant(e, consts));
    match &e.kind {
        ExprKind::Range {
            lo, hi, step: None, ..
        } => end(lo) && end(hi),
        ExprKind::List(xs) => matches!(&xs[..], [x] if constant(x, consts)),
        _ => false,
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
    /// A library `const`'s global, checked once, when a read first reaches it: it is set at
    /// program start, above the program's code, after the consts its initializer reads (R01.6.1).
    fn library_const(&mut self, c: decls::ConstId, span: Span) -> Option<(VarRef, Ty)> {
        if let Some(g) = self.consts[c.0] {
            return Some((VarRef::Global(g), self.globals[g].ty.clone()));
        }
        let d = self.d;
        let k = &d.consts[c.0];
        if self.const_busy[c.0] {
            let code = cpl_diag::codes::DEFINITION_CYCLE.0;
            let msg = format!("{code}: const `{}` is read by its own initializer", k.name);
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
            std::mem::replace(&mut self.item_off, k.item),
            std::mem::replace(&mut self.module, k.module),
        );
        let mut out = vec![TStmt::Item(k.item)];
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
        (self.joining, self.item_off, self.module) = (body.12, body.13, body.14);
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
