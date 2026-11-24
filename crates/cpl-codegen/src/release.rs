//! Freeing a list of `main` once a loop has read it for the last time, so the
//! memory it held is free for what the rest of the run builds.

use std::collections::BTreeSet;

use cpl_ir::{Block, Expr, ExprKind, Iter, PlaceKind, Stmt, Ty};

use crate::emit::Emitter;
use crate::names::global_ident;
use crate::uses::walked::{closure_names, names_after, used_last, walked_last};

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
                    && self.owns_heap_storage(*g)
            })
            .collect()
    }

    /// Whether global `g` is laid out on the heap (a `std::vector` or a
    /// `cpl::Grid`), not in a fixed `std::array` that holds nothing to free.
    fn owns_heap_storage(&self, g: usize) -> bool {
        self.fixed_array_len(cpl_ir::ElemLoc::Global(g)).is_none()
            && self.ty_fixed_len(&self.program.globals[g].ty).is_none()
    }

    /// The lists `main` holds as its own locals that statement `k` names for
    /// the last time, in a call's argument or a loop's body, say, other than
    /// the walks [`Emitter::released_at`] frees: each is emptied right after
    /// the statement. Nothing still live may point into them: no later
    /// statement names a variable that may borrow ([`Ty::may_borrow`]),
    /// and no closure names them; a view dropped inside the statement is gone.
    pub(crate) fn dead_after(&self, block: &Block, k: usize) -> BTreeSet<usize> {
        if self.fn_id.is_some()
            || self.closure_id.is_some()
            || self.in_setup
            || !std::ptr::eq(block, &self.program.main)
        {
            return BTreeSet::new();
        }
        let lends = names_after(block, k, &|ty| {
            ty.may_borrow(self.program)
                || matches!(ty, &Ty::Struct(s) if self.lent_fields.fields.range((s, 0)..=(s, usize::MAX)).next().is_some())
        });
        if lends {
            return BTreeSet::new();
        }
        let walked = walked_last(block, k);
        used_last(block, k)
            .into_iter()
            .filter(|g| {
                !walked.contains(g)
                    && self.main_only.contains(g)
                    && !self.lent_fields.held_globals.contains(g)
                    && !self.lent_main.contains(g)
                    && !self.lent_main_roots(*g)
                    && !closure_names(self.program, *g)
                    && matches!(self.program.globals[*g].ty, Ty::List(_) | Ty::Array { .. })
                    && self.owns_heap_storage(*g)
            })
            .collect()
    }

    /// Whether a `main` global kept as a view ([`Emitter::lends_main_let`]) points into `g`.
    fn lent_main_roots(&self, g: usize) -> bool {
        self.program.main.stmts.iter().any(|s| {
            matches!(s, Stmt::Assign(p, _, init)
                if matches!(p.kind, PlaceKind::Global(id) if self.lent_main.contains(&id))
                    && crate::stmt::place_root(init).is_none_or(|(global, root)| !global || root == g))
        })
    }

    /// After statement `k` of `main`: empty the lists [`Emitter::dead_after`] found.
    pub(crate) fn release_dead(&self, dead: &BTreeSet<usize>, out: &mut String) {
        for &g in dead {
            let name = global_ident(self.program, g);
            self.line(out, &format!("decltype({name})().swap({name});"));
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
