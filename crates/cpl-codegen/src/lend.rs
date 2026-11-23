//! `let xs = if c: f(a) else: a[i]`: the branch that names a place is read
//! in place, so only the other one builds a value. And a view that lends
//! from a temporary, `let v = chain([1, 3, 2], r).take(n)`, keeps the
//! temporary alive as long as the view.

use crate::emit::{Code, Emitter, COND};
use crate::walk::{self, Effects};
use cpl_ir::{Arg, AssignOp, BinOp, Block, Expr, ExprKind, Place, PlaceKind, Stmt, Ty};

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
    pub(crate) fn of(program: &cpl_ir::Program, stmts: &[Stmt]) -> Tail {
        let block = Block {
            stmts: stmts.to_vec(),
        };
        let (mut calls, mut opaque, mut closures) = (false, false, Vec::new());
        walk::deep_exprs(&block, &mut |e| match e.kind {
            ExprKind::Call(..) => calls = true,
            ExprKind::CallExpr(..) => {
                calls = true;
                opaque = true;
            }
            ExprKind::Closure(id) => closures.push(id),
            _ => {}
        });
        let mut effects = Effects::of_block(&block);
        for id in closures {
            opaque |= !closure_effects(&program.closures[id], &mut effects, &mut calls);
        }
        Tail {
            effects,
            calls,
            opaque,
            tested: false,
            lendable: false,
        }
    }
}

/// Adds what running closure `c` can change to `effects`, with its
/// by-reference captures as the caller's variables. False when that is not
/// known: it calls a fn value, makes a closure, or takes `mut` parameters.
fn closure_effects(c: &cpl_ir::ClosureDef, effects: &mut Effects, calls: &mut bool) -> bool {
    let mut known = c.muts.is_empty();
    walk::deep_exprs(&c.body, &mut |e| {
        known &= !matches!(e.kind, ExprKind::CallExpr(..) | ExprKind::Closure(_))
    });
    let inner = Effects::of_block(&c.body);
    for cap in c
        .captures
        .iter()
        .filter(|cap| inner.locals.contains(&cap.inner))
    {
        match cap.is_global {
            true => effects.globals.insert(cap.outer),
            false => effects.locals.insert(cap.outer),
        };
    }
    *calls |= !inner.calls.is_empty();
    effects.globals.extend(inner.globals);
    effects.calls.extend(inner.calls);
    known
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

/// Whether every value `block` gives is a variable declared outside it,
/// of a type held on the heap: the block may give a reference to it.
fn gives_outer_place(block: &cpl_ir::Block, ty: &Ty) -> bool {
    fn gives(block: &cpl_ir::Block, own: &mut Vec<usize>, out: &mut Vec<Option<usize>>) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let(slot, _) => own.push(*slot),
                Stmt::Give(value) => out.push(variable(value)),
                _ => {}
            }
            walk::stmt_blocks(stmt, &mut |inner| gives(inner, own, out));
        }
    }
    // The variable `e` names, `usize::MAX` for a global; a list given as an
    // array over a domain is the same C++ value.
    fn variable(e: &Expr) -> Option<usize> {
        match &e.kind {
            ExprKind::Local(slot) => Some(*slot),
            ExprKind::Global(_) => Some(usize::MAX),
            ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => variable(of.first()?),
            _ => None,
        }
    }
    if !matches!(
        ty,
        Ty::List(_) | Ty::Array { .. } | Ty::Str | Ty::Map(..) | Ty::HashMap(..)
    ) {
        return false;
    }
    let (mut own, mut out) = (Vec::new(), Vec::new());
    gives(block, &mut own, &mut out);
    !out.is_empty()
        && out
            .iter()
            .all(|g| g.is_some_and(|slot| !own.contains(&slot)))
}

/// The variable `value` reads in place: a path, or a `Do` block that gives
/// one (see [`gives_outer_place`]).
pub(crate) fn given_root(value: &Expr) -> Option<(bool, usize)> {
    match &value.kind {
        ExprKind::Do(block) if gives_outer_place(block, &value.ty) => {
            let mut root = None;
            walk::deep_stmts(block, &mut |s| {
                if let Stmt::Give(g) = s {
                    root = root.or_else(|| given_root(g));
                }
            });
            root
        }
        ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => given_root(of.first()?),
        _ => walk::path_root(value),
    }
}

impl<'p> Emitter<'p> {
    /// A `Do` block as a lambda called on the spot. One that gives a
    /// variable from outside it, `{ D_len = xs.len; xs }`, returns a
    /// reference to it, never a copy.
    pub(crate) fn do_lambda(&mut self, block: &cpl_ir::Block, ty: &Ty, ret: String) -> Code {
        let by_ref = gives_outer_place(block, ty);
        let ret = if by_ref {
            "decltype(auto)".to_string()
        } else {
            ret
        };
        let outer = std::mem::replace(&mut self.give_ref, by_ref);
        let code = self.lambda(&ret, |this, out| this.block(block, out));
        self.give_ref = outer;
        code
    }

