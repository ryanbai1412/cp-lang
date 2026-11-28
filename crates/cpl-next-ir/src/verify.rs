//! The stage contracts, executable (contracts 4 to 6): what a fn's IR promises at each stage,
//! checked. Each check gives what the fn breaks, so a stage that breaks its contract fails
//! where it ran, not later in a backend. The pipeline runs them after every stage in a debug
//! build, which the programs test runs.
use crate::facts::Proved;
use crate::ir::*;
use crate::msg::Msg;
use crate::prog::CTy;
use crate::ty::{params_in, ConstArg, End, SetTy, Ty};

/// What a call site may assume of the instance it calls: whether it is an operator
/// (`Func::fast`), and which of its params it views (`Func::views`).
#[derive(Clone, Debug, Default)]
pub struct Callee {
    pub fast: bool,
    pub views: Vec<bool>,
    /// What a message calls it (`Func::shown`).
    pub shown: String,
}

/// Each instance of `fs` as a callee.
pub fn callees<T>(fs: &[Func<T>]) -> Vec<Callee> {
    (fs.iter())
        .map(|f| Callee {
            fast: f.fast.is_some(),
            views: f.views.clone(),
            shown: f.shown.clone(),
        })
        .collect()
}

/// The stage a fn's IR is in, in the order they run.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    /// As lowered (contract 4): its types solved, no lend, copy or drop chosen yet, and a call
    /// the whole of a statement's value, so expressions are pure; until the optional passes.
    Generic,
    /// As link made it (contract 5): every call names an instance.
    Instances,
    /// After ownership (contract 5's raw IR): every passed place settled.
    Raw,
    /// Canonical (contract 6): a fn value is called through its set's instances.
    Canonical,
    /// After the optional passes (contract 6).
    Optimized,
}

/// What a type in a fn's IR may not be at a stage.
pub trait StageTy {
    fn wrong(&self, stage: Stage) -> Option<&'static str>;

    /// An error type, which a program that checked clean should not hold but does today
    /// (`error_typed`).
    fn error(&self) -> bool {
        false
    }

    /// The type as `error_typed` names it.
    fn shown(&self) -> String {
        String::new()
    }
}

impl StageTy for Ty {
    fn wrong(&self, _: Stage) -> Option<&'static str> {
        let set = |s: &SetTy| matches!(s, SetTy::Var(_)).then_some("an unsolved set variable");
        match self {
            Ty::Var(_) => Some("an unsolved type variable"),
            Ty::Const(ConstArg::At(_)) => Some("a constant argument check did not evaluate"),
            Ty::Lent(_) | Ty::Ref(_) => Some("a lend or reference chosen before ownership"),
            Ty::Raw(t) | Ty::Mut(t) | Ty::RangeTo(t, ..) => t.wrong(Stage::Generic),
            Ty::RangeEnds { lo, hi, .. } => {
                let ends = [lo, hi].map(|e| match e {
                    End::Of(t) => Some(&**t),
                    End::Lit(_) | End::Global { .. } => None,
                });
                let global = [lo, hi].iter().any(|e| matches!(e, End::Global { .. }));
                let mut ps = std::collections::BTreeSet::new();
                ends.iter().flatten().for_each(|t| params_in(t, &mut ps));
                if ps.is_empty() && !global {
                    return Some("a range whose ends are known values, not made `Range`");
                }
                ends.iter().flatten().find_map(|t| t.wrong(Stage::Generic))
            }
            Ty::Struct(_, ts, ss) => (ts.iter().find_map(|t| t.wrong(Stage::Generic)))
                .or_else(|| ss.iter().find_map(set)),
            Ty::Fn(ps, r, s) => (ps.iter().chain([&**r]))
                .find_map(|t| t.wrong(Stage::Generic))
                .or_else(|| set(s)),
            _ => None,
        }
    }
}

