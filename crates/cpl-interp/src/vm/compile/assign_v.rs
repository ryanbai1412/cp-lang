use super::Compiler;
use crate::vm::ir::*;
use cpl_ir::{AssignOp, Expr, Place, PlaceKind};

impl<'p> Compiler<'p, '_> {
    /// `place op= value` for a boxed value (a float, a wide int, a list, a
    /// string): one `AssignPlaceV` on the place's path.
    pub(super) fn try_compile_assign_op_v(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        let root = match place.kind {
            PlaceKind::Local(slot) => match self.slot_loc(slot) {
                SlotLoc::V(v) => Some(PlaceRoot::LocalV(v)),
                _ => return false,
            },
            PlaceKind::Global(gid) => Some(PlaceRoot::Global(gid as u16)),
            _ => None,
        };
        let path = match root {
            Some(root) => {
                let Ok(idx) = u16::try_from(self.places.len()) else {
                    return false;
                };
                self.places.push(PlacePath {
                    root,
                    steps: Vec::new(),
                    keys: Vec::new(),
                });
                idx
            }
            None => {
                let (ops, spans, places) = (self.ops.len(), self.spans.len(), self.places.len());
                let Some(path) = self.compile_place_path(place) else {
                    self.ops.truncate(ops);
                    self.spans.truncate(spans);
                    self.places.truncate(places);
                    return false;
                };
                path
            }
        };
        let r_val = self.compile_expr_v(value);
        self.emit(Op::AssignPlaceV { path, r_val, op }, place.span);
        if r_val >= self.num_local_v {
            self.emit(Op::ClearV { reg: r_val }, place.span);
        }
        true
    }
}
