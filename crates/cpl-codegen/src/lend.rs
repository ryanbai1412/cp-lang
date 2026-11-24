//! `let xs = if c: f(a) else: a[i]`: the branch that names a place is read
//! in place, so only the other one builds a value. And a view that lends
//! from a temporary, `let v = chain([1, 3, 2], r).take(n)`, keeps the
//! temporary alive as long as the view.

use crate::emit::{Code, Emitter, COND};
use crate::uses::effects::{may_touch, Touch};
use crate::walk;
use cpl_ir::{Arg, AssignOp, BinOp, Block, Expr, ExprKind, Place, PlaceKind, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

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
    whole_reads_by(block, &|_, _| false)
}

/// [`whole_reads`], not counting a local passed as argument `i` to fn `f`
/// when `by_ref(f, i)`: the callee takes it by const reference, so the
/// C++ cannot move it there.
fn whole_reads_by(
    block: &Block,
    by_ref: &dyn Fn(usize, usize) -> bool,
) -> std::collections::BTreeSet<usize> {
    let (mut uses, mut parts) = (Vec::new(), Vec::new());
    crate::uses::places::each_local(block, &mut |s| uses.push(s));
    crate::uses::places::each_call(block, &mut |f, args| {
        for (i, a) in args.iter().enumerate() {
            if let cpl_ir::Arg::Value(Expr {
                kind: ExprKind::Local(s),
                ..
            }) = a
            {
                if by_ref(f, i) {
                    parts.push(*s);
                }
            }
        }
    });
    crate::uses::places::each_part_base(block, &mut |base| {
        if let ExprKind::Local(s) = base.kind {
            parts.push(s);
        }
    });
    crate::uses::loops::each_loop(block, &mut |_, head| {
        if let crate::uses::loops::Loop::For(_, cpl_ir::Iter::Seq(seq, _)) = head {
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
    crate::uses::places::each_local(&later, &mut |s| seen |= s == slot);
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
    // The variable `e` names or is a part of (`sa.sa`, `rows[i]`), `usize::MAX`
    // for a global; a list given as an array over a domain is the same C++ value.
    fn variable(e: &Expr) -> Option<usize> {
        match &e.kind {
            ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => variable(of.first()?),
            _ => match crate::stmt::place_root(e)? {
                (true, _) => Some(usize::MAX),
                (false, slot) => Some(slot),
            },
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
    crate::uses::places::each_declared(block, &mut |slot| own.push(slot));
    crate::uses::split::each_give(block, &mut |value| {
        out.push(match &value.kind {
            ExprKind::Local(slot) => Some(*slot),
            ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => match of.first() {
                Some(Expr {
                    kind: ExprKind::Local(slot),
                    ..
                }) => Some(*slot),
                _ => None,
            },
            _ => None,
        })
    });
    let all = out.iter().all(|g| g.is_some_and(|s| own.contains(&s)));
    (!out.is_empty() && all).then(|| out.into_iter().flatten().collect())
}

/// What each `return` of a fn body gives, with a conditional's two arms
/// as two (`c ? self.a : self.b` is a place only when both arms are), or
/// `None` when a `return` gives nothing. The returns of a fn that
/// returns `const T&` must each be a place that outlives the call.
pub(crate) fn returned_places(body: &Block) -> Option<Vec<&Expr>> {
    fn arms<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
        match &e.kind {
            ExprKind::If(_, a, b) => {
                arms(a, out);
                arms(b, out);
            }
            _ => out.push(e),
        }
    }
    let (mut out, mut bare) = (Vec::new(), false);
    crate::uses::places::each_return(body, &mut |value| match value {
        Some(e) => arms(e, &mut out),
        None => bare = true,
    });
    (!bare && !out.is_empty()).then_some(out)
}

/// The variable `value` reads in place: a path, a `Do` block that gives
/// one (see [`gives_outer_place`]), or what a fn in `lent_roots` lends
/// from its argument (`seg[a].l` through `Arena.get`).
pub(crate) fn given_root(
    value: &Expr,
    lent_roots: &BTreeMap<cpl_ir::FnId, usize>,
) -> Option<(bool, usize)> {
    match &value.kind {
        ExprKind::Do(block) if gives_outer_place(block, &value.ty) => {
            let mut root = None;
            crate::uses::split::each_give(block, &mut |g| {
                root = root.or_else(|| given_root(g, lent_roots));
            });
            root
        }
        ExprKind::Builtin(cpl_ir::Builtin::ListToArray, of) => given_root(of.first()?, lent_roots),
        _ => storage_root(value, lent_roots),
    }
}

/// The fields from its variable down to the storage `e` reads, an index
/// as `None`, when `e` is a path; `self.act!` is `[act]`.
fn expr_fields(e: &Expr) -> Option<Vec<Option<usize>>> {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => Some(Vec::new()),
        ExprKind::Unwrap(base, _) => expr_fields(base),
        ExprKind::Field(base, i) => expr_fields(base).map(|mut p| {
            p.push(Some(*i));
            p
        }),
        ExprKind::Index(base, _) => expr_fields(base).map(|mut p| {
            p.push(None);
            p
        }),
        _ => None,
    }
}

fn place_fields(place: &cpl_ir::Place) -> Vec<Option<usize>> {
    use cpl_ir::PlaceKind;
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => Vec::new(),
        PlaceKind::Some(base) => place_fields(base),
        PlaceKind::Field(base, i) => {
            let mut p = place_fields(base);
            p.push(Some(*i));
            p
        }
        PlaceKind::Index(base, _) => {
            let mut p = place_fields(base);
            p.push(None);
            p
        }
    }
}

/// Whether `value` and `place`, of one variable, are apart: they part at
/// two different fields, `self.act` and `self.lz[p]`.
pub(crate) fn apart(value: &Expr, place: &cpl_ir::Place) -> bool {
    let Some(read) = expr_fields(value) else {
        return false;
    };
    (read.iter().zip(place_fields(place)))
        .any(|(a, b)| matches!((a, b), (Some(a), Some(b)) if *a != b))
}

/// The variable whose storage a place lives in, whatever its indices
/// compute: `self.keys[i * 2]` is in `self`.
fn storage_root(e: &Expr, lent_roots: &BTreeMap<cpl_ir::FnId, usize>) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Field(base, _)
        | ExprKind::Index(base, _)
        | ExprKind::Unwrap(base, _)
        | ExprKind::Payload(base, ..) => storage_root(base, lent_roots),
        ExprKind::Call(callee, args) => match lent_roots.get(callee).map(|&s| &args[s]) {
            Some(cpl_ir::Arg::Value(arg)) => given_root(arg, lent_roots),
            _ => None,
        },
        _ => walk::path_root(e),
    }
}

