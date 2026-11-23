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
use cpl_ir::{Arg, Block, Builtin, Expr, ExprKind, FnId, Program, Stmt};
use std::collections::{BTreeMap, BTreeSet};

/// What a call of a fn can do, in the shape `cpl_ir::effects::Effects`
/// will give; built here from the fn summaries and `sink::pure_fns` until
/// that lands. Codegen reads `writes` today; the rest is the agreed shape.
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub(crate) struct Effects {
    /// The globals it may write, itself or through what it calls.
    pub(crate) writes: BTreeSet<usize>,
    /// It reads input or writes output.
    pub(crate) io: bool,
    /// It may trap in this build.
    pub(crate) traps: bool,
    /// It calls a fn value whose target is not known, which may write what
    /// that value captured.
    pub(crate) unknown_call: bool,
}

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
    pub(crate) fn of(
        program: &Program,
        devirt: &crate::devirt::Devirt,
        summaries: &[crate::emit::FnSummary],
        traps: bool,
    ) -> Uses {
        let pure = crate::sink::pure_fns(program, devirt, traps);
        let effects = summaries
            .iter()
            .enumerate()
            .map(|(f, s)| Effects {
                writes: s.writes.clone(),
                io: s.io,
                traps: !pure.contains(&f),
                unknown_call: s.opaque || (s.calls_values && !pure.contains(&f)),
            })
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
    let mut closure = false;
    walk::deep_exprs(block, &mut |e| {
        closure |= matches!(e.kind, ExprKind::Closure(_))
    });
    declared && !closure && !mentions(program, &block.stmts[k + 1..], slot).any
}

impl crate::emit::Emitter<'_> {
    /// What the statements after `stmts[k]` do while the `let` there lives,
    /// when that `let` might lend.
    pub(crate) fn tail_at(&self, stmts: &[Stmt], k: usize) -> Option<Tail> {
        let Stmt::Let(slot, Some(init)) = &stmts[k] else {
            return None;
        };
        if !crate::lend::may_lend(init) {
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
