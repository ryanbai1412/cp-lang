//! Freeing a list of `main` once a loop has read it for the last time, so the
//! memory it held is free for what the rest of the run builds.

use std::collections::BTreeSet;

use cpl_ir::{Block, Expr, ExprKind, Iter, Ty};

use crate::emit::Emitter;
use crate::names::global_ident;
use crate::uses::walked::walked_last;

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
