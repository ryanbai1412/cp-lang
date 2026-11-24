//! `let xs = if c: f(a) else: a[i]`: the branch that names a place is read
//! in place, so only the other one builds a value. And a view that lends
//! from a temporary, `let v = chain([1, 3, 2], r).take(n)`, keeps the
//! temporary alive as long as the view.

use crate::emit::{Code, Emitter, COND};
use crate::uses::effects::{may_touch, Touch};
use crate::walk;
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

/// What the statements after a `let` in its block can do, for
/// [`Emitter::lends`].
pub(crate) struct Tail {
    /// What the statements touch, a closure they make counting as run.
    pub(crate) touch: crate::emit::Touch,
    /// The `let` is the optional an `if let` tests, read only by that test
    /// and by the copies at the top of its branch (see [`copied_at_test`]).
    pub(crate) tested: bool,
    /// Every read of the `let` works on a `cpl::Lent` (see [`Emitter::lendable_reads`]).
    pub(crate) lendable: bool,
    /// Locals read whole, not only through an index, field or length: the
    /// C++ may move them, so nothing may hold a reference into them.
    pub(crate) whole: std::collections::BTreeSet<usize>,
}

/// The map and key of `m.get(k)`: the one read whose found value is an
/// entry the C++ can point into (`cpl::map_lend`) instead of copying, so
/// every lending and moving choice about map entries starts here.
pub(crate) fn entry_read(e: &Expr) -> Option<(&Expr, &Expr)> {
    match &e.kind {
        ExprKind::Builtin(cpl_ir::Builtin::MapGet, args) => match args.as_slice() {
            [map, key] => Some((map, key)),
            _ => None,
        },
        _ => None,
    }
}

/// Locals `block` reads whole, not only through an index, field, variant
/// payload or tag, length or the elements a `for` walks.
pub(crate) fn whole_reads(block: &Block) -> std::collections::BTreeSet<usize> {
    let (mut uses, mut parts) = (Vec::new(), Vec::new());
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Local(s) => uses.push(*s),
        ExprKind::Index(..)
        | ExprKind::Field(..)
        | ExprKind::Payload(..)
        | ExprKind::IsVariant(..)
        | ExprKind::Builtin(cpl_ir::Builtin::Len, _) => {
            let base = match &e.kind {
                ExprKind::Builtin(_, args) if args.len() == 1 => &args[0],
                ExprKind::Index(base, _)
                | ExprKind::Field(base, _)
                | ExprKind::Payload(base, ..)
                | ExprKind::IsVariant(base, _) => &**base,
                _ => return,
            };
            if let ExprKind::Local(s) = base.kind {
                parts.push(s);
            }
        }
        _ => {}
    });
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::For(_, _, cpl_ir::Iter::Seq(seq, _), _) = stmt {
            if let ExprKind::Local(s) = seq.kind {
                parts.push(s);
            }
        }
    });
    for s in parts {
        if let Some(at) = uses.iter().position(|&u| u == s) {
            uses.swap_remove(at);
        }
    }
    uses.into_iter().collect()
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
    let mut seen = Touch::of_block(&later).closures;
    walk::deep_exprs(&later, &mut |e| seen |= local(e));
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

