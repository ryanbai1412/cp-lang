//! Uses: what may happen to a heap binding while it lives, the one source
//! for whether a read lends (`const&`), copies or moves.
//!
//! A binding lives from its `let` to the last statement that mentions it,
//! not to the end of its block: a write to its source after that last read
//! cannot change what it reads. A mention that could keep a reference to it
//! past that statement (a `let` built from it, a closure capturing it, a
//! block's value) keeps it live to the end of the block.

use crate::lend::{copied_at_test, Tail};
use crate::walk::{self, Effects as Stores};
use cpl_ir::{Arg, AssignOp, Block, Builtin, Expr, ExprKind, FnId, PlaceKind, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) use cpl_ir::effects::Effects;

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
    /// Why it may outlive its home's frame, when it may.
    #[allow(dead_code)]
    pub(crate) escapes: Option<&'static str>,
}

pub(crate) const RETURNED: &str = "it is returned";
pub(crate) const STORED: &str = "it is stored in another variable";
pub(crate) const KEPT: &str = "a closure that may be kept captures it";

/// The Uses table: each heap binding's [`Use`] and each fn's [`Effects`],
/// computed once per program.
#[derive(Default)]
pub(crate) struct Uses {
    bindings: BTreeMap<(Home, usize), Use>,
    effects: Vec<Effects>,
}

impl Uses {
    pub(crate) fn of(program: &Program, devirt: &crate::devirt::Devirt, traps: bool) -> Uses {
        let target = |home: Option<FnId>, callee: &Expr| devirt.target(home, callee);
        let table = cpl_ir::effects::Table::with_targets(program, traps, &target);
        let effects = (0..program.fns.len())
            .map(|f| table.effects(f).clone())
            .collect();
        let mut uses = Uses {
            bindings: BTreeMap::new(),
            effects,
        };
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
        let mut escapes = BTreeMap::new();
        mutated.extend(muts.iter().copied());
        for block in blocks {
            mutated.extend(Stores::of_block(block).locals);
            moved.extend(crate::lend::whole_reads(block));
            let mut called = BTreeSet::new();
            walk::deep_stmts(block, &mut |stmt| match stmt {
                Stmt::Let(s, Some(e)) if matches!(e.kind, ExprKind::Closure(_)) => {
                    if crate::lend::only_called(program, block, *s, 3) {
                        called.insert(e as *const Expr);
                    }
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) => {
                    if let ExprKind::Local(s) = e.kind {
                        escapes.entry(s).or_insert(RETURNED);
                    }
                }
                Stmt::Assign(_, _, e) => {
                    if let ExprKind::Local(s) = e.kind {
                        escapes.entry(s).or_insert(STORED);
                    }
                }
                _ => {}
            });
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Call(f, args) = &e.kind {
                    for (j, arg) in args.iter().enumerate() {
                        if let Arg::Value(v) = arg {
                            if matches!(v.kind, ExprKind::Closure(_))
                                && crate::lend::fn_param_only_called(program, *f, j)
                            {
                                called.insert(v as *const Expr);
                            }
                        }
                    }
                }
            });
            walk::deep_exprs(block, &mut |e| {
                let ExprKind::Closure(cid) = e.kind else {
                    return;
                };
                for c in program.closures[cid]
                    .captures
                    .iter()
                    .filter(|c| !c.is_global)
                {
                    if c.by_ref {
                        mutated.insert(c.outer);
                    } else if !called.contains(&(e as *const Expr)) {
                        escapes.entry(c.outer).or_insert(KEPT);
                    }
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
                escapes: escapes.get(&slot).copied(),
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
        &self.effects[f]
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
        any: Stores::of_block(&block).locals.contains(&slot),
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

/// Whether `e` mentions the local `slot`, directly or through a closure.
pub(crate) fn mentions_any(program: &Program, e: &Expr, slot: usize) -> bool {
    mentions(program, &[Stmt::Give(e.clone())], slot).any
}

/// The statements of `rest` (those after a `let` of `slot`) while `slot`
/// lives: up to the last one that mentions it.
pub(crate) fn live<'s>(program: &Program, rest: &'s [Stmt], slot: usize) -> &'s [Stmt] {
    if mentions(program, rest, slot).kept {
        return rest;
    }
    let end = rest
        .iter()
        .rposition(|stmt| mentions(program, std::slice::from_ref(stmt), slot).any)
        .map_or(0, |i| i + 1);
    &rest[..end]
}

/// Whether the local `slot`, which statement `k` of `block` reads, is
/// dead after it: the block declared it (the next pass of a loop declares
/// it afresh), no closure of the block can reach it, and no later
/// statement mentions it. That read may move it.
pub(crate) fn dead_after(program: &Program, block: &Block, k: usize, slot: usize) -> bool {
    let declared = block.stmts[..k]
        .iter()
        .any(|stmt| matches!(stmt, Stmt::Let(s, _) if *s == slot));
    declared && unread_after(program, block, k, slot)
}

/// True when nothing after statement `k` of `block`, nor a closure in it,
/// reads `slot`.
pub(crate) fn unread_after(program: &Program, block: &Block, k: usize, slot: usize) -> bool {
    let mut closure = false;
    walk::deep_exprs(block, &mut |e| {
        closure |= matches!(e.kind, ExprKind::Closure(_))
    });
    !closure && !mentions(program, &block.stmts[k + 1..], slot).any
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
pub(crate) const IN_CLOSURE: &str = "a `let` inside a closure owns its value";
pub(crate) const NAME_MOVED: &str = "the name is moved later, so it owns its value";
/// Why a `.clone()` of a place copies: it asks for a second value.
pub(crate) const CLONED: &str = "written `.clone()`, so it is a second value";
pub(crate) const WHOLE: &str = "a `let` of a whole variable is a second value, not a second name";

impl crate::emit::Emitter<'_> {
    /// Where the code being written binds its locals.
    pub(crate) fn home(&self) -> Home {
        match (self.closure_id, self.fn_id) {
            (Some(cid), _) => Home::Closure(cid),
            (None, Some(id)) => Home::Fn(id),
            (None, None) => Home::Top,
        }
    }

    /// Whether the `let` of `slot` can be a reference to the place `init`
    /// names instead of a copy of it: it is a heap value outside a closure,
    /// and nothing changes either for as long as the name lives
    /// ([`Emitter::source_change`]). `tail` is what the statements after the
    /// `let` in its block do; without one the whole fn is held to that.
    pub(crate) fn lends(&self, slot: usize, init: &Expr, tail: Option<&Tail>) -> bool {
        self.closure_id.is_none()
            && !init.ty.is_scalar()
            && !matches!(init.ty, Ty::Fn(..))
            && !self.spent.contains(&slot)
            && match init.kind {
                ExprKind::Field(..) | ExprKind::Index(..) | ExprKind::Unwrap(..) => true,
                // The optional an `if let` or `while let` tests.
                ExprKind::Local(_) | ExprKind::Global(_) => self.locals()[slot].name.is_empty(),
                _ => false,
            }
            && self.source_change(slot, init, tail).is_none()
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
        if self.closure_id.is_some() {
            return IN_CLOSURE;
        }
        if let Some(why) = self.source_change(slot, init, tail) {
            return why;
        }
        match (self.spent.contains(&slot), whole) {
            (true, _) => NAME_MOVED,
            (false, true) => WHOLE,
            (false, false) => SOURCE_MAY_CHANGE,
        }
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
        let Some((global, root)) = crate::stmt::place_root(init) else {
            return Some(SOURCE_CALLED);
        };
        let home = self.home();
        if self.uses.of_binding(home, slot).mutated
            || self.facts.stores((self.fn_id, false, slot)) > 1
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
                self.facts.stores((self.fn_id, false, root)) != 1,
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
                Tail::of(self.program, &body.stmts).calls,
                SOURCE_CALLED,
            ));
        };
        if tail.opaque
            && !(named_local && !crate::lend::closure_writes(self.program, self.fn_id, root))
        {
            return Some(SOURCE_CALLED);
        }
        let effects = &tail.effects;
        match global {
            // A parameter may be another name for the global.
            true => changed(
                effects.globals.contains(&root) || effects.locals.iter().any(|l| *l < params),
                SOURCE_SET,
            )
            .or(changed(
                effects
                    .calls
                    .iter()
                    .any(|f| self.uses.effects(*f).writes.contains(&root)),
                SOURCE_CALLED,
            )),
            // No fn sees a local it isn't passed `mut`, and that is a store.
            // Nor may it read the local whole: that read could move it.
            false if named_local => changed(effects.locals.contains(&root), SOURCE_SET)
                .or(changed(tail.whole.contains(&root), SOURCE_READ_WHOLE)),
            // A parameter may be another name for a global a call writes,
            // or for another parameter a store sets.
            false => changed(
                effects.locals.iter().any(|l| *l == root || *l < params),
                SOURCE_SET,
            )
            .or(changed(
                effects.calls.iter().any(|f| {
                    let e = self.uses.effects(*f);
                    !e.writes.is_empty() || e.unknown_call
                }),
                SOURCE_CALLED,
            )),
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
            ..Tail::of(self.program, live)
        })
    }
}

