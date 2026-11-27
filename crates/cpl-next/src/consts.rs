//! The `const_value` query (`handbook/design/next/incremental.md`): a constant expression's
//! value, worked out by running it on the engine, the one evaluator. The checker asks through
//! `check::Consts`, so it holds no evaluator of its own.
use crate::{check, diag::Msg, inst, lower, own};
use check::ConstError;
use cpl_diag::{codes, Code};
use cpl_next_decls::{decls::Decls, modules::Loaded};
use cpl_next_engine::Stop;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::ops::{Fault, Scalar};
use cpl_next_ir::prog::CTy;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast;
use cpl_syntax::visit::{walk_expr, Visitor};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// The steps a constant may take (each fn entered and each jump taken back is one), the same
/// on every machine.
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
        stmts.push((item.clone(), value_of(e)));
        let open = Err(ConstError::Open);
        let t = check::check(self.d, &stmts, self.loaded, self);
        let global = t.main.iter().rev().find_map(|s| match s {
            check::TStmt::Let(check::VarRef::Global(g), _) => Some(*g),
            _ => None,
        });
        // The program's own domains are declared by statements this run leaves out; one the
        // expression reads fails to lower.
        let failed = (t.errors.iter()).any(|m| m.refusal != Some(Unbuilt::UndeclaredTopDomain));
        let (Some(g), false) = (global, failed) else {
            return open;
        };
        let low = lower::lower(self.d, &t, self.loaded);
        if !low.errors.is_empty() {
            return open;
        }
        let mut prog = inst::link(self.d, &t, &low.funcs, &low.globals);
        let owned = own::check(self.d, &t);
        if !owned.errors.is_empty() {
            return open;
        }
        own::place_copies(&mut prog, &owned.copies);
        let Ok(canon) = inst::canonicalize(&prog) else {
            return open;
        };
        if !prog.errors.is_empty() {
            return open;
        }
        let failed = |code, text: String| {
            Err(ConstError::Failed(Box::new(Msg::at(
                item.at(e.span),
                Msg::verdict(code, text),
            ))))
        };
        let w = match cpl_next_engine::eval(&prog, &canon, g, BUDGET) {
            Ok(words) => words.first().copied().unwrap_or(0),
            // What the engine cannot run yet runs at start-up instead.
            Err(Stop::Input) => return open,
            Err(Stop::Trap(t)) if matches!(t.fault, Fault::Internal(_)) => return open,
            Err(Stop::Trap(t)) => {
                let what = if t.msg.is_empty() {
                    t.fault.message().to_string()
                } else {
                    t.msg
                };
                return failed(
                    Code(t.fault.code()),
                    format!("working out this constant traps: {what}"),
                );
            }
            Err(Stop::Budget) => {
                return failed(
                    codes::CONST_BUDGET,
                    format!(
                        "working out this constant took more than {BUDGET} steps; compute it at run time with `let`"
                    ),
                )
            }
        };
        Ok(match prog.globals.get(g) {
            Some(CTy::Int) => Some(Scalar::Int(w as i64)),
            Some(CTy::U64) => Some(Scalar::U64(w)),
            Some(CTy::F64) => Some(Scalar::F64(w)),
            Some(CTy::Bool) => Some(Scalar::Bool(w != 0)),
            Some(CTy::Char) => Some(Scalar::Char(w as u8)),
            _ => None,
        })
    }
}

impl check::Consts for Evaluator<'_> {
    fn value(&self, item: &ItemId, e: &ast::Expr) -> Value {
        let key = (item.clone(), e.span);
        if let Some(v) = self.done.borrow().get(&key) {
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

/// `let $value = e`: the statement whose value a run reads.
fn value_of(e: &ast::Expr) -> ast::Stmt {
    ast::Stmt {
        kind: ast::StmtKind::Let {
            mutable: false,
            pat: ast::Pattern {
                kind: ast::PatternKind::Name("$value".to_string()),
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