/// The locals `block` gives, when each give is a heap `let` of the block.
fn gives_own_locals(block: &cpl_ir::Block, ty: &Ty) -> Option<Vec<usize>> {
    if !matches!(
        ty,
        Ty::List(_) | Ty::Array { .. } | Ty::Str | Ty::Map(..) | Ty::HashMap(..)
    ) {
        return None;
    }
    let (mut own, mut out) = (Vec::new(), Vec::new());
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, _) => own.push(*slot),
        Stmt::Give(value) => out.push(match &value.kind {
            ExprKind::Local(slot) => Some(*slot),
            ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => match of.first() {
                Some(Expr {
                    kind: ExprKind::Local(slot),
                    ..
                }) => Some(*slot),
                _ => None,
            },
            _ => None,
        }),
        _ => {}
    });
    let all = out.iter().all(|g| g.is_some_and(|s| own.contains(&s)));
    (!out.is_empty() && all).then(|| out.into_iter().flatten().collect())
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
        // A block that gives its own `let` of a place, `{ const auto& t =
        // a[r]; D_len = t.size(); t }`, returns a reference when that `let`
        // was bound as one.
        let own = (!by_ref).then(|| gives_own_locals(block, ty)).flatten();
        let placeholder = format!("@ret{:p}@", block);
        let header = match &own {
            Some(slots) => {
                for slot in slots {
                    self.ref_bound.remove(slot);
                }
                placeholder.clone()
            }
            None => ret.clone(),
        };
        let outer = std::mem::replace(&mut self.give_ref, by_ref || own.is_some());
        let mut code = self.lambda(&header, |this, out| this.block(block, out));
        self.give_ref = outer;
        if let Some(slots) = own {
            let lent = slots.iter().all(|s| self.ref_bound.contains(s));
            let ret = if lent { "decltype(auto)" } else { &ret };
            code.text = code.text.replacen(&placeholder, ret, 1);
        }
        code
    }

    /// Marks a `let f = closure` that may borrow its captures, and records
    /// why each capture that may not owns its copy (see
    /// [`Emitter::named_closure_owners`]).
    pub(crate) fn borrow_if_named_closure(&mut self, slot: usize, init: Option<&Expr>) {
        if let Some(ExprKind::Closure(cid)) = init.map(|e| &e.kind) {
            if let Some(owners) = self.named_closure_owners(slot, *cid) {
                self.borrow_closures.insert(*cid);
                for (outer, why) in owners {
                    self.capture_owns.insert((*cid, outer), why);
                }
            }
        }
    }

    /// Whether the `let f = closure` of `slot` may borrow its heap captures:
    /// it is only ever called. Then gives the captures that still own a
    /// copy, each with its reason: one its fn changes, or reads whole (which
    /// could move it away).
    pub(crate) fn named_closure_owners(
        &self,
        slot: usize,
        cid: usize,
    ) -> Option<Vec<(usize, &'static str)>> {
        if self.closure_id.is_some() {
            return None;
        }
        let program = self.program;
        let blocks: Vec<&Block> = match self.fn_id {
            Some(id) => vec![&program.fns[id].body],
            None => vec![&program.setup, &program.main],
        };
        if self.facts.stores((self.fn_id, false, slot)) > 1
            || !blocks.iter().all(|b| only_called(program, b, slot, 3))
        {
            return None;
        }
        let home = self
            .fn_id
            .map_or(crate::uses::Home::Top, crate::uses::Home::Fn);
        let sunk = self.fn_id.map(|id| &self.sunk[id]);
        let def = &program.closures[cid];
        let owners = def
            .captures
            .iter()
            .filter(|c| !c.is_global)
            .filter_map(|c| {
                let o = c.outer;
                let found = self.uses.of_binding(home, o);
                if found.mutated {
                    Some((o, CAPTURE_SET))
                } else if (found.moved && blocks.iter().any(|b| moved_while_live(b, slot, o)))
                    || sunk.is_some_and(|s| s.contains(&o))
                {
                    Some((o, CAPTURE_MOVED))
                } else {
                    None
                }
            });
        Some(owners.collect())
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
        let mut marked = Vec::new();
        for &value in operands {
            let ExprKind::Binary(BinOp::Coalesce, got, _) = &value.kind else {
                continue;
            };
            let Some(root) = entry_read(got).and_then(|(map, _)| crate::stmt::place_root(map))
            else {
                continue;
            };
            let callee_writes =
                callee.is_some_and(|f| root.0 && self.uses.effects(f).writes.contains(&root.1));
            if value.ty.is_scalar()
                || got.ty == value.ty
                || muts.contains(&root)
                || callee_writes
                || !operands.iter().all(|e| self.leaves(e, root))
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
        let (map, key) = entry_read(init)?;
        let pattern_temp = self.locals()[slot].name.is_empty();
        let named = tail.is_some_and(|tail| tail.lendable);
        let (Ty::Map(_, value) | Ty::HashMap(_, value)) = &map.ty else {
            return None;
        };
        ((pattern_temp || named) && self.lends_entry(slot, map, value, tail)).then_some([map, key])
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
        let Some((map, key)) = entry_read(got) else {
            return false;
        };
        if got.ty == init.ty || !self.lends_entry(slot, map, &init.ty, tail) {
            return false;
        }
        self.need("map_lend");
        let (m, k) = (self.expr(map).text, self.expr(key).text);
        let entry = self.temp();
        self.line(
            out,
            &format!("const auto {entry} = cpl::map_lend({m}, {k});"),
        );
        let other = if self.lends(slot, default, tail) {
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
        let lent = arms.map(|arm| self.lends(slot, arm, tail));
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
            _ => {
                let (map, key) = entry_read(got)?;
                (self.closure_id.is_none()
                    && !seq.ty.is_scalar()
                    && got.ty != seq.ty
                    && crate::stmt::place_root(map).is_some())
                .then_some((Lender::Get(map, key), &**default))
            }
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

impl Emitter<'_> {
    /// `m[k] = (m.get(k) ?? d) + r` copies the entry only to store the
    /// longer list back. When nothing else in the value can see `m`, the
    /// entry is moved out instead (`cpl::map_move_or`) and the store
    /// puts it back.
    pub(crate) fn mark_moved_entry(&mut self, place: &cpl_ir::Place, op: AssignOp, value: &Expr) {
        self.mark_overwritten_entry(place, op, value);
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
        let Some((map, got_key)) = entry_read(got) else {
            return;
        };
        let rest_ok = rest.iter().all(|s| match s {
            Stmt::Expr(e) | Stmt::Give(e) => !may_touch(e, root),
            _ => false,
        });
        if matches!(copy.ty, Ty::List(_))
            && matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_))
            && walk::path_root(map) == Some(root)
            && (same_path(got_key, key) || self.key_temp_of(key, got_key))
            && !may_touch(key, root)
            && !may_touch(d, root)
            && rest_ok
            && self.expr_elem_layout(&of[0]).is_none()
        {
            self.moved_entry.insert(copy as *const Expr as usize);
        }
    }

    /// `m[k] = f(m.get(k) ?? d)`: the store overwrites the entry its value
    /// reads, so that read may move the entry out instead of copying it
    /// when nothing else in the value reaches `m`: no other read, no store,
    /// and no code that may write it. A `mut` parameter is such a map even
    /// under a call of an unknown fn value, as mutation XOR access leaves
    /// that fn no other name for it; anything else asks [`Uses::leaves`].
    fn mark_overwritten_entry(&mut self, place: &cpl_ir::Place, op: AssignOp, value: &Expr) {
        let (AssignOp::Set, PlaceKind::Index(base, keys)) = (op, &place.kind) else {
            return;
        };
        let ([key], PlaceKind::Local(_) | PlaceKind::Global(_)) = (keys.as_slice(), &base.kind)
        else {
            return;
        };
        let root = walk::root(base);
        let (mut found, mut mentions) = (None, 0);
        walk::deep_expr(value, &mut |e| match &e.kind {
            ExprKind::Local(s) if root == (false, *s) => mentions += 1,
            ExprKind::Global(g) if root == (true, *g) => mentions += 1,
            ExprKind::Binary(BinOp::Coalesce, got, _) if !e.ty.is_scalar() => {
                if let Some((map, got_key)) = entry_read(got) {
                    if matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_))
                        && walk::path_root(map) == Some(root)
                        && got.ty != e.ty
                        && (same_path(got_key, key) || self.key_temp_of(key, got_key))
                    {
                        found = Some(e as *const Expr as usize);
                    }
                }
            }
            _ => {}
        });
        let Some(at) = found else {
            return;
        };
        let stored = Touch::of_expr(value).writes.contains(&root);
        let exclusive = !root.0
            && self.closure_id.is_none()
            && self
                .fn_id
                .and_then(|id| self.program.fns[id].params.get(root.1))
                .is_some_and(|p| p.mutable);
        if mentions == 1 && !stored && (exclusive || self.leaves(value, root)) {
            self.moved_entry.insert(at);
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

/// Whether a fn's parameter holding a fn is only ever called, so a closure
/// passed to it may borrow what it captures.
pub(crate) fn fn_param_only_called(program: &cpl_ir::Program, fn_id: usize, slot: usize) -> bool {
    fn_param_only_called_at(program, fn_id, slot, 3)
}

/// Whether `slot` in `body` is only called: never stored, passed `mut`,
/// or captured by a closure that may keep it. A closure that captures it
/// and is itself passed where it is only called (`first_true(m =>
/// ok(m))`), or passed on itself (`bag.first_true(ok)`), counts as a call.
/// Whether, in the block of `body` that declares the closure `slot`, a
/// statement from that `let` to the closure's last mention reads `capture`
/// whole (which may move it). A move before the closure is made or after
/// its last call leaves a borrowed capture alone.
fn moved_while_live(body: &Block, slot: usize, capture: usize) -> bool {
    let mut home = None;
    walk::blocks(body, &mut |b| {
        if b.stmts
            .iter()
            .any(|st| matches!(st, Stmt::Let(s, _) if *s == slot))
        {
            home = Some(b);
        }
    });
    let Some(block) = home else {
        let mut named = false;
        walk::deep_exprs(body, &mut |e| {
            named |= matches!(e.kind, ExprKind::Local(s) if s == slot)
        });
        return named;
    };
    let names = |stmt: &Stmt| {
        let mut found = matches!(stmt, Stmt::Let(s, _) if *s == slot);
        walk::deep_exprs(
            &Block {
                stmts: vec![stmt.clone()],
            },
            &mut |e| found |= matches!(e.kind, ExprKind::Local(s) if s == slot),
        );
        found
    };
    let at: Vec<usize> = (0..block.stmts.len())
        .filter(|&k| names(&block.stmts[k]))
        .collect();
    let (Some(&first), Some(&last)) = (at.first(), at.last()) else {
        return true;
    };
    whole_reads(&Block {
        stmts: block.stmts[first..=last].to_vec(),
    })
    .contains(&capture)
}

pub(crate) fn only_called(program: &cpl_ir::Program, body: &Block, slot: usize, depth: u8) -> bool {
    let (mut uses, mut calls, mut escapes) = (0usize, 0usize, false);
    let mut passed_on = std::collections::BTreeSet::new();
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Assign(place, _, _) = stmt {
            escapes |= walk::root(place) == (false, slot);
        }
    });
    let lent = |args: &[Arg]| {
        args.iter()
            .any(|a| matches!(a, Arg::Mut(place) if walk::root(place) == (false, slot)))
    };
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => uses += 1,
        ExprKind::CallExpr(callee, args) => {
            calls += usize::from(matches!(callee.kind, ExprKind::Local(s) if s == slot));
            escapes |= lent(args);
        }
        ExprKind::Call(f, args) => {
            escapes |= lent(args);
            for (j, arg) in args.iter().enumerate() {
                let Arg::Value(value) = arg else { continue };
                let onward = || depth > 0 && fn_param_only_called_at(program, *f, j, depth - 1);
                match value.kind {
                    ExprKind::Closure(cid) if onward() => _ = passed_on.insert(cid),
                    ExprKind::Local(s) if s == slot && onward() => calls += 1,
                    _ => {}
                }
            }
        }
        ExprKind::Closure(cid) => {
            let def = &program.closures[*cid];
            for cap in def
                .captures
                .iter()
                .filter(|c| !c.is_global && c.outer == slot)
            {
                let held = passed_on.contains(cid)
                    || closure_name(body, *cid).is_some_and(|s| {
                        s == slot || (depth > 0 && only_called(program, body, s, depth - 1))
                    });
                escapes |=
                    !held || !only_called(program, &def.body, cap.inner, depth.saturating_sub(1));
            }
        }
        _ => {}
    });
    !escapes && uses == calls
}

