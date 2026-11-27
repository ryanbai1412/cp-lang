//! Ownership: what each fn reads, writes and holds, decided per fn on the typed tree,
//! before any instance exists (spec chapter 5, R7, R9, R10, R12).
//!
//! A fn's summary names the places it touches: top-level bindings and its own parameters,
//! with the field path kept, so `p.a` and `p.b` do not overlap (R05.8.1). A call through a
//! function value is the join of the closures its set can hold; a set that is a parameter
//! of the fn stays symbolic in the summary and is filled in at each call.
use cpl_diag::{codes, Code, Span};
use cpl_next_check::*;
use cpl_next_decls::decls::{Decls, Owner, SelfMode};
use cpl_next_decls::diag::Msg;
use cpl_next_decls::plain::{plain, plain_in};
use cpl_next_ir::ir::{Arg, Expr, Place, PrintPiece, Site, Stmt, EK};
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::ty::*;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Root {
    Global(usize),
    Local(usize),
}
/// A variable and a path under it. An element at a computed key is the whole container; one at
/// a literal key is apart from the container's other literal keys (R05.8.2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Pl {
    root: Root,
    path: Vec<Step>,
}
impl Pl {
    fn whole(root: Root) -> Pl {
        Pl { root, path: vec![] }
    }
}
fn roots(ps: &BTreeSet<Pl>) -> BTreeSet<Root> {
    ps.iter().map(|p| p.root).collect()
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Step {
    Field(usize),
    Key(i64),
}
/// A summary through a call that recurses on a field (`Desc<Desc<T>>`'s `lt` calling `T`'s) would
/// name ever deeper places; past this depth it names the place holding them, which overlaps all.
const MAX_PATH: usize = 8;
fn overlap(a: &Pl, b: &Pl) -> bool {
    a.root == b.root && a.path.iter().zip(&b.path).all(|(x, y)| x == y)
}
fn hits<'a>(xs: impl IntoIterator<Item = &'a Pl>, ys: &'a BTreeSet<Pl>) -> Option<&'a Pl> {
    xs.into_iter().find(|x| ys.iter().any(|y| overlap(x, y)))
}

#[derive(Default, Clone, PartialEq, Debug)]
struct Fx {
    reads: BTreeSet<Pl>,
    writes: BTreeSet<Pl>,
    /// Lent to something that is still running when this fn yields or lends.
    held: BTreeSet<Pl>,
    /// Function values it calls whose set is a parameter of the fn.
    calls: BTreeSet<SetTy>,
    /// It yields a value that borrows its own locals, so a walk's item lends from the walk.
    yields_own: bool,
}
impl Fx {
    fn add(&mut self, o: &Fx) {
        self.reads.extend(o.reads.iter().cloned());
        self.writes.extend(o.writes.iter().cloned());
        self.held.extend(o.held.iter().cloned());
        self.calls.extend(o.calls.iter().cloned());
        self.yields_own |= o.yields_own;
    }
}

/// What a call reaches: what the callee does apart from its arguments, what it does to them,
/// and which of them it stores (receiver first).
#[derive(Default)]
struct Reach {
    other: Fx,
    through: Fx,
    keeps: BTreeSet<usize>,
}

struct Cx<'a> {
    d: &'a Decls,
    t: &'a Typed,
    fns: HashMap<FnId, Fx>,
    lams: HashMap<LambdaId, Fx>,
    /// What each nested fn reads from around it, its group's together (`check::group_free`).
    group_free: HashMap<LambdaId, BTreeSet<VarRef>>,
    /// The fn being walked; `None` for the file body.
    cur: Option<FnId>,
    off: u32,
    /// Enclosing loops over a generator or lender: what each holds for reading and for writing.
    regions: Vec<(BTreeSet<Pl>, BTreeSet<Pl>)>,
    report: bool,
    errors: Vec<Msg>,
    deferred: HashMap<FnId, Vec<Msg>>,
    /// Block nesting at this point, and for each binding where it was declared and in what order.
    scopes: Vec<u32>,
    next: u32,
    decl: HashMap<Root, (Vec<u32>, u32)>,
    /// R6: the variables a binding's value borrows (a closure borrows what it captures by reference).
    lent: HashMap<Root, BTreeSet<Pl>>,
    in_lambda: usize,
    /// R2: heap places moved out and not assigned since: a binding, or a field path under one.
    moved: BTreeSet<Pl>,
    /// Bindings that are themselves lends of a heap value: heap parameters and heap loop items.
    borrowed: BTreeSet<Root>,
    /// R7: for a binding that holds a heap borrow, the bindings that must not change while it is used.
    frozen: HashMap<Root, BTreeSet<Pl>>,
    /// A write or move that is an error if the binding is used afterwards.
    /// Errors that hold only if a binding is used again: the binding, where in the file the
    /// error is, and the error.
    pending: Vec<(Root, Span, Code, String)>,
    /// The heap bindings a function value held by a binding uses, top-level ones included.
    caps: HashMap<Root, BTreeSet<Root>>,
    /// Where each argument that is copied in starts.
    copies: BTreeSet<Span>,
    /// For each fn, the arguments it stores where they outlive the call (receiver first), so
    /// an owned heap value passed there moves in (R05.2.2).
    keeps: HashMap<FnId, BTreeSet<usize>>,
    /// The fn being walked's receiver and parameters, in call order.
    params: Vec<Root>,
    /// Those of them it stores where they outlive the call.
    kept: BTreeSet<Root>,
}

/// What ownership decides: the errors, and which passed places (`EK::Pass`) are copies, by
/// their span in the loaded source (`place_copies`).
#[derive(Default)]
pub struct Owned {
    pub errors: Vec<Msg>,
    /// What is wrong in a fn reached only through a trait, said if an instance runs it.
    pub deferred: HashMap<FnId, Vec<Msg>>,
    pub copies: BTreeSet<Span>,
}

/// The parameters a call to `f` lends to its result, receiver first, from its signature: the
/// heap ones it declares `lends` (R05.6.1), else each whose type is a type parameter the result
/// type holds, and, for a result whose type is a type parameter, each that holds it: the result
/// may be a cell of it (R05.5.3).
pub fn lends(d: &Decls, f: FnId) -> BTreeSet<usize> {
    let fd = d.f(f);
    let mut held = BTreeSet::new();
    params_in(&fd.ret, &mut held);
    let recv = (fd.self_mode != SelfMode::None).then_some(fd.self_ty.as_ref());
    let tys: Vec<&Ty> = (recv.flatten().into_iter())
        .chain(fd.params.iter().map(|p| &p.ty))
        .collect();
    (tys.iter().enumerate())
        .filter(|(i, t)| match &fd.lends {
            Some(ls) => ls.contains(i) && !plain(d, t, 0),
            None => match (t, &fd.ret) {
                (Ty::Param(k), _) => held.contains(k),
                (_, Ty::Param(k)) => {
                    let mut ps = BTreeSet::new();
                    params_in(t, &mut ps);
                    ps.contains(k)
                }
                _ => false,
            },
        })
        .map(|(i, _)| i)
        .collect()
}

