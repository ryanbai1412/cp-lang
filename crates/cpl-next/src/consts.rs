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
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::ops::{Fault, Scalar};
use cpl_next_ir::prog::CTy;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast;
use cpl_syntax::visit::{walk_expr, Visitor};
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
    main: &'a [(ItemId, ast::Stmt)],
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
    pub(crate) fn new(d: &'a Decls, loaded: &'a Loaded, main: &'a [(ItemId, ast::Stmt)]) -> Self {
        Evaluator {
            d,
            loaded,
            main,
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
        for (_, s) in self.main {
            match &s.kind {
                ast::StmtKind::Let { pat, .. } => {
                    if let ast::PatternKind::Name(n) = &pat.kind {
                        bound.insert(n.as_str());
                    }
                }
                ast::StmtKind::Domain { name, .. } => {
                    bound.insert(name.text.as_str());
                }
                ast::StmtKind::Const { name, .. } => {
                    consts.insert(name.text.as_str());
                }
                _ => {}
            }
        }
        let mut admitted = HashSet::new();
        let mut stmts = Vec::new();
        let mut keys = Vec::new();
        for (k, s) in self.main {
            let closed = |e: &ast::Expr, admitted: &HashSet<&str>| {
                (names(e).iter()).all(|n| {
                    admitted.contains(n.as_str())
                        || !(bound.contains(n.as_str()) || consts.contains(n.as_str()))
                })
            };
            match &s.kind {
                ast::StmtKind::Const {
                    name,
                    generics,
                    init: Some(init),
                    ..
                } if generics.is_empty() && closed(init, &admitted) => {
                    stmts.push((k.clone(), binding(&name.text, init)));
                    keys.push((k.clone(), init.span));
                    admitted.insert(name.text.as_str());
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
                        _ => vec![],
                    };
                    for e in ends.into_iter().filter(|e| closed(e, &admitted)) {
                        stmts.push((k.clone(), binding("$value", e)));
                        keys.push((k.clone(), e.span));
                    }
                }
                _ => {}
            }
        }
        if stmts.is_empty() {
            return HashMap::new();
        }
        self.batching.set(true);
        let ran = self.execute(&stmts, stmts.len());
        self.batching.set(false);
        match ran {
            Ran::Values(vs) => keys.into_iter().zip(vs.into_iter().map(Ok)).collect(),
            Ran::Open | Ran::Stop(_) => HashMap::new(),
        }
    }

    /// `e` checked, lowered and linked as the value of a program of its own, after the
    /// program's constants it names that have values, then run on the engine.
    fn run(&self, item: &ItemId, e: &ast::Expr) -> Value {
        let named = names(e);
        let mut stmts = Vec::new();
        for (k, s) in self.main {
            if let ast::StmtKind::Const {
                name,
                generics,
                init: Some(init),
                ..
            } = &s.kind
            {
                if generics.is_empty()
                    && named.contains(&name.text)
                    && matches!(check::Consts::value(self, k, init), Ok(Some(_)))
                {
                    stmts.push((k.clone(), s.clone()));
                }
            }
        }
        stmts.push((item.clone(), binding("$value", e)));
        let failed = |code, text: String| {
            Err(ConstError::Failed(Box::new(Msg::at(
                item.at(e.span),
                Msg::verdict(code, text),
            ))))
        };
        match self.execute(&stmts, 1) {
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

    /// Runs the program `stmts` on the engine: the values of the globals its last `lets`
    /// top-level `let`s bind.
    fn execute(&self, stmts: &[(ItemId, ast::Stmt)], lets: usize) -> Ran {
        let t = check::check(
            self.d,
            stmts,
            self.loaded,
            self,
            &[],
            // A constant is worked out in the compiler, so each check costs nothing there and
            // one that fails is a compile error, whatever checks the build keeps at run time.
            Build {
                judge: None,
                checks: Checks::CHECKED,
            },
            &check::Ids::default(),
        );
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
        let (mut prog, owned) =
            own::link(d, &mut low.funcs, &mut low.globals, &names, &|fns, tys| {
                let checked = Answers {
                    whole: &low.whole,
                    deferred: &low.deferred,
                    globals: tys,
                };
                let v = inst::Inputs {
                    sigs: d,
                    fns,
                    sums: &low.adds,
                    checked: &checked,
                };
                inst::link(v, &crate::roots(fns, tys.len()), false)
            });
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
    fn value(&self, item: &ItemId, e: &ast::Expr) -> Value {
        if self.batching.get() {
            return Err(ConstError::Open);
        }
        let key = (item.clone(), e.span);
        if let Some(v) = self.done.borrow().get(&key) {
            return v.clone();
        }
        if self.batch.borrow().is_none() {
            let b = self.batch();
            *self.batch.borrow_mut() = Some(b);
        }
        if let Some(v) = self.batch.borrow().as_ref().and_then(|b| b.get(&key)) {
            return v.clone();
        }
        if !self.busy.borrow_mut().insert(key.clone()) {
            self.cyclic.borrow_mut().insert(key);
            return Err(ConstError::Open);
        }
        let mut v = self.run(item, e);
        self.busy.borrow_mut().remove(&key);
        if self.cyclic.borrow_mut().remove(&key) {
            let msg = Msg::verdict(
                codes::DEFINITION_CYCLE,
                "this constant reads itself while it is worked out, so it has no value",
            );
            v = Err(ConstError::Failed(Box::new(Msg::at(item.at(e.span), msg))));
        }
        self.done.borrow_mut().insert(key, v.clone());
        v
    }
}

/// `let name = e`: a statement whose value a run reads.
fn binding(name: &str, e: &ast::Expr) -> ast::Stmt {
    ast::Stmt {
        kind: ast::StmtKind::Let {
            mutable: false,
            pat: ast::Pattern {
                kind: ast::PatternKind::Name(name.to_string()),
                span: e.span,
            },
            ty: None,
            init: Some(e.clone()),
            otherwise: None,
            guard: None,
        },
        span: e.span,
        decorators: Vec::new(),
        public: false,
    }
}

/// The names `e` reads.
fn names(e: &ast::Expr) -> HashSet<String> {
    struct Names(HashSet<String>);
    impl Visitor for Names {
        fn expr(&mut self, e: &ast::Expr) {
            if let ast::ExprKind::Name(n) = &e.kind {
                self.0.insert(n.clone());
            }
        }
    }
    let mut v = Names(HashSet::new());
    walk_expr(e, &mut v);
    v.0
}
