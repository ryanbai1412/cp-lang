//! A loop whose one store to an array is `a[j] = v`, last in its body,
//! and that reads `a[j.prev!]`, keeps the cell it stored in a local: the
//! read is that local, not a load of what the round before stored. The
//! C++ compiler leaves that store-to-load chain in memory, where it is
//! the loop's critical path (ioi00p1's row DP: 23.0 -> 10.3 ms against 17.9 ms
//! for the accepted C++). The cell is carried wide, before its narrow
//! store, so the chain has no truncation or zero-extension either.
//!
//! Nothing else may store to `a`, take it `mut`, or leave the round
//! before the store, so every round stores the cell the next one reads.

use crate::emit::Emitter;
use crate::uses::effects::{writes_nothing, Touch};
use crate::uses::loops::{carried_read, prev_cell};
use cpl_ir::{AssignOp, Block, Expr, ExprKind, Iter, LoopId, Place, PlaceKind, Stmt, Ty};

pub struct Carry {
    root: (bool, usize),
    slot: usize,
    name: String,
}

impl<'p> Emitter<'p> {
    /// The cell `for slot in iter: body` carries, with the C++ that
    /// declares it ahead of the loop.
    pub(crate) fn loop_carry(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &Iter,
        body: &Block,
    ) -> Option<(Carry, String)> {
        if self.closure_id.is_some() || !self.facts.stable(self.facts_local(slot)) {
            return None;
        }
        let (lo, hi, closed) = match iter {
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed,
            } => (lo, hi, *closed),
            Iter::RangeVal(e, false) => match &e.kind {
                ExprKind::Range { lo, hi } => (&**lo, &**hi, false),
                _ => return None,
            },
            _ => return None,
        };
        let Some(Stmt::Assign(place, AssignOp::Set, _)) = body.stmts.last() else {
            return None;
        };
        let PlaceKind::Index(base, idx) = &place.kind else {
            return None;
        };
        let at = match base.kind {
            PlaceKind::Local(slot) => (false, slot),
            PlaceKind::Global(id) => (true, id),
            _ => return None,
        };
        let one_dim = matches!(&base.ty, Ty::Array { dims, .. } if dims.len() == 1);
        let by_slot =
            matches!(idx.as_slice(), [i] if matches!(i.kind, ExprKind::Local(s) if s == slot));
        if !one_dim || !by_slot || place.ty != Ty::Int || !writes_nothing(lo) || !writes_nothing(hi)
        {
            return None;
        }
        if self.dropped_store(place, AssignOp::Set) {
            return None;
        }
        let j = carried_read(body, id, slot, at)?;
        if !self.facts.prev_exists(self.facts_home(), j) {
            return None;
        }
        let effects = Touch::of_block(body);
        if effects.unnamed() {
            return None;
        }
        let params = self.fn_id.map_or(0, |f| self.program.fns[f].params.len());
        let calls_ok = effects.calls.iter().all(|f| {
            self.summaries.get(*f).is_some_and(|s| {
                !s.opaque && !s.calls_values && !(at.0 && s.writes.contains(&at.1))
            })
        });
        if !calls_ok || (!at.0 && at.1 < params && !effects.calls.is_empty()) {
            return None;
        }
        let a = self.place(base);
        let (lo, hi) = (self.expr(lo).text, self.expr(hi).text);
        let name = self.unique("carry");
        let cmp = if closed { "<=" } else { "<" };
        let decl = format!("ll {name} = 0; if (({lo}) {cmp} ({hi})) {name} = {a}[({lo}) - 1];");
        Some((
            Carry {
                root: at,
                slot,
                name,
            },
            decl,
        ))
    }

    /// The local a loop being written keeps `a[j.prev!]` in.
    pub(crate) fn carried(&self, e: &Expr) -> Option<String> {
        let (at, j) = prev_cell(e)?;
        let c = self
            .carries
            .iter()
            .find(|c| c.root == at && matches!(j.kind, ExprKind::Local(s) if s == c.slot))?;
        Some(c.name.clone())
    }

    /// The local `a[j] = v` stores `v` in first, wide, so the next round
    /// reads it without the narrow cell's truncation in its chain.
    pub(crate) fn carried_store(&self, place: &Place) -> Option<String> {
        let PlaceKind::Index(base, idx) = &place.kind else {
            return None;
        };
        let at = match base.kind {
            PlaceKind::Local(slot) => (false, slot),
            PlaceKind::Global(id) => (true, id),
            _ => return None,
        };
        let c = self.carries.iter().find(|c| {
            c.root == at
                && matches!(idx.as_slice(), [i] if matches!(i.kind, ExprKind::Local(s) if s == c.slot))
        })?;
        Some(c.name.clone())
    }
}