    /// Marks each `m.get(k) ?? d` among `operands` to be written
    /// `cpl::map_or(m, k, d)`: a reference to the entry, or to the default
    /// temporary, that lives to the end of the full expression. Only when
    /// nothing the expression runs can change `m`: no operand stores to it,
    /// calls a fn value or makes a closure, and `callee` and the places
    /// passed `mut` leave it alone. Gives the marks it made.
    pub(crate) fn mark_read_in_place(
        &mut self,
        operands: &[&Expr],
        callee: Option<usize>,
        muts: &[(bool, usize)],
    ) -> Vec<usize> {
        let leaves = |this: &Self, e: &Expr, root: (bool, usize)| {
            let effects = Effects::of_expr(e);
            let mut opaque = false;
            walk::deep_expr(e, &mut |x| {
                opaque |= matches!(x.kind, ExprKind::Closure(_) | ExprKind::CallExpr(..))
            });
            let writes = |f: &usize| root.0 && this.summaries[*f].writes.contains(&root.1);
            !opaque
                && !effects.calls.iter().any(writes)
                && !if root.0 {
                    &effects.globals
                } else {
                    &effects.locals
                }
                .contains(&root.1)
        };
        let mut marked = Vec::new();
        for &value in operands {
            let ExprKind::Binary(BinOp::Coalesce, got, _) = &value.kind else {
                continue;
            };
            let ExprKind::Builtin(cpl_ir::Builtin::MapGet, args) = &got.kind else {
                continue;
            };
            let Some(root) = args.first().and_then(crate::stmt::place_root) else {
                continue;
            };
            let callee_writes =
                callee.is_some_and(|f| root.0 && self.summaries[f].writes.contains(&root.1));
            if value.ty.is_scalar()
                || got.ty == value.ty
                || muts.contains(&root)
                || callee_writes
                || !operands.iter().all(|e| leaves(self, e, root))
            {
                continue;
            }
            let key = value as *const Expr as usize;
            self.read_in_place.insert(key);
            marked.push(key);
        }
        marked
    }

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

    /// Pins the containers a stored view lends from (`vs.push(a.sorted().runs())`)
    /// for as long as the store; returns the mark to truncate `pinned` to.
    pub(crate) fn pin_stored(&mut self, e: &Expr, out: &mut String) -> usize {
        let mark = self.pinned.len();
        if let ExprKind::Mutate(_, _, args) = &e.kind {
            for arg in args {
                self.pin_owners(arg, Hold::Pool, out);
            }
        }
        mark
    }

    /// `place = value` where `value` holds views of temporaries: the place
    /// outlives the statement, so the owners its views read do too.
    pub(crate) fn store_pinned(&mut self, place: &Place, value: &Expr, out: &mut String) -> bool {
        let hold = match place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => Hold::Static,
            _ => Hold::Pool,
        };
        let Some(text) = self.pinned_view(value, hold, out) else {
            return false;
        };
        let name = self.place(place);
        self.line(out, &format!("{name} = {text};"));
        true
    }

    /// The C++ for a view-typed call `e` with every temporary container it
    /// may lend from held in a named local first, so the view does not
    /// dangle; `None` when there is none. `hold` says how long. The call itself is written as usual,
    /// with each owner's name in place of its expression.
    pub(crate) fn pinned_view(&mut self, e: &Expr, hold: Hold, out: &mut String) -> Option<String> {
        let mark = self.pinned.len();
        self.pin_owners(e, hold, out);
        if self.pinned.len() == mark {
            return None;
        }
        let text = self.expr(e).text;
        self.pinned.truncate(mark);
        Some(text)
    }

    /// Bind each temporary container a view-typed call lends from, at any
    /// depth of view-typed calls, to a named local.
    pub(crate) fn pin_owners(&mut self, e: &Expr, hold: Hold, out: &mut String) {
        if !e.ty.holds_view() {
            return;
        }
        let args = match &e.kind {
            ExprKind::Call(_, args) => args,
            ExprKind::Tuple(parts) | ExprKind::Variant(_, parts) => {
                for part in parts {
                    self.pin_owners(part, hold, out);
                }
                return;
            }
            ExprKind::Some(inner) => return self.pin_owners(inner, hold, out),
            _ => return,
        };
        for arg in args {
            let Arg::Value(v) = arg else {
                continue;
            };
            if !temporary_owner(v) {
                self.pin_owners(v, hold, out);
                continue;
            }
            let owner = self.temp();
            let owner_ty = self.ty(&v.ty);
            let init = self.expr(v).text;
            let name = match hold {
                Hold::Block => {
                    self.line(out, &format!("{owner_ty} {owner} = {init};"));
                    owner
                }
                Hold::Static => {
                    self.line(out, &format!("static {owner_ty} {owner};"));
                    self.line(out, &format!("{owner} = {init};"));
                    owner
                }
                Hold::Pool => {
                    self.used.insert("deque");
                    self.line(out, &format!("static std::deque<{owner_ty}> {owner};"));
                    self.line(out, &format!("{owner}.push_back({init});"));
                    format!("{owner}.back()")
                }
            };
            self.pinned.push((v as *const Expr, name));
        }
    }
}