/// True when statement `k` of `block` reads a heap variable for the last
/// time and may move it: `x = y` or `let x = y` with `y` dead after it
/// (`uses::dead_after`), or a `let` whose source is set again before
/// anything reads it.
pub(crate) fn last_read(program: &Program, block: &Block, k: usize) -> bool {
    if let Stmt::Let(_, Some(init)) = &block.stmts[k] {
        return crate::lend::overwritten_before_read(init, &block.stmts[k + 1..])
            || matches!(init.kind, ExprKind::Local(src)
                if dead_after(program, block, k, src));
    }
    let Stmt::Assign(place, AssignOp::Set, value) = &block.stmts[k] else {
        return false;
    };
    let (ExprKind::Local(slot), PlaceKind::Local(_) | PlaceKind::Global(_)) =
        (&value.kind, &place.kind)
    else {
        return false;
    };
    walk::root(place) != (false, *slot) && dead_after(program, block, k, *slot)
}

/// Value parameters whose one read copies them into a local the body
/// changes, `var l = limbs`: taken by value, the copy moves in instead, and
/// a caller at its last use moves its own value in.
pub(crate) fn copied_once(f: &cpl_ir::Function) -> BTreeSet<usize> {
    let stored = walk::Effects::of_block(&f.body).locals;
    let mut reads = vec![0usize; f.params.len()];
    walk::deep_exprs(&f.body, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            if s < reads.len() {
                reads[s] += 1;
            }
        }
    });
    f.body
        .stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Let(
                to,
                Some(Expr {
                    kind: ExprKind::Local(p),
                    ty,
                    ..
                }),
            ) if *p < f.params.len()
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
