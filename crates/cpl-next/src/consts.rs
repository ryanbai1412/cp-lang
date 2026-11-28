//! The `const_value` query (`handbook/design/next/incremental.md`): a constant expression's
//! value, worked out by running it on the engine, the one evaluator. The checker asks through
//! `check::Consts`, so it holds no evaluator of its own.
use crate::{check, diag::Msg, inst, lower, own};
use check::ConstError;
use cpl_diag::{codes, Code};
use cpl_next_decls::views::Answers;
use cpl_next_decls::{decls::Decls, modules::Loaded};
use cpl_next_engine::Stop;
use cpl_next_ir::build::{Build, Checks};
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::ops::{Fault, Scalar};
use cpl_next_ir::prog::CTy;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ast::{self as next, AstNode};
use cpl_next_syntax::TokenKind;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

/// The steps a constant may take (each fn entered and each jump taken back is one), the same
/// on every machine (R01.6.8).
pub(crate) const BUDGET: u64 = 10_000_000;

type Key = (ItemId, cpl_diag::Span);
type Value = Result<Option<Scalar>, ConstError>;

/// Works out each constant expression of the program once.
pub(crate) struct Evaluator<'a> {
    d: &'a Decls,
    loaded: &'a Loaded,
    main: Vec<(ItemId, next::Item)>,
    done: RefCell<HashMap<Key, Value>>,
    busy: RefCell<HashSet<Key>>,
    /// The expressions read while they were being worked out.
    cyclic: RefCell<HashSet<Key>>,
    /// The values one run worked out together, once asked.
    batch: RefCell<Option<HashMap<Key, Value>>>,
    /// Whether that run is being checked: a constant it reads is a `let` of it.
    batching: Cell<bool>,
}

impl<'a> Evaluator<'a> {
    pub(crate) fn new(d: &'a Decls, loaded: &'a Loaded) -> Self {
        Evaluator {
            d,
            loaded,
            main: (d.statements.iter())
                .filter_map(|id| {
                    Some((id.clone(), cpl_next_decls::views::Bodies::item(loaded, id)?))
                })
                .collect(),
            done: RefCell::default(),
            busy: RefCell::default(),
            cyclic: RefCell::default(),
            batch: RefCell::default(),
            batching: Cell::default(),
        }
    }

    /// The values of the program's constants and top-level domain ends that read only each
    /// other, worked out by one run: each is a `let` of a program of its own. Empty when that
    /// run does not finish; each is then worked out alone.
    fn batch(&self) -> HashMap<Key, Value> {
        let mut bound = HashSet::new();
        let mut consts = HashSet::new();
        for (_, s) in &self.main {
            match (s.stmt(), s.decl()) {
                (Some(next::Stmt::Let(l)), _) => {
                    let mut pats = l.patterns();
                    if let (Some(p), None) = (pats.next(), pats.next()) {
                        bound.extend(check::plain_name(&p));
                    }
                }
                (_, Some(next::Decl::Domain(dom))) => {
                    bound.extend(dom.name().map(|n| n.text().to_string()));
                }
                (_, Some(next::Decl::Const(c))) => {
                    consts.extend(c.name().map(|n| n.text().to_string()));
                }
                _ => {}
            }
        }
        let mut admitted: HashSet<String> = HashSet::new();
        let mut keys = Vec::new();
        for (k, s) in &self.main {
            let closed = |e: &next::Expr, admitted: &HashSet<String>| {
                (names(e).iter())
                    .all(|n| admitted.contains(n) || !(bound.contains(n) || consts.contains(n)))
            };
            match s.decl() {
                Some(next::Decl::Const(c)) => {
                    let (Some(name), Some(init)) = (c.name(), c.value()) else {
                        continue;
                    };
                    if closed(&init, &admitted) {
                        keys.push((k.clone(), init.syntax().span()));
                        admitted.insert(name.text().to_string());
                    }
                }
                Some(next::Decl::Domain(dom)) => {
                    let ends: Vec<next::Expr> = match dom.size() {
                        Some(next::Expr::Range(r)) if r.step().is_none() => {
                            match (r.lo(), r.hi()) {
                                (Some(lo), Some(hi)) => vec![lo, hi],
                                _ => vec![],
                            }
                        }
                        Some(next::Expr::List(l)) => {
                            let xs: Vec<next::Expr> = l.elems().collect();
                            if xs.len() == 1 {
                                xs
                            } else {
                                vec![]
                            }
                        }
                        _ => vec![],
                    };
                    for e in ends.into_iter().filter(|e| closed(e, &admitted)) {
                        keys.push((k.clone(), e.syntax().span()));
                    }
                }
                _ => {}
            }
        }
        if keys.is_empty() {
            return HashMap::new();
        }
        let values: Vec<SourceId> = keys.iter().map(|(k, span)| k.at(*span)).collect();
        self.batching.set(true);
        let ran = self.execute(&[], &values);
        self.batching.set(false);
        match ran {
            Ran::Values(vs) => keys.into_iter().zip(vs.into_iter().map(Ok)).collect(),
            Ran::Open | Ran::Stop(_) => HashMap::new(),
        }
    }