/// How long the containers a view lends from are kept.
#[derive(Clone, Copy)]
pub(crate) enum Hold {
    /// To the end of the enclosing block.
    Block,
    /// Until the same statement runs again, which also replaces the view.
    Static,
    /// To the end of the run: the view goes into a place that may keep
    /// it past the next run of the statement.
    Pool,
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
                    && args
                        .first()
                        .is_some_and(|m| crate::stmt::place_root(m).is_some()) =>
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

/// Equal pure paths, ignoring spans: `masks[v]` twice.
fn same_path(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Local(x), ExprKind::Local(y)) => x == y,
        (ExprKind::Global(x), ExprKind::Global(y)) => x == y,
        (ExprKind::Cast(c, x), ExprKind::Cast(d, y)) => c == d && same_path(x, y),
        (ExprKind::Field(x, f), ExprKind::Field(y, g)) => f == g && same_path(x, y),
        (ExprKind::Binary(o, x, l), ExprKind::Binary(p, y, r)) => {
            o == p && same_path(x, y) && same_path(l, r)
        }
        (ExprKind::Index(x, i), ExprKind::Index(y, j)) => {
            i.len() == j.len() && same_path(x, y) && i.iter().zip(j).all(|(a, b)| same_path(a, b))
        }
        _ => false,
    }
}

/// Whether running `e` may read or write the variable `root`: anything
/// that calls code counts.
fn may_touch(e: &Expr, root: (bool, usize)) -> bool {
    let mut hit = false;
    walk::deep_expr(e, &mut |x| {
        hit |= match &x.kind {
            ExprKind::Local(s) => !root.0 && *s == root.1,
            ExprKind::Global(g) => root.0 && *g == root.1,
            ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, ..) | ExprKind::MutSlice(p, ..) => {
                walk::root(p) == root
            }
            ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Closure(_) => true,
            _ => false,
        }
    });
    hit
}

impl Emitter<'_> {
    /// `m[k] = (m.get(k) ?? d) + r` copies the entry only to store the
    /// longer list back. When nothing else in the value can see `m`, the
    /// entry is moved out instead (`cpl::map_move_or`) and the store
    /// puts it back.
    pub(crate) fn mark_moved_entry(&mut self, place: &cpl_ir::Place, op: AssignOp, value: &Expr) {
        let (AssignOp::Set, PlaceKind::Index(base, keys), ExprKind::Do(block)) =
            (op, &place.kind, &value.kind)
        else {
            return;
        };
        let ([key], PlaceKind::Local(_) | PlaceKind::Global(_)) = (keys.as_slice(), &base.kind)
        else {
            return;
        };
        let root = walk::root(base);
        let [Stmt::Let(_, Some(copy)), rest @ ..] = block.stmts.as_slice() else {
            return;
        };
        let ExprKind::Builtin(cpl_ir::Builtin::Copy, of) = &copy.kind else {
            return;
        };
        let Some(ExprKind::Binary(BinOp::Coalesce, got, d)) = of.first().map(|e| &e.kind) else {
            return;
        };
        let ExprKind::Builtin(cpl_ir::Builtin::MapGet, args) = &got.kind else {
            return;
        };
        let rest_ok = rest.iter().all(|s| match s {
            Stmt::Expr(e) | Stmt::Give(e) => !may_touch(e, root),
            _ => false,
        });
        if matches!(copy.ty, Ty::List(_))
            && matches!(args[0].kind, ExprKind::Local(_) | ExprKind::Global(_))
            && walk::path_root(&args[0]) == Some(root)
            && (same_path(&args[1], key) || self.key_temp_of(key, &args[1]))
            && !may_touch(key, root)
            && !may_touch(d, root)
            && rest_ok
            && self.expr_elem_layout(&of[0]).is_none()
        {
            self.moved_entry.insert(copy as *const Expr as usize);
        }
    }

    /// Whether `key` is the temporary the store's key was evaluated into,
    /// and `got` the same key written again.
    fn key_temp_of(&self, key: &Expr, got: &Expr) -> bool {
        let ExprKind::Local(slot) = key.kind else {
            return false;
        };
        let v = (self.fn_id, false, slot);
        self.facts.stable(v) && self.facts.init(v).is_some_and(|init| same_path(init, got))
    }

    pub(crate) fn moved_entry_code(&mut self, of: &Expr) -> Code {
        let ExprKind::Binary(_, got, d) = &of.kind else {
            unreachable!("marked by mark_moved_entry")
        };
        let ExprKind::Builtin(_, args) = &got.kind else {
            unreachable!("marked by mark_moved_entry")
        };
        self.need("map_move_or");
        let (m, k, d) = (
            self.expr(&args[0]).text,
            self.expr(&args[1]).text,
            self.expr(d).text,
        );
        Code::primary(format!("cpl::map_move_or({m}, {k}, {d})"))
    }
}
