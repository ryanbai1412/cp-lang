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
        if self.return_cells(chain_expr(value), &value.ty, out) {
            return;
        }
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

    /// `if c: return a else: return b` in an into fn, when it is a
    /// [`chain`] of cells: one test after the chain (see
    /// [`Self::return_cells`]).
    pub(crate) fn if_into(&mut self, c: &Expr, a: &Block, b: &Block, out: &mut String) -> bool {
        let (Some(x), Some(y)) = (chain_block(a), chain_block(b)) else {
            return false;
        };
        let Chain::Leaf(leaf) = leftmost(&x) else {
            return false;
        };
        if !self.returns_into(leaf) {
            return false;
        }
        let ty = leaf.ty.clone();
        self.return_cells(Chain::If(c, Box::new(x), Box::new(y)), &ty, out)
    }

    /// An `if` chain whose every arm is `D.at(i).map(x => (.., x, ..))` of
    /// one tuple of positions: each arm only sets the cell's ordinals, and
    /// one test of every ordinal against its domain follows the chain. The
    /// other parts are positions already, so the test is each arm's own.
    fn return_cells(&mut self, chain: Chain<'_>, ty: &Ty, out: &mut String) -> bool {
        let Ty::Opt(inner) = ty else {
            return false;
        };
        let Ty::Tuple(parts) = &**inner else {
            return false;
        };
        let doms: Option<Vec<_>> = (parts.iter())
            .map(|t| match t {
                Ty::Pos(d) => Some(*d),
                _ => None,
            })
            .collect();
        let Some(doms) = doms else {
            return false;
        };
        if !matches!(chain, Chain::If(..)) || !cells(&chain) {
            return false;
        }
        let names: Vec<String> = doms.iter().map(|_| self.temp()).collect();
        self.line(out, &format!("ll {};", names.join(", ")));
        self.set_cells(&chain, &names, out);
        let test: Vec<String> = (doms.iter().zip(&names))
            .map(|(d, n)| {
                let d = crate::names::dom_ident(self.program, *d);
                format!("((cpl::u64){n} < (cpl::u64){d}_len)")
            })
            .collect();
        let ty = self.ty(inner);
        let parts: Vec<String> = names.iter().map(|n| format!("(int){n}")).collect();
        self.line(out, &format!("if ({}) {{", test.join(" & ")));
        self.indent += 1;
        self.line(out, &format!("cpl_out = {ty}{{{}}};", parts.join(", ")));
        self.line(out, "return true;");
        self.indent -= 1;
        self.line(out, "}");
        self.line(out, "return false;");
        true
    }

    fn set_cells(&mut self, chain: &Chain<'_>, names: &[String], out: &mut String) {
        let leaf = match chain {
            Chain::If(c, a, b) => {
                let c = self.expr(c).text;
                self.line(out, &format!("if ({c}) {{"));
                self.indent += 1;
                self.set_cells(a, names, out);
                self.indent -= 1;
                self.line(out, "} else {");
                self.indent += 1;
                self.set_cells(b, names, out);
                self.indent -= 1;
                self.line(out, "}");
                return;
            }
            Chain::Leaf(leaf) => leaf,
        };
        let Some((at, slot, items)) = cell(leaf) else {
            return;
        };
        for (k, item) in items.iter().enumerate() {
            let v = self.expr(if k == slot { at } else { item }).text;
            self.line(out, &format!("{} = {v};", names[k]));
        }
    }
}

/// An `if` chain of returned values, as an `if` expression or statements.
enum Chain<'e> {
    If(&'e Expr, Box<Chain<'e>>, Box<Chain<'e>>),
    Leaf(&'e Expr),
}

fn chain_expr(e: &Expr) -> Chain<'_> {
    match &e.kind {
        ExprKind::If(c, a, b) => Chain::If(c, Box::new(chain_expr(a)), Box::new(chain_expr(b))),
        _ => Chain::Leaf(e),
    }
}

/// A block that is one `return v` or one `if` of such blocks.
fn chain_block(b: &Block) -> Option<Chain<'_>> {
    match &b.stmts[..] {
        [Stmt::Return(Some(v))] => Some(chain_expr(v)),
        [Stmt::If(c, x, y)] => Some(Chain::If(
            c,
            Box::new(chain_block(x)?),
            Box::new(chain_block(y)?),
        )),
        _ => None,
    }
}

fn leftmost<'a, 'e>(c: &'a Chain<'e>) -> &'a Chain<'e> {
    match c {
        Chain::If(_, a, _) => leftmost(a),
        leaf => leaf,
    }
}

/// Whether every arm of `c` is a [`cell`].
fn cells(c: &Chain<'_>) -> bool {
    match c {
        Chain::If(_, a, b) => cells(a) && cells(b),
        Chain::Leaf(e) => cell(e).is_some(),
    }
}

/// `D.at(i).map(x => (.., x, ..))` as it lowers: `i`, the part `x` is, and
/// the parts, when the others read no local the block makes.
fn cell(e: &Expr) -> Option<(&Expr, usize, &[Expr])> {
    let ExprKind::Do(block) = &e.kind else {
        return None;
    };
    let [Stmt::Let(t, Some(made)), Stmt::Let(r, Some(none)), Stmt::If(test, then, otherwise), Stmt::Give(given)] =
        &block.stmts[..]
    else {
        return None;
    };
    let ExprKind::Builtin(cpl_ir::Builtin::PosTryOrd(_), args) = &made.kind else {
        return None;
    };
    let tested = matches!(&test.kind, ExprKind::IsSome(x) if matches!(x.kind, ExprKind::Local(s) if s == *t));
    if !matches!(none.kind, ExprKind::None)
        || !tested
        || !otherwise.stmts.is_empty()
        || !matches!(given.kind, ExprKind::Local(s) if s == *r)
    {
        return None;
    }
    let (Stmt::Assign(place, AssignOp::Set, set), lets) = then.stmts.split_last()? else {
        return None;
    };
    let ExprKind::Some(tuple) = &set.kind else {
        return None;
    };
    let ExprKind::Tuple(items) = &tuple.kind else {
        return None;
    };
    if place.kind != PlaceKind::Local(*r) {
        return None;
    }
    let mut made_here = vec![*t, *r];
    for (k, s) in lets.iter().enumerate() {
        let Stmt::Let(x, Some(v)) = s else {
            return None;
        };
        let from_t = k == 0
            && matches!(&v.kind, ExprKind::Unwrap(u, _) if matches!(u.kind, ExprKind::Local(s) if s == *t));
        let alias = k > 0 && matches!(v.kind, ExprKind::Local(s) if made_here[2..].contains(&s));
        if !from_t && !alias {
            return None;
        }
        made_here.push(*x);
    }
    if made_here.len() < 3 {
        return None;
    }
    let is_x = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if made_here[2..].contains(&s));
    let slot = items.iter().position(is_x)?;
    let others_read = (items.iter().enumerate())
        .filter(|&(k, _)| k != slot)
        .all(|(_, e)| reads_outside(e, &made_here));
    others_read.then_some((&args[0], slot, &items[..]))
}

/// Whether `e` only reads locals not in `made` and their fields.
fn reads_outside(e: &Expr, made: &[usize]) -> bool {
    match &e.kind {
        ExprKind::Local(s) => !made.contains(s),
        ExprKind::Global(_) => true,
        ExprKind::Field(b, _) | ExprKind::Cast(_, b) => reads_outside(b, made),
        _ => false,
    }
}