pub fn check(d: &Decls, t: &Typed) -> Owned {
    let mut cx = Cx {
        d,
        t,
        fns: HashMap::new(),
        lams: HashMap::new(),
        group_free: HashMap::new(),
        cur: None,
        off: 0,
        regions: vec![],
        report: false,
        errors: vec![],
        deferred: HashMap::new(),
        scopes: vec![],
        next: 0,
        decl: HashMap::new(),
        lent: HashMap::new(),
        in_lambda: 0,
        moved: BTreeSet::new(),
        borrowed: BTreeSet::new(),
        frozen: HashMap::new(),
        pending: Vec::new(),
        caps: HashMap::new(),
        copies: BTreeSet::new(),
        keeps: HashMap::new(),
        params: Vec::new(),
        kept: BTreeSet::new(),
    };
    let mut ids: Vec<FnId> = t.fns.keys().copied().collect();
    ids.sort();
    for round in 0..40 {
        let before = (cx.fns.clone(), cx.lams.clone(), cx.keeps.clone());
        cx.pass(&ids);
        if before == (cx.fns.clone(), cx.lams.clone(), cx.keeps.clone()) {
            break;
        }
        assert!(round < 39, "effect summaries did not settle");
    }
    cx.report = true;
    cx.pass(&ids);
    cx.errors.sort();
    cx.errors.dedup();
    Owned {
        errors: cx.errors,
        deferred: cx.deferred,
        copies: cx.copies,
    }
}

