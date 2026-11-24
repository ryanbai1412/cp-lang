//! The receiver of a method that changes it in place.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Where `method`'s `mut self` lives. A temporary has nowhere to keep
    /// the change, so the error names the method, not an assignment.
    pub(in crate::lower) fn mut_receiver(&mut self, recv: &ast::Expr, method: &str) -> Place {
        // `g.rows[r]` is a line of the grid's own cells, not a slot lent by `GridRows.at`.
        if let ast::ExprKind::Index { base, indices } = &recv.kind {
            if let Some(line) = self.grid_line_place(base, indices, recv.span) {
                return line;
            }
        }
        if let Some(place) = self.lent_receiver(recv) {
            return place;
        }
        if !is_place(recv) && self.builtin_at(recv).is_none() {
            let span = self.span(recv.span);
            self.report(
                recv.span,
                msgs::ir::ASSIGN_TO_LET_CHANGES_ITS_RECEIVER_IN,
                &[&method],
            );
            return Place {
                kind: PlaceKind::Local(0),
                ty: Ty::Error,
                span,
            };
        }
        self.place(recv)
    }

    /// `a.at(i).kids.push(x)`: a receiver at or under a `lends mut` call is the lent slot
    /// itself (R05.6.3). The call's lead runs ahead of the statement, its tail after it.
    fn lent_receiver(&mut self, recv: &ast::Expr) -> Option<Place> {
        let (mut lead, mut tail) = (Vec::new(), Vec::new());
        let (path, alias) = self.lent_path(recv, &mut lead, &mut tail)?;
        let place = self.place(&path);
        self.cx.mut_aliases.remove(&alias);
        self.cx.pending.append(&mut lead);
        self.cx.after.append(&mut tail);
        Some(place)
    }

    /// The place a `mut` argument names; a receiver goes through
    /// [`Self::mut_receiver`].
    pub(in crate::lower) fn mut_arg(
        &mut self,
        value: &ast::Expr,
        param: &str,
        method: &str,
    ) -> Place {
        if param == "self" {
            self.mut_receiver(value, method)
        } else {
            self.place(value)
        }
    }
}
