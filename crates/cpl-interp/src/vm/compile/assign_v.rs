use super::Compiler;
use crate::vm::ir::*;
use cpl_ir::{AssignOp, Expr, Place, PlaceKind};

impl<'p> Compiler<'p, '_> {
    /// `place op= value` through the place's boxed value: one `AssignPlaceV`
    /// on its path. A local held in integer registers is boxed for it.
    pub(super) fn try_compile_assign_op_v(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        let undo = self.code_mark();
        let (global, slot) = cpl_ir::walk::root(place);
        let (root, held) = if global {
            (PlaceRoot::Global(slot as u16), None)
        } else {
            match self.slot_loc(slot) {
                SlotLoc::V(v) => (PlaceRoot::LocalV(v), None),
                loc => {
                    let v = self.box_slot(loc, place.span);
                    (PlaceRoot::LocalV(v), Some((loc, v)))
                }
            }
        };
        let path = if matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) {
            let Ok(idx) = u16::try_from(self.places.len()) else {
                return false;
            };
            let (steps, keys) = (Vec::new(), Vec::new());
            self.places.push(PlacePath { root, steps, keys });
            idx
        } else {
            match self.place_path_rooted(place, false, held.map(|h| h.1), false) {
                Some(path) => path,
                None => {
                    self.rewind(undo);
                    return false;
                }
            }
        };
        let r_val = self.compile_expr_v(value);
        self.emit(Op::AssignPlaceV { path, r_val, op }, place.span);
        if r_val >= self.num_local_v {
            self.emit(Op::ClearV { reg: r_val }, place.span);
        }
        if let Some((loc, v)) = held {
            self.unbox_slot(loc, v, place.span);
            self.emit(Op::ClearV { reg: v }, place.span);
        }
        true
    }
}
