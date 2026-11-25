//! Contract 3: declarations -> typed tree. Decides every type, target and
//! conversion. Nothing after this infers anything.
use crate::decls::{self, Decls, ModId, Named, Owner, SelfMode};
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
mod stmt;
mod unify;
mod zonk;

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
    Bool(bool),
    Unit,
    Var(VarRef),
    Field(Box<TExpr>, usize),
    Call {
        f: FnRef,
        recv: Option<Box<TArg>>,
        args: Vec<TArg>,
    },
    CallTrait {
        tr: TraitId,
        method: String,
        recv: Box<TArg>,
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
    /// A closure flows into a function-value slot.
    Inject(Box<TExpr>),
    /// A function value flows into a slot whose set is larger.
    Widen(Box<TExpr>),
    Bin(BinOp, Box<TExpr>, Box<TExpr>),
    Cmp(CmpOp, Box<TExpr>, Box<TExpr>),
    Not(Box<TExpr>),
    Neg(Box<TExpr>),
    If(Box<TExpr>, Box<TExpr>, Box<TExpr>),
    Block(Vec<TStmt>, Option<Box<TExpr>>),
    Print(Vec<Piece>),
    ReadInt,
    /// The size of a domain type: `D.count`.
    DomLen(Ty),
    /// `D(i)`: an Int becomes a value of the domain, checked.
    DomFrom(Ty, Box<TExpr>),
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
    Error,
}
#[derive(Clone, Debug)]
pub enum Piece {
    Text(String),
    Value(TExpr),
}
#[derive(Clone, Debug)]
pub enum TArg {
    Value(TExpr),
    /// A place lent for the call.
    Mut(TExpr),
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
        f: FnRef,
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
}
pub struct LambdaInfo {
    pub params: Vec<Ty>,
    pub ret: Ty,
    /// The fn it sits in; `None` for a top-level statement.
    pub owner: Option<FnId>,
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
}
pub struct Typed {
    pub domains: Vec<DomInfo>,
    pub fns: HashMap<FnId, TypedFn>,
    pub globals: Vec<Local>,
    pub main: Vec<TStmt>,
    pub lambdas: Vec<LambdaInfo>,
    /// The set a fn returns, for fns whose result is a function value.
    pub ret_sets: HashMap<(FnId, usize), SetTy>,
    pub errors: Vec<String>,
}

enum SetVar {
    Link(usize),
    Root(Vec<Member>),
    Bound(SetTy),
}

struct Cx<'a> {
    d: &'a Decls,
    src: &'a str,
    tys: Vec<Option<Ty>>,
    sets: Vec<SetVar>,
    /// Set variables being written out, to catch a set that contains itself.
    zonking: std::cell::RefCell<Vec<usize>>,
    cyclic: std::cell::Cell<bool>,
    errors: Vec<String>,
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
    ret: Ty,
    /// What the fn being checked yields or lends.
    region: Option<(bool, Ty)>,
    /// How many loops enclose the statement being checked, and a marker for `For`'s two-step entry.
    loops: u32,
    in_for: bool,
    /// Set while checking the one call that may be a generator or a lender.
    allow_region: bool,
    /// Set while converting the branches of an `if` into their common type: sets are joined, never merged.
    joining: bool,
    item_off: u32,
    /// The module whose names the code being checked sees.
    module: ModId,
    /// Fns a reference reached, to be checked; and the library places already reported.
    reached: Vec<FnId>,
    reported: HashSet<(u8, usize)>,
}

