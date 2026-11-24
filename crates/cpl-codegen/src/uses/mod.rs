//! Uses: what may happen to a heap binding while it lives, the one source
//! for whether a read lends (`const&`), copies or moves.
//!
//! A binding lives from its `let` to the last statement that mentions it,
//! not to the end of its block: a write to its source after that last read
//! cannot change what it reads. A mention that could keep a reference to it
//! past that statement (a `let` built from it, a closure capturing it, a
//! block's value) keeps it live to the end of the block.

pub(crate) mod effects;
pub(crate) mod fn_refs;
pub(crate) mod loops;
pub(crate) mod message;
pub(crate) mod places;
pub(crate) mod rows;
pub(crate) mod sinks;
pub(crate) mod split;
pub(crate) mod walked;

use crate::lend::{copied_at_test, Tail};
use crate::walk;
use cpl_ir::{
    Arg, AssignOp, Block, Builtin, Expr, ExprKind, FnId, Iter, Place, PlaceKind, Program, Stmt, Ty,
};
use effects::Touch;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) use effects::Effects;

/// Where a binding lives: top-level code (setup and main), a fn or a closure.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Home {
    Top,
    Fn(FnId),
    Closure(usize),
}

/// What may happen to one heap binding (a `let`, parameter, capture or
/// loop binding) in its home.
#[derive(Clone, Default, Debug)]
pub(crate) struct Use {
    /// Stored to or passed `mut` after it is bound, captured by reference,
    /// or a `mut` parameter.
    pub(crate) mutated: bool,
    /// Read whole, not only through an index, field, length or `for`: the
    /// C++ may move it there.
    pub(crate) moved: bool,
}

/// The Uses table: each heap binding's [`Use`] and each fn's [`Effects`],
/// computed once per program.
#[derive(Default)]
pub(crate) struct Uses {
    bindings: BTreeMap<(Home, usize), Use>,
    pub(crate) table: cpl_ir::effects::Table,
    /// The `let`s that read their source for the last time, so may move
    /// it: those a statement runs once at its source's last read
    /// ([`dying_lets`]), and those of fns that read a global for the last
    /// time ([`dying_global_lets`]).
    pub(crate) dying: BTreeSet<*const Stmt>,
}

impl Uses {
    pub(crate) fn of(program: &Program, devirt: &crate::devirt::Devirt, traps: bool) -> Uses {
        let target = |home: Option<FnId>, callee: &Expr| devirt.target(home, callee);
        let table = cpl_ir::effects::Table::with_targets(program, traps, &target);
        let mut uses = Uses {
            bindings: BTreeMap::new(),
            table,
            dying: dying_global_lets(program),
        };
        let bodies = [&program.setup, &program.main]
            .into_iter()
            .chain(program.fns.iter().map(|f| &f.body))
            .chain(program.closures.iter().map(|c| &c.body));
        for body in bodies {
            walk::blocks(body, &mut |block| {
                for k in 0..block.stmts.len() {
                    uses.dying.extend(dying_lets(program, block, k));
                }
            });
        }
        uses.add(
            program,
            Home::Top,
            &[&program.setup, &program.main],
            &program.main_locals,
            &[],
        );
        for (id, f) in program.fns.iter().enumerate() {
            let muts: Vec<usize> = (0..f.params.len())
                .filter(|&p| f.params[p].mutable)
                .collect();
            uses.add(program, Home::Fn(id), &[&f.body], &f.locals, &muts);
        }
        for (cid, c) in program.closures.iter().enumerate() {
            uses.add(program, Home::Closure(cid), &[&c.body], &c.locals, &c.muts);
        }
        uses
    }

    fn add(
        &mut self,
        program: &Program,
        home: Home,
        blocks: &[&Block],
        locals: &[cpl_ir::Local],
        muts: &[usize],
    ) {
        let (mut mutated, mut moved) = (BTreeSet::new(), BTreeSet::new());
        mutated.extend(muts.iter().copied());
        for block in blocks {
            mutated.extend(Touch::of_block(block).stored_locals());
            moved.extend(crate::lend::whole_reads(block));
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Closure(cid) = e.kind {
                    let captures = program.closures[cid].captures.iter();
                    mutated.extend(
                        captures
                            .filter(|c| !c.is_global && c.by_ref)
                            .map(|c| c.outer),
                    );
                }
            });
        }
        for (slot, local) in locals.iter().enumerate() {
            if local.ty.is_scalar() {
                continue;
            }
            let found = Use {
                mutated: mutated.contains(&slot),
                moved: moved.contains(&slot),
            };
            self.bindings.insert((home, slot), found);
        }
    }

    /// The [`Use`] of `slot` in `home`; a scalar's is empty.
    pub(crate) fn of_binding(&self, home: Home, slot: usize) -> Use {
        self.bindings
            .get(&(home, slot))
            .cloned()
            .unwrap_or_default()
    }

    /// What calling fn `f` can do.
    pub(crate) fn effects(&self, f: FnId) -> &Effects {
        self.table.effects(f)
    }
}

/// The expression a read of `e` reads in place: `e` itself, or the array a
/// list view of it is (`ArrayToList`), which is the same storage.
pub(crate) fn in_place(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Builtin(Builtin::ArrayToList | Builtin::ListToArray, args) if args.len() == 1 => {
            in_place(&args[0])
        }
        _ => e,
    }
}

/// How `stmts` mention the local `slot`.
#[derive(Default)]
struct Mentions {
    any: bool,
    /// A mention that can hold a reference to it past its statement.
    kept: bool,
}

