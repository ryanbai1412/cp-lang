//! How code runs: its loops, how far its labeled jumps reach, and its size.

use crate::walk;
use cpl_ir::{Arg, Block, Builtin, Expr, ExprKind, Iter, LoopId, Slot, Stmt};
use std::collections::BTreeSet;

/// A loop's head.
#[derive(Clone, Copy)]
pub(crate) enum Loop<'a> {
    /// `for slot in iter`.
    For(Slot, &'a Iter),
    While,
}

/// Every loop in `block`, at any depth: its id and head.
pub(crate) fn each_loop(block: &Block, f: &mut dyn FnMut(LoopId, Loop)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::For(id, slot, iter, _) => f(*id, Loop::For(*slot, iter)),
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

/// `j` of a read `a[j.prev!]`, and the variable `a`.
pub(crate) fn prev_cell(e: &Expr) -> Option<((bool, usize), &Expr)> {
    let ExprKind::Index(base, idx) = &e.kind else {
        return None;
    };
    let [i] = idx.as_slice() else {
        return None;
    };
    let ExprKind::Unwrap(v, true) = &i.kind else {
        return None;
    };
    let ExprKind::Builtin(Builtin::PosPrev, args) = &v.kind else {
        return None;
    };
    let at = match base.kind {
        ExprKind::Local(slot) => (false, slot),
        ExprKind::Global(id) => (true, id),
        _ => return None,
    };
    Some((at, args.first()?))
}

/// The `slot` of a read `at[slot.prev!]` in the body of loop `id` over
/// `slot`, when the body stores to `at` once, never takes it `mut` or
/// rebinds it, and no `break` or `continue` ends a round early: then
/// every round's one store is the cell the next round reads.
pub(crate) fn carried_read(
    block: &Block,
    id: LoopId,
    slot: Slot,
    at: (bool, usize),
) -> Option<&Expr> {
    let (mut read, mut stores, mut other) = (None, 0, false);
    walk::deep_exprs(block, &mut |e| {
        if let Some((a, j)) = prev_cell(e) {
            if a == at && matches!(j.kind, ExprKind::Local(s) if s == slot) {
                read.get_or_insert(j);
            }
        }
        other |= match &e.kind {
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => args
                .iter()
                .any(|a| matches!(a, Arg::Mut(p) if walk::root(p) == at)),
            ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, ..) | ExprKind::MutSlice(p, ..) => {
                walk::root(p) == at
            }
            _ => false,
        };
    });
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Assign(p, ..) if walk::root(p) == at => stores += 1,
        Stmt::Break(to) | Stmt::Continue(to) => other |= to.is_none() || *to == Some(id),
        Stmt::Let(s, _) | Stmt::For(_, s, ..) => other |= (false, *s) == at,
        _ => {}
    });
    read.filter(|_| stores == 1 && !other)
}

/// Whether `block` has a `break`, `continue` or `return` at any depth: a
/// way out of it other than its end.
pub(crate) fn has_jump(block: &Block) -> bool {
    let mut found = false;
    walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(_));
    });
    found
}

/// Every statement of `block` at any depth, in order, by address: the sites
/// a profile keys.
pub(crate) fn stmt_sites(block: &Block) -> Vec<*const Stmt> {
    let mut sites = Vec::new();
    walk::deep_stmts(block, &mut |s| sites.push(s as *const Stmt));
    sites
}

/// Whether `block` has a `break` or a `return` at any depth: a way out of the
/// loop it is the body of (a `continue` stays in it).
pub(crate) fn breaks_or_returns(block: &Block) -> bool {
    let mut found = false;
    walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::Break(_) | Stmt::Return(_));
    });
    found
}

/// Whether `block` jumps anywhere but its own end, at any depth: a `break`,
/// `continue`, `return` or `give`.
pub(crate) fn jumps_out(block: &Block) -> bool {
    let mut found = false;
    walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(
            stmt,
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(_) | Stmt::Give(_)
        );
    });
    found
}

/// An innermost loop that walks a grid's last axis, `c[i, l] max= a + b[j, l]` in `for l`.
/// Each cell's address is its own base plus `l`, and at -O2 g++ steps one pointer per grid
/// or an index plus a compare, by register pressure elsewhere in the function: a round of
/// about 8 uops then takes 2 or 3 cycles. Two cells a round take one loop's overhead
/// between them, as a row of `List<List<Int>>` does in hand C++. An AVX2 judge unrolls already.
pub(crate) fn sweeps_grid_row(slot: usize, body: &Block) -> bool {
    let row = |ty: &cpl_ir::Ty, at: &[Expr]| {
        matches!(ty, cpl_ir::Ty::Array { dims, .. } if dims.len() > 1)
            && matches!(at.last().map(|e| &e.kind), Some(ExprKind::Local(s)) if *s == slot)
    };
    let (mut nested, mut sweeps) = (false, false);
    walk::deep_stmts(body, &mut |stmt| match stmt {
        Stmt::For(..) | Stmt::While(..) => nested = true,
        Stmt::Assign(place, ..) => {
            if let cpl_ir::PlaceKind::Index(base, at) = &place.kind {
                sweeps |= row(&base.ty, at);
            }
        }
        _ => {}
    });
    walk::deep_exprs(body, &mut |e| {
        if let ExprKind::Index(base, at) = &e.kind {
            sweeps |= row(&base.ty, at);
        }
    });
    sweeps && !nested
}
