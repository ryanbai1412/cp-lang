use super::Compiler;
use crate::vm::helpers::*;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{AssignOp, BinOp, CmpOp, Expr, ExprKind, Place, PlaceKind, Ty, UnOp};

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn try_compile_assign(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        if self.try_compile_assign_fast(place, op, value) {
            return true;
        }
        self.try_compile_assign_place_path(place, op, value)
    }

    pub(super) fn try_compile_assign_fast(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        let val_kind = ScalarKind::of_ty(&place.ty, self.program);
        let opt_kind = ScalarKind::of_opt_ty(&place.ty, self.program);
        match (&place.kind, op, val_kind, opt_kind) {
            (PlaceKind::Local(slot), AssignOp::Set, Some(_), _) => {
                let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                    return false;
                };
                let r = self.compile_expr_i(value);
                if r != dst {
                    self.emit(Op::MoveI { dst, src: r }, place.span);
                }
                true
            }
            (PlaceKind::Local(slot), AssignOp::Set, None, Some(_)) => {
                let SlotLoc::OptI { val, some, .. } = self.slot_loc(*slot) else {
                    return false;
                };
                let (rv, rs) = self.compile_expr_opt_i(value);
                if rv != val {
                    self.emit(Op::MoveI { dst: val, src: rv }, place.span);
                }
                if rs != some {
                    self.emit(Op::MoveI { dst: some, src: rs }, place.span);
                }
                true
            }
            (PlaceKind::Local(slot), AssignOp::Set, None, None) => match self.slot_loc(*slot) {
                SlotLoc::SmallTuple { base, len, .. } => {
                    self.compile_expr_small_tuple(value, base, len);
                    true
                }
                SlotLoc::V(dst) => {
                    let r = self.compile_expr_v(value);
                    if r != dst {
                        self.emit(Op::MoveV { dst, src: r }, place.span);
                    }
                    true
                }
                _ => false,
            },
            (PlaceKind::Local(slot), op, Some(ScalarKind::Int), _) => {
                let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                    return false;
                };
                let r = self.compile_expr_i(value);
                self.emit_assign_op_i(dst, dst, r, op, place.span)
            }
            (PlaceKind::Global(gid), AssignOp::Set, Some(kind), _) => {
                let r = self.compile_expr_i(value);
                self.emit(
                    Op::StoreGlobalI {
                        gid: *gid as u16,
                        src: r,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Global(gid), AssignOp::Set, None, Some(kind)) => {
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreGlobalOptI {
                        gid: *gid as u16,
                        val,
                        some,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Global(gid), AssignOp::Set, None, None) => {
                let r = self.compile_expr_v(value);
                self.emit(
                    Op::StoreGlobalV {
                        gid: *gid as u16,
                        src: r,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Global(gid), op, Some(ScalarKind::Int), _) => {
                let cur = self.alloc_i();
                self.emit(
                    Op::LoadGlobalI {
                        dst: cur,
                        gid: *gid as u16,
                    },
                    place.span,
                );
                let r = self.compile_expr_i(value);
                if !self.emit_assign_op_i(cur, cur, r, op, place.span) {
                    return false;
                }
                self.emit(
                    Op::StoreGlobalI {
                        gid: *gid as u16,
                        src: cur,
                        kind: ScalarKind::Int,
                    },
                    place.span,
                );
                true
            }
            // Array[i] = val or Array[i, j] = val (scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, Some(kind), _)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2I {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    r_val,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1I {
                                    gid: *gid as u16,
                                    r_i,
                                    r_val,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2I {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    r_val,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(Op::StoreLArr1I { arr_v, r_i, r_val }, place.span);
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // Array[i] = opt_val or Array[i, j] = opt_val (unboxed ?Int / ?Scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, None, Some(kind))
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let (r_val, r_some) = self.compile_expr_opt_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2OptI {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1OptI {
                                    gid: *gid as u16,
                                    r_i,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let (r_val, r_some) = self.compile_expr_opt_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2OptI {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreLArr1OptI {
                                    arr_v,
                                    r_i,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let (val, some) = self.compile_expr_opt_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1OptI {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                val,
                                some,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // List[i] = val (scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, Some(kind), _)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreGList1I {
                                gid: *gid as u16,
                                r_i,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLList1I {
                                list_v,
                                r_i,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // Array[i] op= val (Int)
            (PlaceKind::Index(base, indices), op, Some(ScalarKind::Int), _)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadGArr2I {
                                    dst: cur,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreGArr2I {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    r_val: cur,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1I {
                                    dst: cur,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreGArr1I {
                                    gid: *gid as u16,
                                    r_i,
                                    r_val: cur,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadLArr2I {
                                    dst: cur,
                                    arr_v,
                                    r_i,
                                    r_j,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreLArr2I {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    r_val: cur,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::LoadLArr1I {
                                    dst: cur,
                                    arr_v,
                                    r_i,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreLArr1I {
                                    arr_v,
                                    r_i,
                                    r_val: cur,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadFieldArr1I {
                                dst: cur,
                                obj_v,
                                field: *field as u16,
                                r_i,
                            },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // List[i] op= val (Int)
            (PlaceKind::Index(base, indices), op, Some(ScalarKind::Int), _)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadGList1I {
                                dst: cur,
                                gid: *gid as u16,
                                r_i,
                            },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreGList1I {
                                gid: *gid as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadLList1I {
                                dst: cur,
                                list_v,
                                r_i,
                            },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLList1I {
                                list_v,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadFieldList1I {
                                dst: cur,
                                obj_v,
                                field: *field as u16,
                                r_i,
                            },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // seq[i].field = val or seq[i].field op= val (scalar field on Array/List of Struct)
            (PlaceKind::Field(obj_place, field), op, Some(kind), _)
                if matches!(obj_place.kind, PlaceKind::Index(..)) =>
            {
                let PlaceKind::Index(base, indices) = &obj_place.kind else {
                    return false;
                };
                if !matches!(base.ty, Ty::Array { .. } | Ty::List(_)) || indices.len() != 1 {
                    return false;
                }
                if op != AssignOp::Set && kind != ScalarKind::Int {
                    return false;
                }
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = if op == AssignOp::Set {
                            self.compile_expr_i(value)
                        } else {
                            let r_rhs = self.compile_expr_i(value);
                            let cur = self.alloc_i();
                            self.emit(
                                Op::LoadGIdxFieldI {
                                    dst: cur,
                                    gid: *gid as u16,
                                    r_i,
                                    field: *field as u16,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            cur
                        };
                        self.emit(
                            Op::StoreGIdxFieldI {
                                gid: *gid as u16,
                                r_i,
                                field: *field as u16,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(seq_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = if op == AssignOp::Set {
                            self.compile_expr_i(value)
                        } else {
                            let r_rhs = self.compile_expr_i(value);
                            let cur = self.alloc_i();
                            self.emit(
                                Op::LoadLIdxFieldI {
                                    dst: cur,
                                    seq_v,
                                    r_i,
                                    field: *field as u16,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            cur
                        };
                        self.emit(
                            Op::StoreLIdxFieldI {
                                seq_v,
                                r_i,
                                field: *field as u16,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Min | AssignOp::Max, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && indices.len() == 1
                    && small_tuple_shape(&place.ty, self.program).is_some() =>
            {
                let (len, _) = small_tuple_shape(&place.ty, self.program).unwrap();
                let is_max = op == AssignOp::Max;
                let src_base = self.next_i;
                self.next_i += len as u16;
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        self.compile_expr_small_tuple(value, src_base, len);
                        self.emit(
                            Op::MinMaxGArr1SmallTuple {
                                gid: *gid as u16,
                                r_i,
                                src_base,
                                len,
                                is_max,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        self.compile_expr_small_tuple(value, src_base, len);
                        self.emit(
                            Op::MinMaxLArr1SmallTuple {
                                arr_v,
                                r_i,
                                src_base,
                                len,
                                is_max,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2)
                    && small_tuple_shape(&place.ty, self.program).is_some() =>
            {
                let (len, kinds) = small_tuple_shape(&place.ty, self.program).unwrap();
                let src_base = self.next_i;
                self.next_i += len as u16;
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        self.compile_expr_small_tuple(value, src_base, len);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2SmallTuple {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1SmallTuple {
                                    gid: *gid as u16,
                                    r_i,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        self.compile_expr_small_tuple(value, src_base, len);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2SmallTuple {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreLArr1SmallTuple {
                                    arr_v,
                                    r_i,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_v(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2V {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    r_val,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1V {
                                    gid: *gid as u16,
                                    r_i,
                                    r_val,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_v(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2V {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    r_val,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(Op::StoreLArr1V { arr_v, r_i, r_val }, place.span);
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(
                            Op::StoreLFieldIdx1V {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(
                            Op::StoreGList1V {
                                gid: *gid as u16,
                                r_i,
                                r_val,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(Op::StoreLList1V { list_v, r_i, r_val }, place.span);
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(
                            Op::StoreLFieldIdx1V {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, Some(kind))
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                let PlaceKind::Field(obj_place, field) = &base.kind else {
                    return false;
                };
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let r_i = self.compile_expr_i(&indices[0]);
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreLFieldIdx1OptI {
                        obj_v,
                        field: *field as u16,
                        r_i,
                        val,
                        some,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), op, Some(kind), _) => {
                if op != AssignOp::Set && kind != ScalarKind::Int {
                    return false;
                }
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let src_i = if op == AssignOp::Set {
                    self.compile_expr_i(value)
                } else {
                    let r_rhs = self.compile_expr_i(value);
                    let cur = self.alloc_i();
                    self.emit(
                        Op::FieldI {
                            dst: cur,
                            obj_v,
                            field: *field as u16,
                        },
                        place.span,
                    );
                    if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                        return false;
                    }
                    cur
                };
                self.emit(
                    Op::StoreLFieldI {
                        obj_v,
                        field: *field as u16,
                        src_i,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), AssignOp::Set, None, Some(kind)) => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreLFieldOptI {
                        obj_v,
                        field: *field as u16,
                        val,
                        some,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), AssignOp::Set, None, None) => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let src_v = self.compile_expr_v(value);
                self.emit(
                    Op::StoreLFieldV {
                        obj_v,
                        field: *field as u16,
                        src_v,
                    },
                    place.span,
                );
                true
            }
            _ => false,
        }
    }

    pub(super) fn compile_place_path(&mut self, place: &'p Place) -> Option<u16> {
        self.compile_place_path_to(place, false)
    }

    fn keyed_map(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Map(k, _) | Ty::HashMap(k, _) if !crate::value::ty_has_cached(k, self.program))
    }

    /// A path to `place`, through map entries too. A path that ends at a
    /// map entry is only for an op that walks it whole (`key_leaf`): the
    /// ops that store to the last step take list and array cells.
    pub(super) fn compile_place_path_to(
        &mut self,
        place: &'p Place,
        key_leaf: bool,
    ) -> Option<u16> {
        if let PlaceKind::Index(base, indices) = &place.kind {
            if !key_leaf && indices.len() == 1 && self.keyed_map(&base.ty) {
                return None;
            }
        }
        let mut chain: Vec<&'p Place> = Vec::new();
        let mut cur = place;
        let root = loop {
            match &cur.kind {
                PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                    SlotLoc::V(v) => break PlaceRoot::LocalV(v),
                    _ => return None,
                },
                PlaceKind::Global(gid) => break PlaceRoot::Global(*gid as u16),
                PlaceKind::Field(base, _) => {
                    chain.push(cur);
                    cur = base;
                }
                PlaceKind::Index(base, indices)
                    if indices.len() == 1 && self.keyed_map(&base.ty) =>
                {
                    chain.push(cur);
                    cur = base;
                }
                PlaceKind::Index(base, indices)
                    if matches!(base.ty, Ty::Array { .. } | Ty::List(_))
                        && (indices.len() == 1
                            || indices.len() == 2
                            || Self::indexes_nd_cell(&base.ty, indices)) =>
                {
                    if matches!(base.ty, Ty::List(_)) && indices.len() != 1 {
                        return None;
                    }
                    chain.push(cur);
                    cur = base;
                }
                _ => return None,
            }
        };
        if chain.is_empty()
            && !matches!(
                place.ty,
                Ty::List(_)
                    | Ty::Array { .. }
                    | Ty::Str
                    | Ty::Set(_)
                    | Ty::HashSet(_)
                    | Ty::Map(_, _)
                    | Ty::HashMap(_, _)
            )
        {
            return None;
        }
        let mut steps = Vec::with_capacity(chain.len());
        let mut keys = Vec::new();
        for p in chain.into_iter().rev() {
            match &p.kind {
                PlaceKind::Index(base, indices) if self.keyed_map(&base.ty) => {
                    steps.push(PlaceStep::Key(keys.len() as u16));
                    keys.push(self.compile_expr_v(&indices[0]));
                }
                PlaceKind::Field(_, field) => steps.push(PlaceStep::Field(*field as u16)),
                PlaceKind::Index(_, indices) if indices.len() == 1 => {
                    let r_i = self.compile_expr_i(&indices[0]);
                    steps.push(PlaceStep::Index1(r_i));
                }
                PlaceKind::Index(base, indices) if Self::indexes_nd_cell(&base.ty, indices) => {
                    let (idx, n) = self.compile_index_run(indices);
                    steps.push(PlaceStep::IndexN(idx, n));
                }
                PlaceKind::Index(_, indices) if indices.len() == 2 => {
                    let r_i = self.compile_expr_i(&indices[0]);
                    let r_j = self.compile_expr_i(&indices[1]);
                    steps.push(PlaceStep::Index2(r_i, r_j));
                }
                _ => return None,
            }
        }
        let idx = u16::try_from(self.places.len()).ok()?;
        self.places.push(PlacePath { root, steps, keys });
        Some(idx)
    }

    pub(super) fn try_compile_assign_place_path(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        if let PlaceKind::Index(map_place, indices) = &place.kind {
            if matches!(&map_place.ty, Ty::Map(kty, _) | Ty::HashMap(kty, _) if !crate::value::ty_has_cached(kty, self.program))
                && indices.len() == 1
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                let saved_fill_tys = self.fill_tys.len();
                if let Some(path) = self.compile_place_path(map_place) {
                    let key_v = self.compile_expr_v(&indices[0]);
                    let val_v = self.compile_expr_v(value);
                    let val_ty = self.fill_tys.len() as u16;
                    self.fill_tys.push(&place.ty);
                    self.emit(
                        Op::MapPlaceAssign {
                            path,
                            key_v,
                            val_v,
                            op,
                            val_ty,
                        },
                        place.span,
                    );
                    if val_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: val_v }, place.span);
                    }
                    if key_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: key_v }, place.span);
                    }
                    return true;
                }
                self.ops.truncate(saved_ops);
                self.spans.truncate(saved_spans);
                self.places.truncate(saved_places);
                self.fill_tys.truncate(saved_fill_tys);
            }
        }
        let val_kind = ScalarKind::of_ty(&place.ty, self.program);
        let opt_kind = ScalarKind::of_opt_ty(&place.ty, self.program);
        if val_kind == Some(ScalarKind::U64) && op != AssignOp::Set {
            match &place.kind {
                PlaceKind::Local(slot) => {
                    let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                        return false;
                    };
                    let r_rhs = self.compile_expr_i(value);
                    return self.emit_assign_op_u64(dst, dst, r_rhs, op, place.span);
                }
                PlaceKind::Global(gid) => {
                    let r_rhs = self.compile_expr_i(value);
                    let cur = self.alloc_i();
                    self.emit(
                        Op::LoadGlobalI {
                            dst: cur,
                            gid: *gid as u16,
                        },
                        place.span,
                    );
                    if !self.emit_assign_op_u64(cur, cur, r_rhs, op, place.span) {
                        return false;
                    }
                    self.emit(
                        Op::StoreGlobalI {
                            gid: *gid as u16,
                            src: cur,
                            kind: ScalarKind::U64,
                        },
                        place.span,
                    );
                    return true;
                }
                _ => {}
            }
        }
        if matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) {
            return false;
        }
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let Some(path) = self.compile_place_path(place) else {
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
            return false;
        };
        match (op, val_kind, opt_kind) {
            (AssignOp::Set, Some(kind), _) => {
                let r_val = self.compile_expr_i(value);
                self.emit(Op::StorePlaceI { path, r_val, kind }, place.span);
                true
            }
            (AssignOp::Set, None, Some(kind)) => {
                let (r_val, r_some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StorePlaceOptI {
                        path,
                        r_val,
                        r_some,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (AssignOp::Set, None, None) => {
                let r_val = self.compile_expr_v(value);
                self.emit(Op::StorePlaceV { path, r_val }, place.span);
                true
            }
            (op, Some(kind @ (ScalarKind::Int | ScalarKind::U64)), _)
                if matches!(
                    op,
                    AssignOp::Binary(
                        BinOp::Add
                            | BinOp::Sub
                            | BinOp::Mul
                            | BinOp::Div
                            | BinOp::Rem
                            | BinOp::BitAnd
                            | BinOp::BitOr
                            | BinOp::BitXor
                            | BinOp::Shl
                            | BinOp::Shr
                    ) | AssignOp::Min
                        | AssignOp::Max
                ) =>
            {
                let r_rhs = self.compile_expr_i(value);
                self.emit(
                    Op::AssignPlaceI {
                        path,
                        r_rhs,
                        op,
                        kind,
                    },
                    place.span,
                );
                true
            }
            _ => {
                self.ops.truncate(saved_ops);
                self.spans.truncate(saved_spans);
                self.places.truncate(saved_places);
                false
            }
        }
    }

    pub(super) fn emit_assign_op_u64(
        &mut self,
        dst: u16,
        a: u16,
        b: u16,
        op: AssignOp,
        span: Span,
    ) -> bool {
        let inst = match op {
            AssignOp::Binary(
                bop @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Rem
                | BinOp::Shl
                | BinOp::Shr),
            ) => Op::BinU64 { dst, a, b, op: bop },
            AssignOp::Binary(BinOp::BitAnd) => Op::BitAndU64 { dst, a, b },
            AssignOp::Binary(BinOp::BitOr) => Op::BitOrU64 { dst, a, b },
            AssignOp::Binary(BinOp::BitXor) => Op::BitXorU64 { dst, a, b },
            AssignOp::Set => Op::MoveI { dst, src: b },
            _ => return false,
        };
        self.emit(inst, span);
        true
    }

    pub(super) fn try_compile_sort_stmt(
        &mut self,
        place: &'p Place,
        desc: Option<&'p Expr>,
        span: Span,
    ) -> bool {
        let elem_ty_ok = match &place.ty {
            Ty::List(elem) => ty_uses_structural_cmp(elem, self.program),
            Ty::Array { elem, dims } => {
                dims.len() == 1 && ty_uses_structural_cmp(elem, self.program)
            }
            Ty::Str => true,
            _ => false,
        };
        if !elem_ty_ok {
            return false;
        }
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let Some(path) = self.compile_place_path_to(place, true) else {
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
            return false;
        };
        let desc_r = match desc {
            Some(e) => self.compile_expr_i(e),
            None => u16::MAX,
        };
        self.emit(Op::SortPlace { path, desc_r }, span);
        true
    }

    pub(super) fn emit_assign_op_i(
        &mut self,
        dst: u16,
        a: u16,
        b: u16,
        op: AssignOp,
        span: Span,
    ) -> bool {
        let inst = match op {
            AssignOp::Binary(BinOp::Add) => Op::AddI { dst, a, b },
            AssignOp::Binary(BinOp::Sub) => Op::SubI { dst, a, b },
            AssignOp::Binary(BinOp::Mul) => Op::MulI { dst, a, b },
            AssignOp::Binary(BinOp::Div) => Op::DivI { dst, a, b },
            AssignOp::Binary(BinOp::Rem) => Op::RemI { dst, a, b },
            AssignOp::Binary(BinOp::BitAnd) => Op::BitAndI { dst, a, b },
            AssignOp::Binary(BinOp::BitOr) => Op::BitOrI { dst, a, b },
            AssignOp::Binary(BinOp::BitXor) => Op::BitXorI { dst, a, b },
            AssignOp::Binary(BinOp::Shl) => Op::ShlI { dst, a, b },
            AssignOp::Binary(BinOp::Shr) => Op::ShrI { dst, a, b },
            AssignOp::Min => Op::MinI { dst, a, b },
            AssignOp::Max => Op::MaxI { dst, a, b },
            AssignOp::Set => Op::MoveI { dst, src: b },
            _ => return false,
        };
        self.emit(inst, span);
        true
    }

    pub(super) fn emit_branch_if_false(&mut self, cond: &'p Expr) -> usize {
        if let ExprKind::IsSome(inner) = &cond.kind {
            if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                let (_, r_some) = self.compile_expr_opt_i(inner);
                return self.emit(
                    Op::JmpIfZero {
                        reg: r_some,
                        target: 0,
                    },
                    cond.span,
                );
            } else {
                let src_v = self.compile_expr_v(inner);
                let r_some = self.alloc_i();
                self.emit(
                    Op::IsSomeV {
                        dst_i: r_some,
                        src_v,
                    },
                    cond.span,
                );
                return self.emit(
                    Op::JmpIfZero {
                        reg: r_some,
                        target: 0,
                    },
                    cond.span,
                );
            }
        }
        if let ExprKind::Cmp(op, lhs, rhs) = &cond.kind {
            if ScalarKind::of_ty(&lhs.ty, self.program).is_some()
                && ScalarKind::of_ty(&rhs.ty, self.program).is_some()
                && lhs.ty != Ty::U64
                && rhs.ty != Ty::U64
                && ty_uses_structural_cmp(&lhs.ty, self.program)
                && matches!(
                    op,
                    CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                )
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let inv = match op {
                    CmpOp::Lt => Op::JmpGeI { a, b, target: 0 },
                    CmpOp::Le => Op::JmpGtI { a, b, target: 0 },
                    CmpOp::Gt => Op::JmpLeI { a, b, target: 0 },
                    CmpOp::Ge => Op::JmpLtI { a, b, target: 0 },
                    CmpOp::Eq => Op::JmpNeI { a, b, target: 0 },
                    CmpOp::Ne => Op::JmpEqI { a, b, target: 0 },
                    _ => unreachable!(),
                };
                return self.emit(inv, cond.span);
            }
        }
        if let ExprKind::Unary(UnOp::Not, inner) = &cond.kind {
            let r = self.compile_expr_i(inner);
            return self.emit(Op::JmpIfNonZero { reg: r, target: 0 }, cond.span);
        }
        let r = self.compile_expr_i(cond);
        self.emit(Op::JmpIfZero { reg: r, target: 0 }, cond.span)
    }
}