/// The slot of the `let f = closure` that names closure `cid` in `body`.
fn closure_name(body: &Block, cid: usize) -> Option<usize> {
    let mut name = None;
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Let(s, Some(init)) = stmt {
            if matches!(init.kind, ExprKind::Closure(c) if c == cid) {
                name = Some(*s);
            }
        }
    });
    name
}

fn fn_param_only_called_at(
    program: &cpl_ir::Program,
    fn_id: usize,
    slot: usize,
    depth: u8,
) -> bool {
    let f = &program.fns[fn_id];
    slot < f.params.len() && !f.params[slot].mutable && only_called(program, &f.body, slot, depth)
}

/// The C++ naming map entry `e` (`it.` or `it->`) as loop variable `var`:
/// an entry holding a list is a pair of names for it, not a copy.
pub(crate) fn entry_binding(ty: &Ty, cpp: &str, var: &str, e: &str) -> String {
    match matches!(ty, Ty::Tuple(parts) if parts.iter().any(|p| !p.is_scalar())) {
        true => format!("auto {var} = std::tie({e}first, {e}second);"),
        false => format!("{cpp} {var}({e}first, {e}second);"),
    }
}

/// A non-scalar value the C++ copies: a `let` of a place (at the span of
/// what it reads) or a closure's by-value capture (closure id, name).
pub(crate) enum Copied {
    Let(cpl_diag::Span, &'static str),
    Capture(usize, String, &'static str),
}

/// Whether `e` reads part of a named value: `a.f`, `xs[i]!`. Part of a
/// call's result (`xs.pop()!`, `s.parse()!.0`) is a temporary the C++
/// moves from, so binding it copies nothing.
fn rooted_in_a_name(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Field(base, _)
        | ExprKind::Payload(base, ..)
        | ExprKind::Index(base, _)
        | ExprKind::Unwrap(base, _) => rooted_in_a_name(base),
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        _ => false,
    }
}

const CAPTURE_SET: &str =
    "the captured name is changed while the closure may still run, so it owns a copy";
const CAPTURE_MOVED: &str = "the captured name is moved later, so the closure owns a copy";
pub(crate) const KEPT: &str = "the closure may be kept (stored, returned or passed where it is kept), so it owns what it reads";

impl Emitter<'_> {
    /// Records the copy naming operand `e` before a later one runs makes,
    /// when `e` is a place holding a heap value: `up[find(u)]` with a
    /// `find` that writes `up` reads `up` as it was before the call.
    pub(crate) fn note_operand_copy(&mut self, e: &Expr) {
        let place = matches!(
            e.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
                | ExprKind::Unwrap(..)
        );
        if place && holds_heap(self.program, &e.ty) {
            self.copies.push(Copied::Let(
                e.span,
                "a later operand may change it, and operands are read left to right",
            ));
        }
    }

