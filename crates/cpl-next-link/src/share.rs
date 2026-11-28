//! Which types share a representation, decided once here and read by every backend: types
//! whose values differ only in which domain a position counts, or in the range an `Int` is
//! held to, lie in memory alike, since a position and a ranged `Int` are one word, so a value
//! of one can be stored where the other is wanted.
use cpl_next_ir::ir::*;
use cpl_next_ir::prog::{CTy, Program};
use std::collections::{HashMap, HashSet};

/// Each type `funcs` and `p` use whose representation is an earlier type's, mapped to that type.
pub(crate) fn shares(p: &Program, funcs: &[Func<CTy>]) -> HashMap<CTy, CTy> {
    let mut s = Shares {
        p,
        first: HashMap::new(),
        seen: HashSet::new(),
        out: HashMap::new(),
    };
    p.globals.iter().for_each(|t| s.ty(t));
    for f in funcs {
        f.locals.iter().for_each(|t| s.ty(t));
        s.ty(&f.ret);
        if let Some((_, t)) = &f.region {
            s.ty(t);
        }
        s.block(&f.body);
    }
    s.out
}

struct Shares<'a> {
    p: &'a Program,
    /// The first type seen with each representation.
    first: HashMap<CTy, CTy>,
    seen: HashSet<CTy>,
    out: HashMap<CTy, CTy>,
}

impl Shares<'_> {
    fn ty(&mut self, t: &CTy) {
        if !self.seen.insert(t.clone()) {
            return;
        }
        let first = self.first.entry(erased(t)).or_insert_with(|| t.clone());
        // A bare domain or range type keeps its own identity: its size or bounds are read
        // through it.
        if first != t && !matches!(t, CTy::Dom(_) | CTy::Range(..) | CTy::RangeTo(..)) {
            self.out.insert(t.clone(), first.clone());
        }
        let held = match t {
            CTy::Raw(e) | CTy::Ref(e) | CTy::Lent(e) => vec![(**e).clone()],
            CTy::Struct(..) => self.p.parts(t),
            CTy::Clo(k) => self.p.captures(k),
            CTy::Fn(_, _, ms) => ms.iter().map(|k| CTy::Clo(Box::new(k.clone()))).collect(),
            _ => Vec::new(),
        };
        held.iter().for_each(|t| self.ty(t));
    }
    fn block(&mut self, b: &[Stmt<CTy>]) {
        for st in b {
            match st {
                Stmt::Let(_, e)
                | Stmt::Assign(_, e)
                | Stmt::Expr(e)
                | Stmt::Yield(e)
                | Stmt::Fact(e)
                | Stmt::Check { cond: e, .. } => self.expr(e),
                Stmt::Return(e) => e.iter().for_each(|e| self.expr(e)),
                Stmt::If(c, a, b) => {
                    self.expr(c);
                    self.block(a);
                    self.block(b);
                }
                Stmt::Loop(a, b, _) => {
                    self.block(a);
                    self.block(b);
                }
                Stmt::Switch(c, arms) => {
                    self.expr(c);
                    arms.iter().for_each(|a| self.block(a));
                }
                Stmt::Region { args, body, .. } => {
                    self.args(args);
                    self.block(body);
                }
                Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                    xs.iter().for_each(|x| self.expr(x))
                }
                Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
            }
        }
    }
    fn expr(&mut self, e: &Expr<CTy>) {
        self.ty(&e.ty);
        match &e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| self.expr(x)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => self.args(xs),
            EK::CallValue(c, xs, _) => {
                self.expr(c);
                self.args(xs);
            }
            _ => {}
        }
    }
    fn args(&mut self, xs: &[Arg<CTy>]) {
        for a in xs {
            if let Arg::Value(x) = a {
                self.expr(x);
            }
        }
    }
}

/// `t` with every domain the same one and every ranged `Int` an `Int`: what tells
/// representations apart.
fn erased(t: &CTy) -> CTy {
    match t {
        CTy::Dom(_) => CTy::Dom(cpl_next_ir::ty::DomId::Decl(0)),
        CTy::Range(..) | CTy::RangeTo(..) => CTy::Int,
        CTy::Raw(e) => CTy::Raw(Box::new(erased(e))),
        CTy::Ref(e) => CTy::Ref(Box::new(erased(e))),
        CTy::Lent(e) => CTy::Lent(Box::new(erased(e))),
        CTy::Struct(d, ts, ss) => CTy::Struct(*d, ts.iter().map(erased).collect(), ss.clone()),
        other => other.clone(),
    }
}