/// A type parameter or set variable has no `CTy`: an instance's types are concrete by theirs.
impl StageTy for CTy {
    fn wrong(&self, _: Stage) -> Option<&'static str> {
        None
    }

    fn shown(&self) -> String {
        format!("{self:?}")
    }

    fn error(&self) -> bool {
        match self {
            CTy::Error => true,
            CTy::Raw(t) | CTy::Ref(t) | CTy::Lent(t) => t.error(),
            CTy::Struct(_, ts, _) => ts.iter().any(CTy::error),
            CTy::Fn(ps, r, _) => ps.iter().chain([&**r]).any(CTy::error),
            _ => false,
        }
    }
}

/// What `f` breaks of `stage`'s contract, where `callees` are the program's instances as a
/// call sees them, and the program has `globals` globals:
/// its locals, globals, instances and sites exist, a local is set before it is read (a drop
/// of a local releases what it holds, if anything), `break` is in a loop or a region's body,
/// and each stage's own promises above.
pub fn check<T: StageTy + PartialEq + std::fmt::Debug>(
    f: &Func<T>,
    stage: Stage,
    callees: &[Callee],
    globals: usize,
) -> Vec<String> {
    walk(f, stage, callees, globals).out
}

/// `all` on a program's instances, which also checks that each argument a call gives has the
/// type of the param it fills (a lend aside), so an instance picked for the wrong marks
/// (`Add<S>` given a `Rev<lent List>`) fails here rather than in a backend.
pub fn instances(fs: &[Func<CTy>], stage: Stage, globals: usize) -> Vec<Msg> {
    let callees = callees(fs);
    (fs.iter())
        .flat_map(|f| walk_in(f, stage, &callees, globals, fs).out)
        .map(|e| Msg::internal(format!("{stage:?} IR: {e}")))
        .collect()
}

/// The instances holding an error type, by name. The checker should have refused or resolved
/// each; until it does, the programs test keeps them in a list that only shrinks
/// (`tests/error-typed.txt`), rather than the verifier refusing a program that runs.
pub fn error_typed(fs: &[Func<CTy>], callees: &[Callee], globals: usize) -> Vec<String> {
    (fs.iter())
        .filter_map(|f| {
            let of = walk(f, Stage::Raw, callees, globals).error_typed?;
            Some(format!("{}: {of}", f.shown))
        })
        .collect()
}

/// `check` on each fn, as internal messages.
pub fn all<'a, T: StageTy + PartialEq + std::fmt::Debug + 'a>(
    fs: impl IntoIterator<Item = &'a Func<T>>,
    stage: Stage,
    callees: &[Callee],
    globals: usize,
) -> Vec<Msg> {
    (fs.into_iter())
        .flat_map(|f| check(f, stage, callees, globals))
        .map(|e| Msg::internal(format!("{stage:?} IR: {e}")))
        .collect()
}

/// What check left unworked in the structs' field types (`Whole::fields`), as `all` says it
/// of a generic fn's types: one message per fault.
pub fn fields(fields: &[Vec<crate::ty::Ty>]) -> Vec<Msg> {
    let faults: std::collections::BTreeSet<_> = (fields.iter().flatten())
        .filter_map(|t| t.wrong(Stage::Generic))
        .collect();
    (faults.into_iter())
        .map(|e| Msg::internal(format!("Generic IR: a field's type: {e}")))
        .collect()
}

/// What instance `func` breaks of the optimized contract against its canonical IR: each check
/// the optional passes left out is named by a `Proved` (contract 6, proof-accounted).
pub fn accounted(
    func: usize,
    canon: &Func<CTy>,
    opt: &Func<CTy>,
    proved: &[Proved],
) -> Vec<String> {
    let mut left = walk(opt, Stage::Optimized, &[], usize::MAX).checks;
    let mut out = Vec::new();
    for c in walk(canon, Stage::Canonical, &[], usize::MAX).checks {
        match left.iter().position(|x| *x == c) {
            Some(i) => {
                left.swap_remove(i);
            }
            None if (proved.iter()).any(|p| (p.func, p.kind, p.site) == (func, c.0, c.1)) => {}
            None => out.push(format!(
                "{}: its {:?} check at site {} left out with no fact proving it",
                opt.shown, c.0, c.1
            )),
        }
    }
    out
}