    /// Records the copy a `let slot = init` emitted as `T name = text;`
    /// makes, when `init` names a place holding a heap value.
    /// A copy the contestant wrote as a method on a heap place,
    /// `a.clone()` or `a.sorted()`, not one a generated constructor makes
    /// of its parameter at the call's span.
    pub(crate) fn note_written_copy(&mut self, e: &Expr, args: &[Expr], why: &'static str) {
        let [arg, ..] = args else { return };
        let place = matches!(
            arg.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
        );
        let written = arg.span.start >= e.span.start && arg.span.end < e.span.end;
        if place && written && holds_heap(self.program, &arg.ty) {
            self.copies.push(Copied::Let(e.span, why));
        }
    }

    pub(crate) fn note_let_copy(
        &mut self,
        slot: usize,
        init: &Expr,
        text: &str,
        tail: Option<&Tail>,
    ) {
        // A conversion the C++ spells as its argument (`sorted`'s list
        // to sort in place) copies that argument.
        let mut init = init;
        while let ExprKind::Builtin(_, args) = &init.kind {
            match args.as_slice() {
                [arg]
                    if text
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.') =>
                {
                    init = arg
                }
                _ => break,
            }
        }
        let place = matches!(
            init.kind,
            ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
                | ExprKind::Unwrap(..)
        ) && rooted_in_a_name(init);
        // A temporary bound by reference names the place it was bound to.
        let whole = matches!(init.kind, ExprKind::Local(s)
                if !self.locals()[s].name.is_empty() || self.ref_bound.contains(&s))
            || matches!(init.kind, ExprKind::Global(_));
        if !holds_heap(self.program, &init.ty) || !(place || whole) || moved(text) {
            return;
        }
        let why = self.copy_reason(slot, init, tail, whole);
        self.copies.push(Copied::Let(init.span, why));
    }

