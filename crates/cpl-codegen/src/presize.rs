//! `[f(x) for x in xs]`: the list a comprehension fills takes its source's
//! size up front, so it grows once instead of doubling.

use crate::emit::{Emitter, ADD, PRIMARY};
use crate::walk;
use cpl_ir::{ElemLoc, Expr, ExprKind, Iter, Mutation, PlaceKind, Stmt, Ty};

/// `t.push(e)` as a statement.
fn pushes_to(stmt: &Stmt, slot: usize) -> bool {
    matches!(stmt, Stmt::Expr(Expr { kind: ExprKind::Mutate(Mutation::Push, place, _), .. })
        if place.kind == PlaceKind::Local(slot))
}

impl<'p> Emitter<'p> {
    /// Writes `stmts[k]`, then the `reserve` [`Self::presize`] finds for it.
    pub(crate) fn stmt_presized(&mut self, stmts: &[Stmt], k: usize, out: &mut String) {
        self.stmt(&stmts[k], out);
        if let Some(line) = self.presize(&stmts[k..]) {
            self.line(out, &line);
        }
    }

    /// `t.reserve(n);` for `let t = []` that the next loop pushes to once a
    /// round over `n` rounds it knows ahead: a list or a plain range.
    pub(crate) fn presize(&mut self, stmts: &[Stmt]) -> Option<String> {
        let [Stmt::Let(slot, Some(init)), Stmt::For(_, _, iter, body), ..] = stmts else {
            return None;
        };
        let empty = matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty());
        let loc = ElemLoc::Local(self.current_scope(), *slot);
        if !empty
            || !matches!(init.ty, Ty::List(_))
            || self.reused.contains(slot)
            || (self.pushing == Some(*slot) && self.closure_id.is_none())
            || self.elem_layout(loc).is_some()
            || !self.ty_for_loc(&init.ty, loc).starts_with("std::vector<")
        {
            return None;
        }
        let once = body.stmts.iter().filter(|s| pushes_to(s, *slot)).count() == 1
            && body
                .stmts
                .iter()
                .all(|s| matches!(s, Stmt::Let(..)) || pushes_to(s, *slot));
        if !once {
            return None;
        }
        let n = match iter {
            Iter::Seq(src, _)
                if walk::is_simple(src)
                    && (matches!(src.ty, Ty::List(_) | Ty::ListView(..))
                        || matches!(&src.ty, Ty::Array { dims, .. } if dims.len() == 1)) =>
            {
                format!("{}.size()", self.expr(src).at(PRIMARY))
            }
            Iter::Range {
                lo,
                hi,
                step: None,
                closed,
                ..
            } if [lo, hi]
                .iter()
                .all(|e| walk::is_simple(e) && e.ty == Ty::Int) =>
            {
                let (lo, hi) = (self.expr(lo).at(ADD + 1), self.expr(hi).at(ADD));
                let one = if *closed { " + 1" } else { "" };
                format!("(size_t)std::max<ll>(0, {hi} - {lo}{one})")
            }
            _ => return None,
        };
        Some(format!("{}.reserve({n});", self.names[*slot]))
    }
}
