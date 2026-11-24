//! A fn returning `?T` whose every call is tested where it is made, `if
//! let x = f(a)`, writes its value through a `T&` and returns whether there
//! is one: `bool f(a, T& cpl_out)`. `return Some(v)` is `cpl_out = v;
//! return true;`, `return None` is `return false;`, an `if` returns from
//! each branch, and a `.map` returns from inside its test. The call is
//! `T x; if (f(a, x)) { ... }` (see [`crate::opened`]): neither side
//! builds the optional. Only a `T` of scalars, which `T x;` makes for free.

use crate::devirt::Devirt;
use crate::emit::Emitter;
use crate::uses::places;
use cpl_ir::{walk, AssignOp, Block, Expr, ExprKind, FnId, PlaceKind, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

/// A value `T x;` declares and a copy moves for free.
pub(crate) fn plain(ty: &Ty) -> bool {
    match ty {
        Ty::Tuple(items) => !items.is_empty() && items.iter().all(plain),
        t => t.is_scalar() && *t != Ty::Unit,
    }
}

/// `e` as statements that return it, when its value is given only by
/// `Some`s and `None`s: an `if`, or a block whose value is a local that
/// starts `None` and is set to `Some(v)` last in its last statement's
/// branch (what `.map` and `.filter` lower to).
fn returned(e: &Expr) -> Option<Block> {
    let ret = |e: Expr| Stmt::Return(Some(e));
    match &e.kind {
        ExprKind::Some(_) | ExprKind::None => Some(Block {
            stmts: vec![ret(e.clone())],
        }),
        ExprKind::If(c, a, b) => {
            let (a, b) = (returned(a)?, returned(b)?);
            Some(Block {
                stmts: vec![Stmt::If((**c).clone(), a, b)],
            })
        }
        ExprKind::Do(block) => {
            let (Stmt::Give(value), rest) = block.stmts.split_last()? else {
                return None;
            };
            if walk::has_give(rest) {
                return None;
            }
            if let Some(mut b) = returned(value) {
                let mut stmts = rest.to_vec();
                stmts.append(&mut b.stmts);
                return Some(Block { stmts });
            }
            let ExprKind::Local(r) = value.kind else {
                return None;
            };
            let at = rest
                .iter()
                .position(|s| matches!(s, Stmt::Let(s, Some(n)) if *s == r && matches!(n.kind, ExprKind::None)))?;
            let mut stmts = rest.to_vec();
            stmts.remove(at);
            let Some(Stmt::If(_, then, otherwise)) = stmts.last_mut() else {
                return None;
            };
            let last = then.stmts.last_mut()?;
            let Stmt::Assign(place, AssignOp::Set, v) = last else {
                return None;
            };
            if place.kind != PlaceKind::Local(r)
                || !matches!(v.kind, ExprKind::Some(_))
                || !otherwise.stmts.is_empty()
            {
                return None;
            }
            *last = ret(v.clone());
            let none = Expr {
                kind: ExprKind::None,
                ty: e.ty.clone(),
                span: e.span,
            };
            stmts.push(ret(none));
            let body = Block { stmts };
            (!mentions(&body, r)).then_some(body)
        }
        _ => None,
    }
}

fn mentions(block: &Block, slot: usize) -> bool {
    let mut seen = false;
    places::each_local(block, &mut |s| seen |= s == slot);
    places::each_written(block, &mut |p| seen |= walk::root(p) == (false, slot));
    seen
}

/// Every body of the program.
fn bodies(program: &Program) -> impl Iterator<Item = &Block> {
    let fns = program.fns.iter().map(|f| &f.body);
    let closures = program.closures.iter().map(|c| &c.body);
    fns.chain(closures).chain([&program.main, &program.setup])
}

/// The fns written as `bool f(.., T& cpl_out)`.
pub(crate) fn into_fns(program: &Program, devirt: &Devirt<'_>) -> BTreeSet<FnId> {
    let mut named: BTreeMap<FnId, (usize, usize)> = BTreeMap::new();
    for body in bodies(program) {
        places::each_call(body, &mut |f, _| named.entry(f).or_default().0 += 1);
        walk::blocks(body, &mut |b| {
            for k in 0..b.stmts.len() {
                if let Stmt::Let(
                    _,
                    Some(Expr {
                        kind: ExprKind::Call(f, _),
                        ..
                    }),
                ) = &b.stmts[k]
                {
                    if crate::opened::tested_here(program, &b.stmts, k).is_some() {
                        named.entry(*f).or_default().1 += 1;
                    }
                }
            }
        });
    }
    named
        .into_iter()
        .filter(|&(id, (calls, tested))| {
            let f = &program.fns[id];
            calls == tested
                && matches!(&f.ret, Ty::Opt(inner) if plain(inner))
                && devirt.template_params(id).is_empty()
                && !devirt.returns_lambda(id)
                && !program.entry_fns.contains(&id)
                && !program.judge_fns.contains(&id)
                && !program.is_hook(id)
                && !program.cpp_bodies.iter().any(|(c, _)| *c == id)
                && returns_parts(&f.body)
        })
        .map(|(id, _)| id)
        .collect()
}

/// Whether every `return` of `body` is one [`returned`] rewrites.
fn returns_parts(body: &Block) -> bool {
    let mut ok = true;
    places::each_return(body, &mut |e| ok &= e.and_then(returned).is_some());
    ok
}

impl Emitter<'_> {
    /// Whether `return value` here writes `cpl_out`.
    pub(crate) fn returns_into(&self, value: &Expr) -> bool {
        returned(value).is_some()
            && self.closure_id.is_none()
            && self.bodies.is_empty()
            && self.pushing.is_none()
            && self.fn_id.is_some_and(|id| self.into.contains(&id))
    }

    /// `return value` in an into fn: `cpl_out = v; return true;` for
    /// `Some(v)`, `return false;` for `None`, else its statements.
    pub(crate) fn return_into(&mut self, value: &Expr, out: &mut String) {
        match &value.kind {
            ExprKind::Some(v) => {
                let v = self.expr(v).text;
                self.line(out, &format!("cpl_out = {v};"));
                self.line(out, "return true;");
            }
            ExprKind::None => self.line(out, "return false;"),
            _ => {
                if let Some(body) = returned(value) {
                    self.block(&body, out);
                }
            }
        }
    }

    /// The C++ type `cpl_out` holds in into fn `id`.
    pub(crate) fn out_param(&self, id: FnId) -> Option<String> {
        let Ty::Opt(inner) = &self.program.fns[id].ret else {
            return None;
        };
        self.into
            .contains(&id)
            .then(|| format!("{}& cpl_out", self.ty(inner)))
    }
}