    /// Records a closure capturing the heap value `name` by value.
    pub(crate) fn note_capture_copy(
        &mut self,
        cid: usize,
        heap: bool,
        name: &str,
        why: &'static str,
    ) {
        if heap {
            self.copies
                .push(Copied::Capture(cid, name.to_string(), why));
        }
    }
}

/// Each copy as the span of its source, what is copied and why.
/// True when a value of `ty` owns heap memory, so copying it is a deep
/// copy `cpl explain` lists; a `Mod` or a tuple of ints copies its bytes.
/// Whether the C++ initialiser moves its value: `std::move(t)`, or a part
/// of a moved tuple, `std::get<0>(std::move(t))`.
fn moved(text: &str) -> bool {
    let part = text
        .strip_prefix("std::get<")
        .and_then(|rest| rest.split_once(">("))
        .map_or(text, |(_, inner)| inner);
    part.starts_with("std::move(")
}

pub(crate) fn holds_heap(program: &cpl_ir::Program, ty: &Ty) -> bool {
    fn go(program: &cpl_ir::Program, ty: &Ty, depth: u32) -> bool {
        let inner = |t: &Ty| depth < 16 && go(program, t, depth + 1);
        match ty {
            Ty::Str
            | Ty::Array { .. }
            | Ty::List(_)
            | Ty::Keyed(..)
            | Ty::Map(..)
            | Ty::HashMap(..)
            | Ty::Set(_)
            | Ty::HashSet(_) => true,
            Ty::Opt(t) | Ty::Mut(t) => inner(t),
            Ty::Tuple(parts) => parts.iter().any(inner),
            Ty::Record(fields) => fields.iter().any(|(_, t)| inner(t)),
            Ty::Struct(id) => program.structs[*id].fields.iter().any(|(_, t)| inner(t)),
            Ty::Enum(id) => program.enums[*id]
                .variants
                .iter()
                .any(|(_, payload)| payload.iter().any(inner)),
            _ => false,
        }
    }
    go(program, ty, 0)
}

