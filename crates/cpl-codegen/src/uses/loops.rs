//! How code runs: its loops, how far its labeled jumps reach, and its size.

use crate::walk;
use cpl_ir::{Block, Iter, LoopId, Stmt};
use std::collections::BTreeSet;

/// A loop's head.
#[derive(Clone, Copy)]
pub(crate) enum Loop<'a> {
    /// `for _ in iter`.
    For(&'a Iter),
    While,
}

/// Every loop in `block`, at any depth: its id and head.
pub(crate) fn each_loop(block: &Block, f: &mut dyn FnMut(LoopId, Loop)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::For(id, _, iter, _) => f(*id, Loop::For(iter)),
        Stmt::While(id, ..) => f(*id, Loop::While),
        _ => {}
    });
}

/// The loops `block`'s labeled `break`s and `continue`s name, at any depth.
pub(crate) fn labeled_jumps(block: &Block) -> (BTreeSet<LoopId>, BTreeSet<LoopId>) {
    let (mut breaks, mut continues) = (BTreeSet::new(), BTreeSet::new());
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Break(Some(id)) => {
            breaks.insert(*id);
        }
        Stmt::Continue(Some(id)) => {
            continues.insert(*id);
        }
        _ => {}
    });
    (breaks, continues)
}

/// How many statements `block` holds, at any depth.
pub(crate) fn stmt_count(block: &Block) -> usize {
    let mut count = 0;
    walk::deep_stmts(block, &mut |_| count += 1);
    count
}