impl<'p> Emitter<'p> {
    /// A `Do` block as a lambda called on the spot. One that gives a
    /// variable from outside it, `{ D_len = xs.len; xs }`, returns a
    /// reference to it, never a copy.
    pub(crate) fn do_lambda(&mut self, block: &cpl_ir::Block, ty: &Ty, ret: String) -> Code {
        let by_ref = gives_outer_place(block, ty) && self.keeps_given_roots(block);
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
        let owned = (!by_ref && own.is_none()).then(|| crate::stmt::block_lets(block));
        let outer_owned = std::mem::replace(&mut self.give_owned, owned);
        let mut code = self.lambda(&header, |this, out| this.block(block, out));
        self.give_ref = outer;
        self.give_owned = outer_owned;
        if let Some(slots) = own {
            let lent = slots.iter().all(|s| self.ref_bound.contains(s));
            let ret = if lent { "decltype(auto)" } else { &ret };
            code.text = code.text.replacen(&placeholder, ret, 1);
        }
        code
    }

    /// Whether running `block` leaves the variables it gives parts of as
    /// they were, so the reference it returns still names what it gave.
    fn keeps_given_roots(&self, block: &cpl_ir::Block) -> bool {
        let mut roots = Vec::new();
        crate::uses::split::each_give(block, &mut |g| {
            roots.push(given_root(g, &self.lent_roots));
        });
        let touch = self.block_touch(block);
        roots
            .into_iter()
            .all(|r| r.is_some_and(|r| !touch.opaque && !touch.writes.contains(&r)))
    }

    /// Marks a `let f = closure` that may borrow its captures, and records
    /// why each capture that may not owns its copy (see
    /// [`Emitter::named_closure_owners`]).
    pub(crate) fn borrow_if_named_closure(&mut self, slot: usize, init: Option<&Expr>) {
        if let Some(ExprKind::Closure(cid)) = init.map(|e| &e.kind) {
            if let Some(owners) = self.named_closure_owners(slot, *cid) {
                self.borrow_closures.insert(*cid);
                if self.only_a_sort_key(slot) {
                    // Held for one sort, which only calls it: it lives only
                    // through those calls, as a key written in place does.
                    self.call_scoped.insert(*cid);
                }
                for (outer, why) in owners {
                    self.capture_owns.insert((*cid, outer), why);
                }
            }
        }
    }