pub(crate) fn copy_sites(
    program: &cpl_ir::Program,
    copies: &[Copied],
) -> Vec<(cpl_diag::Span, String, &'static str)> {
    let mut at = std::collections::BTreeMap::new();
    let mut visit = |e: &Expr| {
        if let ExprKind::Closure(cid) = e.kind {
            at.entry(cid).or_insert(e.span);
        }
    };
    cpl_ir::walk::deep_exprs(&program.setup, &mut visit);
    cpl_ir::walk::deep_exprs(&program.main, &mut visit);
    for f in &program.fns {
        cpl_ir::walk::deep_exprs(&f.body, &mut visit);
    }
    for c in &program.closures {
        cpl_ir::walk::deep_exprs(&c.body, &mut visit);
    }
    let mut sites: Vec<_> = copies
        .iter()
        .filter_map(|c| match c {
            Copied::Let(span, why) => Some((*span, String::new(), *why)),
            Copied::Capture(cid, name, why) => {
                Some((*at.get(cid)?, format!("`{name}` captured by value"), *why))
            }
        })
        .collect();
    sites.sort_by_key(|(span, what, _)| (span.start, what.clone()));
    sites.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    sites
}

impl Emitter<'_> {
    /// Statement `k` of `block`, moving each variable it reads for the
    /// last time instead of copying it.
    pub(crate) fn stmt_owning(&mut self, block: &Block, k: usize, out: &mut String) {
        self.moves_last = crate::uses::last_read(self.program, block, k)
            || self.takes_sunk_param(block, k)
            || self.dies_here(&block.stmts[k]);
        self.stmt_presized(&block.stmts, k, out);
        self.moves_last = false;
    }

    /// A `let` an enclosing statement found reads its source for the last
    /// time, unless that source is itself a reference or the `let` lends.
    pub(crate) fn dies_here(&self, stmt: &Stmt) -> bool {
        let Stmt::Let(slot, Some(init)) = stmt else {
            return false;
        };
        let at = stmt as *const Stmt;
        self.uses.dying.contains(&at)
            && !matches!(crate::uses::in_place(init).kind,
                ExprKind::Local(src) if self.ref_bound.contains(&src))
            && !self.lends(*slot, init, self.tail.as_ref())
    }

    /// `std::move(text)` for a `let` of a heap variable (or a field path on
    /// one) the statements after it set again before anything reads it:
    /// `let old_k = ks; ks = List.filled(..)` takes the list, not a copy.
    pub(crate) fn take_if_overwritten(&mut self, init: &Expr, text: String) -> String {
        let taken = std::mem::take(&mut self.moves_last);
        // A `const T&` name cannot be moved from; `std::move` of it only copies.
        let lent =
            path_of(init).is_some_and(|((global, s), _)| !global && self.ref_bound.contains(&s));
        match taken && !lent && !init.ty.is_scalar() && !text.starts_with("std::move(") {
            true => format!("std::move({text})"),
            false => text,
        }
    }
}