fn mentions(program: &Program, stmts: &[Stmt], slot: usize) -> Mentions {
    let block = Block {
        stmts: stmts.to_vec(),
    };
    let mut m = Mentions {
        any: Touch::of_block(&block).stored_locals().contains(&slot),
        kept: false,
    };
    let reads = |e: &Expr| {
        let mut found = false;
        walk::deep_expr(e, &mut |inner| match &inner.kind {
            ExprKind::Local(s) => found |= *s == slot,
            ExprKind::Closure(cid) => {
                found |= program.closures[*cid]
                    .captures
                    .iter()
                    .any(|c| !c.is_global && c.outer == slot)
            }
            _ => {}
        });
        found
    };
    walk::deep_exprs(&block, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => m.any = true,
        ExprKind::Closure(_) if reads(e) => (m.any, m.kept) = (true, true),
        _ => {}
    });
    // A `let` inside a nested block ends with that block, within its
    // statement; only one of these statements outlives it.
    m.kept |= stmts.iter().any(|stmt| match stmt {
        Stmt::Let(_, Some(e)) | Stmt::Give(e) => reads(e),
        _ => false,
    });
    m
}

/// Which of the `n` fields of the local tuple `slot` `block` reads, when it
/// reads `slot` only through its fields and no closure captures it.
pub(crate) fn fields_only(
    program: &Program,
    block: &Block,
    slot: usize,
    n: usize,
) -> Option<Vec<bool>> {
    let (mut whole, mut read, mut captured) = (0isize, vec![false; n], false);
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => whole += 1,
        ExprKind::Field(base, i) if matches!(base.kind, ExprKind::Local(s) if s == slot) => {
            whole -= 1;
            if let Some(r) = read.get_mut(*i) {
                *r = true;
            }
        }
        ExprKind::Closure(c) => {
            captured |= program.closures[*c]
                .captures
                .iter()
                .any(|c| !c.is_global && c.outer == slot)
        }
        _ => {}
    });
    (whole == 0 && !captured).then_some(read)
}

/// Whether any of `exprs` reads a local `hit` picks.
pub(crate) fn reads_local(exprs: &[Expr], hit: &dyn Fn(usize) -> bool) -> bool {
    let mut found = false;
    for e in exprs {
        walk::deep_expr(e, &mut |e| {
            found |= matches!(e.kind, ExprKind::Local(s) if hit(s))
        });
    }
    found
}

/// Whether `stmts` mention the local `slot`, directly or through a closure.
pub(crate) fn mentions_in(program: &Program, stmts: &[Stmt], slot: usize) -> bool {
    mentions(program, stmts, slot).any
}

/// Whether `e` mentions the local `slot`, directly or through a closure.
pub(crate) fn mentions_any(program: &Program, e: &Expr, slot: usize) -> bool {
    mentions(program, &[Stmt::Give(e.clone())], slot).any
}

/// Whether every read of the local `slot` in `e` leaves the path `fields`
/// at a different field (`s.b` beside `s.a`), so it reads none of it.
pub(crate) fn reads_apart(e: &Expr, slot: usize, fields: &[usize]) -> bool {
    let (mut reads, mut apart, mut closure) = (0, 0, false);
    walk::deep_expr(e, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => reads += 1,
        ExprKind::Field(base, f) => {
            apart += usize::from(field_path(base).is_some_and(|(s, at)| {
                s == slot
                    && at.len() < fields.len()
                    && fields.starts_with(&at)
                    && fields[at.len()] != *f
            }));
        }
        ExprKind::Closure(_) => closure = true,
        _ => {}
    });
    !closure && reads == apart
}

/// The code of `rest` (the statements after a `let` of `slot`) that runs
/// while `slot` lives: up to the last statement that mentions it or a `let`
/// built from it (which may point into it), and of a last `if`, its test
/// and the same of each branch: a write after the last read cannot change
/// what was read.
pub(crate) fn live(program: &Program, rest: &[Stmt], slot: usize) -> Vec<Stmt> {
    live_of(program, rest, BTreeSet::from([slot]))
}

fn live_of(program: &Program, stmts: &[Stmt], mut names: BTreeSet<usize>) -> Vec<Stmt> {
    let mut end = 0;
    for (i, stmt) in stmts.iter().enumerate() {
        let one = std::slice::from_ref(stmt);
        let (mut any, mut kept) = (false, false);
        for &name in &names {
            let m = mentions(program, one, name);
            (any, kept) = (any || m.any, kept || m.kept);
        }
        if !any {
            continue;
        }
        end = i + 1;
        match stmt {
            Stmt::Let(bound, Some(init)) if !holds_closure(init) => _ = names.insert(*bound),
            _ if kept => return stmts.to_vec(),
            _ => {}
        }
    }
    let mut out = stmts[..end].to_vec();
    if let Some(Stmt::If(_, then, otherwise)) = out.last_mut() {
        let branch = |b: &Block| Block {
            stmts: live_of(program, &b.stmts, names.clone()),
        };
        *then = branch(then);
        *otherwise = branch(otherwise);
    }
    out
}

fn holds_closure(e: &Expr) -> bool {
    let mut found = false;
    walk::deep_expr(e, &mut |x| found |= matches!(x.kind, ExprKind::Closure(_)));
    found
}

/// Whether the local `slot`, which statement `k` of `block` reads, is
/// dead after it: the block declared it (the next pass of a loop declares
/// it afresh), no closure of the block can reach it, and no later
/// statement mentions it. That read may move it.
/// `outer` are the locals dead once `block`, a branch of an `if`, ends.
pub(crate) fn dead_after(
    program: &Program,
    block: &Block,
    k: usize,
    slot: usize,
    outer: &BTreeSet<usize>,
) -> bool {
    let declared = outer.contains(&slot)
        || block.stmts[..k]
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Let(s, _) if *s == slot));
    declared && unread_after(program, block, k, slot)
}

/// The locals dead once statement `k` of `block`, an `if`, ends: what
/// each of its branches may move. `outer` is the same for `block`. A local
/// the statements after `k` set before reading it is dead there too.
pub(crate) fn dead_past(
    program: &Program,
    block: &Block,
    k: usize,
    outer: &BTreeSet<usize>,
) -> BTreeSet<usize> {
    let lets = block.stmts[..k].iter().filter_map(|stmt| match stmt {
        Stmt::Let(s, _) => Some(*s),
        _ => None,
    });
    let mut read = BTreeSet::new();
    let here = Block {
        stmts: vec![block.stmts[k].clone()],
    };
    walk::deep_exprs(&here, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            read.insert(s);
        }
    });
    let set_again = read.into_iter().filter(|&s| {
        !captured_in(program, block, s)
            && crate::lend::local_set_before_read(s, &block.stmts[k + 1..])
    });
    (outer.iter().copied().chain(lets))
        .filter(|&s| unread_after(program, block, k, s))
        .chain(set_again)
        .collect()
}

