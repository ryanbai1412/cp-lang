//! `let xs = if c: f(a) else: a[i]`: the branch that names a place is read
//! in place, so only the other one builds a value. And a view that lends
//! from a temporary, `let v = chain([1, 3, 2], r).take(n)`, keeps the
//! temporary alive as long as the view.

use crate::emit::{Emitter, COND};
use crate::names::ident;
use crate::walk::{self, Effects};
use cpl_ir::{Arg, Block, Expr, ExprKind, Stmt, Ty};

/// A container argument that is no place: a temporary C++ destroys at the
/// end of the statement.
fn temporary_owner(e: &Expr) -> bool {
    matches!(e.ty, Ty::List(_) | Ty::Array { .. } | Ty::Str) && walk::path_root(e).is_none()
}

/// The `if` whose branches a `let` may read in place.
pub(crate) fn lending_if(init: &Expr) -> Option<[&Expr; 2]> {
    match &init.kind {
        ExprKind::If(_, then, otherwise) => Some([then, otherwise]),
        _ => None,
    }
}

/// What the statements after a `let` in its block can do, for
/// [`Emitter::reads_in_place`].
pub(crate) struct Tail {
    pub(crate) effects: Effects,
    /// A call of a fn of the program.
    pub(crate) calls: bool,
    /// A call through a fn value, or a closure: code whose stores are not
    /// in `effects`.
    pub(crate) opaque: bool,
    /// The `let` is the optional an `if let` tests, read only by that test
    /// and by the copies at the top of its branch (see [`copied_at_test`]).
    pub(crate) tested: bool,
}

impl Tail {
    pub(crate) fn of(stmts: &[Stmt]) -> Tail {
        let block = Block {
            stmts: stmts.to_vec(),
        };
        let (mut calls, mut opaque) = (false, false);
        walk::deep_exprs(&block, &mut |e| match e.kind {
            ExprKind::Call(..) => calls = true,
            ExprKind::CallExpr(..) | ExprKind::Closure(_) => {
                calls = true;
                opaque = true;
            }
            _ => {}
        });
        Tail {
            effects: Effects::of_block(&block),
            calls,
            opaque,
            tested: false,
        }
    }
}

/// Whether the temporary `slot` an `if let` tests is read only by that test
/// and by copies of a scalar or a fn at the top of its branch: `t0.has_value()`,
/// then `f = *t0`. A reference into a place is dead before anything else runs,
/// so nothing can change the place under it.
pub(crate) fn copied_at_test(slot: usize, rest: &[Stmt]) -> bool {
    let Some((Stmt::If(cond, then, otherwise), after)) = rest.split_first() else {
        return false;
    };
    let local = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if s == slot);
    if !matches!(&cond.kind, ExprKind::IsSome(x) if local(x)) {
        return false;
    }
    let copied = |stmt: &Stmt| {
        matches!(stmt, Stmt::Let(_, Some(Expr { kind: ExprKind::Unwrap(x, _), ty, .. }))
            if local(x) && (ty.is_scalar() || matches!(ty, Ty::Fn(..))))
    };
    let copies = then.stmts.iter().take_while(|stmt| copied(stmt)).count();
    let later = Block {
        stmts: [&then.stmts[copies..], &otherwise.stmts, after].concat(),
    };
    let mut seen = false;
    walk::deep_exprs(&later, &mut |e| {
        seen |= local(e) || matches!(e.kind, ExprKind::Closure(_))
    });
    !seen
}

impl<'p> Emitter<'p> {
    /// The map and key of `let t = m.get(k)` when `t` is the temporary an
    /// `if let` or `while let` tests, the value is no scalar, and nothing
    /// can change `m` while `t` lives: `t` may then point into `m`.
    pub(crate) fn lent_map_get<'e>(
        &self,
        slot: usize,
        init: &'e Expr,
        tail: Option<&Tail>,
    ) -> Option<[&'e Expr; 2]> {
        let ExprKind::Builtin(cpl_ir::Builtin::MapGet, args) = &init.kind else {
            return None;
        };
        let [map, key] = args.as_slice() else {
            return None;
        };
        let value_ty = match &map.ty {
            Ty::Map(_, v) | Ty::HashMap(_, v) => v,
            _ => return None,
        };
        let pattern_temp = self.locals()[slot].name.is_empty();
        let plain = matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_));
        (self.closure_id.is_none()
            && pattern_temp
            && plain
            && !value_ty.is_scalar()
            && !self.spent.contains(&slot)
            && self.place_stays(slot, map, tail))
        .then_some([map, key])
    }

    /// Writes `cpp own; const cpp& name = c ? (own = f(a)) : a[i];` and gives
    /// true when a branch names a place nothing changes while `name` lives.
    pub(crate) fn let_lending_branch(
        &mut self,
        slot: usize,
        init: &Expr,
        (cpp, name): (&str, &str),
        tail: Option<&Tail>,
        out: &mut String,
    ) -> bool {
        let (ExprKind::If(cond, ..), Some(arms)) = (&init.kind, lending_if(init)) else {
            return false;
        };
        let lent = arms.map(|arm| self.reads_in_place(slot, arm, tail));
        if !lent.contains(&true) {
            return false;
        }
        let c = self.expr(cond).at(COND + 1);
        let own = self.temp();
        let [a, b] = [0, 1].map(|k| {
            let text = self.expr(arms[k]).text;
            if lent[k] {
                format!("({text})")
            } else {
                format!("({own} = {text})")
            }
        });
        if lent.contains(&false) {
            self.line(out, &format!("{cpp} {own};"));
        }
        self.line(out, &format!("const {cpp}& {name} = {c} ? {a} : {b};"));
        true
    }

    /// The C++ for a view-typed call `e` with every temporary container it
    /// may lend from held in a named local first, so the view does not
    /// dangle; `None` when there is none. `keep` holds them in statics, for
    /// a view stored past this block.
    pub(crate) fn pinned_view(&mut self, e: &Expr, keep: bool, out: &mut String) -> Option<String> {
        let ExprKind::Call(f, args) = &e.kind else {
            return None;
        };
        if !matches!(e.ty, Ty::ListView(..) | Ty::StrView)
            || !args.iter().any(|a| self.lends_temporary(a))
        {
            return None;
        }
        let mut rendered = Vec::with_capacity(args.len());
        for arg in args {
            rendered.push(match arg {
                Arg::Value(v) if temporary_owner(v) => {
                    let owner = self.temp();
                    let owner_ty = self.ty(&v.ty);
                    let init = self.expr(v).text;
                    if keep {
                        self.line(out, &format!("static {owner_ty} {owner};"));
                        self.line(out, &format!("{owner} = {init};"));
                    } else {
                        self.line(out, &format!("{owner_ty} {owner} = {init};"));
                    }
                    owner
                }
                Arg::Value(v) => match self.pinned_view(v, keep, out) {
                    Some(text) => text,
                    None => self.expr(v).text,
                },
                Arg::Mut(p) => self.place(p),
            });
        }
        let callee = ident(&self.program.fns[*f].name);
        Some(format!("{callee}({})", rendered.join(", ")))
    }

    fn lends_temporary(&self, arg: &Arg) -> bool {
        match arg {
            Arg::Value(v) if temporary_owner(v) => true,
            Arg::Value(
                v @ Expr {
                    kind: ExprKind::Call(_, args),
                    ..
                },
            ) => {
                matches!(v.ty, Ty::ListView(..) | Ty::StrView)
                    && args.iter().any(|a| self.lends_temporary(a))
            }
            _ => false,
        }
    }
}