impl Cx<'_> {
    fn pass(&mut self, ids: &[FnId]) {
        for &id in ids {
            let tf = &self.t.fns[&id];
            self.cur = Some(id);
            self.off = self.d.f(id).item;
            let mut fx = Fx::default();
            self.decl.clear();
            self.lent.clear();
            self.moved.clear();
            self.frozen.clear();
            self.pending.clear();
            self.caps.clear();
            self.kept.clear();
            self.params = (self.d.f(id).self_mode != SelfMode::None)
                .then_some(tf.self_local)
                .flatten()
                .into_iter()
                .chain(tf.params.iter().copied())
                .map(Root::Local)
                .collect();
            self.borrowed = tf
                .params
                .iter()
                .chain(tf.self_local.iter())
                .filter(|l| !self.plain(&tf.locals[**l].ty, 0))
                .map(|l| Root::Local(*l))
                .collect();
            self.expr(&tf.body, &mut fx);
            self.escapes(result_span(&tf.body), &tf.body);
            // Callers see only what outlives the call: top-level bindings and the parameters.
            let mut ps: Vec<usize> = tf.params.clone();
            ps.extend(tf.self_local);
            let keep = |p: &Pl| match p.root {
                Root::Global(_) => true,
                Root::Local(l) => ps.contains(&l),
            };
            fx.reads.retain(keep);
            fx.writes.retain(keep);
            fx.held.retain(keep);
            self.fns.insert(id, fx);
            let kept = (0..self.params.len())
                .filter(|i| self.kept.contains(&self.params[*i]))
                .collect();
            self.keeps.insert(id, kept);
        }
        self.cur = None;
        self.params.clear();
        self.kept.clear();
        self.decl.clear();
        self.lent.clear();
        self.moved.clear();
        self.frozen.clear();
        self.pending.clear();
        self.caps.clear();
        self.borrowed.clear();
        let mut fx = Fx::default();
        let t = self.t;
        for s in &t.main {
            self.stmt(s, &mut fx);
        }
    }
    /// Where a span is in the file. Spans are relative to the item they are in, so this is
    /// asked while that item is the one being walked.
    fn at(&self, span: Span) -> Span {
        Span::new(self.off + span.start, self.off + span.end)
    }
    fn err(&mut self, span: Span, code: Code, msg: String) {
        self.err_at(self.at(span), code, msg)
    }
    fn err_at(&mut self, at: Span, code: Code, msg: String) {
        if self.report {
            let e = Msg::at(at, Msg::verdict(code, msg));
            match self.cur.filter(|f| self.t.quiet.contains(f)) {
                Some(f) => self.deferred.entry(f).or_default().push(e),
                None => self.errors.push(e),
            }
        }
    }
    fn bind(&mut self, v: VarRef) {
        self.next += 1;
        let r = self.root(v);
        self.decl.insert(r, (self.scopes.clone(), self.next));
    }
    fn root_name(&self, r: Root) -> String {
        self.name(&Pl {
            root: r,
            path: vec![],
        })
    }
    /// R6: the variables this value borrows. A closure borrows the `var`s, heap values and heap
    /// parameters it uses (R05.10.1); a value built from lent values borrows what they do.
    fn sources(&self, e: &TExpr) -> BTreeSet<Pl> {
        let mut out = BTreeSet::new();
        if matches!(
            e.ty,
            Ty::Int
                | Ty::Range(..)
                | Ty::RangeOf(_)
                | Ty::Bool
                | Ty::Char
                | Ty::U64
                | Ty::F64
                | Ty::Unit
                | Ty::Dom(_)
        ) {
            return out;
        }
        match &e.kind {
            TK::Lambda { id, params, body } => {
                let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
                free_vars(body, &mut used, &mut bound);
                used.extend(self.group_free.get(id).into_iter().flatten().copied());
                for v in used.difference(&bound) {
                    let by_ref = match v {
                        VarRef::Local(l) if params.contains(l) && self.cur.is_some() => continue,
                        VarRef::Global(g) if params.contains(g) && self.cur.is_none() => continue,
                        VarRef::Local(l) => {
                            let Some(loc) = self
                                .cur
                                .and_then(|c| self.t.fns.get(&c))
                                .and_then(|f| f.locals.get(*l))
                            else {
                                continue;
                            };
                            loc.mutable || !self.plain(&loc.ty, 0)
                        }
                        // A generator value's arguments are its own, even the file's top-level ones.
                        VarRef::Global(g) => {
                            let loc = &self.t.globals[*g];
                            let held = self.t.lambdas[id.0].yields.is_some();
                            (held || !loc.top) && (loc.mutable || !self.plain(&loc.ty, 0))
                        }
                    };
                    let r = self.root(*v);
                    if by_ref {
                        out.insert(Pl::whole(r));
                    }
                    out.extend(self.lent.get(&r).into_iter().flatten().cloned());
                }
            }
            TK::Var(v) => {
                let r = self.root(*v);
                if self.borrowed.contains(&r) {
                    out.insert(Pl::whole(r));
                }
                out.extend(self.lent.get(&r).into_iter().flatten().cloned());
            }
            // R05.3.5: `s[i]` of a heap element lends `s`, or only that element of it when `i` is
            // a literal (R05.8.2).
            TK::Elem(x) if !self.plain(&e.ty, 0) && !matches!(e.ty, Ty::Param(_)) => {
                match &x.kind {
                    TK::Call {
                        recv: Some(r),
                        args,
                        ..
                    }
                    | TK::CallTrait {
                        recv: Some(r),
                        args,
                        ..
                    } => {
                        let (TArg::Value(v) | TArg::Mut(v)) = &**r;
                        match self.place(v) {
                            Some(mut p) => {
                                if let [TArg::Value(TExpr {
                                    kind: TK::Int(k), ..
                                })] = &args[..]
                                {
                                    p.path.push(Step::Key(*k));
                                }
                                out.insert(p);
                            }
                            None => out = self.sources(v),
                        }
                    }
                    _ => out = self.sources(x),
                }
            }
            TK::Elem(b) | TK::Slot(b, _) => out = self.sources(b),
            // R05.5.3: a result that holds a heap argument lends it while the result lives.
            TK::Call { f, recv, args } if !self.lent_args(f.decl).is_empty() => {
                let lends = self.lent_args(f.decl);
                let all = recv.as_deref().into_iter().chain(args.iter());
                for (i, a) in all.enumerate() {
                    let (TArg::Value(v) | TArg::Mut(v)) = a;
                    // A value of a type parameter that nothing lent is a copy in each instance.
                    let owned = matches!(v.ty, Ty::Param(_))
                        && (self.place(v)).is_some_and(|p| {
                            !self.borrowed.contains(&p.root) && !self.params.contains(&p.root)
                        });
                    if !lends.contains(&i) || self.plain(&v.ty, 0) || owned {
                        continue;
                    }
                    match self.place(v) {
                        Some(p) => {
                            out.extend(self.lent.get(&p.root).into_iter().flatten().cloned());
                            out.insert(Pl::whole(p.root));
                        }
                        None => out.extend(self.sources(v)),
                    }
                }
            }
            TK::Field(b, _) | TK::Payload(b, _) | TK::Inject(b) | TK::Widen(b) => {
                out = self.sources(b);
                // A heap field of a place is that place: binding it borrows the place.
                if let (true, Some(p), false) = (
                    matches!(e.kind, TK::Field(..) | TK::Payload(..)),
                    self.place(e),
                    self.plain(&e.ty, 0),
                ) {
                    if !self.decl.contains_key(&p.root) || !p.path.is_empty() {
                        out.insert(Pl::whole(p.root));
                    }
                }
            }
            // A field read into a struct moves into it (R05.2.2): the struct holds what the field
            // holds, not the place it was read from.
            TK::Struct(xs) => xs.iter().for_each(|x| {
                out.extend(match &x.kind {
                    TK::Field(b, _) | TK::Payload(b, _) => self.sources(b),
                    _ => self.sources(x),
                })
            }),
            TK::Variant(_, x) => out = self.sources(x),
            TK::If(_, a, b) => {
                out = self.sources(a);
                out.extend(self.sources(b));
            }
            TK::Block(_, Some(t)) => out = self.sources(t),
            _ => {}
        }
        out
    }
    /// The `@cache` field of `self` that the lends result `e` would borrow (R05.10.9).
    fn lends_cache(&self, e: &TExpr) -> Option<String> {
        let f = self.d.f(self.cur?);
        let (Some(Owner::Struct(s)), Some(_), true) =
            (f.owner, &f.lends, f.self_mode != SelfMode::None)
        else {
            return None;
        };
        match &e.kind {
            TK::Block(_, Some(t)) | TK::Elem(t) | TK::Slot(t, _) => self.lends_cache(t),
            TK::Field(..) => {
                let p = self.place(e)?;
                let Some(Step::Field(i)) = p.path.first() else {
                    return None;
                };
                let sd = self.d.s(s);
                (Some(&p.root) == self.params.first() && sd.cached.get(*i) == Some(&true))
                    .then(|| sd.fields[*i].0.clone())
            }
            TK::Call { f, recv, args } => {
                let lends = self.lent_args(f.decl);
                (recv.as_deref().into_iter().chain(args.iter()))
                    .enumerate()
                    .filter(|(i, _)| lends.contains(i))
                    .find_map(|(_, TArg::Value(v) | TArg::Mut(v))| self.lends_cache(v))
            }
            _ => None,
        }
    }
    /// R05.7.8: a lent value is not returned.
    fn escapes(&mut self, span: Span, e: &TExpr) {
        let out = &self.sources(e);
        if let Some(n) = self.lends_cache(e) {
            return self.err(
                span,
                codes::BAD_PROJECTION,
                format!(
                    "this lends `@cache` field `{n}`, which a later read may change (R05.10.9)"
                ),
            );
        }
        let mut out = roots(out);
        // A type-parameter value the result holds of a parameter is not a borrow here: an
        // instance whose type argument is lent copies it where it is kept (`EK::Lent` in inst.rs).
        out.retain(|r| !(self.params.contains(r) && self.generic(*r)));
        // The result borrows what the signature lends it (R05.5.3), or what a fn declared
        // `lends` names (R05.6.1).
        let declared = self.cur.and_then(|f| self.d.f(f).lends.clone());
        let lent = self.cur.map(|f| lends(self.d, f)).unwrap_or_default();
        for (i, r) in self.params.iter().enumerate() {
            if lent.contains(&i) || declared.as_ref().is_some_and(|ls| ls.contains(&i)) {
                out.remove(r);
            }
        }
        if let Some(r) = out.iter().next() {
            let n = self.root_name(*r);
            let why = if declared.is_some() && self.params.contains(r) {
                format!("the fn lends only from what its `lends(...)` names, not {n}")
            } else if self.borrowed.contains(r) {
                format!("{n} is lent by the caller; return a `.clone()`")
            } else {
                format!("{n} is gone when the fn returns")
            };
            self.err(
                span,
                codes::LENT_ESCAPES,
                format!("this returns a value that borrows {n}: {why}"),
            );
        }
    }
    /// R8: a lent value is kept only in storage that dies no later than what it borrows.
    fn keep(&mut self, span: Span, storage: Root, what: &BTreeSet<Pl>) {
        if what.is_empty() {
            return;
        }
        let home = self.decl.get(&storage).cloned();
        for s in &roots(what) {
            let ok = match (&home, self.decl.get(s)) {
                // A parameter's storage is the caller's, and a top-level binding outlives any fn.
                (None, _) => false,
                (Some(_), None) => true,
                (Some((hs, ho)), Some((ss, so))) => hs.starts_with(ss) && so < ho,
            };
            if !ok {
                let (a, b) = (self.root_name(storage), self.root_name(*s));
                self.err(
                    span,
                    codes::LENT_ESCAPES,
                    format!("a function that uses {b} is stored in {a}, which outlives {b}"),
                );
            }
        }
        self.lent
            .entry(storage)
            .or_default()
            .extend(what.iter().cloned());
    }
    /// Stores `what` into `storage`. A parameter whose type holds a type parameter, stored where
    /// it outlives the call, is one the fn keeps: each call moves an owned heap argument in
    /// (R05.2.2) and holds a lent one to R8 (R05.5.3). Anything else stored is a lend, held to R8
    /// here. A value lent from `storage` itself is free again once it is written back.
    fn store(&mut self, span: Span, storage: Root, mut what: BTreeSet<Pl>) {
        what.retain(|p| p.root != storage);
        if !self.decl.contains_key(&storage) {
            self.take_generic_params(&mut what);
        }
        self.keep(span, storage, &what);
    }
    /// A parameter of a type parameter's type that outlives the call, stored or returned, is
    /// kept: the caller moves a heap value in (R05.2.2) and copies a plain one.
    fn take_generic_params(&mut self, what: &mut BTreeSet<Pl>) {
        let own: Vec<Root> = (roots(what).into_iter())
            .filter(|r| self.params.contains(r) && self.generic(*r))
            .collect();
        for r in own {
            what.retain(|p| p.root != r);
            self.kept.insert(r);
        }
    }
    /// Whether a binding holds a fn value, whose lends are captures (R05.10.2).
    fn fn_value(&self, r: Root) -> bool {
        let ty = match r {
            Root::Local(l) => self
                .cur
                .and_then(|c| self.t.fns.get(&c))
                .and_then(|f| f.locals.get(l))
                .map(|loc| &loc.ty),
            Root::Global(g) => self.t.globals.get(g).map(|loc| &loc.ty),
        };
        matches!(ty, Some(Ty::Fn(..) | Ty::Clo(_)))
    }
    /// Whether a local's type holds a type parameter.
    fn generic(&self, r: Root) -> bool {
        fn has(t: &Ty) -> bool {
            match t {
                Ty::Param(_) => true,
                Ty::Struct(_, ts, _) => ts.iter().any(has),
                Ty::Raw(t) | Ty::Ref(t) => has(t),
                _ => false,
            }
        }
        let Root::Local(l) = r else { return false };
        self.cur
            .and_then(|c| self.t.fns.get(&c))
            .and_then(|f| f.locals.get(l))
            .is_some_and(|loc| has(&loc.ty))
    }
    /// The arguments a call to `f` stores where they outlive it, receiver first.
    fn kept_args(&self, f: FnId) -> BTreeSet<usize> {
        match &self.d.f(f).intrinsic {
            Some(n) => cpl_next_ir::ir::Intr::named(n)
                .map(|i| i.keeps().iter().copied().collect())
                .unwrap_or_default(),
            None => self.keeps.get(&f).cloned().unwrap_or_default(),
        }
    }
    /// The arguments a call to `f` lends to its result, receiver first (R05.5.3).
    fn lent_args(&self, f: FnId) -> BTreeSet<usize> {
        match &self.d.f(f).intrinsic {
            Some(n) => cpl_next_ir::ir::Intr::named(n)
                .map(|i| i.lends().iter().copied().collect())
                .unwrap_or_default(),
            None => lends(self.d, f),
        }
    }
    /// The binding `b` now holds `what`. Its heap borrows (not closure captures, R10) are frozen
    /// while it is used; a generator value's arguments are its borrows.
    fn holds(&mut self, b: Root, e: &TExpr, what: &BTreeSet<Pl>) {
        if what.is_empty() {
            return;
        }
        self.lent.entry(b).or_default().extend(what.iter().cloned());
        // A generator value holds what it walks, as a view does (R07.6.2).
        let walks = matches!(e.ty, Ty::Struct(s, ..) if Some(s) == self.d.recipe);
        if walks || !self.plain(&e.ty, 0) {
            self.frozen
                .entry(b)
                .or_default()
                .extend(what.iter().cloned());
        }
    }
    /// A write to `p`: an error if a binding that borrows it is used later.
    fn wrote(&mut self, span: Span, p: &Pl) {
        let n = self.root_name(p.root);
        for (b, srcs) in &self.frozen {
            if *b != p.root && srcs.iter().any(|s| overlap(s, p)) {
                let bn = self.root_name(*b);
                let at = self.at(span);
                self.pending.push((
                    *b,
                    at,
                    codes::USED_WHILE_LENT,
                    format!("{n} is written here while {bn}, which borrows it, is still used"),
                ));
            }
        }
    }
    fn live(&mut self, span: Span, p: &Pl) {
        if let Some(i) = self.pending.iter().position(|x| x.0 == p.root) {
            let (_, at, code, msg) = self.pending.remove(i);
            self.pending.retain(|x| x.0 != p.root);
            self.err_at(at, code, msg);
        }
        if let Some(m) = self.moved.iter().find(|m| overlap(m, p)).cloned() {
            let n = self.name(&m);
            self.err(
                span,
                codes::USE_AFTER_MOVE,
                format!("{n} was moved into another binding; use `.clone()` there to keep both"),
            );
        }
    }
    /// R2: `let b = a` and `x = a` move a heap value out of a binding this block owns. A
    /// field read is a lend of it (R05.3.5), so `let b = a.xs` moves nothing.
    fn take(&mut self, e: &TExpr) {
        if let TK::Var(_) = e.kind {
            self.take_path(e);
        }
    }
    /// A move out of a binding or a field path under it: a struct built from `a.xs` (a record
    /// update `(..a, k: 3)` among them, R12.11.4) takes the path, so `a.xs` and `a` as a whole
    /// are gone, and `a`'s other fields stay.
    fn take_path(&mut self, e: &TExpr) {
        if let Some(p) = self.field_path(e) {
            let r = p.root;
            if self.decl.contains_key(&r)
                && !self.borrowed.contains(&r)
                && !self.lent.contains_key(&r)
                && !self.plain(&e.ty, 0)
            {
                self.moved.insert(p);
                // R05.10.2: a captured variable can't be moved while the fn value is alive.
                let n = self.root_name(r);
                // A value that holds a lend of it is E0602, a fn value that captures it E0607.
                let lent = self.lent.iter().map(|(b, s)| (b, roots(s)));
                let caps = self.caps.iter().map(|(b, s)| (b, s.clone()));
                let users: Vec<(Root, BTreeSet<Root>)> =
                    lent.chain(caps).map(|(b, s)| (*b, s)).collect();
                for (b, srcs) in &users {
                    if srcs.contains(&r) {
                        let code = if self.caps.contains_key(b) || self.fn_value(*b) {
                            codes::MOVED_WHILE_CAPTURED
                        } else {
                            codes::USED_WHILE_LENT
                        };
                        let bn = self.root_name(*b);
                        let at = self.at(e.span);
                        self.pending.push((
                            *b,
                            at,
                            code,
                            format!("{n} is moved here while {bn}, which uses it, is still used"),
                        ));
                    }
                }
            }
        }
    }
    /// A move inside a loop of a binding declared before it leaves nothing for the next turn.
    fn moved_in_loop(&mut self, span: Span, at: u32, before: &BTreeSet<Pl>) {
        let gone: Vec<Pl> = self
            .moved
            .difference(before)
            .filter(|p| self.decl.get(&p.root).is_some_and(|(_, o)| *o <= at))
            .cloned()
            .collect();
        for p in gone {
            let n = self.name(&p);
            self.err(
                span,
                codes::USE_AFTER_MOVE,
                format!("{n} is moved inside a loop, so the next turn would use it after the move"),
            );
        }
    }
    fn root(&self, v: VarRef) -> Root {
        match v {
            VarRef::Local(l) => Root::Local(l),
            VarRef::Global(g) => Root::Global(g),
        }
    }
    /// A binding, or a field path under one: a place a value moves out of whole.
    fn field_path(&self, e: &TExpr) -> Option<Pl> {
        match &e.kind {
            TK::Var(_) => self.place(e),
            TK::Field(b, _) => self.field_path(b).and_then(|_| self.place(e)),
            _ => None,
        }
    }
    fn place(&self, e: &TExpr) -> Option<Pl> {
        match &e.kind {
            TK::Var(v) => Some(Pl {
                root: self.root(*v),
                path: vec![],
            }),
            TK::Field(b, i) | TK::Payload(b, i) => {
                let mut p = self.place(b)?;
                p.path.push(Step::Field(*i));
                Some(p)
            }
            // A cell is part of its storage: a write to it writes the storage, or only the part
            // that is that element when its key is a literal.
            TK::Slot(b, k) => {
                let mut p = self.place(b)?;
                if let TK::Int(k) = k.kind {
                    p.path.push(Step::Key(k));
                }
                Some(p)
            }
            _ => None,
        }
    }
    /// The fields of struct type `t` whose declared type is one of its type parameters.
    fn generic_fields(&self, t: &Ty) -> BTreeSet<usize> {
        let Ty::Struct(id, ..) = t else {
            return BTreeSet::new();
        };
        (self.d.s(*id).fields.iter().enumerate())
            .filter(|(_, (_, ft))| matches!(ft, Ty::Param(_)))
            .map(|(i, _)| i)
            .collect()
    }
    fn plain(&self, t: &Ty, depth: usize) -> bool {
        let doms = (self.cur).and_then(|c| self.d.domain_params.get(&c));
        plain_in(self.d, t, depth, doms.map_or(&[], |v| v.as_slice()))
    }
    fn name(&self, p: &Pl) -> String {
        let (mut s, mut ty) = match p.root {
            Root::Global(g) => (self.t.globals[g].name.clone(), self.t.globals[g].ty.clone()),
            Root::Local(l) => match self
                .cur
                .and_then(|c| self.t.fns.get(&c))
                .and_then(|f| f.locals.get(l))
            {
                Some(loc) => (loc.name.clone(), loc.ty.clone()),
                None => return "a local value".into(),
            },
        };
        for step in &p.path {
            let i = match step {
                Step::Field(i) => i,
                Step::Key(k) => {
                    s = format!("{s}[{k}]");
                    break;
                }
            };
            let Ty::Struct(id, targs, sets) = &ty else {
                break;
            };
            let sd = self.d.s(*id);
            let Some((f, ft)) = sd.fields.get(*i) else {
                break;
            };
            s = format!("{s}.{f}");
            ty = Subst { tys: targs, sets }.ty(ft);
        }
        format!("`{s}`")
    }

    /// What calling any closure of this set does.
    fn set_fx(&self, s: &SetTy, depth: usize) -> Fx {
        let mut out = Fx::default();
        if depth > 12 {
            return out;
        }
        match s {
            SetTy::Param(_) => {
                out.calls.insert(s.clone());
            }
            SetTy::Closed(ms) => {
                for m in ms {
                    match m {
                        Member::Sub(s) => out.add(&self.set_fx(s, depth + 1)),
                        Member::Clo(CloRef::Item(f)) => {
                            let (other, _) = self.callee(f, &[], depth + 1, true);
                            out.add(&other);
                        }
                        Member::Clo(CloRef::Lambda(id, ts, ss)) => {
                            let Some(l) = self.lams.get(id) else { continue };
                            let here = self.t.lambdas[id.0].owner == self.cur;
                            let mut fx = l.clone();
                            if !here {
                                // Its captured variables are not names of this fn (R8 keeps it from outliving them).
                                let g = |p: &Pl| matches!(p.root, Root::Global(_));
                                fx.reads.retain(g);
                                fx.writes.retain(g);
                                fx.held.retain(g);
                            }
                            let calls = std::mem::take(&mut fx.calls);
                            out.add(&fx);
                            for c in calls {
                                let c = if here {
                                    c
                                } else {
                                    Subst { tys: ts, sets: ss }.set(&c)
                                };
                                if here && matches!(c, SetTy::Param(_)) {
                                    out.calls.insert(c);
                                } else {
                                    out.add(&self.set_fx(&c, depth + 1));
                                }
                            }
                        }
                    }
                }
            }
            SetTy::RetOf(f, k) => {
                if let Some(rs) = self.t.ret_sets.get(&(f.decl, *k)) {
                    let rs = Subst {
                        tys: &f.targs,
                        sets: &f.sets,
                    }
                    .set(rs);
                    out.add(&self.set_fx(&rs, depth + 1));
                }
            }
            SetTy::AddedAt(c) => {
                if let Some((f, k)) = self.t.call_adds.get(*c) {
                    if let Some(s) = self.t.param_adds.get(&(f.decl, *k)) {
                        let s = Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        }
                        .set(s);
                        out.add(&self.set_fx(&s, depth + 1));
                    }
                }
            }
            SetTy::RetSlot(_) | SetTy::Var(_) => {}
        }
        out
    }
    /// A call to `f` with these argument places (receiver first): what it does apart from
    /// its arguments, and what it does to them.
    /// What a call of `f` touches. `exact` says `f`'s parameters have the arguments' types, so
    /// a place inside a parameter is the same place inside its argument; a trait call's impls
    /// are of other types, so there each place is its whole argument.
    fn callee(&self, f: &FnRef, args: &[Option<Pl>], depth: usize, exact: bool) -> (Fx, Fx) {
        let (mut other, mut through) = (Fx::default(), Fx::default());
        let fd = self.d.f(f.decl);
        let Some(tf) = self.t.fns.get(&f.decl) else {
            return (other, through);
        };
        let Some(sum) = self.fns.get(&f.decl) else {
            return (other, through);
        };
        let mut ps: Vec<usize> = Vec::new();
        if fd.self_mode != SelfMode::None {
            ps.extend(tf.self_local);
        }
        ps.extend(tf.params.iter().copied());
        let map = |p: &Pl| -> Option<(bool, Pl)> {
            match p.root {
                Root::Global(_) => Some((false, p.clone())),
                Root::Local(l) => {
                    let i = ps.iter().position(|x| *x == l)?;
                    let mut q = args.get(i)?.clone()?;
                    if exact {
                        q.path.extend(p.path.iter().copied());
                        q.path.truncate(MAX_PATH);
                    }
                    Some((true, q))
                }
            }
        };
        for (from, pick) in [(&sum.reads, 0), (&sum.writes, 1), (&sum.held, 2)] {
            for p in from {
                if let Some((arg, q)) = map(p) {
                    let fx = if arg { &mut through } else { &mut other };
                    match pick {
                        0 => fx.reads.insert(q),
                        1 => fx.writes.insert(q),
                        _ => fx.held.insert(q),
                    };
                }
            }
        }
        let s = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        for c in &sum.calls {
            other.add(&self.set_fx(&s.set(c), depth + 1));
        }
        (other, through)
    }

    /// One call (R10): its arguments, what the callee touches by name, and what the closures
    /// it can call touch, are one access. Returns what it holds for reading and for writing.
    fn call(
        &mut self,
        who: &str,
        span: Span,
        args: &[&TArg],
        callee: Reach,
        fx: &mut Fx,
    ) -> (BTreeSet<Pl>, BTreeSet<Pl>) {
        let Reach {
            other,
            through,
            keeps,
        } = callee;
        let (mut held, mut lent) = (BTreeSet::new(), BTreeSet::new());
        for a in args {
            match a {
                TArg::Value(v) => match self.place(v) {
                    Some(p) if !self.plain(&v.ty, 0) => {
                        self.live(v.span, &p);
                        fx.reads.insert(p.clone());
                        // R4: the callee keeps the value it was called with, so a place the
                        // call also writes is copied in, not viewed.
                        if other.writes.iter().any(|w| overlap(w, &p)) {
                            self.copies.insert(Span {
                                start: self.off + v.span.start,
                                end: self.off + v.span.end,
                            });
                        } else {
                            held.insert(p);
                        }
                    }
                    _ => self.expr(v, fx),
                },
                TArg::Mut(v) => {
                    let Some(p) = self.place(v) else {
                        self.expr(v, fx);
                        continue;
                    };
                    self.live(v.span, &p);
                    if let Some(q) = lent.iter().find(|q| overlap(q, &p)) {
                        let n = self.name(q);
                        self.err(
                            span,
                            codes::LENT_TWICE,
                            format!("{n} is lent to `{who}` for writing twice in the same call"),
                        );
                    }
                    lent.insert(p);
                }
            }
        }
        if let Some(p) = hits(&held, &lent) {
            let n = self.name(p);
            self.err(
                span,
                codes::LENT_TWICE,
                format!("{n} is passed to `{who}` and lent to it for writing in the same call"),
            );
        }
        if let Some(p) = hits(&lent, &other.writes).or_else(|| hits(&lent, &other.reads)) {
            let n = self.name(p);
            self.err(
                span,
                codes::USED_WHILE_LENT,
                format!(
                    "{n} is lent to `{who}` for writing, and the call also reaches it another way"
                ),
            );
        }
        // What the callee stores: an owned heap value moves in, a lent one is kept by what the
        // call writes, so it must outlive that (R8).
        let mut kept = BTreeSet::new();
        for (i, a) in args.iter().enumerate() {
            if let (TArg::Value(v), true) = (a, keeps.contains(&i)) {
                self.take(v);
                kept.extend(self.sources(v));
            }
        }
        for p in &lent {
            self.wrote(span, p);
            self.store(span, p.root, kept.clone());
        }
        fx.writes.extend(lent.iter().cloned());
        fx.add(&other);
        fx.reads.extend(through.reads);
        fx.writes.extend(through.writes);
        held.extend(other.held.iter().cloned());
        held.extend(through.held);
        (held, lent)
    }
    fn arg_places(&self, args: &[&TArg]) -> Vec<Option<Pl>> {
        args.iter()
            .map(|a| match a {
                TArg::Value(v) | TArg::Mut(v) => self.place(v),
            })
            .collect()
    }
    fn fn_call(
        &mut self,
        f: &FnRef,
        span: Span,
        recv: Option<&TArg>,
        args: &[TArg],
        fx: &mut Fx,
    ) -> (BTreeSet<Pl>, BTreeSet<Pl>) {
        let all: Vec<&TArg> = recv.into_iter().chain(args.iter()).collect();
        let (other, through) = self.callee(f, &self.arg_places(&all), 0, true);
        let who = self.d.f(f.decl).name.clone();
        let keeps = self.kept_args(f.decl);
        let callee = Reach {
            other,
            through,
            keeps,
        };
        self.call(&who, span, &all, callee, fx)
    }

    /// A call of a bound's method: which impl runs is known per instance, so the summary is every impl's.
    fn trait_call(
        &mut self,
        tr: TraitId,
        method: &str,
        span: Span,
        recv: Option<&TArg>,
        args: &[TArg],
        fx: &mut Fx,
    ) -> (BTreeSet<Pl>, BTreeSet<Pl>) {
        // Which impl runs is known per instance; the summary is every impl's.
        let all: Vec<&TArg> = recv.into_iter().chain(args.iter()).collect();
        let places = self.arg_places(&all);
        let (mut other, mut through) = (Fx::default(), Fx::default());
        let mut keeps = BTreeSet::new();
        // An impl that leaves the fn out runs the trait's default.
        let default =
            (self.d.traits[tr.0].methods.get(method)).filter(|m| self.t.fns.contains_key(m));
        let impls = self.d.trait_impls(tr).map(|(_, im)| im);
        for m in impls.filter_map(|im| im.methods.get(method).or(default)) {
            let f = FnRef {
                decl: *m,
                targs: vec![],
                sets: vec![],
                ctx: 0,
            };
            let (o, t) = self.callee(&f, &places, 0, false);
            other.add(&o);
            through.add(&t);
            keeps.extend(self.kept_args(*m));
        }
        let callee = Reach {
            other,
            through,
            keeps,
        };
        self.call(method, span, &all, callee, fx)
    }

    fn expr(&mut self, e: &TExpr, fx: &mut Fx) {
        match &e.kind {
            TK::Int(_)
            | TK::Float(_)
            | TK::Bool(_)
            | TK::Unit
            | TK::Unset
            | TK::Text(_)
            | TK::ReadInt
            | TK::DomLen(_)
            | TK::FnItem(_)
            | TK::Itself(_)
            | TK::Error => {}
            TK::Slot(b, i) => {
                self.expr(i, fx);
                match self.place(b) {
                    Some(p) => {
                        self.live(e.span, &p);
                        fx.reads.insert(p);
                    }
                    None => self.expr(b, fx),
                }
            }
            TK::Var(_) | TK::Field(..) | TK::Payload(..) => match self.place(e) {
                Some(p) => {
                    self.live(e.span, &p);
                    fx.reads.insert(p);
                }
                None => {
                    if let TK::Field(b, _) | TK::Payload(b, _) = &e.kind {
                        self.expr(b, fx)
                    }
                }
            },
            TK::Call { f, recv, args } => {
                self.fn_call(f, e.span, recv.as_deref(), args, fx);
            }
            TK::CallTrait {
                tr,
                method,
                recv,
                args,
                ..
            } => {
                self.trait_call(tr.tr, method, e.span, recv.as_deref(), args, fx);
            }
            TK::CallValue { callee, args } => {
                self.expr(callee, fx);
                let other = match &callee.ty {
                    Ty::Fn(_, _, s) => self.set_fx(s, 0),
                    Ty::Clo(c) => self.set_fx(&SetTy::Closed(vec![Member::Clo((**c).clone())]), 0),
                    _ => Fx::default(),
                };
                let vals: Vec<TArg> = args.iter().cloned().map(TArg::Value).collect();
                let all: Vec<&TArg> = vals.iter().collect();
                let callee = Reach {
                    other,
                    ..Reach::default()
                };
                self.call("the function value", e.span, &all, callee, fx);
            }
            TK::Struct(xs) => {
                let generic = self.generic_fields(&e.ty);
                for (i, x) in xs.iter().enumerate() {
                    self.expr(x, fx);
                    // R05.5.3: a type-parameter field holds a heap place by lending it; any other
                    // field keeps its argument, so an owned heap binding moves in (R05.5).
                    if !(generic.contains(&i) && self.place(x).is_some()) {
                        self.take_path(x);
                    }
                }
            }
            TK::Lambda { id, body, .. } => {
                let mut inner = Fx::default();
                let saved = std::mem::take(&mut self.regions);
                self.in_lambda += 1;
                self.expr(body, &mut inner);
                self.in_lambda -= 1;
                self.regions = saved;
                self.lams.insert(*id, inner);
            }
            TK::Inject(x)
            | TK::Widen(x)
            | TK::Elem(x)
            | TK::Not(x)
            | TK::CharOf(x)
            | TK::CodeOf(x)
            | TK::Neg(x)
            | TK::ToU64(x)
            | TK::DomFrom(_, x)
            | TK::DomNth(_, x)
            | TK::DomHas(_, x)
            | TK::DomOrd(x)
            | TK::Ordinal(x)
            | TK::Variant(_, x)
            | TK::Tag(x) => self.expr(x, fx),
            TK::Bin(_, a, b, _) | TK::Cmp(_, a, b) => {
                self.expr(a, fx);
                self.expr(b, fx);
            }
            TK::If(c, a, b) => {
                self.expr(c, fx);
                let before = self.moved.clone();
                self.expr(a, fx);
                let after_a = std::mem::replace(&mut self.moved, before);
                self.expr(b, fx);
                self.moved.extend(after_a);
            }
            TK::Block(ss, tail) => {
                self.next += 1;
                self.scopes.push(self.next);
                let groups = group_free(&self.t.lambdas, ss);
                self.group_free.extend(groups.clone());
                ss.iter().for_each(|s| self.stmt(s, fx));
                // Calling one fn of a group may run any of them.
                for id in groups.keys() {
                    let mut all = Fx::default();
                    for s in &self.t.lambdas[id.0].group {
                        all.add(self.lams.get(s).unwrap_or(&Fx::default()));
                    }
                    self.lams.insert(*id, all);
                }
                if let Some(t) = tail {
                    self.expr(t, fx)
                }
                self.scopes.pop();
            }
            TK::Print(ps) => {
                for p in ps {
                    if let Piece::Value(v, _) = p {
                        self.expr(v, fx)
                    }
                }
            }
        }
    }
    fn stmt(&mut self, s: &TStmt, fx: &mut Fx) {
        match s {
            TStmt::Expr(e) | TStmt::Assert(e, ..) => self.expr(e, fx),
            TStmt::Return(Some(e)) => {
                self.expr(e, fx);
                self.escapes(e.span, e);
            }
            TStmt::Let(v, e) => {
                self.expr(e, fx);
                self.take(e);
                self.bind(*v);
                let what = self.sources(e);
                self.holds(self.root(*v), e, &what);
                let mut inner = e;
                while let TK::Inject(x) | TK::Widen(x) = &inner.kind {
                    inner = x;
                }
                if let TK::Lambda { body, .. } = &inner.kind {
                    let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
                    free_vars(body, &mut used, &mut bound);
                    let roots: BTreeSet<Root> =
                        used.difference(&bound).map(|u| self.root(*u)).collect();
                    // What a captured view lends stays lent while the closure lives (R05.7.1);
                    // a captured variable itself is shared, not frozen (R05.10.1).
                    let through: Vec<Pl> = (what.iter())
                        .filter(|p| !roots.contains(&p.root))
                        .cloned()
                        .collect();
                    if !through.is_empty() {
                        self.frozen
                            .entry(self.root(*v))
                            .or_default()
                            .extend(through);
                    }
                    self.caps.insert(self.root(*v), roots);
                }
            }
            TStmt::Assign(target, e) => {
                self.expr(e, fx);
                match self.place(target) {
                    Some(p) => {
                        self.take(e);
                        // Assigning a place refills it and every path under it.
                        (self.moved).retain(|m| !(overlap(m, &p) && m.path.len() >= p.path.len()));
                        let what = self.sources(e);
                        self.wrote(target.span, &p);
                        self.store(target.span, p.root, what);
                        fx.writes.insert(p);
                    }
                    None => self.expr(target, fx),
                }
            }
            TStmt::While(c, b) => {
                self.expr(c, fx);
                let at = self.next;
                let before = self.moved.clone();
                self.expr(b, fx);
                self.moved_in_loop(b.span, at, &before);
            }
            TStmt::For(v, lo, hi, b) => {
                self.bind(*v);
                self.expr(lo, fx);
                self.expr(hi, fx);
                let at = self.next;
                let before = self.moved.clone();
                self.expr(b, fx);
                self.moved_in_loop(b.span, at, &before);
            }
            TStmt::Return(None) | TStmt::Break | TStmt::Continue => {}
            TStmt::Item(off) => self.off = *off,
            // A loop: what one case moves out of the code above it is gone for the next.
            TStmt::Cases(count, body) => {
                if let Some(c) = count {
                    self.expr(c, fx)
                }
                let at = self.next;
                let before = self.moved.clone();
                self.next += 1;
                self.scopes.push(self.next);
                body.iter().for_each(|s| self.stmt(s, fx));
                self.scopes.pop();
                self.moved_in_loop(Span::new(0, 0), at, &before);
            }
            TStmt::Yield(e) | TStmt::LendOut(e) => {
                if let (TStmt::LendOut(_), Some(p)) = (s, self.place(e)) {
                    fx.writes.insert(p);
                } else {
                    self.expr(e, fx);
                }
                // R05.10.2: a yielded fn value over the generator's own locals lives only as
                // long as the walk.
                if matches!(s, TStmt::Yield(_)) {
                    let own =
                        |p: &Pl| matches!(p.root, Root::Local(_)) && !self.params.contains(&p.root);
                    fx.yields_own |= self.sources(e).iter().any(own);
                }
                for (h, m) in &self.regions {
                    fx.held.extend(h.iter().chain(m.iter()).cloned());
                }
            }
            // R12: the callee's lends are live for the walk, and the body runs inside the call.
            TStmt::Region {
                callee,
                recv,
                args,
                var,
                body,
                ..
            } => {
                // R4 for a cell: a value written into the cell being lent that borrows the container is
                // copied in, so the write does not run while that value still views it.
                let mut parked = Vec::new();
                if let (Some(rv), TK::Block(stmts, _)) = (recv.as_deref(), &body.kind) {
                    let (TArg::Value(rv) | TArg::Mut(rv)) = rv;
                    if let Some(root) = self.place(rv).map(|p| p.root) {
                        for st in stmts {
                            let TStmt::Assign(_, v) = st else { continue };
                            let TK::Var(x) = &v.kind else { continue };
                            let r = self.root(*x);
                            if self
                                .lent
                                .get(&r)
                                .is_some_and(|s| s.iter().any(|p| p.root == root))
                            {
                                parked.push((r, self.lent.remove(&r), self.frozen.remove(&r)));
                            }
                        }
                    }
                }
                // A cell lent at a literal key is that element of the receiver, apart from its
                // other literal keys (R05.8.2).
                let keyed = match (recv.as_deref(), &args[..]) {
                    (
                        Some(TArg::Mut(rv)),
                        [TArg::Value(
                            k @ TExpr {
                                kind: TK::Int(_), ..
                            },
                        )],
                    ) => Some(TArg::Mut(TExpr {
                        kind: TK::Slot(Box::new(rv.clone()), Box::new(k.clone())),
                        ty: rv.ty.clone(),
                        span: rv.span,
                    })),
                    _ => None,
                };
                let recv = keyed.as_ref().or(recv.as_deref());
                let (mut held, lent) = match (callee, recv) {
                    (Callee::Fn(f), _) => self.fn_call(f, body.span, recv, args, fx),
                    (Callee::Trait(tr, m), Some(r)) => {
                        self.trait_call(tr.tr, m, body.span, Some(r), args, fx)
                    }
                    (Callee::Trait(..), None) => Default::default(),
                    (Callee::Value(c), _) => {
                        self.expr(c, fx);
                        (self.sources(c), BTreeSet::new())
                    }
                };
                let who = match callee {
                    Callee::Fn(f) => self.d.f(f.decl).name.clone(),
                    Callee::Trait(_, m) => m.clone(),
                    Callee::Value(_) => "the generator".to_string(),
                };
                // What the walked value lends is held as long as it is (R05.5.3).
                for a in recv.into_iter().chain(args.iter()) {
                    let TArg::Value(v) = a else { continue };
                    match self.place(v) {
                        Some(p) => {
                            held.extend(self.lent.get(&p.root).into_iter().flatten().cloned())
                        }
                        None => held.extend(self.sources(v)),
                    }
                }
                self.bind(*var);
                let item = self.root(*var);
                let item_ty = match var {
                    VarRef::Local(l) => self
                        .cur
                        .and_then(|c| self.t.fns.get(&c))
                        .and_then(|f| f.locals.get(*l))
                        .map(|l| l.ty.clone()),
                    VarRef::Global(g) => self.t.globals.get(*g).map(|l| l.ty.clone()),
                };
                let frame = match callee {
                    Callee::Fn(f) => self.fns.get(&f.decl).is_some_and(|x| x.yields_own),
                    _ => false,
                };
                // An item of a type parameter is not a borrow: each instance copies it if it must.
                if frame
                    || item_ty.is_some_and(|t| !matches!(t, Ty::Param(_)) && !self.plain(&t, 0))
                {
                    self.borrowed.insert(item);
                }
                self.regions.push((held.clone(), lent.clone()));
                let mut inner = Fx::default();
                self.expr(body, &mut inner);
                self.regions.pop();
                let own = self.root(*var);
                inner.reads.retain(|p| p.root != own);
                inner.writes.retain(|p| p.root != own);
                if let Some(p) = hits(&held, &inner.writes) {
                    let n = self.name(p);
                    self.err(
                        body.span,
                        codes::USED_WHILE_LENT,
                        format!(
                            "{n} is lent to `{who}` for the whole loop, and the loop body writes it"
                        ),
                    );
                }
                if let Some(p) = hits(&lent, &inner.writes).or_else(|| hits(&lent, &inner.reads)) {
                    let n = self.name(p);
                    self.err(body.span, codes::USED_WHILE_LENT, format!("{n} is lent to `{who}` for writing for the whole loop, and the loop body uses it"));
                }
                for (r, l, f) in parked {
                    if let Some(l) = l {
                        self.lent.insert(r, l);
                    }
                    if let Some(f) = f {
                        self.frozen.insert(r, f);
                    }
                }
                fx.add(&inner);
            }
        }
    }
}