/// True when nothing after statement `k` of `block`, nor a closure in it
/// that captures `slot`, reads `slot`.
pub(crate) fn unread_after(program: &Program, block: &Block, k: usize, slot: usize) -> bool {
    !captured_in(program, block, slot) && !mentions(program, &block.stmts[k + 1..], slot).any
}

/// Whether a closure in `block` captures the local `slot`: one made before a
/// statement may hold it by reference and run later.
fn captured_in(program: &Program, block: &Block, slot: usize) -> bool {
    let mut held = false;
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            held |=
                (program.closures[cid].captures.iter()).any(|c| !c.is_global && c.outer == slot);
        }
    });
    held
}

/// Why the place a lent `let` reads may change while the name lives.
pub(crate) const SET_AGAIN: &str = "the name is changed later, so it owns its value";
pub(crate) const SOURCE_MOVED: &str = "what it reads is moved while the name lives";
pub(crate) const SOURCE_SET: &str = "what it reads is set while the name lives";
pub(crate) const SOURCE_CALLED: &str = "a call while the name lives may change what it reads";
pub(crate) const SOURCE_MAY_CHANGE: &str = "what it reads may change while the name lives";
pub(crate) const SOURCE_READ_WHOLE: &str =
    "what it reads is read whole, so may move, while the name lives";
/// Why a `let` the table finds no change for still copies.
pub(crate) const NAME_MOVED: &str = "the name is moved later, so it owns its value";
/// Why a `.clone()` of a place copies: it asks for a second value.
pub(crate) const CLONED: &str = "written `.clone()`, so it is a second value";
/// Why `.to_str()` / `.to_list()` of a view copies: what it shows becomes
/// a value of its own.
pub(crate) const OWNED_VIEW: &str = "a view made owned copies what it shows";
/// Why a loop over a place walks a copy: the body may change the place.
pub(crate) const WALKED_COPY: &str = "the loop body may change what it walks, so it walks a copy";
/// Why `.sorted()` of a place copies: it sorts a copy, where `.sort()`
/// sorts in place.
pub(crate) const SORTED: &str = "`.sorted()` sorts a copy; `.sort()` sorts in place";
/// Why an argument the same call also changes through a `mut` one is
/// passed as a copy: the callee must see the value from before the call.
pub(crate) const SHARED_ARG: &str =
    "the call also changes it through a `mut` argument, so it passes a copy";
pub(crate) const WHOLE: &str = "a `let` of a whole variable is a second value, not a second name";

