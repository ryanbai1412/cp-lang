//! `let xs = if c: f(a) else: a[i]`: the branch that names a place is read
//! in place, so only the other one builds a value. And a view that lends
//! from a temporary, `let v = chain([1, 3, 2], r).take(n)`, keeps the
//! temporary alive as long as the view.

use crate::emit::{Emitter, COND};
use crate::names::ident;
use crate::walk::{self, Effects};
use cpl_ir::{Arg, BinOp, Block, Expr, ExprKind, Stmt, Ty};

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

/// Whether a `let` of `init` may read a place in place, so its block works
/// out what the statements after it do: `a[i]`, `if c: f(a) else: a[i]`,
/// `m.get(k)`, `m.get(k) ?? d`.
pub(crate) fn may_lend(init: &Expr) -> bool {
    let map_get = |e: &Expr| {
        matches!(&e.kind, ExprKind::Builtin(cpl_ir::Builtin::MapGet, a)
            if a.first().is_some_and(|m| crate::stmt::place_root(m).is_some()))
    };
    crate::stmt::place_root(init).is_some()
        || lending_if(init)
            .is_some_and(|arms| arms.iter().any(|a| crate::stmt::place_root(a).is_some()))
        || map_get(init)
        || matches!(&init.kind, ExprKind::Binary(BinOp::Coalesce, got, _) if map_get(got))
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
    /// Every read of the `let` works on a `cpl::Lent` (see [`Emitter::lendable_reads`]).
    pub(crate) lendable: bool,
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
            lendable: false,
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
        let named = tail.is_some_and(|tail| tail.lendable);
        let plain = matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_));
        (self.closure_id.is_none()
            && (pattern_temp || named)
            && plain
            && !value_ty.is_scalar()
            && !self.spent.contains(&slot)
            && self.place_stays(slot, map, tail))
        .then_some([map, key])
    }

    /// Writes `const auto t = cpl::map_lend(m, k); const cpp& name = t ? *t : (own = d);`
    /// for `let name = m.get(k) ?? d` when nothing changes `m` while `name`
    /// lives, so a found entry is read in place, never copied.
    pub(crate) fn let_lending_coalesce(
        &mut self,
        slot: usize,
        init: &Expr,
        (cpp, name): (&str, &str),
        tail: Option<&Tail>,
        out: &mut String,
    ) -> bool {
        let ExprKind::Binary(BinOp::Coalesce, got, default) = &init.kind else {
            return false;
        };
        let ExprKind::Builtin(cpl_ir::Builtin::MapGet, args) = &got.kind else {
            return false;
        };
        let [map, key] = args.as_slice() else {
            return false;
        };
        let plain = matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_));
        if got.ty == init.ty
            || init.ty.is_scalar()
            || self.closure_id.is_some()
            || !plain
            || self.spent.contains(&slot)
            || !self.place_stays(slot, map, tail)
        {
            return false;
        }
        self.need("map_lend");
        let (m, k) = (self.expr(map).text, self.expr(key).text);
        let entry = self.temp();
        self.line(
            out,
            &format!("const auto {entry} = cpl::map_lend({m}, {k});"),
        );
        let other = if self.reads_in_place(slot, default, tail) {
            format!("({})", self.expr(default).text)
        } else {
            let own = self.temp();
            self.line(out, &format!("{cpp} {own};"));
            format!("({own} = {})", self.expr(default).text)
        };
        self.line(
            out,
            &format!("const {cpp}& {name} = {entry} ? *{entry} : {other};"),
        );
        true
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

/// Where a lent optional comes from: `m.get(k)`, or a name already holding a `cpl::Lent`.
pub(crate) enum Lender<'e> {
    Get(&'e Expr, &'e Expr),
    Named(&'e Expr),
}

impl<'p> Emitter<'p> {
    /// Whether every read of the named optional `slot` in `rest` works on a
    /// `cpl::Lent`: `o.is_some`, `o ?? d` with a plain default, and the
    /// temporary an `if let l = o` tests, which then lends too.
    pub(crate) fn lendable_reads(&self, slot: usize, rest: &[Stmt]) -> bool {
        if self.locals()[slot].name.is_empty() {
            return false;
        }
        let block = Block {
            stmts: rest.to_vec(),
        };
        let local = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if s == slot);
        let (mut reads, mut lendable) = (0, 0);
        walk::deep_exprs(&block, &mut |e| match &e.kind {
            ExprKind::Local(s) if *s == slot => reads += 1,
            ExprKind::IsSome(x) if local(x) => lendable += 1,
            ExprKind::Binary(BinOp::Coalesce, x, _) if local(x) && x.ty != e.ty => lendable += 1,
            _ => {}
        });
        walk::deep_stmts(&block, &mut |stmt| {
            if let Stmt::Let(t, Some(x)) = stmt {
                if local(x) && self.locals()[*t].name.is_empty() {
                    lendable += 1;
                }
            }
        });
        reads == lendable
    }

    /// `let t = o` for the temporary an `if let` tests, where `o` holds a `cpl::Lent`.
    pub(crate) fn let_copying_lent(&mut self, name: &str, init: &Expr, out: &mut String) -> bool {
        if !matches!(init.kind, ExprKind::Local(s) if self.lent.contains(&s)) {
            return false;
        }
        let o = self.expr(init).text;
        self.line(out, &format!("const auto {name} = {o};"));
        true
    }

    /// `m.get(k) ?? d` over a plain map with a non-scalar value, or `o ?? d`
    /// where `o` holds a `cpl::Lent`: where the value comes from, and the default.
    pub(crate) fn lent_coalesce<'e>(&self, seq: &'e Expr) -> Option<(Lender<'e>, &'e Expr)> {
        let ExprKind::Binary(BinOp::Coalesce, got, default) = &seq.kind else {
            return None;
        };
        match &got.kind {
            ExprKind::Local(s) if self.lent.contains(s) && got.ty != seq.ty => {
                Some((Lender::Named(got), &**default))
            }
            ExprKind::Builtin(cpl_ir::Builtin::MapGet, args)
                if self.closure_id.is_none()
                    && !seq.ty.is_scalar()
                    && got.ty != seq.ty
                    && matches!(
                        args.first().map(|m| &m.kind),
                        Some(ExprKind::Local(_) | ExprKind::Global(_))
                    ) =>
            {
                Some((Lender::Get(&args[0], &args[1]), &**default))
            }
            _ => None,
        }
    }

    /// Opens a scope holding `m.get(k) ?? d` by reference, so `for v in` walks the entry in place.
    pub(crate) fn loop_over_lent(
        &mut self,
        from: Lender<'_>,
        default: &Expr,
        out: &mut String,
    ) -> String {
        self.line(out, "{");
        self.indent += 1;
        let entry = match from {
            Lender::Get(map, key) => {
                self.need("map_lend");
                let (m, k) = (self.expr(map).text, self.expr(key).text);
                let entry = self.temp();
                self.line(
                    out,
                    &format!("const auto {entry} = cpl::map_lend({m}, {k});"),
                );
                entry
            }
            Lender::Named(o) => self.expr(o).text,
        };
        let (own, held) = (self.temp(), self.temp());
        self.line(out, &format!("std::decay_t<decltype(*{entry})> {own};"));
        let d = self.expr(default).text;
        self.line(
            out,
            &format!("const auto& {held} = {entry} ? *{entry} : ({own} = {d});"),
        );
        held
    }
}
