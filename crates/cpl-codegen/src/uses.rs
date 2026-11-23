//! Uses: what may happen to a heap binding while it lives, the one source
//! for whether a read lends (`const&`), copies or moves.
//!
//! A binding lives from its `let` to the last statement that mentions it,
//! not to the end of its block: a write to its source after that last read
//! cannot change what it reads. A mention that could keep a reference to it
//! past that statement (a `let` built from it, a closure capturing it, a
//! block's value) keeps it live to the end of the block.

use crate::lend::{copied_at_test, Tail};
use crate::walk::{self, Effects};
use cpl_ir::{Block, Builtin, Expr, ExprKind, Program, Stmt};

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
        any: Effects::of_block(&block).locals.contains(&slot),
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