impl crate::emit::Emitter<'_> {
    /// Where the code being written binds its locals.
    /// The key `Facts` has for the body being emitted.
    pub(crate) fn facts_home(&self) -> Option<usize> {
        self.facts.key(self.home())
    }

    /// The variable local `slot` of the body being emitted is to `Facts`.
    pub(crate) fn facts_local(&self, slot: usize) -> crate::ranges::Var {
        self.facts.local(self.facts_home(), slot)
    }

    /// Whether local `slot` is a parameter of the fn being emitted: a `let`
    /// of one the fn only reads parts of names the caller's value.
    pub(crate) fn is_param(&self, slot: usize) -> bool {
        self.closure_id.is_none()
            && self
                .fn_id
                .is_some_and(|id| slot < self.program.fns[id].params.len())
    }

    pub(crate) fn home(&self) -> Home {
        match (self.closure_id, self.fn_id) {
            (Some(cid), _) => Home::Closure(cid),
            (None, Some(id)) => Home::Fn(id),
            (None, None) => Home::Top,
        }
    }

    /// Whether the `let` of `slot` can be a reference to the place `init`
    /// names instead of a copy of it: it is a heap value and nothing changes
    /// either for as long as the name lives
    /// ([`Emitter::source_change`]). `tail` is what the statements after the
    /// `let` in its block do; without one the whole fn is held to that.
    pub(crate) fn lends(&self, slot: usize, init: &Expr, tail: Option<&Tail>) -> bool {
        // A long literal is one `static const std::string`, which nothing writes.
        if matches!(&init.kind, ExprKind::Str(bytes) if bytes.len() > crate::expr::SHORT_STR) {
            return !self.uses.of_binding(self.home(), slot).mutated
                && self.facts.stores(self.facts_local(slot)) <= 1;
        }
        let init = in_place(init);
        !init.ty.is_scalar()
            && !matches!(init.ty, Ty::Fn(..))
            && !self.spent.contains(&slot)
            && match init.kind {
                ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
                | ExprKind::Unwrap(..) => true,
                // The optional an `if let` or `while let` tests.
                ExprKind::Local(src) => {
                    self.locals()[slot].name.is_empty()
                        || self.locals()[src].name.is_empty()
                        || (self.is_param(src) && !self.uses.of_binding(self.home(), slot).moved)
                }
                ExprKind::Global(_) => self.locals()[slot].name.is_empty(),
                // `let n = arena.get(id)`: a fn that lends from an argument.
                ExprKind::Call(callee, _) => self.lent_roots.contains_key(&callee),
                _ => false,
            }
            && self.source_change(slot, init, tail).is_none()
    }

    /// The call a `let` of `slot` holds in place when `init` calls a fn that returns
    /// `cpl::Held<T>` and [`Emitter::lends`] would let it read a lent result: `text` untaken.
    pub(crate) fn held_call(
        &mut self,
        slot: usize,
        init: &Expr,
        tail: Option<&Tail>,
        text: &str,
    ) -> Option<String> {
        let ExprKind::Call(callee, _) = init.kind else {
            return None;
        };
        let root = *self.held_roots.get(&callee)?;
        let call = text.strip_suffix(".take()")?.to_string();
        self.lent_roots.insert(callee, root);
        let lends = self.lends(slot, init, tail);
        self.lent_roots.remove(&callee);
        lends.then_some(call)
    }

    /// Whether the `let` of `slot` from an entry of `map` (`m.get(k)`, or
    /// `m.get(k) ?? d`), a `value` once found, can point into `map` instead
    /// of copying the entry: the general rule of [`Emitter::lends`], asked
    /// of the map itself.
    pub(crate) fn lends_entry(
        &self,
        slot: usize,
        map: &Expr,
        value: &Ty,
        tail: Option<&Tail>,
    ) -> bool {
        !value.is_scalar()
            && matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_))
            && !self.spent.contains(&slot)
            && self.source_stays(slot, map, tail)
    }

    /// Why the `let` of `slot` from `init` copies, for `cpl explain`: the
    /// table's reason ([`Emitter::source_change`]) when it has one.
    pub(crate) fn copy_reason(
        &self,
        slot: usize,
        init: &Expr,
        tail: Option<&Tail>,
        whole: bool,
    ) -> &'static str {
        if let Some(why) = self.source_change(slot, init, tail) {
            return why;
        }
        match (self.spent.contains(&slot), whole) {
            (true, _) => NAME_MOVED,
            (false, true) => WHOLE,
            (false, false) => SOURCE_MAY_CHANGE,
        }
    }

    /// The root of a place whose indices call fns (`at[p.cut(r)]`) when
    /// none of them can change it: they run before the name binds.
    fn keyed_root(&self, e: &Expr) -> Option<(bool, usize)> {
        let mut keys = Vec::new();
        let mut at = e;
        let root = loop {
            at = match &at.kind {
                ExprKind::Unwrap(base, _)
                | ExprKind::Field(base, _)
                | ExprKind::Payload(base, ..) => base,
                ExprKind::Index(base, indices) => {
                    keys.extend(indices);
                    base
                }
                ExprKind::Call(callee, args) => {
                    let lent = *self.lent_roots.get(callee)?;
                    let mut from = None;
                    for (k, arg) in args.iter().enumerate() {
                        let cpl_ir::Arg::Value(v) = arg else {
                            return None;
                        };
                        if k == lent {
                            from = Some(v);
                        } else {
                            keys.push(v);
                        }
                    }
                    from?
                }
                _ => break walk::path_root(at)?,
            };
        };
        keys.iter()
            .all(|key| self.leaves(key, root))
            .then_some(root)
    }

    pub(crate) fn source_stays(&self, slot: usize, init: &Expr, tail: Option<&Tail>) -> bool {
        self.source_change(slot, init, tail).is_none()
    }

    /// Why the place `init` reads may change, or the name `slot` be changed,
    /// while the `let` of `slot` lives; `None` when neither can.
    pub(crate) fn source_change(
        &self,
        slot: usize,
        init: &Expr,
        tail: Option<&Tail>,
    ) -> Option<&'static str> {
        let temp = |slot: usize| self.locals()[slot].name.is_empty();
        let changed = |yes: bool, why| yes.then_some(why);
        let Some((global, root)) = crate::stmt::place_root(init).or_else(|| self.keyed_root(init))
        else {
            return Some(SOURCE_CALLED);
        };
        let home = self.home();
        if self.uses.of_binding(home, slot).mutated || self.facts.stores(self.facts_local(slot)) > 1
        {
            return Some(SET_AGAIN);
        }
        let params = self.fn_id.map_or(0, |id| self.program.fns[id].params.len());
        let named_local = !global && root >= params && !temp(root);
        if !global && root >= params && !named_local {
            // A temporary set once is read only by the pattern that takes
            // it apart, and a reference it holds was checked for all the
            // statements after it, which cover this name.
            return changed(self.spent.contains(&root), SOURCE_MOVED).or(changed(
                self.facts.stores(self.facts_local(root)) != 1,
                SOURCE_SET,
            ));
        }
        if (named_local && self.spent.contains(&root))
            || (!global && self.fn_id.is_some_and(|id| self.sunk[id].contains(&root)))
        {
            return Some(SOURCE_MOVED);
        }
        if tail.is_some_and(|tail| tail.tested) && temp(slot) {
            return None;
        }
        let Some(tail) = tail else {
            let Some(id) = self.fn_id else {
                return Some(SOURCE_MAY_CHANGE);
            };
            let body = &self.program.fns[id].body;
            let own = crate::ranges::Facts::of_fn(id, body);
            let stored = match global {
                true => own.stores((None, true, root)),
                false => own.stores((Some(id), false, root)),
            };
            return changed(stored != 0, SOURCE_SET).or(changed(
                {
                    let t = self.tail(&body.stmts).touch;
                    !t.calls.is_empty() || t.values
                },
                SOURCE_CALLED,
            ));
        };
        if tail.touch.opaque
            && !(named_local && !effects::closure_writes(self.program, self.fn_id, root))
        {
            return Some(SOURCE_CALLED);
        }
        let t = &tail.touch;
        let calls_writing =
            |f: &dyn Fn(&Effects) -> bool| t.calls.iter().any(|&id| f(self.uses.effects(id)));
        let sets_param = t.writes.iter().any(|&(g, l)| !g && l < params);
        match global {
            // A parameter may be another name for the global.
            true => changed(calls_writing(&|e| e.writes.contains(&root)), SOURCE_CALLED).or(
                changed(t.writes.contains(&(true, root)) || sets_param, SOURCE_SET),
            ),
            // No fn sees a local it isn't passed `mut`, and that is a store.
            // Nor may it read the local whole: that read could move it.
            false if named_local => changed(t.writes.contains(&(false, root)), SOURCE_SET)
                .or(changed(tail.whole.contains(&root), SOURCE_READ_WHOLE)),
            // A parameter may be another name for a global a call writes,
            // or for another parameter a store sets.
            false => {
                changed(t.writes.contains(&(false, root)) || sets_param, SOURCE_SET).or(changed(
                    calls_writing(&|e| !e.writes.is_empty() || e.unknown_call),
                    SOURCE_CALLED,
                ))
            }
        }
    }

    /// What the statements after `stmts[k]` do while the `let` there lives,
    /// when that `let` binds a heap value.
    pub(crate) fn tail_at(&self, stmts: &[Stmt], k: usize) -> Option<Tail> {
        let Stmt::Let(slot, Some(init)) = &stmts[k] else {
            return None;
        };
        if init.ty.is_scalar() || matches!(init.ty, Ty::Fn(..)) {
            return None;
        }
        let rest = &stmts[k + 1..];
        let live = live(self.program, rest, *slot);
        Some(Tail {
            tested: copied_at_test(*slot, rest),
            lendable: self.lendable_reads(*slot, rest),
            ..self.tail(&live)
        })
    }
}

