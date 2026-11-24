//! Freeing a list once the run has read it for the last time, so the
//! memory it held is free for what the rest of the run builds.

use std::collections::BTreeSet;

use cpl_ir::{Block, Builtin, Expr, ExprKind, Iter, Stmt, Ty};

use crate::emit::Emitter;
use crate::names::global_ident;
use crate::uses::walked::{read_last, walked_last};

impl Emitter<'_> {
    /// The lists `main` holds as its own locals that statement `k` walks
    /// for the last time ([`walked_last`]) and nothing lends or holds:
    /// each is emptied right after the loop that walks it.
    pub(crate) fn released_at(&self, block: &Block, k: usize) -> BTreeSet<usize> {
        if self.fn_id.is_some()
            || self.closure_id.is_some()
            || self.in_setup
            || !std::ptr::eq(block, &self.program.main)
        {
            return BTreeSet::new();
        }
        walked_last(block, k)
            .into_iter()
            .filter(|g| {
                self.main_only.contains(g)
                    && !self.lent_fields.held_globals.contains(g)
                    && matches!(self.program.globals[*g].ty, Ty::List(_) | Ty::Array { .. })
            })
            .collect()
    }

    /// After statement `k` of `block`, frees each list or string it read for
    /// the last time ([`read_last`]) that the C++ owns, unless what the
    /// statement keeps could point into it: `let t = ups.sorted(by: f)`
    /// then `taken = t.take(k).collect()` frees `t` before the rest runs.
    pub(crate) fn release_read(&mut self, block: &Block, k: usize, out: &mut String) {
        if self.closure_id.is_some() || self.in_setup {
            return;
        }
        // What the statement keeps must be a value of its own: a place it
        // names (`let r = rows[1]`) or what a lending fn gives may point
        // into what it read.
        let kept = match &block.stmts[k] {
            Stmt::Let(_, Some(e)) | Stmt::Assign(_, _, e) => Some(e),
            _ => None,
        };
        let fresh = |e: &Expr| {
            e.ty.is_scalar()
                || (matches!(e.ty, Ty::List(_) | Ty::Str)
                    && !e.ty.holds_view()
                    && match &e.kind {
                        ExprKind::Call(f, _) => !self.lent_roots.contains_key(f),
                        ExprKind::Builtin(b, _) => {
                            !matches!(b, Builtin::ArrayToList | Builtin::ListToArray)
                        }
                        _ => false,
                    })
        };
        if kept.is_some_and(|e| !fresh(e)) {
            return;
        }
        let main = std::ptr::eq(block, &self.program.main);
        for (global, id) in read_last(self.program, block, k) {
            let name = match global {
                true if main
                    && self.main_only.contains(&id)
                    && !self.released.contains(&id)
                    && !self.lent_fields.held_globals.contains(&id)
                    && matches!(self.program.globals[id].ty, Ty::List(_) | Ty::Str) =>
                {
                    global_ident(self.program, id)
                }
                false
                    if !self.ref_bound.contains(&id)
                        && !self.hoisted.contains(&id)
                        && !self.reused.contains(&id)
                        && matches!(self.locals()[id].ty, Ty::List(_) | Ty::Str) =>
                {
                    self.names[id].clone()
                }
                _ => continue,
            };
            self.line(out, &format!("{{ auto cpl_freed = std::move({name}); }}"));
        }
    }

    /// After a loop over a list [`Emitter::released_at`] found: empty it.
    pub(crate) fn release_walked(&self, iter: &Iter, out: &mut String) {
        if let Iter::Seq(
            Expr {
                kind: ExprKind::Global(g),
                ..
            },
            _,
        ) = iter
        {
            if self.released.contains(g) {
                let name = global_ident(self.program, *g);
                self.line(out, &format!("decltype({name})().swap({name});"));
            }
        }
    }
}