/// A variable and the fields read on it, outermost first: `self.ks`.
fn path_of(e: &Expr) -> Option<((bool, usize), Vec<usize>)> {
    match &e.kind {
        ExprKind::Local(s) => Some(((false, *s), Vec::new())),
        ExprKind::Global(g) => Some(((true, *g), Vec::new())),
        ExprKind::Field(base, f) => path_of(base).map(|(root, mut fields)| {
            fields.push(*f);
            (root, fields)
        }),
        _ => None,
    }
}

fn place_path(p: &Place) -> Option<((bool, usize), Vec<usize>)> {
    match &p.kind {
        PlaceKind::Local(s) => Some(((false, *s), Vec::new())),
        PlaceKind::Global(g) => Some(((true, *g), Vec::new())),
        PlaceKind::Field(base, f) => place_path(base).map(|(root, mut fields)| {
            fields.push(*f);
            (root, fields)
        }),
        _ => None,
    }
}

/// Whether `stmt` may read or change the variable path `path`, or leave
/// the block early: anything but `let`s and stores that stay off it.
fn touches(stmt: &Stmt, (root, fields): &((bool, usize), Vec<usize>)) -> bool {
    let block = Block {
        stmts: vec![stmt.clone()],
    };
    let opaque = Touch::of_block(&block).code();
    let (mut uses, mut elsewhere) = (0usize, 0usize);
    walk::deep_exprs(&block, &mut |e| match &e.kind {
        ExprKind::Local(s) if (false, *s) == *root => uses += 1,
        ExprKind::Global(g) if (true, *g) == *root => uses += 1,
        ExprKind::Field(base, f) if fields.first().is_some_and(|f0| f0 != f) => {
            elsewhere +=
                usize::from(path_of(base).is_some_and(|(r, fs)| r == *root && fs.is_empty()));
        }
        _ => {}
    });
    let stored = match stmt {
        Stmt::Let(..) | Stmt::Expr(_) => false,
        Stmt::Assign(place, _, _) => place_path(place)
            .map_or(walk::root(place) == *root, |(r, fs)| {
                r == *root && fs.iter().zip(fields).all(|(a, b)| a == b)
            }),
        _ => true,
    };
    opaque || stored || uses != elsewhere
}

/// Whether the variable path `init` names is set whole by one of the
/// statements after its `let` before any of them reads it, changes it or
/// leaves: the `let` may take its value instead of copying it.
pub(crate) fn overwritten_before_read(init: &Expr, rest: &[Stmt]) -> bool {
    let Some(path) = path_of(init) else {
        return false;
    };
    for stmt in rest {
        if let Stmt::Assign(place, AssignOp::Set, value) = stmt {
            if place_path(place).as_ref() == Some(&path) {
                let value = Stmt::Expr(value.clone());
                return !touches(&value, &path);
            }
        }
        if touches(stmt, &path) {
            return false;
        }
    }
    false
}