/// True when statement `k` of `block` reads a heap variable for the last
/// time and may move it: `x = y` or `let x = y` (or `Some(y)`) with `y` dead after it
/// (`uses::dead_after`), or a `let` whose source is set again before
/// anything reads it.
/// Whether a statement of `rest` moves `slot` on whole: returns, gives or
/// stores it, or puts it in an optional, a tuple or a variant.
fn moved_on(rest: &[Stmt], slot: usize) -> bool {
    let whole = |e: &Expr| matches!(e.kind, ExprKind::Local(s) if s == slot);
    let held = |s: &Stmt| match s {
        Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Assign(_, AssignOp::Set, e) => whole(e),
        _ => false,
    };
    let wraps = |e: &Expr| match &e.kind {
        ExprKind::Some(inner) => whole(inner),
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().any(whole),
        _ => false,
    };
    rest.iter().any(|stmt| {
        let mut found = held(stmt);
        walk::stmt_blocks(stmt, &mut |inner| {
            walk::deep_stmts(inner, &mut |s| found |= held(s))
        });
        walk::stmt_exprs(stmt, &mut |e| {
            walk::deep_expr(e, &mut |e| found |= wraps(e))
        });
        walk::stmt_blocks(stmt, &mut |inner| {
            walk::deep_exprs(inner, &mut |e| found |= wraps(e))
        });
        found
    })
}

pub(crate) fn last_read(
    program: &Program,
    block: &Block,
    k: usize,
    outer: &BTreeSet<usize>,
) -> bool {
    if let Stmt::Assign(place, AssignOp::Set, value) = &block.stmts[k] {
        if let ExprKind::Some(inner) = &value.kind {
            if let ExprKind::Local(slot) = inner.kind {
                return walk::root(place) != (false, slot)
                    && dead_after(program, block, k, slot, outer);
            }
        }
    }
    if let Stmt::Let(bound, Some(init)) = &block.stmts[k] {
        // An `if let` payload stays a reference unless the name moves on.
        if let ExprKind::Unwrap(..) = init.kind {
            return crate::lend::overwritten_before_read(init, &block.stmts[k + 1..])
                || (place_dies(program, block, k, init, outer)
                    && moved_on(&block.stmts[k + 1..], *bound));
        }
        let init = match &init.kind {
            ExprKind::Some(inner) => &**inner,
            ExprKind::If(_, a, b) => match some_local(a).or(some_local(b)) {
                Some(slot) => return dead_after(program, block, k, slot, outer),
                None => init,
            },
            _ => init,
        };
        return crate::lend::overwritten_before_read(init, &block.stmts[k + 1..])
            || place_dies(program, block, k, in_place(init), outer)
            || copied(init).is_some_and(|src| match src.kind {
                ExprKind::Global(g) => dies_in(program, block, k, (true, g), None),
                _ => false,
            });
    }
    let Stmt::Assign(place, AssignOp::Set, value) = &block.stmts[k] else {
        return false;
    };
    let source = match value.kind {
        ExprKind::Local(slot) => (false, slot),
        ExprKind::Global(g) => (true, g),
        _ => return false,
    };
    field_store(place)
        && walk::root(place) != source
        && (crate::lend::overwritten_before_read(value, &block.stmts[k + 1..])
            || match source {
                (false, slot) => dead_after(program, block, k, slot, outer),
                // Nothing runs after the program's last top-level statement.
                (true, _) => {
                    std::ptr::eq(block, &program.main)
                        && crate::lend::untouched_in(value, &block.stmts[k + 1..])
                }
            })
}

/// A variable or a field path on one: `x`, `s.a.b`.
fn field_store(place: &Place) -> bool {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => true,
        PlaceKind::Field(base, _) => field_store(base),
        _ => false,
    }
}

/// The locals `xs.push(y)` or `xs.push((i, y))`, statement `k` of `block`,
/// reads for the last time: the list takes them by move.
pub(crate) fn pushed_last_reads(
    program: &Program,
    block: &Block,
    k: usize,
    outer: &BTreeSet<usize>,
) -> Vec<*const Expr> {
    let Stmt::Expr(e) = &block.stmts[k] else {
        return Vec::new();
    };
    let ExprKind::Mutate(cpl_ir::Mutation::Push, place, args) = &e.kind else {
        return Vec::new();
    };
    let items: Vec<&Expr> = match args.first().map(|a| &a.kind) {
        Some(ExprKind::Tuple(items)) => items.iter().collect(),
        Some(_) => args.iter().take(1).collect(),
        None => Vec::new(),
    };
    let slot_of = |e: &Expr| match e.kind {
        ExprKind::Local(slot) => Some(slot),
        _ => None,
    };
    items
        .iter()
        .filter_map(|item| slot_of(item).map(|slot| (*item, slot)))
        .filter(|&(_, slot)| {
            walk::root(place) != (false, slot)
                && items.iter().filter(|i| slot_of(i) == Some(slot)).count() == 1
                && dead_after(program, block, k, slot, outer)
        })
        .map(|(item, _)| item as *const Expr)
        .collect()
}