    /// Whether `slot` is read once, as the key fn of a sort.
    fn only_a_sort_key(&self, slot: usize) -> bool {
        let program = self.program;
        let blocks: Vec<&Block> = match (self.closure_id, self.fn_id) {
            (Some(c), _) => vec![&program.closures[c].body],
            (None, Some(id)) => vec![&program.fns[id].body],
            (None, None) => vec![&program.setup, &program.main],
        };
        let (mut uses, mut keys) = (0usize, 0usize);
        for block in blocks {
            crate::uses::places::each_local(block, &mut |s| uses += usize::from(s == slot));
            crate::uses::places::each_sort_key(block, &mut |func| {
                keys += usize::from(matches!(func.kind, ExprKind::Local(s) if s == slot));
            });
            crate::uses::places::each_closure(block, &mut |cid| {
                let caps = &program.closures[cid].captures;
                uses += caps
                    .iter()
                    .filter(|c| !c.is_global && c.outer == slot)
                    .count();
            });
        }
        uses == 1 && keys == 1
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
        use crate::uses::Home;
        let program = self.program;
        let (blocks, home): (Vec<&Block>, _) = match (self.closure_id, self.fn_id) {
            (Some(c), _) => (vec![&program.closures[c].body], Home::Closure(c)),
            (None, Some(id)) => (vec![&program.fns[id].body], Home::Fn(id)),
            (None, None) => (vec![&program.setup, &program.main], Home::Top),
        };
        if self.facts.stores(self.facts_local(slot)) > 1
            || !blocks.iter().all(|b| only_called(program, b, slot, 3))
        {
            return None;
        }
        let sunk = self
            .fn_id
            .filter(|_| self.closure_id.is_none())
            .map(|id| &self.sunk[id]);
        let by_ref = |f: usize, i: usize| {
            program.fns[f].params.get(i).is_some_and(|p| !p.mutable)
                && !self.sunk.get(f).is_some_and(|s| s.contains(&i))
        };
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
                } else if (found.moved
                    && blocks.iter().any(|b| moved_while_live(b, slot, o, &by_ref)))
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
        if !(e.ty.holds_view() || self.lent_fields.holds_lent(&e.ty)) {
            return;
        }
        let (callee, args) = match &e.kind {
            ExprKind::Call(g, args) => (Some(*g), args),
            ExprKind::Tuple(parts) | ExprKind::Variant(_, parts) => {
                for part in parts {
                    self.pin_owners(part, hold, out);
                }
                return;
            }
            ExprKind::Some(inner) => return self.pin_owners(inner, hold, out),
            _ => return,
        };
        for (k, arg) in args.iter().enumerate() {
            let Arg::Value(v) = arg else {
                continue;
            };
            // A lent struct a lent field points to is an owner too, after its own.
            let lent_owner = walk::path_root(v).is_none()
                && self.lent_fields.holds_lent(&v.ty)
                && callee.is_some_and(|g| self.lent_fields.sources[g].contains(&k));
            if !temporary_owner(v) && !lent_owner {
                self.pin_owners(v, hold, out);
                continue;
            }
            if lent_owner {
                self.pin_owners(v, hold, out);
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
        crate::uses::places::each_local(&block, &mut |s| reads += usize::from(s == slot));
        crate::uses::places::each_tested(&block, &mut |x| lendable += usize::from(local(x)));
        crate::uses::places::each_coalesce(&block, &mut |whole, x| {
            lendable += usize::from(local(x) && x.ty != whole.ty);
        });
        crate::uses::split::each_let_run(&block, &mut |run| {
            if let Some((t, x)) = run {
                if local(x) && self.locals()[t].name.is_empty() {
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
        crate::uses::places::each_part(value, &mut |e| match &e.kind {
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
    pub(crate) fn key_temp_of(&self, key: &Expr, got: &Expr) -> bool {
        let ExprKind::Local(slot) = key.kind else {
            return false;
        };
        let v = (self.fn_id, false, slot);
        self.facts.stable(v) && self.facts.init(v).is_some_and(|init| same_path(init, got))
    }

    /// `m[k] ?? d` at `at`, marked by [`Self::mark_moved_entry`], as a move
    /// of the entry out of the map.
    pub(crate) fn moved_entry_code(&mut self, at: &Expr, of: &Expr) -> Option<Code> {
        let ExprKind::Binary(_, got, d) = &of.kind else {
            return None;
        };
        let ExprKind::Builtin(_, args) = &got.kind else {
            return None;
        };
        let [m, k, ..] = args.as_slice() else {
            return None;
        };
        if !self.moved_entry.remove(&(at as *const Expr as usize)) {
            return None;
        }
        self.need("map_move_or");
        let (m, k, d) = (self.expr(m).text, self.expr(k).text, self.expr(d).text);
        Some(Code::primary(format!("cpl::map_move_or({m}, {k}, {d})")))
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
fn moved_while_live(
    body: &Block,
    slot: usize,
    capture: usize,
    by_ref: &dyn Fn(usize, usize) -> bool,
) -> bool {
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
        crate::uses::places::each_local(body, &mut |s| named |= s == slot);
        return named;
    };
    let names = |stmt: &Stmt| {
        let mut found = matches!(stmt, Stmt::Let(s, _) if *s == slot);
        crate::uses::places::each_variable_in(stmt, &mut |v| found |= v == (false, slot));
        found
    };
    let at: Vec<usize> = (0..block.stmts.len())
        .filter(|&k| names(&block.stmts[k]))
        .collect();
    let (Some(&first), Some(&last)) = (at.first(), at.last()) else {
        return true;
    };
    whole_reads_by(
        &Block {
            stmts: block.stmts[first..=last].to_vec(),
        },
        by_ref,
    )
    .contains(&capture)
}

/// Whether `?fn` param `slot` of fn `fn_id` is only tested and its payload
/// only called: a closure passed in it lives no longer than the call.
pub(crate) fn opt_fn_param_only_called(
    program: &cpl_ir::Program,
    fn_id: usize,
    slot: usize,
) -> bool {
    let f = &program.fns[fn_id];
    slot < f.params.len()
        && !f.params[slot].mutable
        && matches!(f.locals[slot].ty, Ty::Opt(ref inner) if matches!(**inner, Ty::Fn(..)))
        && opt_only_called(program, &f.body, slot, 3)
}

/// Whether optional local `slot` of `body` is only tested (`IsSome`), bound
/// whole to another such local, or unwrapped into a name [`only_called`].
fn opt_only_called(program: &cpl_ir::Program, body: &Block, slot: usize, depth: u8) -> bool {
    let is_slot = |e: &cpl_ir::Expr| matches!(e.kind, ExprKind::Local(s) if s == slot);
    let (mut uses, mut ok, mut escapes) = (0usize, 0usize, false);
    crate::uses::places::each_put(body, &mut |at, init| match at {
        crate::uses::places::Put::Store(place, _) => escapes |= walk::root(place) == (false, slot),
        crate::uses::places::Put::Let(s) => match &init.kind {
            ExprKind::Local(_) if is_slot(init) => {
                ok += 1;
                escapes |= depth == 0 || !opt_only_called(program, body, s, depth - 1);
            }
            ExprKind::Unwrap(inner, _) if is_slot(inner) => {
                ok += 1;
                escapes |= !only_called(program, body, s, depth);
            }
            ExprKind::Binary(cpl_ir::BinOp::Coalesce, opt, default)
                if is_slot(opt) && plain_closure(program, default) =>
            {
                ok += 1;
                escapes |= !only_called(program, body, s, depth);
            }
            _ => {}
        },
    });
    crate::uses::places::each_local(body, &mut |s| uses += usize::from(s == slot));
    crate::uses::places::each_tested(body, &mut |inner| ok += usize::from(is_slot(inner)));
    crate::uses::places::each_closure(body, &mut |cid| {
        escapes |= program.closures[cid]
            .captures
            .iter()
            .any(|c| !c.is_global && c.outer == slot);
    });
    !escapes && uses == ok
}

/// Whether `e` is a closure that captures nothing: a plain fn.
pub(crate) fn plain_closure(program: &cpl_ir::Program, e: &cpl_ir::Expr) -> bool {
    matches!(e.kind, ExprKind::Closure(cid) if program.closures[cid].captures.is_empty())
}

/// The closures an optional argument holds: `Some(closure)`, through the
/// branches of an `if`.
pub(crate) fn optional_closures(value: &cpl_ir::Expr) -> Vec<usize> {
    match &value.kind {
        ExprKind::Some(inner) => match inner.kind {
            ExprKind::Closure(cid) => vec![cid],
            _ => Vec::new(),
        },
        ExprKind::If(_, then, otherwise) => {
            let mut cids = optional_closures(then);
            cids.extend(optional_closures(otherwise));
            cids
        }
        _ => Vec::new(),
    }
}

pub(crate) fn only_called(program: &cpl_ir::Program, body: &Block, slot: usize, depth: u8) -> bool {
    only_called_in(program, body, slot, depth, None)
}

/// [`only_called`] for the body of fn `this.0`, whose param `this.1` is
/// `slot`: handing it to the fn's own call as that param is a call too.
fn only_called_in(
    program: &cpl_ir::Program,
    body: &Block,
    slot: usize,
    depth: u8,
    this: Option<(usize, usize)>,
) -> bool {
    let (mut uses, mut calls, mut escapes) = (0usize, 0usize, false);
    let mut passed_on = std::collections::BTreeSet::new();
    // A closure bound by `let` is kept only as long as that name is.
    let mut bound = std::collections::BTreeMap::new();
    crate::uses::places::each_put(body, &mut |at, init| match at {
        crate::uses::places::Put::Store(place, _) => escapes |= walk::root(place) == (false, slot),
        crate::uses::places::Put::Let(s) => {
            if let ExprKind::Closure(cid) = init.kind {
                bound.insert(cid, s);
            }
        }
    });
    let lent = |args: &[Arg]| {
        args.iter()
            .any(|a| matches!(a, Arg::Mut(place) if walk::root(place) == (false, slot)))
    };
    crate::uses::places::each_local(body, &mut |s| uses += usize::from(s == slot));
    crate::uses::places::each_value_call(body, &mut |callee, args| {
        calls += usize::from(matches!(callee.kind, ExprKind::Local(s) if s == slot));
        escapes |= lent(args);
    });
    crate::uses::places::each_call(body, &mut |f, args| {
        escapes |= lent(args);
        for (j, arg) in args.iter().enumerate() {
            let Arg::Value(value) = arg else { continue };
            let onward = || depth > 0 && fn_param_only_called_at(program, f, j, depth - 1);
            match value.kind {
                ExprKind::Closure(cid) if this == Some((f, j)) || onward() => {
                    _ = passed_on.insert(cid)
                }
                ExprKind::Local(s) if s == slot && (this == Some((f, j)) || onward()) => calls += 1,
                _ => {}
            }
        }
    });
    crate::uses::places::each_sort_key(body, &mut |func| {
        calls += usize::from(matches!(func.kind, ExprKind::Local(s) if s == slot));
    });
    crate::uses::places::each_closure(body, &mut |cid| {
        let def = &program.closures[cid];
        for cap in def
            .captures
            .iter()
            .filter(|c| !c.is_global && c.outer == slot)
        {
            let held = passed_on.contains(&cid)
                || closure_name(body, cid).is_some_and(|s| {
                    s == slot || (depth > 0 && only_called(program, body, s, depth - 1))
                });
            escapes |=
                !held || !only_called(program, &def.body, cap.inner, depth.saturating_sub(1));
        }
    });
    !escapes && uses == calls
}

/// The slot of the `let f = closure` that names closure `cid` in `body`.
fn closure_name(body: &Block, cid: usize) -> Option<usize> {
    let mut name = None;
    crate::uses::split::each_let_run(body, &mut |run| {
        if let Some((s, init)) = run {
            if matches!(init.kind, ExprKind::Closure(c) if c == cid) {
                name = Some(s);
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
    slot < f.params.len()
        && !f.params[slot].mutable
        && only_called_in(program, &f.body, slot, depth, Some((fn_id, slot)))
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
/// what it reads, with the fn instance that makes it: each instance of a
/// generic fn is its own C++) or a closure's by-value capture (closure id, name).
pub(crate) enum Copied {
    Let(cpl_diag::Span, &'static str, Option<usize>),
    /// A copy in a library fn, at the span of the call that runs it, with
    /// the fn and the name it binds: two in one call are two copies.
    Library(cpl_diag::Span, String, &'static str),
    Capture(usize, String, &'static str),
}

/// C++ that names a place as is, `self.rows`, `sa.sa` or `adj[u]`.
fn is_place_text(text: &str) -> bool {
    let mut depth = 0;
    text.chars().all(|c| {
        depth += match c {
            '[' => 1,
            ']' => -1,
            _ => 0,
        };
        depth >= 0 && (depth > 0 || c.is_alphanumeric() || "_.]".contains(c))
    }) && depth == 0
        && !text.starts_with('[')
}

/// Whether stored text `text` names a place (`v`, `t[f(i)].xs`,
/// `cpl::at(ss, 0)`), not a call whose result is a value of its own.
fn names_a_place(text: &str) -> bool {
    if let Some(at) = text.strip_prefix("cpl::at(") {
        return !at.contains("::");
    }
    let mut depth = 0;
    text.chars().all(|c| {
        depth += match c {
            '[' => 1,
            ']' => -1,
            _ => 0,
        };
        depth > 0 || c != '('
    })
}

/// What tuple part `std::get<1>(marks[i])` reads from: `marks[i]`.
fn tuple_part_of(text: &str) -> &str {
    text.strip_prefix("std::get<")
        .and_then(|t| t.split_once(">("))
        .and_then(|(_, t)| t.strip_suffix(')'))
        .map_or(text, tuple_part_of)
}

/// Why `return self.f` copies.
const GIVEN: &str = "a part of a value that still holds it, given by value";

/// Why `p = q` of a place that still holds its value copies.
pub(crate) const SET_COPY: &str =
    "a store keeps its own value, and the place it reads still holds it";

/// Why `Some(xs)` of a place copies.
const SOME_COPY: &str = "an optional holds its own value";
pub(crate) const STATIC_COPY: &str =
    "a long literal is one shared static, so a value that keeps it copies it";

/// Whether `e` is a long literal, which the C++ names as its one static.
fn names_a_static(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Str(bytes) if bytes.len() > crate::expr::SHORT_STR)
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

/// What the field, element or payload path `e` reads from: `a.xs` in
/// `a.at(i).kids`.
fn part_base(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Field(base, _)
        | ExprKind::Payload(base, ..)
        | ExprKind::Index(base, _)
        | ExprKind::Unwrap(base, _) => part_base(base),
        // An inlined lend, `{ let n = a.xs[i]; n }`, names what it binds.
        ExprKind::Do(block) => match block.stmts.as_slice() {
            [.., Stmt::Let(k, Some(init)), Stmt::Give(g)] if matches!(g.kind, ExprKind::Local(s) if s == *k) => {
                part_base(init)
            }
            _ => e,
        },
        _ => e,
    }
}

const CAPTURE_SET: &str =
    "the captured name is changed while the closure may still run, so it owns a copy";
const CAPTURE_MOVED: &str = "the captured name is moved later, so the closure owns a copy";
pub(crate) const KEPT: &str = "the closure may be kept (stored, returned or passed where it is kept), so it owns what it reads";

impl Emitter<'_> {
    /// `return Some(xs)`, or `return if c: Some(xs) else: None`, for a
    /// local `xs` the fn or closure owns: the optional takes `xs` by move,
    /// as `return xs` would.
    pub(crate) fn moved_some(&mut self, value: &Expr) -> Option<String> {
        if let ExprKind::If(cond, then, otherwise) = &value.kind {
            let (a, b) = (self.moved_some(then), self.moved_some(otherwise));
            if a.is_none() && b.is_none() {
                return None;
            }
            let c = self.expr(cond).text;
            let a = a.unwrap_or_else(|| self.expr(then).text);
            let b = b.unwrap_or_else(|| self.expr(otherwise).text);
            return Some(format!("({c} ? {a} : {b})"));
        }
        let ExprKind::Some(inner) = &value.kind else {
            return None;
        };
        if matches!(inner.kind, ExprKind::Field(..) | ExprKind::Index(..)) {
            // `return Some(p.x0)`: the return is the owner's last use.
            if self.closure_id.is_some()
                || !holds_heap(self.program, &inner.ty)
                || !self.returned_item_dies(std::slice::from_ref(&**inner), 0)
            {
                return None;
            }
            let text = self.expr(inner).text;
            let text = self.moved_if(true, text);
            return Some(format!("{}({text})", self.ty(&value.ty)));
        }
        let ExprKind::Local(slot) = inner.kind else {
            return None;
        };
        let borrowed = match self.closure_id {
            Some(cid) => {
                let c = &self.program.closures[cid];
                slot < c.params.len() || c.captures.iter().any(|cap| cap.inner == slot)
            }
            None => self
                .fn_id
                .is_some_and(|id| slot < self.program.fns[id].params.len()),
        };
        if borrowed {
            return None;
        }
        self.some_moved(inner, &value.ty)
    }

    /// `Some(xs)` taking the local `xs` by move, when it is read for the
    /// last time and owns its value; the copy noted for it is not made.
    pub(crate) fn some_moved(&mut self, inner: &Expr, ty: &Ty) -> Option<String> {
        let ExprKind::Local(slot) = inner.kind else {
            return None;
        };
        if self.ref_bound.contains(&slot) || !holds_heap(self.program, &inner.ty) {
            return None;
        }
        self.copies.retain(|c| {
            !matches!(c, Copied::Let(span, why, _) | Copied::Library(span, _, why)
                if *span == inner.span && *why == SOME_COPY)
        });
        Some(format!("{}(std::move({}))", self.ty(ty), self.names[slot]))
    }

    /// Records the copy `.to_str()` / `.to_list()` makes of the view `e`
    /// converts.
    pub(crate) fn note_owned_view(&mut self, e: &Expr) {
        self.note_copy(e.span, crate::uses::OWNED_VIEW, None);
    }

    /// Records the copy a call passes of argument `e` it also changes
    /// through a `mut` argument.
    pub(crate) fn note_shared_arg_copy(&mut self, e: &Expr) {
        if holds_heap(self.program, &e.ty) {
            self.note_copy(e.span, crate::uses::SHARED_ARG, None);
        }
    }

    pub(crate) fn note_walked_copy(&mut self, e: &Expr) {
        self.note_copy(e.span, crate::uses::WALKED_COPY, None);
    }

    /// Records the copy `Some(xs)` makes when `xs` is a place holding a
    /// heap value: the `std::optional` holds a value of its own.
    pub(crate) fn note_some_copy(&mut self, inner: &Expr, text: &str) {
        let place = matches!(inner.kind, ExprKind::Local(_) | ExprKind::Global(_))
            || (matches!(
                inner.kind,
                ExprKind::Field(..) | ExprKind::Payload(..) | ExprKind::Index(..)
            ) && rooted_in_a_name(inner));
        if place && holds_heap(self.program, &inner.ty) && !moved(text) {
            self.note_copy(inner.span, SOME_COPY, None);
        }
    }

    /// Records the copy `p = q` makes when the stored text `text` names
    /// the place `value`, whose heap value it still holds after, or `value`
    /// calls a fn that lends what it returns, `n = a.get(i)`.
    pub(crate) fn note_set_copy(&mut self, value: &Expr, text: &str) {
        let place = matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_))
            || (matches!(
                value.kind,
                ExprKind::Field(..) | ExprKind::Payload(..) | ExprKind::Index(..)
            ) && rooted_in_a_name(value));
        // A part of an unnamed temporary (`let (a, b) = f()`) is the part of a dead value.
        let temp =
            matches!(part_base(value).kind, ExprKind::Local(s) if self.locals()[s].name.is_empty());
        let named = names_a_place(tuple_part_of(text)) && !(temp && tuple_part_of(text) != text);
        let lent = matches!(value.kind, ExprKind::Call(f, _) if self.lent_roots.contains_key(&f));
        if names_a_static(value) && !moved(text) {
            self.note_copy(value.span, STATIC_COPY, None);
        } else if (place && named || lent) && holds_heap(self.program, &value.ty) && !moved(text) {
            self.note_copy(value.span, SET_COPY, None);
        }
    }

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
            self.note_copy(
                e.span,
                "a later operand may change it, and operands are read left to right",
                None,
            );
        }
    }

    /// Records the copy a `let slot = init` emitted as `T name = text;`
    /// makes, when `init` names a place holding a heap value.
    /// A copy the contestant wrote as a method on a heap place,
    /// `a.clone()` or `a.sorted()`, not one a generated constructor makes
    /// of its parameter at the call's span.
    pub(crate) fn note_written_copy(
        &mut self,
        e: &Expr,
        args: &[Expr],
        text: &str,
        why: &'static str,
    ) {
        let [first, ..] = args else { return };
        let mut arg = first;
        // A conversion the C++ spells as its argument (`p.sorted()` of an
        // array is `cpl::sorted(p)`) copies that argument.
        while let ExprKind::Builtin(_, inner) = &arg.kind {
            match inner.as_slice() {
                [one] if is_place_text(text) => arg = one,
                _ => break,
            }
        }
        let place = matches!(
            arg.kind,
            ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
        );
        let written = arg.span.start >= e.span.start && arg.span.end < e.span.end;
        if place && (written || self.in_library()) && holds_heap(self.program, &arg.ty) {
            self.note_copy(e.span, why, None);
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
        let is_part = |e: &Expr| {
            matches!(
                e.kind,
                ExprKind::Field(..)
                    | ExprKind::Payload(..)
                    | ExprKind::Index(..)
                    | ExprKind::Unwrap(..)
            ) && (rooted_in_a_name(e) || given_root(part_base(e), &self.lent_roots).is_some())
                || matches!(e.kind, ExprKind::Call(f, _) if self.lent_roots.contains_key(&f))
        };
        // A temporary bound by reference names the place it was bound to.
        let is_whole = |e: &Expr| {
            matches!(e.kind, ExprKind::Local(s)
                if !self.locals()[s].name.is_empty() || self.ref_bound.contains(&s))
                || matches!(e.kind, ExprKind::Global(_))
        };
        // `T v = c ? f(a) : a;` copies the arm the C++ names as is.
        let named_arm = |e: &Expr| match e.kind {
            ExprKind::Local(s) => {
                let name = &self.names[s];
                text.ends_with(&format!(" : {name}")) || text.contains(&format!(" ? {name} : "))
            }
            _ => false,
        };
        // `T v = c ? f(a) : self.t[p];` and `T v = o ?? self.d;` copy the arm that is a place.
        let arms = match &init.kind {
            ExprKind::Binary(BinOp::Coalesce, a, b) => Some([&**a, &**b]),
            _ => lending_if(init),
        };
        if let Some(arm) = arms.and_then(|arms| {
            (arms.into_iter()).find(|a| is_part(a) || (is_whole(a) && named_arm(a)))
        }) {
            init = arm;
        }
        if names_a_static(init) && !moved(text) && !self.lent_slot(slot) {
            self.note_copy(init.span, STATIC_COPY, Some(slot));
            return;
        }
        let place = is_part(init);
        let whole = is_whole(init);
        if !holds_heap(self.program, &init.ty)
            || !(place || whole)
            || moved(text)
            || self.lent_slot(slot)
        {
            return;
        }
        let why = self.copy_reason(slot, init, tail, whole);
        self.note_copy(init.span, why, Some(slot));
    }

    /// Records the copy `return place;` makes when the fn or block gives a
    /// part of a named value it does not own by value: `return self.id;`.
    pub(crate) fn note_given_copy(&mut self, value: &Expr, text: &str, is_return: bool) {
        let by_ref = self.give_ref || (is_return && self.closure_id.is_none() && self.lends_result);
        let mut value = value;
        while let (ExprKind::Builtin(_, args), true) = (&value.kind, is_place_text(text)) {
            match args.as_slice() {
                [arg] => value = arg,
                _ => break,
            }
        }
        let part = matches!(
            value.kind,
            ExprKind::Field(..) | ExprKind::Payload(..) | ExprKind::Index(..)
        ) && rooted_in_a_name(value);
        let named = names_a_place(text);
        if part && named && !by_ref && holds_heap(self.program, &value.ty) && !moved(text) {
            self.note_copy(value.span, GIVEN, None);
        }
    }

    /// Records a copy at `span`; one in a library fn also names the fn and
    /// the local `slot` it binds, so `cpl explain` lists each copy once.
    pub(crate) fn note_copy(
        &mut self,
        span: cpl_diag::Span,
        why: &'static str,
        slot: Option<usize>,
    ) {
        let Some(id) = self.fn_id.filter(|_| self.in_library()) else {
            return self.copies.push(Copied::Let(span, why, self.fn_id));
        };
        let name = slot.map(|s| self.locals()[s].name.clone());
        let fun = &self.program.fn_display[id];
        let what = match name.filter(|n| !n.is_empty()) {
            Some(name) => format!("`{name}` in {fun}"),
            None => format!("a read in {fun}"),
        };
        self.copies.push(Copied::Library(span, what, why));
    }

    /// Whether local `slot` of the fn being emitted is a `cpl::Lent`
    /// ([`crate::lentopt::lent_opts`]): a pointer to the caller's value, so
    /// binding it copies no container.
    pub(crate) fn lent_slot(&self, slot: usize) -> bool {
        matches!(
            self.current_scope(),
            cpl_ir::ScopeId::Fn(id) if self.lent_opts[id].contains(&slot)
        )
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
    let mut visit = |cid, span| {
        at.entry(cid).or_insert(span);
    };
    crate::uses::places::each_closure_at(&program.setup, &mut visit);
    crate::uses::places::each_closure_at(&program.main, &mut visit);
    for f in &program.fns {
        crate::uses::places::each_closure_at(&f.body, &mut visit);
    }
    for c in &program.closures {
        crate::uses::places::each_closure_at(&c.body, &mut visit);
    }
    let mut once: Vec<_> = copies
        .iter()
        .filter_map(|c| match c {
            Copied::Let(span, why, fun) => Some((*span, String::new(), *why, *fun)),
            Copied::Library(..) => None,
            Copied::Capture(cid, name, why) => Some((
                *at.get(cid)?,
                format!("`{name}` captured by value"),
                *why,
                None,
            )),
        })
        .collect();
    once.sort_by_key(|(span, what, why, fun)| (span.start, what.clone(), *why, *fun));
    once.dedup_by(|a, b| a == b);
    let mut sites: Vec<_> = once
        .into_iter()
        .map(|(span, what, why, _)| (span, what, why))
        .collect();
    // Each library copy is its own site in the C++, though several share
    // the call that runs them: `return id` twice in one fn is two copies.
    sites.extend(copies.iter().filter_map(|c| match c {
        Copied::Library(span, what, why) => Some((*span, what.clone(), *why)),
        _ => None,
    }));
    sites.sort_by_key(|(span, what, why)| (span.start, what.clone(), *why));
    sites
}

impl Emitter<'_> {
    /// Statement `k` of `block`, moving each variable it reads for the
    /// last time instead of copying it.
    pub(crate) fn stmt_owning(&mut self, block: &Block, k: usize, out: &mut String) {
        // A loop's variable is a new value each turn, dead past its last
        // read in the body (moving a `const auto&` one only copies).
        let outer = (self.branch_dead.iter().rev())
            .find(|(b, _)| std::ptr::eq(*b, block))
            .map(|(_, dead)| dead.clone())
            .or_else(|| {
                (self.loop_vars.get(&(block as *const Block))).map(|&v| BTreeSet::from([v]))
            })
            .unwrap_or_default();
        let branches = match &block.stmts[k] {
            Stmt::If(_, then, otherwise) => {
                let dead = crate::uses::dead_past(self.program, block, k, &outer);
                self.branch_dead.push((then as *const Block, dead.clone()));
                self.branch_dead.push((otherwise as *const Block, dead));
                2
            }
            // A block run once ends as a branch does.
            Stmt::Expr(Expr {
                kind: ExprKind::Do(inner),
                ..
            }) => {
                let dead = crate::uses::dead_past(self.program, block, k, &outer);
                self.branch_dead.push((inner as *const Block, dead));
                1
            }
            _ => 0,
        };
        self.stmt_owning_in(block, k, &outer, out);
        let kept = self.branch_dead.len() - branches;
        self.branch_dead.truncate(kept);
    }

    fn stmt_owning_in(
        &mut self,
        block: &Block,
        k: usize,
        outer: &BTreeSet<usize>,
        out: &mut String,
    ) {
        self.moves_last = crate::uses::last_read(self.program, block, k, outer)
            || self.takes_sunk_param(block, k)
            || self.dies_here(&block.stmts[k])
            || self.unwraps_into_spent(block, k, outer);
        self.taken_items = self.taken_items(block, k, outer);
        if self.reads_held(&block.stmts[k]) {
            self.moves_last = false;
            self.taken_items.clear();
        }
        self.pushed_moves = crate::uses::pushed_last_reads(self.program, block, k, outer);
        // A fn's params live through its whole body: one dead after a call
        // there may move into it, if the fn owns it.
        let params: BTreeSet<usize> = match self.fn_id {
            Some(id) if std::ptr::eq(block, &self.program.fns[id].body) => {
                (0..self.program.fns[id].params.len())
                    .chain(outer.iter().copied())
                    .collect()
            }
            _ => outer.clone(),
        };
        self.called_moves = crate::uses::called_last_reads(self.program, block, k, &params);
        self.stmt_presized(&block.stmts, k, out);
        self.moves_last = false;
        self.taken_items.clear();
        self.pushed_moves.clear();
        self.called_moves.clear();
    }

    /// `let t = o!` of an optional read for the last time into a tuple
    /// whose fields are moved out (`while let (k, s) = xs.pop()`): the
    /// payload moves, as a copy would only be moved from.
    fn unwraps_into_spent(&self, block: &Block, k: usize, outer: &BTreeSet<usize>) -> bool {
        matches!(&block.stmts[k], Stmt::Let(bound, Some(init))
            if matches!(init.kind, ExprKind::Unwrap(..))
                && self.spent.contains(bound)
                && crate::uses::place_dies(self.program, block, k, init, outer))
    }

    /// Whether `stmt` reads a variable a lent struct field may point to,
    /// which must then never be moved from.
    fn reads_held(&self, stmt: &Stmt) -> bool {
        let (held, globals) = (&self.lent_fields.held, &self.lent_fields.held_globals);
        if held.is_empty() && globals.is_empty() {
            return false;
        }
        let scope = match (self.closure_id, self.fn_id) {
            (Some(c), _) => cpl_ir::ScopeId::Closure(c),
            (None, Some(f)) => cpl_ir::ScopeId::Fn(f),
            (None, None) => cpl_ir::ScopeId::Main,
        };
        let mut reads = false;
        crate::uses::places::each_variable_in(stmt, &mut |(global, s)| {
            reads |= match global {
                false => held.contains(&(scope, s)),
                true => globals.contains(&s),
            };
        });
        reads
    }

    /// `std::move(text)` when `item` is one of the literal's taken items.
    pub(crate) fn taken_item(&self, item: &Expr, text: String) -> String {
        match self.taken_items.contains(&(item as *const Expr)) {
            true => format!("std::move({text})"),
            false => text,
        }
    }

    /// Whether item `idx` of a returned literal reads a place on a local
    /// the function owns (`s.a`, `st[0].1`) that no other item reads: the
    /// return is the local's last use, so the item may move.
    pub(crate) fn returned_item_dies(&self, items: &[Expr], idx: usize) -> bool {
        let item = &items[idx];
        let Some(slot) = owned_root(item) else {
            return false;
        };
        let params = self.fn_id.map_or(0, |id| self.program.fns[id].params.len());
        (slot >= params || self.fn_id.is_some_and(|id| self.sunk[id].contains(&slot)))
            && (!self.ref_bound.contains(&slot) || self.mut_bound.contains(&slot))
            && !item.ty.is_scalar()
            && !matches!(item.ty, Ty::Fn(..))
            && !items.iter().enumerate().any(|(j, other)| {
                j != idx
                    && crate::uses::mentions_any(self.program, other, slot)
                    && !crate::uses::reads_apart(other, slot, &field_prefix(item))
            })
    }

    /// `std::move(text)` when `dies`.
    pub(crate) fn moved_if(&self, dies: bool, text: String) -> String {
        match dies && !text.starts_with("std::move(") {
            true => format!("std::move({text})"),
            false => text,
        }
    }

    /// The items of `let x = T(a, b.c, ..)` that read a place `rest` sets
    /// before reading it, which no other item of the literal reads either:
    /// g++ builds a `std::tuple` from its last item first, so an earlier
    /// item that reads the place too would see it moved out.
    fn taken_items(
        &self,
        block: &Block,
        k: usize,
        outer: &BTreeSet<usize>,
    ) -> BTreeSet<*const Expr> {
        let rest = &block.stmts[k + 1..];
        let (Stmt::Let(_, Some(init)) | Stmt::Assign(_, AssignOp::Set, init)) = &block.stmts[k]
        else {
            return BTreeSet::new();
        };
        let ExprKind::Tuple(items) = &init.kind else {
            return BTreeSet::new();
        };
        items
            .iter()
            .enumerate()
            .filter(|&(i, item)| {
                let Some(path) = path_of(item) else {
                    return false;
                };
                !item.ty.is_scalar()
                    && !matches!(item.ty, Ty::Fn(..))
                    && !(!path.0 .0 && self.ref_bound.contains(&path.0 .1))
                    && !items
                        .iter()
                        .enumerate()
                        .any(|(j, other)| j != i && touches(&Stmt::Expr(other.clone()), &path))
                    && (overwritten_before_read(item, rest)
                        || crate::uses::place_dies(self.program, block, k, item, outer)
                        || self.sunk_dies(block, k, item))
            })
            .map(|(_, item)| item as *const Expr)
            .collect()
    }

    /// A by-value param of the fn being written that nothing after statement
    /// `k` of its body reads.
    fn sunk_dies(&self, block: &Block, k: usize, item: &Expr) -> bool {
        let (ExprKind::Local(p), Some(id)) = (&item.kind, self.fn_id) else {
            return false;
        };
        self.closure_id.is_none()
            && std::ptr::eq(block, &self.program.fns[id].body)
            && self.sunk[id].contains(p)
            && crate::uses::unread_after(self.program, block, k, *p)
    }

    /// A `let` an enclosing statement found reads its source for the last
    /// time, unless that source is itself a reference or the `let` lends.
    pub(crate) fn dies_here(&self, stmt: &Stmt) -> bool {
        let Stmt::Let(slot, Some(init)) = stmt else {
            return false;
        };
        let at = stmt as *const Stmt;
        self.uses.dying.contains(&at)
            && !matches!(crate::uses::copied(init).unwrap_or_else(|| crate::uses::in_place(init)).kind,
                ExprKind::Local(src) if self.ref_bound.contains(&src))
            && !self.lends(*slot, init, self.tail.as_ref())
    }

    /// `std::move(text)` for a `let` of a heap variable (or a field path on
    /// one) the statements after it set again before anything reads it:
    /// `let old_k = ks; ks = List.filled(..)` takes the list, not a copy.
    pub(crate) fn take_if_overwritten(&mut self, init: &Expr, text: String) -> String {
        let taken = std::mem::take(&mut self.moves_last);
        if let ExprKind::Some(inner) = &init.kind {
            return match taken {
                true => self.some_moved(inner, &init.ty).unwrap_or(text),
                false => text,
            };
        }
        if let (ExprKind::If(..), true) = (&init.kind, taken) {
            return self.moved_some(init).unwrap_or(text);
        }
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
        ExprKind::Unwrap(base, _) => path_of(base),
        ExprKind::Field(base, f) => path_of(base).map(|(root, mut fields)| {
            fields.push(*f);
            (root, fields)
        }),
        _ => None,
    }
}

/// The local a place expression reads through fields and indexes.
/// The local whose part `e` gives when returned: `p` of `p.x0`, of
/// `Some(p.x0)` and of `t.b[0]`.
pub(crate) fn given_part_root(e: &Expr) -> Option<usize> {
    let part = match &e.kind {
        ExprKind::Some(inner) => inner,
        _ => e,
    };
    match part.kind {
        ExprKind::Field(..) | ExprKind::Index(..) => owned_root(part),
        _ => None,
    }
}

/// The local a binding of `init` names a place on: `t0` of `*t0` (the
/// optional an `if let` tests), of `t0.a` and of `t0`.
pub(crate) fn bound_root(e: &Expr) -> Option<usize> {
    match &e.kind {
        ExprKind::Local(s) => Some(*s),
        ExprKind::Unwrap(base, _)
        | ExprKind::Payload(base, ..)
        | ExprKind::Field(base, _)
        | ExprKind::Index(base, _) => bound_root(base),
        _ => None,
    }
}

fn owned_root(e: &Expr) -> Option<usize> {
    match &e.kind {
        ExprKind::Local(s) => Some(*s),
        ExprKind::Field(base, _) | ExprKind::Index(base, _) => owned_root(base),
        _ => None,
    }
}

/// The fields `e` follows from its root before any index: `s.a[0].b` is `[a]`.
fn field_prefix(e: &Expr) -> Vec<usize> {
    match &e.kind {
        ExprKind::Field(base, f) => {
            let mut fields = field_prefix(base);
            if path_of(base).is_some() {
                fields.push(*f);
            }
            fields
        }
        ExprKind::Index(base, _) => field_prefix(base),
        _ => Vec::new(),
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
    // Code called by name sees a local only through its arguments, which
    // `uses` counts; a global it may read itself.
    let access = Touch::of_block(&block);
    let opaque = match root.0 {
        true => access.code(),
        false => access.unnamed(),
    };
    let (mut uses, mut elsewhere) = (0usize, 0usize);
    crate::uses::places::each_variable(&block, &mut |v| uses += usize::from(v == *root));
    crate::uses::places::each_field(&block, &mut |base, f| {
        if fields.first().is_some_and(|f0| *f0 != f) {
            elsewhere +=
                usize::from(path_of(base).is_some_and(|(r, fs)| r == *root && fs.is_empty()));
        }
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
    path_of(init).is_some_and(|path| set_before_read(&path, rest))
}

/// Whether `rest` sets the local `slot` whole before anything reads it.
pub(crate) fn local_set_before_read(slot: usize, rest: &[Stmt]) -> bool {
    set_before_read(&((false, slot), Vec::new()), rest)
}

fn set_before_read(path: &((bool, usize), Vec<usize>), rest: &[Stmt]) -> bool {
    for stmt in rest {
        if let Stmt::Assign(place, AssignOp::Set, value) = stmt {
            if place_path(place).as_ref() == Some(path) {
                let value = Stmt::Expr(value.clone());
                return !touches(&value, path);
            }
        }
        if touches(stmt, path) {
            return false;
        }
    }
    false
}

/// Whether nothing in `rest`, nor code it calls, reads or sets the
/// variable path `e` names.
pub(crate) fn untouched_in(e: &Expr, rest: &[Stmt]) -> bool {
    path_of(e).is_some_and(|path| !rest.iter().any(|stmt| touches(stmt, &path)))
}

/// Each `for` body of `program` and the variable it binds.
pub(crate) fn loop_vars(program: &cpl_ir::Program) -> BTreeMap<*const Block, usize> {
    let mut vars = BTreeMap::new();
    let bodies = [&program.setup, &program.main]
        .into_iter()
        .chain(program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body));
    for body in bodies {
        crate::uses::places::each_for_body(body, &mut |slot, body| {
            vars.insert(body as *const Block, slot);
        });
    }
    vars
}

impl Emitter<'_> {
    /// The parts of `yield (xs[i], xs[j])` in a `cpl_each_` form, each the
    /// text of a place, when a heap one is among them: the callback gets
    /// references to them, not copies. A tuple of lent parts is lent: the
    /// loop body reads it, and storing it copies. The body cannot change
    /// what the fn reads (`cpl_ir::pushed` feeds it no such loop), and the
    /// fn's own locals stay put until the callback returns.
    pub(crate) fn lent_parts(&mut self, item: &Expr) -> Option<Vec<String>> {
        let (ExprKind::Tuple(parts), Ty::Tuple(tys)) = (&item.kind, &item.ty) else {
            return None;
        };
        let place = |e: &Expr| {
            matches!(
                e.kind,
                ExprKind::Local(_)
                    | ExprKind::Global(_)
                    | ExprKind::Field(..)
                    | ExprKind::Index(..)
            ) && crate::stmt::place_root(e).is_some()
        };
        let lends = parts.len() == tys.len()
            && self.lends_tuple_parts(&item.ty)
            && parts.iter().all(|p| p.ty.is_scalar() || place(p))
            && self
                .fn_id
                .is_some_and(|id| !self.devirt.recursive_fns.contains(&id));
        lends.then(|| parts.iter().map(|p| self.expr(p).text).collect())
    }

    /// Whether a yielded value of type `ty` is a tuple with a heap part, which a
    /// `cpl_each_` form hands its callback as references ([`Emitter::lent_parts`]).
    /// A range is two numbers, not a heap part.
    pub(crate) fn lends_tuple_parts(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Tuple(tys) if tys.iter().any(|t| !t.is_scalar() && !matches!(t, Ty::Fn(..) | Ty::Range(_))))
    }
}