/// Where the value of a body is written: the last expression of a block, not the block.
fn result_span(e: &TExpr) -> Span {
    match &e.kind {
        TK::Block(_, Some(t)) => result_span(t),
        _ => e.span,
    }
}

/// Settles each place passed to a call (`EK::Pass`) in the linked program: a copy taken in its
/// turn where ownership found the call also writes it (R4), else a view from the call's start
/// (R11) or a read. `sites` is where each site is in the loaded source.
pub fn place_copies(p: &mut Program, sites: &[Span], copies: &BTreeSet<Span>) {
    for f in &mut p.insts {
        Settle {
            locals: &mut f.locals,
            copy: |s: Site| copies.contains(&sites[s as usize]),
            saved: HashMap::new(),
        }
        .block(&mut f.body);
    }
}
struct Settle<'a, C> {
    locals: &'a mut Vec<CTy>,
    copy: C,
    /// A passed place saved before a later operand ran, by site: the temporary its copy is in.
    saved: HashMap<Site, usize>,
}
impl<C: Fn(Site) -> bool> Settle<'_, C> {
    fn block(&mut self, b: &mut Vec<Stmt<CTy>>) {
        for mut st in std::mem::take(b) {
            if let Stmt::Let(t, e) = &mut st {
                if let EK::Pass(p, view, s) = &e.kind {
                    if *view && !(self.copy)(*s) {
                        continue;
                    }
                    if *view {
                        self.saved.insert(*s, *t);
                    }
                    e.kind = EK::Copy(p.clone());
                    b.push(st);
                    continue;
                }
            }
            let mut pre = Vec::new();
            self.stmt(&mut st, &mut pre);
            b.extend(pre);
            b.push(st);
        }
    }
    fn stmt(&mut self, st: &mut Stmt<CTy>, pre: &mut Vec<Stmt<CTy>>) {
        match st {
            Stmt::Let(_, e)
            | Stmt::Assign(_, e)
            | Stmt::Expr(e)
            | Stmt::Return(Some(e))
            | Stmt::Yield(e)
            | Stmt::Fact(e)
            | Stmt::Check { cond: e, .. } => self.expr(e, pre),
            Stmt::If(c, a, b) => {
                self.expr(c, pre);
                self.block(a);
                self.block(b);
            }
            Stmt::Loop(a, b) => {
                self.block(a);
                self.block(b);
            }
            Stmt::Switch(e, bs) => {
                self.expr(e, pre);
                bs.iter_mut().for_each(|b| self.block(b));
            }
            Stmt::Print(ps) => {
                for x in ps {
                    if let PrintPiece::Value(e, _) = x {
                        self.expr(e, pre);
                    }
                }
            }
            Stmt::Event(_, xs) => xs.iter_mut().for_each(|x| self.expr(x, pre)),
            Stmt::Region { args, body, .. } => {
                self.args(args, pre);
                self.block(body);
            }
            Stmt::Return(None) | Stmt::LendOut(_) | Stmt::Break | Stmt::Continue => {}
        }
    }
    fn args(&mut self, xs: &mut [Arg<CTy>], pre: &mut Vec<Stmt<CTy>>) {
        for a in xs {
            let Arg::Value(v) = a else { continue };
            match &v.kind {
                EK::Pass(p, true, s) if !(self.copy)(*s) => *a = Arg::View(p.clone()),
                EK::Pass(_, true, s) if self.saved.contains_key(s) => {
                    v.kind = EK::Copy(Place::Local(self.saved[s]));
                }
                _ => self.expr(v, pre),
            }
        }
    }
    fn expr(&mut self, e: &mut Expr<CTy>, pre: &mut Vec<Stmt<CTy>>) {
        if let EK::Pass(p, _, s) = &e.kind {
            let p = p.clone();
            e.kind = if (self.copy)(*s) {
                let t = self.locals.len();
                self.locals.push(e.ty.clone());
                let ty = e.ty.clone();
                pre.push(Stmt::Let(
                    t,
                    Expr {
                        kind: EK::Copy(p),
                        ty,
                    },
                ));
                EK::Copy(Place::Local(t))
            } else {
                EK::Copy(p)
            };
            return;
        }
        match &mut e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                xs.iter_mut().for_each(|x| self.expr(x, pre))
            }
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x, pre),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => self.args(xs, pre),
            EK::CallValue(c, xs, _) => {
                self.expr(c, pre);
                xs.iter_mut().for_each(|x| self.expr(x, pre));
            }
            _ => {}
        }
    }
}
