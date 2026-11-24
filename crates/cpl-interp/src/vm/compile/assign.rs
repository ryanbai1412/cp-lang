use super::jump::{Jump, JumpOp};
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
            // A field of a local held in integer registers is its register.
            (PlaceKind::Field(obj, f), op, Some(_), _)
                if (op == AssignOp::Set || place.ty == Ty::Int)
                    && self.field_reg(obj, *f).is_some() =>
            {
                let Some(dst) = self.field_reg(obj, *f) else {
                    return false;
                };
                let r = self.compile_expr_i(value);
                self.emit_assign_op_i(dst, dst, r, op, place.span)
            }
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
                let Some((len, _)) = small_tuple_shape(&place.ty, self.program) else {
                    return false;
                };
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
                let Some((len, kinds)) = small_tuple_shape(&place.ty, self.program) else {
                    return false;
                };
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
                        if let Some((len, kinds)) =
                            crate::vm::scalar::small_tuple_shape(&value.ty, self.program)
                                .filter(|_| self.small_tuple_is_cheap(value))
                        {
                            let src_base = self.small_tuple_operand(value, len);
                            let store = Op::StoreLFieldIdx1SmallTuple {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                src_base,
                                len,
                                kinds,
                            };
                            self.emit(store, place.span);
                            return true;
                        }
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
            _ => false,
        }
    }

    pub(super) fn compile_place_path(&mut self, place: &'p Place) -> Option<u16> {
        self.compile_place_path_to(place, false)
    }

    fn field_reg(&self, obj: &Place, f: usize) -> Option<u16> {
        let PlaceKind::Local(slot) = obj.kind else {
            return None;
        };
        match self.slot_loc(slot) {
            SlotLoc::I(r, ScalarKind::OneIntStruct | ScalarKind::OneU64Struct) if f == 0 => Some(r),
            SlotLoc::SmallTuple { base, len, .. } if f < usize::from(len) => Some(base + f as u16),
            _ => None,
        }
    }

    /// A map whose entries a path step walks to: with `any_keys`, also one
    /// keyed by a user `Ord` or cached fields, for an op that searches with
    /// `Interp::cmp_ty_val`.
    fn keyed_map(&self, ty: &Ty, any_keys: bool) -> bool {
        matches!(ty, Ty::Map(k, _) | Ty::HashMap(k, _) if any_keys || (!crate::value::ty_has_cached(k, self.program) && !crate::value::keys::user_ordered(k, self.program)))
    }

    /// A path to `place`, through map entries too. A path that ends at a
    /// map entry is only for an op that walks it whole (`key_leaf`) or puts
    /// the entry back whole (`StorePlaceV`): the other ops that store to
    /// the last step take list and array cells.
    pub(super) fn compile_place_path_to(
        &mut self,
        place: &'p Place,
        key_leaf: bool,
    ) -> Option<u16> {
        self.place_path_rooted(place, key_leaf, None, false)
    }

    /// The path to `place`, from `boxed_root` when its local is boxed there;
    /// with `any_keys`, through entries of any [`Self::keyed_map`].
    pub(super) fn place_path_rooted(
        &mut self,
        place: &'p Place,
        key_leaf: bool,
        boxed_root: Option<u16>,
        any_keys: bool,
    ) -> Option<u16> {
        if let PlaceKind::Index(base, indices) = &place.kind {
            if !key_leaf && indices.len() == 1 && self.keyed_map(&base.ty, any_keys) {
                return None;
            }
        }
        let mut chain: Vec<&'p Place> = Vec::new();
        let mut cur = place;
        let root = loop {
            match &cur.kind {
                PlaceKind::Local(slot) => match (boxed_root, self.slot_loc(*slot)) {
                    (Some(v), _) | (None, SlotLoc::V(v)) => break PlaceRoot::LocalV(v),
                    _ => return None,
                },
                PlaceKind::Global(gid) => break PlaceRoot::Global(*gid as u16),
                PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    chain.push(cur);
                    cur = base;
                }
                PlaceKind::Index(base, indices)
                    if indices.len() == 1 && self.keyed_map(&base.ty, any_keys) =>
                {
                    chain.push(cur);
                    cur = base;
                }
                PlaceKind::Index(base, indices)
                    if matches!(base.ty, Ty::Array { .. } | Ty::List(_) | Ty::Str)
                        && (indices.len() == 1
                            || indices.len() == 2
                            || Self::indexes_nd_cell(&base.ty, indices)) =>
                {
                    if matches!(base.ty, Ty::List(_) | Ty::Str) && indices.len() != 1 {
                        return None;
                    }
                    chain.push(cur);
                    cur = base;
                }
                _ => return None,
            }
        };
        if chain.is_empty()
            && boxed_root.is_none()
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
                PlaceKind::Index(base, indices) if self.keyed_map(&base.ty, any_keys) => {
                    steps.push(PlaceStep::Key(keys.len() as u16));
                    keys.push(self.compile_expr_v(&indices[0]));
                }
                PlaceKind::Field(_, field) => steps.push(PlaceStep::Field(*field as u16)),
                PlaceKind::Some(_) => steps.push(PlaceStep::Some),
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
            if matches!(&map_place.ty, Ty::Map(..) | Ty::HashMap(..)) && indices.len() == 1 {
                let undo = self.code_mark();
                let saved_fill_tys = self.fill_tys.len();
                if let Some(path) = self.compile_place_path(map_place) {
                    let key_v = self.compile_expr_v(&indices[0]);
                    let val_v = self.compile_expr_v(value);
                    let map_ty = self.fill_tys.len() as u16;
                    self.fill_tys.push(&map_place.ty);
                    self.emit(
                        Op::MapPlaceAssign {
                            path,
                            key_v,
                            val_v,
                            op,
                            map_ty,
                        },
                        place.span,
                    );
                    self.free_temp_v(val_v, place.span);
                    self.free_temp_v(key_v, place.span);
                    return true;
                }
                self.rewind(undo);
                self.fill_tys.truncate(saved_fill_tys);
            }
        }
        let val_kind = ScalarKind::of_ty(&place.ty, self.program);
        let opt_kind = ScalarKind::of_opt_ty(&place.ty, self.program);
        if val_kind == Some(ScalarKind::U64) && matches!(op, AssignOp::Binary(_)) {
            match &place.kind {
                PlaceKind::Local(slot) => {
                    let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                        return self.try_compile_assign_op_v(place, op, value);
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
        if op != AssignOp::Set && val_kind.is_none() && opt_kind.is_none() {
            return self.try_compile_assign_op_v(place, op, value);
        }
        if let (AssignOp::Max, None, Some(kind)) = (op, val_kind, opt_kind) {
            match place.kind {
                PlaceKind::Local(slot) => {
                    let SlotLoc::OptI { val, some, .. } = self.slot_loc(slot) else {
                        return self.try_compile_assign_op_v(place, op, value);
                    };
                    let (b_val, b_some) = self.compile_expr_opt_i(value);
                    let max = Op::MaxOptI {
                        val,
                        some,
                        b_val,
                        b_some,
                    };
                    self.emit(max, place.span);
                    return true;
                }
                PlaceKind::Global(gid) => {
                    let (b_val, b_some) = self.compile_expr_opt_i(value);
                    let (val, some, gid) = (self.alloc_i(), self.alloc_i(), gid as u16);
                    let load = Op::LoadGlobalOptI {
                        dst_val: val,
                        dst_some: some,
                        gid,
                    };
                    self.emit(load, place.span);
                    let max = Op::MaxOptI {
                        val,
                        some,
                        b_val,
                        b_some,
                    };
                    self.emit(max, place.span);
                    let store = Op::StoreGlobalOptI {
                        gid,
                        val,
                        some,
                        kind,
                    };
                    self.emit(store, place.span);
                    return true;
                }
                _ => {}
            }
        }
        if matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) {
            return self.try_compile_assign_op_v(place, op, value);
        }
        let undo = self.code_mark();
        let Some(path) = self.compile_place_path(place) else {
            self.rewind(undo);
            return self.try_compile_assign_op_v(place, op, value);
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
            // An optional's `max=` orders it as a value does.
            (AssignOp::Max, None, Some(_)) => {
                let r_val = self.compile_expr_v(value);
                self.emit(Op::AssignPlaceV { path, r_val, op }, place.span);
                self.free_temp_v(r_val, place.span);
                true
            }
            _ => {
                self.rewind(undo);
                self.try_compile_assign_op_v(place, op, value)
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
        let undo = self.code_mark();
        let Some(path) = self.compile_place_path_to(place, true) else {
            self.rewind(undo);
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

    pub(super) fn emit_branch_if_false(&mut self, cond: &'p Expr) -> Vec<Jump> {
        let mut out = Vec::new();
        self.emit_branch(cond, false, &mut out);
        out
    }

    /// Pushes onto `out` the jumps taken when `cond` is `on` and falls through
    /// otherwise, so a condition of `and`, `or`, `not` and compares never stores a Bool.
    fn emit_branch(&mut self, cond: &'p Expr, on: bool, out: &mut Vec<Jump>) {
        let jump = match &cond.kind {
            ExprKind::Unary(UnOp::Not, inner) => return self.emit_branch(inner, !on, out),
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), lhs, rhs) => {
                // `and` leaves on a false lhs, `or` on a true one.
                let short = *op == BinOp::Or;
                if short == on {
                    self.emit_branch(lhs, short, out);
                    return self.emit_branch(rhs, on, out);
                }
                let mut past = Vec::new();
                self.emit_branch(lhs, short, &mut past);
                self.emit_branch(rhs, on, out);
                let here = self.ops.len() as u32;
                return self.patch_all(past, here);
            }
            ExprKind::Cmp(op, lhs, rhs)
                if ScalarKind::of_ty(&lhs.ty, self.program).is_some()
                    && ScalarKind::of_ty(&rhs.ty, self.program).is_some()
                    && lhs.ty != Ty::U64
                    && rhs.ty != Ty::U64
                    && ty_uses_structural_cmp(&lhs.ty, self.program)
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let op = match (on, op) {
                    (true, op) => *op,
                    (_, CmpOp::Lt) => CmpOp::Ge,
                    (_, CmpOp::Le) => CmpOp::Gt,
                    (_, CmpOp::Gt) => CmpOp::Le,
                    (_, CmpOp::Ge) => CmpOp::Lt,
                    (_, CmpOp::Eq) => CmpOp::Ne,
                    _ => CmpOp::Eq,
                };
                match op {
                    CmpOp::Lt => JumpOp::JmpLtI { a, b },
                    CmpOp::Le => JumpOp::JmpLeI { a, b },
                    CmpOp::Gt => JumpOp::JmpGtI { a, b },
                    CmpOp::Ge => JumpOp::JmpGeI { a, b },
                    CmpOp::Eq => JumpOp::JmpEqI { a, b },
                    _ => JumpOp::JmpNeI { a, b },
                }
            }
            _ => {
                let reg = self.compile_expr_i(cond);
                match on {
                    true => JumpOp::JmpIfNonZero { reg },
                    false => JumpOp::JmpIfZero { reg },
                }
            }
        };
        out.push(self.emit_jump(jump, cond.span));
    }
}