/// The arguments of the calls statement `k` of `block` runs once, `f(xs, ..)`,
/// `let r = f(xs, ..)`, `r = g(f(xs))` or `print(f(xs))`, that are locals
/// named once in it and dead after it, or globals that die there: each may
/// move into a parameter taken by value.
pub(crate) fn called_last_reads(
    program: &Program,
    block: &Block,
    k: usize,
    outer: &BTreeSet<usize>,
) -> Vec<*const Expr> {
    let e = match &block.stmts[k] {
        Stmt::Expr(e) | Stmt::Let(_, Some(e)) | Stmt::Assign(_, _, e) => e,
        _ => return Vec::new(),
    };
    // The calls the statement runs once each: `f(x)`, `g(f(x)).len`,
    // `print(f(x))` and the `let`s of a `do` block, never one inside a
    // closure or a loop.
    fn once<'e>(e: &'e Expr, out: &mut Vec<&'e Arg>) {
        match &e.kind {
            ExprKind::Call(_, args) => {
                for arg in args {
                    out.push(arg);
                    if let Arg::Value(v) = arg {
                        once(v, out);
                    }
                }
            }
            ExprKind::Builtin(_, args) => args.iter().for_each(|a| once(a, out)),
            ExprKind::Print(args, _) => args.iter().for_each(|a| once(&a.value, out)),
            ExprKind::Do(block) => {
                for stmt in &block.stmts {
                    if let Stmt::Let(_, Some(x)) | Stmt::Expr(x) = stmt {
                        once(x, out);
                    }
                }
            }
            _ => {}
        }
    }
    let mut args = Vec::new();
    once(e, &mut args);
    let named = |slot: usize| {
        let mut n = 0;
        walk::deep_expr(e, &mut |x| {
            n += usize::from(x.kind == ExprKind::Local(slot))
        });
        n
    };
    args.into_iter()
        .filter_map(|arg| match arg {
            Arg::Value(value) => match value.kind {
                ExprKind::Local(slot) => Some((value, (false, slot))),
                ExprKind::Global(g) => Some((value, (true, g))),
                _ => None,
            },
            Arg::Mut(_) => None,
        })
        .filter(|&(_, src)| match src {
            (false, slot) => named(slot) == 1 && dead_after(program, block, k, slot, outer),
            (true, _) => dies_in(program, block, k, src, None),
        })
        .map(|(value, _)| value as *const Expr)
        .collect()
}

/// The local `y` of `Some(y)`.
fn some_local(e: &Expr) -> Option<usize> {
    match &e.kind {
        ExprKind::Some(inner) => match inner.kind {
            ExprKind::Local(slot) => Some(slot),
            _ => None,
        },
        _ => None,
    }
}

/// The local a place expression starts from and the fields it follows:
/// `m.a.b` is `m` and `[a, b]`, `m` alone is `m` and `[]`.
fn field_path(e: &Expr) -> Option<(usize, Vec<usize>)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((*slot, Vec::new())),
        // The payload of an optional is the optional's place: `if let x = t`
        // binds `*t`, which dies with `t`.
        ExprKind::Unwrap(base, _) => field_path(base),
        ExprKind::Field(base, f) => field_path(base).map(|(slot, mut fields)| {
            fields.push(*f);
            (slot, fields)
        }),
        _ => None,
    }
}

/// Whether the place `init` names, a local this block declared or a field
/// path into one, is dead after statement `k`: no closure of the block can
/// reach the local, nothing after `k` sets it, and every later read of it
/// leaves the path at a different field (`var acc = m.a.id` when only
/// `m.a.op` and `m.n` are read later). That read may move it.
pub(crate) fn place_dies(
    program: &Program,
    block: &Block,
    k: usize,
    init: &Expr,
    outer: &BTreeSet<usize>,
) -> bool {
    let Some((slot, fields)) = field_path(init) else {
        return false;
    };
    if fields.is_empty() {
        return dead_after(program, block, k, slot, outer);
    }
    let declared = block.stmts[..k]
        .iter()
        .any(|stmt| matches!(stmt, Stmt::Let(s, _) if *s == slot));
    let rest = Block {
        stmts: block.stmts[k + 1..].to_vec(),
    };
    let (mut reads, mut apart) = (0, 0);
    walk::deep_exprs(&rest, &mut |e| match &e.kind {
        ExprKind::Local(s) if *s == slot => reads += 1,
        ExprKind::Field(base, f) => {
            let leaves = |(s, at): (usize, Vec<usize>)| {
                s == slot
                    && at.len() < fields.len()
                    && fields.starts_with(&at)
                    && fields[at.len()] != *f
            };
            apart += usize::from(field_path(base).is_some_and(leaves));
        }
        _ => {}
    });
    declared
        && !Touch::of_block(block).closures
        && reads == apart
        && !Touch::of_block(&rest).stored_locals().contains(&slot)
}

/// Value parameters whose one read copies them into a local the body
/// changes, `var l = limbs`: taken by value, the copy moves in instead, and
/// a caller at its last use moves its own value in.
/// The params a struct literal of the body's own statements takes as an item
/// at their last read: by value, they move into the field.
pub(crate) fn stored_last(program: &Program, f: &cpl_ir::Function) -> BTreeSet<usize> {
    let body = &f.body;
    let mut sunk = BTreeSet::new();
    for (k, stmt) in body.stmts.iter().enumerate() {
        let (Stmt::Let(_, Some(init)) | Stmt::Assign(_, AssignOp::Set, init)) = stmt else {
            continue;
        };
        let ExprKind::Tuple(items) = &init.kind else {
            continue;
        };
        for (i, item) in items.iter().enumerate() {
            let ExprKind::Local(p) = item.kind else {
                continue;
            };
            let elsewhere = items
                .iter()
                .enumerate()
                .any(|(j, other)| j != i && mentions(program, &[Stmt::Expr(other.clone())], p).any);
            if p < f.params.len()
                && !f.params[p].mutable
                && !item.ty.is_scalar()
                && !matches!(item.ty, Ty::Fn(..))
                && !elsewhere
                && unread_after(program, body, k, p)
            {
                sunk.insert(p);
            }
        }
    }
    sunk
}