pub fn check(d: &Decls, main: &[(u32, ast::Stmt)], src: &str) -> Typed {
    let mut cx = Cx {
        d,
        src,
        tys: Vec::new(),
        sets: Vec::new(),
        zonking: Default::default(),
        cyclic: Default::default(),
        errors: Vec::new(),
        lambdas: Vec::new(),
        globals: Vec::new(),
        global_names: HashMap::new(),
        ret_sets: HashMap::new(),
        domains: Vec::new(),
        doms: HashMap::new(),
        ranges: Vec::new(),
        decl: None,
        generics: Vec::new(),
        n_sets: 0,
        locals: Vec::new(),
        scopes: vec![HashMap::new()],
        self_local: None,
        ret: Ty::Unit,
        region: None,
        loops: 0,
        in_for: false,
        allow_region: false,
        joining: false,
        item_off: 0,
        module: d.program,
        reached: Vec::new(),
        reported: HashSet::new(),
    };
    // The set each fn returns is one variable per declaration, filled as bodies are checked.
    for (id, f) in d.fns.iter().enumerate() {
        for k in 0..f.n_ret_sets {
            let v = cx.new_set();
            cx.ret_sets.insert((FnId(id), k), v);
        }
    }
    for (at, code, name) in &d.dup_fns {
        let line = src[..(*at).min(src.len())].matches('\n').count() + 1;
        cx.errors.push(format!(
            "line {line}: {code}: `{name}` is already a fn; one name is one fn"
        ));
    }
    for (i, s) in d.structs.iter().enumerate() {
        if let Some(f) = decls::field_holding_itself(d, StructId(i)) {
            let line = src[..s.field_at[f].min(src.len())].matches('\n').count() + 1;
            cx.errors.push(format!(
                "line {line}: {}: a `{}` would hold another `{}` inline, so it has no size; \
                 keep the others in a list and hold an index",
                cpl_diag::codes::RECURSIVE_FIELD.0,
                s.name,
                s.name
            ));
        }
    }
    for (i, a) in d.aliases.iter().enumerate() {
        if a.module == d.program && decls::alias_cycle(d, decls::AliasId(i)) {
            let line = src[..a.at.min(src.len())].matches('\n').count() + 1;
            cx.errors.push(format!(
                "line {line}: {}: type alias `{}` is defined by itself, so it names no type",
                cpl_diag::codes::ALIAS_CYCLE.0,
                a.name
            ));
        }
    }
    let mut main_out = Vec::new();
    let mut cases: Option<(Option<TExpr>, Vec<TStmt>)> = None;
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
        let into = match &mut cases {
            Some((_, body)) => body,
            None => &mut main_out,
        };
        into.push(TStmt::Item(*off));
        cx.stmt(s, into);
    }
    if let Some((count, body)) = cases {
        cx.scopes.pop();
        main_out.push(TStmt::Cases(count, body));
    }
    // The core's and the program's fns are all checked; a library's only once a reference
    // reaches them, so what a module declares and the program never uses refuses nothing.
    let mut fns = HashMap::new();
    let mut todo: Vec<FnId> = (0..d.fns.len())
        .map(FnId)
        .filter(|id| !d.is_library(d.f(*id).module))
        .rev()
        .collect();
    loop {
        todo.append(&mut cx.reached);
        let Some(id) = todo.pop() else { break };
        let f = d.f(id);
        if fns.contains_key(&id) || f.intrinsic || matches!(f.body, FnBody::None) {
            continue;
        }
        let tf = cx.check_fn(id);
        fns.insert(id, tf);
    }
    // Zonk: every variable is replaced by what it was solved to.
    let mut main_z = main_out;
    for s in &mut main_z {
        cx.zonk_stmt(s);
    }
    for tf in fns.values_mut() {
        cx.zonk_expr(&mut tf.body);
        for l in &mut tf.locals {
            l.ty = cx.zonk(&l.ty);
        }
        tf.ret = cx.zonk(&tf.ret);
    }
    let mut globals = std::mem::take(&mut cx.globals);
    for g in &mut globals {
        g.ty = cx.zonk(&g.ty);
    }
    let mut lambdas = std::mem::take(&mut cx.lambdas);
    for l in &mut lambdas {
        l.params = l.params.iter().map(|t| cx.zonk(t)).collect();
        l.ret = cx.zonk(&l.ret);
    }
    let ret_sets = cx
        .ret_sets
        .clone()
        .into_iter()
        .map(|(k, v)| (k, cx.zonk_set(&v)))
        .collect();
    if cx.cyclic.get() {
        cx.errors.push("a variable holding a function is assigned a function built from its own old value, so the set of functions it can hold contains itself; bind the new function to a new name".to_string());
    }
    Typed {
        domains: std::mem::take(&mut cx.domains),
        fns,
        globals,
        main: main_z,
        lambdas,
        ret_sets,
        errors: cx.errors,
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
    fn err(&mut self, span: Span, msg: impl Into<String>) {
        let at = (self.item_off + span.start) as usize;
        let line = self.src[..at.min(self.src.len())].matches('\n').count() + 1;
        self.errors.push(format!("line {line}: {}", msg.into()));
    }
    /// A reference reached fn `id`: it is checked, and what its library declaration uses that
    /// the slice does not build is said once. A trait's method reaches every impl's.
    pub(super) fn reach(&mut self, id: FnId) {
        let d = self.d;
        let f = d.f(id);
        self.reached.push(id);
        if let Some(Owner::Trait(t)) = f.owner {
            let impls = d.impls.iter().filter(|im| im.trait_ == t);
            self.reached
                .extend(impls.filter_map(|im| im.methods.get(&f.name).copied()));
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
        if self.reported.insert((2, f.module.0)) {
            said.extend(d.module_errors[f.module.0].iter().cloned());
        }
        self.errors
            .extend(said.into_iter().map(|e| format!("{module}: {e}")));
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
        if let Some(owner) = f.owner.filter(|_| f.self_mode != SelfMode::None) {
            let ty = match owner {
                Owner::Struct(id) => {
                    let s = d.s(id);
                    Ty::Struct(
                        id,
                        (0..s.generics.len()).map(Ty::Param).collect(),
                        (0..s.n_sets).map(SetTy::Param).collect(),
                    )
                }
                Owner::Trait(_) => Ty::Error,
            };
            self.self_local = Some(self.add_local("self", ty));
        }
        let mut params = Vec::new();
        for p in &f.params {
            let ty = self.fresh_sets(&p.ty);
            params.push(self.add_local(&p.name, ty));
        }
        self.region = f.region.clone();
        let slots = self.ret_sets.clone();
        self.ret = map_slots(&f.ret, &|k| slots[&(id, k)].clone());
        let ret = self.ret.clone();
        let body = match &f.body {
            FnBody::Block(b) => self.block(b, Some(&ret)),
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
        let out = TypedFn {
            self_local: self.self_local,
            params,
            locals: std::mem::take(&mut self.locals),
            body,
            ret,
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