fn walk<'a, T: StageTy + PartialEq + std::fmt::Debug>(
    f: &'a Func<T>,
    stage: Stage,
    callees: &'a [Callee],
    globals: usize,
) -> V<'a, T> {
    walk_in(f, stage, callees, globals, &[])
}

/// `walk`, with the instances `insts` its calls name when their params are to be checked.
fn walk_in<'a, T: StageTy + PartialEq + std::fmt::Debug>(
    f: &'a Func<T>,
    stage: Stage,
    callees: &'a [Callee],
    globals: usize,
    insts: &'a [Func<T>],
) -> V<'a, T> {
    let mut v = V {
        f,
        stage,
        callees,
        globals,
        insts,
        set: vec![false; f.locals.len()],
        loops: 0,
        out: Vec::new(),
        checks: Vec::new(),
        error_typed: None,
    };
    v.head();
    v.block(&f.body);
    v
}

struct V<'a, T> {
    f: &'a Func<T>,
    stage: Stage,
    callees: &'a [Callee],
    globals: usize,
    insts: &'a [Func<T>],
    set: Vec<bool>,
    loops: usize,
    out: Vec<String>,
    checks: Vec<(CheckKind, Site)>,
    error_typed: Option<String>,
}

impl<T: StageTy + PartialEq + std::fmt::Debug> V<'_, T> {
    fn bad(&mut self, what: String) {
        self.out.push(format!("{}: {what}", self.f.shown));
    }

    fn ty(&mut self, t: &T) {
        self.ty_of(t, "an expression");
    }

    fn ty_of(&mut self, t: &T, of: &str) {
        if let Some(w) = t.wrong(self.stage) {
            self.bad(format!("{w}, as {of}'s type"));
        }
        if t.error() && self.error_typed.is_none() {
            self.error_typed = Some(format!("{of} {}", t.shown()));
        }
    }

    fn local(&mut self, l: usize) -> bool {
        let ok = l < self.f.locals.len();
        if !ok {
            self.bad(format!("local {l} of {}", self.f.locals.len()));
        }
        ok
    }

    fn site(&mut self, s: Site) {
        if s as usize >= self.f.sites.len() {
            self.bad(format!("site {s} of {}", self.f.sites.len()));
        }
    }

    fn head(&mut self) {
        let f = self.f;
        for &(p, _) in &f.params {
            if self.local(p) {
                self.set[p] = true;
            }
        }
        if f.n_captures > f.params.len() {
            self.bad(format!(
                "{} captures of {} params",
                f.n_captures,
                f.params.len()
            ));
        }
        for &l in &f.drops {
            self.local(l);
            let param = f.params.iter().position(|&(p, _)| p == l);
            if param.is_some_and(|k| f.params[k].1 || f.views.get(k) == Some(&true)) {
                self.bad(format!(
                    "local {l}, a lent or viewed param, released by its frame"
                ));
            }
        }
        if self.stage < Stage::Raw && (!f.drops.is_empty() || f.views.contains(&true)) {
            self.bad("a drop or view chosen before ownership".into());
        }
        for (i, t) in f.locals.iter().enumerate() {
            self.ty_of(t, &format!("local {i}"));
        }
        self.ty_of(&f.ret, "the result");
        if let Some((_, t)) = &f.region {
            self.ty_of(t, "the region");
        }
    }

    fn block(&mut self, b: &[Stmt<T>]) {
        b.iter().for_each(|s| self.stmt(s));
    }

    fn stmt(&mut self, s: &Stmt<T>) {
        match s {
            Stmt::Let(l, e) => {
                self.expr(e, "");
                if self.local(*l) {
                    self.set[*l] = true;
                }
            }
            Stmt::Assign(p, e) => {
                self.expr(e, "");
                match p {
                    Place::Local(l) if self.local(*l) => self.set[*l] = true,
                    Place::Local(_) => {}
                    p => self.read(p),
                }
            }
            Stmt::Expr(e) => self.expr(e, ""),
            Stmt::Yield(e) | Stmt::Fact(e) => self.expr(e, "a yield or fact"),
            Stmt::Return(e) => {
                if let Some(e) = e {
                    self.expr(e, "a return");
                    if !self.insts.is_empty() && e.ty != self.f.ret {
                        self.bad(format!("a return of {:?}, not the result", e.ty));
                    }
                }
            }
            Stmt::If(c, a, b) => {
                self.expr(c, "an if's condition");
                self.block(a);
                self.block(b);
            }
            Stmt::Loop(a, b, s) => {
                self.site(*s);
                self.loops += 1;
                self.block(a);
                self.block(b);
                self.loops -= 1;
            }
            Stmt::Switch(c, arms) => {
                self.expr(c, "a switch's value");
                arms.iter().for_each(|a| self.block(a));
                // A switch of no arm is a call of an empty set: nothing after it runs.
                if arms.is_empty() {
                    self.set.iter_mut().for_each(|s| *s = true);
                }
            }
            Stmt::Break | Stmt::Continue if self.loops == 0 => {
                self.bad("a break or continue outside a loop".into())
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Check {
                kind, cond, site, ..
            } => {
                self.site(*site);
                self.checks.push((*kind, *site));
                self.expr(cond, "a check");
                // A check that always fails, as an untyped part's trap: nothing after it runs.
                if matches!(cond.kind, EK::Bool(false)) {
                    self.set.iter_mut().for_each(|s| *s = true);
                }
            }
            Stmt::Region {
                target,
                args,
                var,
                body,
                site,
                body_site,
                ..
            } => {
                self.site(*site);
                self.site(*body_site);
                self.target(target);
                self.viewed(target, args);
                self.args(args, "a region's argument");
                if self.local(*var) {
                    self.set[*var] = true;
                }
                self.loops += 1;
                self.block(body);
                self.loops -= 1;
            }
            Stmt::LendOut(p) => self.read(p),
            Stmt::Drop(p) => {
                if self.stage < Stage::Raw {
                    self.bad("a drop before ownership placed it".into());
                }
                match p {
                    Place::Local(l) => drop(self.local(*l)),
                    p => self.read(p),
                }
            }
            Stmt::Event(crate::ir::EventPoint { vals: es, .. }) => {
                // Each value is a call run as a statement of its own, its writes kept.
                es.iter().for_each(|e| self.expr(e, ""))
            }
        }
    }

    /// `inside`: what holds `e`, empty where it is the whole of a statement's value, the one
    /// place a call may be until the optional passes; an operator's call may be anywhere.
    fn expr(&mut self, e: &Expr<T>, inside: &str) {
        self.site(e.site);
        self.ty(&e.ty);
        let pure = self.stage < Stage::Optimized;
        match &e.kind {
            EK::Copy(p) | EK::AddrOf(p) | EK::Tag(p) => self.read(p),
            EK::Lent(p) => {
                if self.stage == Stage::Generic {
                    self.bad("a lend before ownership chose it".into());
                }
                self.read(p);
            }
            EK::Pass(p, _, s) => {
                if self.stage >= Stage::Raw {
                    self.bad("a passed place ownership left unsettled".into());
                }
                self.site(*s);
                self.read(p);
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                xs.iter().for_each(|x| self.expr(x, "a struct or closure"))
            }
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x, "a conversion"),
            EK::Intr(op, xs, s) => {
                self.site(*s);
                self.args(xs, &format!("{op:?}'s argument"));
            }
            EK::Call(t, xs, s) => {
                let op =
                    matches!(t, Target::Inst(i) if self.callees.get(*i).is_some_and(|c| c.fast));
                if pure && !inside.is_empty() && !op {
                    self.bad(format!("a call inside {inside}"));
                }
                self.site(*s);
                self.target(t);
                self.viewed(t, xs);
                self.args(xs, "a call's argument");
                if let Target::Inst(i) = t {
                    self.params(*i, xs);
                    self.result(*i, &e.ty);
                }
            }
            EK::CallValue(g, xs, s) => {
                if self.stage >= Stage::Canonical {
                    self.bad("a fn value called, not its set's instances".into());
                }
                self.site(*s);
                self.expr(g, "a called value");
                self.args(xs, "a call's argument");
            }
            EK::DomLen(t) | EK::DomLo(t) | EK::DomHeld(t) | EK::DomCounted(t) => self.ty(t),
            EK::TypeValue(t) => {
                if self.stage >= Stage::Instances {
                    self.bad("a value parameter's read left for an instance".into());
                }
                self.ty(t);
            }
            EK::HasBound(t, _, ts) => [t].into_iter().chain(ts).for_each(|t| self.ty(t)),
            EK::Int(_) | EK::F64(_) | EK::Bool(_) | EK::Unit | EK::Text(_) => {}
        }
    }

    fn args(&mut self, xs: &[Arg<T>], inside: &str) {
        for a in xs {
            match a {
                Arg::Value(e) => self.expr(e, inside),
                Arg::View(p) | Arg::Lend(p) => self.read(p),
            }
        }
    }

    /// After ownership, a view is given only to a param its callee views; one the callee
    /// owns takes a copy.
    fn viewed(&mut self, t: &Target<T>, xs: &[Arg<T>]) {
        let Target::Inst(i) = t else { return };
        let Some(c) = self.callees.get(*i).filter(|_| self.stage >= Stage::Raw) else {
            return;
        };
        for (j, x) in xs.iter().enumerate() {
            if matches!(x, Arg::View(_)) && c.views.get(j) != Some(&true) {
                self.bad(format!(
                    "a view given to param {j} of {}, which owns it",
                    c.shown
                ));
            }
        }
    }

    /// Each argument of a call of instance `i` that is a value or a viewed local has the type
    /// of the param it fills.
    fn params(&mut self, i: usize, xs: &[Arg<T>]) {
        let Some(g) = self.insts.get(i) else {
            return;
        };
        for (k, (a, &(p, _))) in xs.iter().zip(&g.params).enumerate() {
            let given = match a {
                Arg::Value(e) => &e.ty,
                Arg::View(Place::Local(l)) if *l < self.f.locals.len() => &self.f.locals[*l],
                _ => continue,
            };
            if g.locals.get(p).is_some_and(|t| t != given) {
                self.bad(format!("{}'s param {k} given a {given:?}", g.shown));
            }
        }
    }

    /// A call of instance `i` has the type its result is made with, lent marks included: a
    /// caller that took a lent view for its own storage would release what it never held.
    fn result(&mut self, i: usize, got: &T) {
        if let Some(g) = self.insts.get(i).filter(|g| g.ret != *got) {
            self.bad(format!(
                "a call of {} typed {got:?}, not its result",
                g.shown
            ));
        }
    }

    fn target(&mut self, t: &Target<T>) {
        let linked = self.stage >= Stage::Instances;
        match t {
            Target::Fn(_) | Target::Case | Target::Trait { .. } if linked => {
                self.bad("a call that names no instance".into())
            }
            Target::Inst(_) if !linked => self.bad("an instance called before link".into()),
            Target::Inst(i) if *i >= self.callees.len() => {
                self.bad(format!("instance {i} of {}", self.callees.len()))
            }
            Target::Value(e) => {
                if self.stage >= Stage::Canonical {
                    self.bad("a fn value called, not its set's instances".into());
                }
                self.expr(e, "a called value");
            }
            Target::Trait {
                args,
                self_ty,
                targs,
                ret,
                ..
            } => (args.iter().chain(targs).chain([self_ty, ret])).for_each(|t| self.ty(t)),
            _ => {}
        }
    }

    fn read(&mut self, p: &Place) {
        match p {
            Place::Local(l) => {
                if self.local(*l) && !self.set[*l] {
                    self.bad(format!("local {l} read before it is set"));
                }
            }
            Place::Global(g) => {
                if *g >= self.globals {
                    self.bad(format!("global {g} of {}", self.globals));
                }
            }
            Place::Slot(b, i) => {
                self.read(b);
                self.read(&Place::Local(*i));
            }
            Place::Field(b, _) | Place::Variant(b, _) | Place::Deref(b) | Place::Through(b) => {
                self.read(b)
            }
        }
    }
}

#[cfg(test)]
mod tests;
