use super::{Compiler, GiveTarget};
use crate::vm::ir::*;
use cpl_ir::{BinOp, Builtin, Expr, ExprKind, Mutation, PlaceKind, Ty};

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn compile_expr_opt_i(&mut self, expr: &'p Expr) -> (u16, u16) {
        if let Some(place) = self.reported_write(expr) {
            return self.reporting_write(expr, place, |c| c.compile_expr_opt_i(expr));
        }
        let span = expr.span;
        if let Some((fn_id, args)) = self.traced_call(expr) {
            let src_v = self.emit_traced_call(expr, fn_id, args, false);
            let (dst_val, dst_some) = (self.alloc_i(), self.alloc_i());
            let unbox = Op::UnboxOptI {
                dst_val,
                dst_some,
                src_v,
            };
            self.emit(unbox, span);
            return (dst_val, dst_some);
        }
        match &expr.kind {
            ExprKind::None => {
                let v = self.alloc_i();
                let s = self.alloc_i();
                self.emit(Op::ConstImmI { dst: s, val: 0 }, span);
                (v, s)
            }
            ExprKind::Some(inner) => {
                let v = self.compile_expr_i(inner);
                let s = self.alloc_i();
                self.emit(Op::ConstImmI { dst: s, val: 1 }, span);
                (v, s)
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::OptI { val, some, .. } => (val, some),
                SlotLoc::V(src_v) => {
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v,
                        },
                        span,
                    );
                    (dst_val, dst_some)
                }
                SlotLoc::I(v, _) => {
                    let s = self.alloc_i();
                    self.emit(Op::ConstImmI { dst: s, val: 1 }, span);
                    (v, s)
                }
                SlotLoc::SmallTuple { base, .. } => {
                    let s = self.alloc_i();
                    self.emit(Op::ConstImmI { dst: s, val: 0 }, span);
                    (base, s)
                }
            },
            ExprKind::Global(gid) => {
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::LoadGlobalOptI {
                        dst_val,
                        dst_some,
                        gid: *gid as u16,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosNext, args) if args.len() == 2 => {
                let src = self.compile_expr_i(&args[0]);
                let len = self.compile_expr_i(&args[1]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosNext {
                        dst_val,
                        dst_some,
                        src,
                        len,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosPrev, args) if args.len() == 1 => {
                let src = self.compile_expr_i(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosPrev {
                        dst_val,
                        dst_some,
                        src,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosOffset(d), args) if args.len() == 2 => {
                let src = self.compile_expr_i(&args[0]);
                let delta = self.compile_expr_i(&args[1]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosOffset {
                        dst_val,
                        dst_some,
                        src,
                        delta,
                        dom: *d as u16,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosTryOrd(d), args) if args.len() == 1 => {
                let src = self.compile_expr_i(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosTryOrd {
                        dst_val,
                        dst_some,
                        src,
                        dom: *d as u16,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(b @ (Builtin::Last | Builtin::First), args) if args.len() == 1 => {
                let is_last = matches!(b, Builtin::Last);
                if let ExprKind::Index(base, indices) = &args[0].kind {
                    if indices.len() == 1 && matches!(base.ty, Ty::Array { .. } | Ty::List(_)) {
                        let root_info = match &base.kind {
                            ExprKind::Global(gid) => Some((*gid as u16, true)),
                            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                SlotLoc::V(v) => Some((v, false)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((root, is_global)) = root_info {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::SeqIdxEndOptI {
                                    dst_val,
                                    dst_some,
                                    root,
                                    is_global,
                                    r_i,
                                    is_last,
                                },
                                span,
                            );
                            return (dst_val, dst_some);
                        }
                    }
                }
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                if is_last {
                    self.emit(
                        Op::SeqLastOptI {
                            dst_val,
                            dst_some,
                            seq_v,
                        },
                        span,
                    );
                } else {
                    self.emit(
                        Op::SeqFirstOptI {
                            dst_val,
                            dst_some,
                            seq_v,
                        },
                        span,
                    );
                }
                if seq_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: seq_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::MapGet, args)
                if args.len() == 2
                    && matches!(&args[0].ty, Ty::Map(kty, _) | Ty::HashMap(kty, _) if !crate::value::ty_has_cached(kty, self.program)) =>
            {
                let map_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let tmp_v = self.alloc_v();
                self.emit(
                    Op::MapGetV {
                        dst: tmp_v,
                        map_v,
                        key_v,
                    },
                    span,
                );
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v: tmp_v,
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Builtin(
                b @ (Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt),
                args,
            ) if args.len() == 2
                && matches!(
                    args[0].ty,
                    Ty::Set(_) | Ty::HashSet(_) | Ty::Map(_, _) | Ty::HashMap(_, _)
                )
                && !crate::value::ty_has_cached(&args[1].ty, self.program) =>
            {
                let mode = match b {
                    Builtin::FirstGe => 0,
                    Builtin::FirstGt => 1,
                    Builtin::LastLe => 2,
                    Builtin::LastLt => 3,
                    _ => unreachable!(),
                };
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let tmp_v = self.alloc_v();
                self.emit(
                    Op::SetMapBoundV {
                        dst: tmp_v,
                        src_v,
                        key_v,
                        mode,
                    },
                    span,
                );
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v: tmp_v,
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Mutate(m @ (Mutation::PopFirst | Mutation::PopLast), place, args)
                if args.is_empty()
                    && matches!(
                        place.ty,
                        Ty::Set(_) | Ty::HashSet(_) | Ty::Map(_, _) | Ty::HashMap(_, _)
                    ) =>
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                if let Some(path) = self.compile_place_path(place) {
                    let tmp_v = self.alloc_v();
                    self.emit(
                        Op::SetMapPopEndPlace {
                            dst: tmp_v,
                            path,
                            is_last: matches!(m, Mutation::PopLast),
                        },
                        span,
                    );
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v: tmp_v,
                        },
                        span,
                    );
                    self.emit(Op::ClearV { reg: tmp_v }, span);
                    (dst_val, dst_some)
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v,
                        },
                        span,
                    );
                    (dst_val, dst_some)
                }
            }
            ExprKind::Mutate(Mutation::Pop, place, args)
                if args.is_empty() && matches!(place.ty, Ty::List(_)) =>
            {
                match &place.kind {
                    PlaceKind::Local(slot) => {
                        if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::ListPopOptIL {
                                    dst_val,
                                    dst_some,
                                    list_v,
                                },
                                span,
                            );
                            (dst_val, dst_some)
                        } else {
                            let src_v = self.emit_fallback_expr_v(expr);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::UnboxOptI {
                                    dst_val,
                                    dst_some,
                                    src_v,
                                },
                                span,
                            );
                            (dst_val, dst_some)
                        }
                    }
                    PlaceKind::Global(gid) => {
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::ListPopOptIG {
                                dst_val,
                                dst_some,
                                gid: *gid as u16,
                            },
                            span,
                        );
                        (dst_val, dst_some)
                    }
                    PlaceKind::Field(obj_place, field) => {
                        if let PlaceKind::Local(slot) = obj_place.kind {
                            if let SlotLoc::V(obj_v) = self.slot_loc(slot) {
                                let dst_val = self.alloc_i();
                                let dst_some = self.alloc_i();
                                self.emit(
                                    Op::ListPopFieldOptIL {
                                        dst_val,
                                        dst_some,
                                        obj_v,
                                        field: *field as u16,
                                    },
                                    span,
                                );
                                return (dst_val, dst_some);
                            }
                        }
                        let src_v = self.emit_fallback_expr_v(expr);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::UnboxOptI {
                                dst_val,
                                dst_some,
                                src_v,
                            },
                            span,
                        );
                        (dst_val, dst_some)
                    }
                    _ => {
                        let src_v = self.emit_fallback_expr_v(expr);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::UnboxOptI {
                                dst_val,
                                dst_some,
                                src_v,
                            },
                            span,
                        );
                        (dst_val, dst_some)
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2OptI {
                                    dst_val,
                                    dst_some,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1OptI {
                                    dst_val,
                                    dst_some,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                span,
                            );
                        }
                        (dst_val, dst_some)
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(
                                    Op::LoadLArr2OptI {
                                        dst_val,
                                        dst_some,
                                        arr_v,
                                        r_i,
                                        r_j,
                                    },
                                    span,
                                );
                            } else {
                                self.emit(
                                    Op::LoadLArr1OptI {
                                        dst_val,
                                        dst_some,
                                        arr_v,
                                        r_i,
                                    },
                                    span,
                                );
                            }
                            (dst_val, dst_some)
                        } else {
                            let src_v = self.emit_fallback_expr_v(expr);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::UnboxOptI {
                                    dst_val,
                                    dst_some,
                                    src_v,
                                },
                                span,
                            );
                            (dst_val, dst_some)
                        }
                    }
                    ExprKind::Index(outer, outer_indices)
                        if indices.len() == 1
                            && outer_indices.len() == 1
                            && matches!(outer.ty, Ty::Array { .. } | Ty::List(_)) =>
                    {
                        let root_info = match &outer.kind {
                            ExprKind::Global(gid) => Some((*gid as u16, true)),
                            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                SlotLoc::V(v) => Some((v, false)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((root, is_global)) = root_info {
                            let r_i = self.compile_expr_i(&outer_indices[0]);
                            let r_j = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::LoadNestedArrOptI {
                                    dst_val,
                                    dst_some,
                                    root,
                                    is_global,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                            return (dst_val, dst_some);
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::LoadLArr1OptI {
                                dst_val,
                                dst_some,
                                arr_v,
                                r_i,
                            },
                            span,
                        );
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                    ExprKind::Field(inner_base, field) if indices.len() == 1 => {
                        if let ExprKind::Index(seq, inner_indices) = &inner_base.kind {
                            if inner_indices.len() == 1
                                && matches!(seq.ty, Ty::List(_) | Ty::Array { .. })
                            {
                                let root_info = match &seq.kind {
                                    ExprKind::Global(gid) => Some((*gid as u16, true)),
                                    ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                        SlotLoc::V(v) => Some((v, false)),
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if let Some((root, is_global)) = root_info {
                                    let r_i = self.compile_expr_i(&inner_indices[0]);
                                    let r_j = self.compile_expr_i(&indices[0]);
                                    let dst_val = self.alloc_i();
                                    let dst_some = self.alloc_i();
                                    self.emit(
                                        Op::LoadIdxFieldIdxOptI {
                                            dst_val,
                                            dst_some,
                                            root,
                                            is_global,
                                            r_i,
                                            field: *field as u16,
                                            r_j,
                                        },
                                        span,
                                    );
                                    return (dst_val, dst_some);
                                }
                            }
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::LoadLArr1OptI {
                                dst_val,
                                dst_some,
                                arr_v,
                                r_i,
                            },
                            span,
                        );
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadLArr2OptI {
                                    dst_val,
                                    dst_some,
                                    arr_v,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadLArr1OptI {
                                    dst_val,
                                    dst_some,
                                    arr_v,
                                    r_i,
                                },
                                span,
                            );
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                let list_v = self.compile_expr_v(base);
                let r_i = self.compile_expr_i(&indices[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::LoadLList1OptI {
                        dst_val,
                        dst_some,
                        list_v,
                        r_i,
                    },
                    span,
                );
                if list_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: list_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Field(base, idx) => {
                let obj_v = self.compile_expr_v(base);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::FieldOptI {
                        dst_val,
                        dst_some,
                        obj_v,
                        field: *idx as u16,
                    },
                    span,
                );
                if obj_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: obj_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Builtin(b @ (Builtin::SetMin | Builtin::SetMax), args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let tmp_v = self.alloc_v();
                self.emit(
                    Op::SetMinMaxV {
                        dst: tmp_v,
                        src_v,
                        is_max: matches!(b, Builtin::SetMax),
                    },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v: tmp_v,
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Call(fn_id, args) => {
                if let Some((dst_val, _, dst_some)) =
                    self.try_compile_call_mix(*fn_id, args, CallRet::OptI, span)
                {
                    (dst_val, dst_some)
                } else {
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v,
                        },
                        span,
                    );
                    (dst_val, dst_some)
                }
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((dst_val, _, dst_some)) =
                    self.try_compile_call_closure(callee, args, CallRet::OptI, span)
                {
                    (dst_val, dst_some)
                } else {
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v,
                        },
                        span,
                    );
                    (dst_val, dst_some)
                }
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs)
                if ScalarKind::of_opt_ty(&lhs.ty, self.program).is_some() =>
            {
                let (dst_val, dst_some) = (self.alloc_i(), self.alloc_i());
                let (lv, ls) = self.compile_expr_opt_i(lhs);
                self.emit(
                    Op::MoveI {
                        dst: dst_val,
                        src: lv,
                    },
                    lhs.span,
                );
                self.emit(
                    Op::MoveI {
                        dst: dst_some,
                        src: ls,
                    },
                    lhs.span,
                );
                let j_done = self.emit(Op::JmpIfNonZero { reg: ls, target: 0 }, span);
                let (rv, rs) = self.compile_expr_opt_i(rhs);
                self.emit(
                    Op::MoveI {
                        dst: dst_val,
                        src: rv,
                    },
                    rhs.span,
                );
                self.emit(
                    Op::MoveI {
                        dst: dst_some,
                        src: rs,
                    },
                    rhs.span,
                );
                let end = self.ops.len() as u32;
                self.patch(j_done, end);
                (dst_val, dst_some)
            }
            ExprKind::If(cond, then_e, else_e) => {
                if let Some(b) = Self::const_bool(cond) {
                    return if b {
                        self.compile_expr_opt_i(then_e)
                    } else {
                        self.compile_expr_opt_i(else_e)
                    };
                }
                if let (ExprKind::Some(inner), ExprKind::None) = (&then_e.kind, &else_e.kind) {
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(
                        Op::ConstImmI {
                            dst: dst_some,
                            val: 0,
                        },
                        span,
                    );
                    let j_else = self.emit_branch_if_false(cond);
                    let tv = self.compile_expr_i(inner);
                    if dst_val != tv {
                        self.emit(
                            Op::MoveI {
                                dst: dst_val,
                                src: tv,
                            },
                            inner.span,
                        );
                    }
                    self.emit(
                        Op::ConstImmI {
                            dst: dst_some,
                            val: 1,
                        },
                        then_e.span,
                    );
                    let end_pc = self.ops.len() as u32;
                    self.patch(j_else, end_pc);
                    return (dst_val, dst_some);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                let j_else = self.emit_branch_if_false(cond);
                let (tv, ts) = self.compile_expr_opt_i(then_e);
                self.emit(
                    Op::MoveI {
                        dst: dst_val,
                        src: tv,
                    },
                    then_e.span,
                );
                self.emit(
                    Op::MoveI {
                        dst: dst_some,
                        src: ts,
                    },
                    then_e.span,
                );
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let (ev, es) = self.compile_expr_opt_i(else_e);
                self.emit(
                    Op::MoveI {
                        dst: dst_val,
                        src: ev,
                    },
                    else_e.span,
                );
                self.emit(
                    Op::MoveI {
                        dst: dst_some,
                        src: es,
                    },
                    else_e.span,
                );
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                (dst_val, dst_some)
            }
            ExprKind::Do(block) => {
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                let prev_pinned = self.pinned_v;
                self.pinned_v = self.next_v;
                self.give_targets.push(GiveTarget::OptI {
                    dst_val,
                    dst_some,
                    exits: Vec::new(),
                });
                self.compile_do_stmts(&block.stmts);
                let mut target = self.give_targets.pop().unwrap();
                self.pinned_v = prev_pinned;
                let end_pc = self.ops.len() as u32;
                for j in target.exits_mut().drain(..) {
                    self.patch(j, end_pc);
                }
                (dst_val, dst_some)
            }
            ExprKind::Mutate(Mutation::Take, place, args) if args.is_empty() => {
                // Taking an optional scalar leaves it in place.
                if let PlaceKind::Local(slot) = place.kind {
                    if let SlotLoc::OptI { val, some, .. } = self.slot_loc(slot) {
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(
                            Op::MoveI {
                                dst: dst_val,
                                src: val,
                            },
                            span,
                        );
                        self.emit(
                            Op::MoveI {
                                dst: dst_some,
                                src: some,
                            },
                            span,
                        );
                        return (dst_val, dst_some);
                    }
                }
                let src_v = match self.compile_place_path(place) {
                    Some(path) => {
                        let dst = self.alloc_v();
                        self.emit(Op::TakePlaceV { dst, path }, span);
                        dst
                    }
                    None => self.emit_fallback_expr_v(expr),
                };
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
            _ => {
                let src_v = self.emit_fallback_expr_v(expr);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v,
                    },
                    span,
                );
                (dst_val, dst_some)
            }
        }
    }
}