    /// The expression at `at`, in any module's item.
    fn expr_at(&self, at: &SourceId) -> Option<next::Expr> {
        let item = (self.loaded.modules.iter())
            .flat_map(|m| m.items.iter())
            .find(|i| i.id == at.item)?;
        (item.item.syntax().descendants())
            .filter_map(next::Expr::cast)
            .find(|e| e.syntax().span() == at.span)
    }

    /// `e` checked, lowered and linked as the value of a program of its own, after the
    /// program's constants it names that have values, then run on the engine.
    fn run(&self, item: &ItemId, e: &next::Expr) -> Value {
        let named = names(e);
        let mut items = Vec::new();
        for (k, s) in &self.main {
            let Some(next::Decl::Const(c)) = s.decl() else {
                continue;
            };
            if let (Some(name), Some(init)) = (c.name(), c.value()) {
                let read = (k.clone(), init.syntax().span());
                if named.contains(name.text()) && self.busy.borrow().contains(&read) {
                    self.cyclic.borrow_mut().insert(read);
                    continue;
                }
                if named.contains(name.text())
                    && matches!(
                        check::Consts::value(self, k.at(init.syntax().span())),
                        Ok(Some(_))
                    )
                {
                    items.push(k.clone());
                }
            }
        }
        let span = e.syntax().span();
        let value = [item.at(span)];
        let failed = |code, text: String| {
            Err(ConstError::Failed(Box::new(Msg::at(
                item.at(span),
                Msg::verdict(code, text),
            ))))
        };
        match self.execute(&items, &value) {
            Ran::Values(mut vs) => Ok(vs.pop().flatten()),
            // What the engine cannot run yet runs at start-up instead.
            Ran::Open | Ran::Stop(Stop::Input) => Err(ConstError::Open),
            Ran::Stop(Stop::Trap(t)) if matches!(t.fault, Fault::Internal(_)) => {
                Err(ConstError::Open)
            }
            Ran::Stop(Stop::Trap(t)) => {
                let what = if t.msg.is_empty() {
                    t.fault.message().to_string()
                } else {
                    t.msg
                };
                failed(
                    Code(t.fault.code()),
                    format!("working out this constant traps: {what}"),
                )
            }
            Ran::Stop(Stop::Budget) => failed(
                codes::CONST_BUDGET,
                format!(
                    "working out this constant took more than {BUDGET} steps; compute it at run time with `let`"
                ),
            ),
        }
    }

    /// Runs the program of items `main` then the expressions at `values`, each bound by a
    /// top-level `let`, on the engine: the values of the globals those `let`s bind.
    fn execute(&self, main: &[ItemId], values: &[SourceId]) -> Ran {
        let lets = values.len();
        let mut c = crate::checker(
            self.d,
            self.loaded,
            self,
            // A constant is worked out in the compiler, so each check costs nothing there and
            // one that fails is a compile error, whatever checks the build keeps at run time.
            Build {
                judge: None,
                checks: Checks::CHECKED,
            },
            None,
            &check::Ids::default(),
            main,
        );
        for at in values {
            c.value(at.clone());
        }
        let t = c.finish();
        // The program's own domains are declared by statements this run leaves out; one the
        // expression reads fails to lower.
        if (t.errors.iter()).any(|m| m.refusal != Some(Unbuilt::UndeclaredTopDomain)) {
            return Ran::Open;
        }
        let gs: Vec<usize> = (t.main.iter())
            .filter_map(|s| match s {
                check::TStmt::Let(check::VarRef::Global(g), _) => Some(*g),
                _ => None,
            })
            .collect();
        let Some(gs) = gs.get(gs.len().wrapping_sub(lets)..) else {
            return Ran::Open;
        };
        let mut low = lower::lower(self.d, &t, self.loaded);
        if !low.errors.is_empty() {
            return Ran::Open;
        }
        let names: Vec<(String, bool)> = (t.globals.iter())
            .map(|g| (g.name.clone(), g.mutable))
            .collect();
        let d = self.d;
        // A field naming the constant being worked out is still open here; link says so if
        // what the constant reads lays that struct out.
        let before = own::before(d, &mut low.funcs, &mut low.globals, &names, &[]);
        let (fns, tys) = (&low.funcs, &low.globals[..]);
        let checked = Answers {
            whole: &low.whole,
            deferred: &low.deferred,
            globals: tys,
        };
        let v = inst::Inputs {
            items: d,
            roles: d,
            fns,
            sums: &low.adds,
            checked: &checked,
            rows: &t.dom_rows,
        };
        // Only what the constants read is linked: a fn whose signature names a constant still
        // being worked out (`fn f(x: Mod<P>) -> Mod<P>` while `P` is) has no instance yet.
        let roots = inst::Roots {
            fns: Default::default(),
            globals: (0..tys.len()).map(cpl_next_ir::whole::GlobalId).collect(),
        };
        let mut prog = inst::link(v, &roots, false);
        let owned = own::after(d, &mut prog, &names, before);
        if !owned.errors.is_empty() {
            return Ran::Open;
        }
        let Ok(canon) = inst::canonicalize(&mut prog, &cpl_next_own::expand_calls) else {
            return Ran::Open;
        };
        if !prog.errors.is_empty() {
            return Ran::Open;
        }
        match cpl_next_engine::eval(&prog, &canon, gs, BUDGET) {
            Ok(words) => Ran::Values(
                (gs.iter().zip(words))
                    .map(|(&g, w)| scalar(prog.globals.get(g), &w))
                    .collect(),
            ),
            Err(stop) => Ran::Stop(stop),
        }
    }
}