/// Whether `block` may run some of its code more than once: it holds a
/// loop, at any depth.
pub(crate) fn repeats(block: &Block) -> bool {
    let mut found = false;
    loops::each_loop(block, &mut |_, _| found = true);
    found
}

/// How many times `block` reads each of the locals below `n`, at any depth.
pub(crate) fn read_counts(block: &Block, n: usize) -> Vec<usize> {
    let mut reads = vec![0usize; n];
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            if let Some(k) = reads.get_mut(s) {
                *k += 1;
            }
        }
    });
    reads
}

pub(crate) fn copied_once(f: &cpl_ir::Function) -> BTreeSet<usize> {
    let stored = Touch::of_block(&f.body).stored_locals();
    let reads = read_counts(&f.body, f.params.len());
    f.body
        .stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Let(to, Some(init)) => Some((to, init, copied(init).unwrap_or(in_place(init)))),
            _ => None,
        })
        .filter_map(|(to, init, src)| match (&src.kind, &init.ty) {
            (ExprKind::Local(p), ty)
                if *p < f.params.len()
                    && reads[*p] == 1
                    && !f.params[*p].mutable
                    && !ty.is_scalar()
                    && !matches!(ty, Ty::Fn(..))
                    && stored.contains(to) =>
            {
                Some(*p)
            }
            _ => None,
        })
        .collect()
}

/// The `let x = src` statements that statement `k` of `block` runs once,
/// in the value blocks it evaluates, at the last read of `src`, a heap
/// variable `block` sets: `for e in edges.sorted(by: ..)` with nothing
/// reading `edges` after it sorts `edges` itself. Such a `let` may move
/// `src`.
fn dying_lets(program: &Program, block: &Block, k: usize) -> Vec<*const Stmt> {
    let mut found = Vec::new();
    once_lets(&block.stmts[k], &mut found);
    found
        .into_iter()
        .filter(|stmt| match stmt {
            Stmt::Let(_, Some(init)) => {
                crate::lend::holds_heap(program, &init.ty)
                    && !matches!(init.ty, Ty::Fn(..))
                    && walk::path_root(moved_source(init))
                        .filter(|_| walk::is_simple(moved_source(init)))
                        .is_some_and(|src| dies_in(program, block, k, src, None))
            }
            _ => false,
        })
        .map(|stmt| stmt as *const Stmt)
        .collect()
}

/// The variable a `let` takes whole: `src` of `let x = src`, of
/// `let x = Some(src)` and of `let x = if c: Some(src) else: None`.
fn moved_source(init: &Expr) -> &Expr {
    fn some(e: &Expr) -> Option<&Expr> {
        match &e.kind {
            ExprKind::Some(inner) if matches!(inner.kind, ExprKind::Local(_)) => Some(inner),
            _ => None,
        }
    }
    match &init.kind {
        ExprKind::Some(_) => some(init).unwrap_or(init),
        ExprKind::If(_, a, b) => match (some(a), some(b)) {
            (Some(src), None) | (None, Some(src)) => src,
            _ => init,
        },
        _ => copied(init).unwrap_or_else(|| in_place(init)),
    }
}

/// The variable `xs` a lowered `Copy(xs)` copies whole: once `xs` dies the
/// copy can take it.
pub(crate) fn copied(init: &Expr) -> Option<&Expr> {
    match &init.kind {
        ExprKind::Builtin(Builtin::Copy, args) => match args.as_slice() {
            [src] if matches!(in_place(src).kind, ExprKind::Local(_) | ExprKind::Global(_)) => {
                Some(in_place(src))
            }
            _ => None,
        },
        _ => None,
    }
}

/// A read of the whole variable `src`.
fn reads_whole(e: &Expr, src: (bool, usize)) -> bool {
    match e.kind {
        ExprKind::Local(s) => src == (false, s),
        ExprKind::Global(g) => src == (true, g),
        _ => false,
    }
}

/// The `let`s of a variable that `stmt` runs at most once: in the value
/// blocks it evaluates and the branches it takes, never in a loop body, a
/// closure or an operand that may run them again.
fn once_lets<'s>(stmt: &'s Stmt, out: &mut Vec<&'s Stmt>) {
    fn stmts<'s>(block: &'s Block, out: &mut Vec<&'s Stmt>) {
        for inner in &block.stmts {
            if let Stmt::Let(_, Some(init)) = inner {
                if matches!(
                    moved_source(init).kind,
                    ExprKind::Local(_) | ExprKind::Global(_)
                ) {
                    out.push(inner);
                }
            }
            once_lets(inner, out);
        }
    }
    // The value blocks `e` runs at most once: its own, and those of the
    // operands it evaluates once, such as a call's arguments (`edges.sorted(by:
    // ..).adjacent` sorts its own copy in the generator's argument), not a
    // fill's per-cell value.
    fn value<'s>(e: &'s Expr, out: &mut Vec<&'s Stmt>) {
        match &e.kind {
            ExprKind::Do(block) => stmts(block, out),
            ExprKind::Fill(..) => {}
            _ => walk::children(e, &mut |child| value(child, out)),
        }
    }
    match stmt {
        Stmt::Let(_, Some(e))
        | Stmt::Assign(_, _, e)
        | Stmt::Expr(e)
        | Stmt::Give(e)
        | Stmt::Return(Some(e))
        | Stmt::For(_, _, Iter::Seq(e, _) | Iter::RangeVal(e, _), _) => value(e, out),
        Stmt::If(cond, then, otherwise) => {
            value(cond, out);
            stmts(then, out);
            stmts(otherwise, out);
        }
        _ => {}
    }
}