/// What running a program of constants made of them.
enum Ran {
    /// It is not a program this run can build.
    Open,
    Stop(Stop),
    Values(Vec<Option<Scalar>>),
}

/// The value the words `w` of a global of type `ty` hold, if it is a scalar.
fn scalar(ty: Option<&CTy>, w: &[u64]) -> Option<Scalar> {
    let w = w.first().copied().unwrap_or(0);
    match ty? {
        CTy::Int => Some(Scalar::Int(w as i64)),
        CTy::U64 => Some(Scalar::U64(w)),
        CTy::F64 => Some(Scalar::F64(w)),
        CTy::Bool => Some(Scalar::Bool(w != 0)),
        CTy::Char => Some(Scalar::Char(w as u8)),
        _ => None,
    }
}

impl check::Consts for Evaluator<'_> {
    /// One asked for by a check run to work out another constant, and decided at run time, is
    /// that outer check's to say.
    fn value(&self, at: SourceId) -> Value {
        if self.batching.get() {
            return Err(ConstError::Pending);
        }
        let key = (at.item.clone(), at.span);
        if let Some(v) = self.done.borrow().get(&key) {
            return pending(&self.busy.borrow(), v.clone());
        }
        if self.batch.borrow().is_none() {
            let b = self.batch();
            *self.batch.borrow_mut() = Some(b);
        }
        if let Some(v) = self.batch.borrow().as_ref().and_then(|b| b.get(&key)) {
            return pending(&self.busy.borrow(), v.clone());
        }
        // A check run to work out this constant that asks for it again (a signature's
        // `Mod<P>` while `P` is worked out) has no value for it yet; only reading it is a cycle.
        if !self.busy.borrow_mut().insert(key.clone()) {
            return Err(ConstError::Pending);
        }
        let mut v = match self.expr_at(&at) {
            // A literal is its value: no program is checked to read it (std's `NTT_P1`, which
            // its own signatures' `Mod<NTT_P1>` ask for while std is checked).
            Some(next::Expr::Literal(l))
                if l.token().is_some_and(|t| t.kind() == TokenKind::Int) =>
            {
                let t = l.token().map(|t| t.text().to_string()).unwrap_or_default();
                Ok(cpl_next_syntax::literal::int_value(&t)
                    .and_then(|v| i64::try_from(v).ok())
                    .map(Scalar::Int))
            }
            Some(e) => self.run(&at.item, &e),
            None => {
                let what = "a constant's expression is not in its item";
                Err(ConstError::Failed(Box::new(Msg::at(
                    at.clone(),
                    Msg::internal(what),
                ))))
            }
        };
        self.busy.borrow_mut().remove(&key);
        if self.cyclic.borrow_mut().remove(&key) {
            let msg = Msg::verdict(
                codes::DEFINITION_CYCLE,
                "this constant reads itself while it is worked out, so it has no value",
            );
            v = Err(ConstError::Failed(Box::new(Msg::at(at, msg))));
        }
        self.done.borrow_mut().insert(key, v.clone());
        pending(&self.busy.borrow(), v)
    }
}

/// A constant decided at run time, asked for by a check run to work out other constants
/// (`busy` is not empty), is that outer check's to say.
fn pending<K>(busy: &HashSet<K>, v: Value) -> Value {
    match v {
        Err(ConstError::Open) if !busy.is_empty() => Err(ConstError::Pending),
        v => v,
    }
}

/// The names `e` reads.
fn names(e: &next::Expr) -> HashSet<String> {
    (e.syntax().descendants())
        .filter_map(next::NameRef::cast)
        .filter_map(|n| n.ident())
        .map(|t| t.text().to_string())
        .collect()
}