/// Whether the variable `src`, which `block` sets before statement `k`
/// (a `let`, or a top-level global's first store in `main`), is read for
/// the last time by that statement, once: read nowhere after it nor
/// elsewhere in it, by no closure or fn, and held by no earlier value
/// that could point into it (a view, a reference).
pub(crate) fn dies_in(
    program: &Program,
    block: &Block,
    k: usize,
    src: (bool, usize),
    callee: Option<FnId>,
) -> bool {
    let before = &block.stmts[..k];
    let set = before.iter().any(|stmt| match (stmt, src) {
        (Stmt::Let(s, _), (false, slot)) => *s == slot,
        (Stmt::Assign(place, AssignOp::Set, _), (true, g)) => {
            (std::ptr::eq(block, &program.main) || std::ptr::eq(block, &program.setup))
                && place.kind == PlaceKind::Global(g)
        }
        _ => false,
    });
    if !set {
        return false;
    }
    let count = |b: &Block| {
        let mut n = 0;
        walk::deep_exprs(b, &mut |e| n += usize::from(reads_whole(e, src)));
        n + usize::from(Touch::of_block(b).writes.contains(&src))
    };
    let rest = Block {
        stmts: block.stmts[k + 1..].to_vec(),
    };
    let here = Block {
        stmts: vec![block.stmts[k].clone()],
    };
    // A closure or fn could run at any later point and read it.
    let elsewhere = match src {
        (false, slot) => {
            let mut captured = false;
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Closure(cid) = e.kind {
                    captured |= program.closures[cid]
                        .captures
                        .iter()
                        .any(|c| !c.is_global && c.outer == slot);
                }
            });
            captured || mentions(program, &rest.stmts, slot).any
        }
        (true, g) => {
            count(&rest) > 0
                || (std::ptr::eq(block, &program.setup) && count(&program.main) > 0)
                || program
                    .fns
                    .iter()
                    .enumerate()
                    .any(|(id, f)| Some(id) != callee && count(&f.body) > 0)
                || program.closures.iter().any(|c| {
                    count(&c.body) > 0
                        || c.captures.iter().any(|cap| cap.is_global && cap.outer == g)
                })
        }
    };
    let mut held = false;
    walk::deep_stmts(
        &Block {
            stmts: before.to_vec(),
        },
        &mut |stmt| match stmt {
            Stmt::Let(_, Some(e)) | Stmt::Assign(_, _, e) => {
                held |= !e.ty.is_scalar() && !reads_whole(e, src) && reads_unbuilt(e, src)
            }
            _ => {}
        },
    );
    let inside = callee.map_or(0, |f| count(&program.fns[f].body));
    !elsewhere && !held && count(&here) + inside == 1
}

/// Whether `e` reads `src` whole other than inside a value a block builds
/// from scratch, which holds no reference into it.
fn reads_unbuilt(e: &Expr, src: (bool, usize)) -> bool {
    if built_fresh(e) && !e.ty.holds_view() {
        return false;
    }
    if reads_whole(e, src) {
        return true;
    }
    let mut found = false;
    if let ExprKind::Do(block) = &e.kind {
        walk::deep_exprs(block, &mut |inner| found |= reads_whole(inner, src));
    }
    walk::children(e, &mut |child| found |= reads_unbuilt(child, src));
    found
}

/// A value a block builds from scratch and gives, `[c.x for c in cs]`:
/// it holds no reference into what it read.
fn built_fresh(e: &Expr) -> bool {
    let ExprKind::Do(block) = &e.kind else {
        return false;
    };
    let Some(Stmt::Give(given)) = block.stmts.last() else {
        return false;
    };
    let ExprKind::Local(built) = given.kind else {
        return false;
    };
    block
        .stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::Let(slot, _) if *slot == built))
}

/// The `let x = g` statements of a fn that read the global `g` for the
/// last time: a statement of `setup` or `main` calls the fn, its only
/// call, after that block sets `g`, and the fn reads `g` once, in a `let` it runs once,
/// while nothing else reads `g` (`var room = room_of` in a fn that builds
/// a table once). Such a `let` may move `g`.
pub(crate) fn dying_global_lets(program: &Program) -> BTreeSet<*const Stmt> {
    let mut calls: BTreeMap<FnId, usize> = BTreeMap::new();
    let mut bodies = vec![&program.setup, &program.main];
    bodies.extend(program.fns.iter().map(|f| &f.body));
    bodies.extend(program.closures.iter().map(|c| &c.body));
    for b in bodies {
        walk::deep_exprs(b, &mut |e| {
            if let ExprKind::Call(f, _) = e.kind {
                *calls.entry(f).or_default() += 1;
            }
        });
    }
    let mut out = BTreeSet::new();
    for block in [&program.setup, &program.main] {
        for (k, stmt) in block.stmts.iter().enumerate() {
            let (Stmt::Let(_, Some(e)) | Stmt::Assign(_, AssignOp::Set, e) | Stmt::Expr(e)) = stmt
            else {
                continue;
            };
            let ExprKind::Call(f, _) = e.kind else {
                continue;
            };
            if calls.get(&f) != Some(&1)
                || program.is_hook(f)
                || program.entry_fns.contains(&f)
                || program.judge_fns.contains(&f)
            {
                continue;
            }
            let mut found = Vec::new();
            for inner in &program.fns[f].body.stmts {
                if let Stmt::Let(_, Some(init)) = inner {
                    if matches!(in_place(init).kind, ExprKind::Global(_)) {
                        found.push(inner);
                    }
                }
                once_lets(inner, &mut found);
            }
            for inner in found {
                let Stmt::Let(_, Some(init)) = inner else {
                    continue;
                };
                let src =
                    walk::path_root(in_place(init)).filter(|_| walk::is_simple(in_place(init)));
                if crate::lend::holds_heap(program, &init.ty)
                    && !matches!(init.ty, Ty::Fn(..))
                    && src.is_some_and(|src| src.0 && dies_in(program, block, k, src, Some(f)))
                {
                    out.insert(inner as *const Stmt);
                }
            }
        }
    }
    out
}

/// `xs.extend(src)` may read `src` in place when nothing in it can reach `xs`
/// (inserting a vector's own range into it is undefined) or run other code.
pub(crate) fn extend_borrows(place: &cpl_ir::Place, src: &Expr) -> bool {
    let a = cpl_ir::effects::Access::of_expr(src);
    !a.opaque && !a.values && !a.mutates && !a.reads.contains(&crate::walk::root(place))
}
