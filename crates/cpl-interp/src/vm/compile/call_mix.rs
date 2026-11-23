//! Calls of compiled fns from compiled code: value args first, then each
//! `mut` arg moved out of its place, written back after the call.

use super::*;

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn try_compile_call_mix(
        &mut self,
        fn_id: FnId,
        args: &'p [Arg],
        ret_mode: CallRet,
        span: Span,
    ) -> Option<(u16, u16, u16)> {
        let f = &self.program.fns[fn_id];
        let has_mut = f.params.iter().any(|p| p.mutable);
        let has_fn_param = f
            .locals
            .iter()
            .take(f.params.len())
            .any(|l| matches!(l.ty, Ty::Fn(..)));
        if f.locals.len() > 512
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || (has_mut && has_fn_param && !self.mut_fn_args_alias_free(fn_id, args))
        {
            return None;
        }
        // Check that all mut args are simple local/global places or a single 1D indexed V place.
        let mut idx_mut_count = 0usize;
        let mut via_path = vec![false; args.len()];
        for (i, a) in args.iter().enumerate() {
            match a {
                Arg::Value(_) => {
                    if f.params[i].mutable {
                        return None;
                    }
                }
                Arg::Mut(place) => {
                    if !f.params[i].mutable {
                        return None;
                    }
                    match &place.kind {
                        PlaceKind::Local(slot) => {
                            if matches!(self.slot_loc(*slot), SlotLoc::SmallTuple { .. }) {
                                return None;
                            }
                        }
                        PlaceKind::Global(_) => {}
                        PlaceKind::Index(base, indices)
                            if indices.len() == 1
                                && idx_mut_count == 0
                                && ScalarKind::of_ty(&indices[0].ty, self.program).is_some()
                                && ScalarKind::of_ty(&f.locals[i].ty, self.program).is_none()
                                && ScalarKind::of_opt_ty(&f.locals[i].ty, self.program)
                                    .is_none() =>
                        {
                            let base_ok = match &base.kind {
                                PlaceKind::Local(slot) => {
                                    matches!(self.slot_loc(*slot), SlotLoc::V(_))
                                }
                                PlaceKind::Global(_) => true,
                                _ => false,
                            };
                            let container_ok = match &base.ty {
                                Ty::Array { dims, .. } => {
                                    dims.len() == 1
                                        && !matches!(
                                            dims[0],
                                            cpl_ir::Dim::IntFrom(_) | cpl_ir::Dim::Sym(_)
                                        )
                                }
                                Ty::List(_) => true,
                                _ => false,
                            };
                            if !base_ok || !container_ok {
                                return None;
                            }
                            idx_mut_count += 1;
                        }
                        PlaceKind::Index(..) | PlaceKind::Field(..)
                            if ScalarKind::of_ty(&f.locals[i].ty, self.program).is_none()
                                && ScalarKind::of_opt_ty(&f.locals[i].ty, self.program)
                                    .is_none() =>
                        {
                            via_path[i] = true;
                            idx_mut_count += 1;
                        }
                        _ => return None,
                    }
                }
            }
        }
        if idx_mut_count > 0 && args.iter().filter(|a| matches!(a, Arg::Mut(_))).count() > 1 {
            return None;
        }
        let mut ni = 0usize;
        let mut nv = 0usize;
        for local in f.locals.iter().take(f.params.len()) {
            let ty = &local.ty;
            if ScalarKind::of_ty(ty, self.program).is_some() {
                ni += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                ni += 2;
            } else {
                nv += 1;
            }
        }
        if ni > 8 || nv > 8 {
            return None;
        }
        enum WriteBack {
            LocalI {
                dst: u16,
                arg_off: u16,
            },
            LocalOptI {
                dst_val: u16,
                dst_some: u16,
                arg_off: u16,
            },
            LocalV {
                dst: u16,
                arg_off: u16,
            },
            GlobalI {
                gid: u16,
                kind: ScalarKind,
                arg_off: u16,
            },
            GlobalOptI {
                gid: u16,
                kind: ScalarKind,
                arg_off: u16,
            },
            GlobalV {
                gid: u16,
                arg_off: u16,
            },
            LocalArr1V {
                arr_v: u16,
                r_i: u16,
                arg_off: u16,
            },
            GlobalArr1V {
                gid: u16,
                r_i: u16,
                arg_off: u16,
            },
            LocalList1V {
                list_v: u16,
                r_i: u16,
                arg_off: u16,
            },
            GlobalList1V {
                gid: u16,
                r_i: u16,
                arg_off: u16,
            },
            Place {
                path: u16,
                arg_off: u16,
            },
        }
        let mut i_args: Vec<u16> = vec![0; ni];
        let mut v_args: Vec<(u16, bool)> = vec![(0, false); nv];
        let mut writebacks: Vec<WriteBack> = Vec::new();
        // Precompute argument slot offsets so all Arg::Value expressions are evaluated
        // before any Arg::Mut place is moved out (e.g. `a.union(a.shifted(w))`).
        let mut param_offs: Vec<usize> = Vec::with_capacity(args.len());
        let mut cur_i = 0usize;
        let mut cur_v = 0usize;
        for local in f.locals.iter().take(f.params.len()) {
            let ty = &local.ty;
            if ScalarKind::of_ty(ty, self.program).is_some() {
                param_offs.push(cur_i);
                cur_i += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                param_offs.push(cur_i);
                cur_i += 2;
            } else {
                param_offs.push(cur_v);
                cur_v += 1;
            }
        }
        let marks = (self.ops.len(), self.spans.len(), self.places.len());
        // Pass 1: evaluate all value arguments first.
        for (i, a) in args.iter().enumerate() {
            if let Arg::Value(ae) = a {
                let ty = &f.locals[i].ty;
                let off = param_offs[i];
                if ScalarKind::of_ty(ty, self.program).is_some() {
                    i_args[off] = self.compile_expr_i(ae);
                } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                    let (v, s) = self.compile_expr_opt_i(ae);
                    i_args[off] = v;
                    i_args[off + 1] = s;
                } else {
                    v_args[off] = (self.compile_expr_v(ae), false);
                }
            }
        }
        // Pass 2: move out all mut arguments after value arguments are evaluated.
        for (i, a) in args.iter().enumerate() {
            if let Arg::Mut(place) = a {
                let ty = &f.locals[i].ty;
                let off = param_offs[i];
                if via_path[i] {
                    let Some(path) = self.compile_place_path_to(place, true) else {
                        self.ops.truncate(marks.0);
                        self.spans.truncate(marks.1);
                        self.places.truncate(marks.2);
                        return None;
                    };
                    let r = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst: r, path }, place.span);
                    writebacks.push(WriteBack::Place {
                        path,
                        arg_off: off as u16,
                    });
                    v_args[off] = (r, true);
                    continue;
                }
                match &place.kind {
                    PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                        SlotLoc::I(r, _) => {
                            i_args[off] = r;
                            writebacks.push(WriteBack::LocalI {
                                dst: r,
                                arg_off: off as u16,
                            });
                        }
                        SlotLoc::OptI { val, some, .. } => {
                            i_args[off] = val;
                            i_args[off + 1] = some;
                            writebacks.push(WriteBack::LocalOptI {
                                dst_val: val,
                                dst_some: some,
                                arg_off: off as u16,
                            });
                        }
                        SlotLoc::V(r) => {
                            v_args[off] = (r, true);
                            writebacks.push(WriteBack::LocalV {
                                dst: r,
                                arg_off: off as u16,
                            });
                        }
                        SlotLoc::SmallTuple { .. } => unreachable!(),
                    },
                    PlaceKind::Global(gid) => {
                        let gid = *gid as u16;
                        if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
                            let r = self.alloc_i();
                            self.emit(Op::LoadGlobalI { dst: r, gid }, place.span);
                            i_args[off] = r;
                            writebacks.push(WriteBack::GlobalI {
                                gid,
                                kind,
                                arg_off: off as u16,
                            });
                        } else if let Some(kind) = ScalarKind::of_opt_ty(ty, self.program) {
                            let rv = self.alloc_i();
                            let rs = self.alloc_i();
                            self.emit(
                                Op::LoadGlobalOptI {
                                    dst_val: rv,
                                    dst_some: rs,
                                    gid,
                                },
                                place.span,
                            );
                            i_args[off] = rv;
                            i_args[off + 1] = rs;
                            writebacks.push(WriteBack::GlobalOptI {
                                gid,
                                kind,
                                arg_off: off as u16,
                            });
                        } else {
                            let r = self.alloc_v();
                            self.emit(Op::LoadGlobalVMove { dst: r, gid }, place.span);
                            v_args[off] = (r, true);
                            writebacks.push(WriteBack::GlobalV {
                                gid,
                                arg_off: off as u16,
                            });
                        }
                    }
                    PlaceKind::Index(base, indices) => {
                        let is_arr = matches!(&base.ty, Ty::Array { .. });
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r = self.alloc_v();
                        match (&base.kind, is_arr) {
                            (PlaceKind::Local(slot), true) => {
                                let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                                    unreachable!();
                                };
                                self.emit(Op::TakeLArr1V { dst: r, arr_v, r_i }, place.span);
                                writebacks.push(WriteBack::LocalArr1V {
                                    arr_v,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Global(gid), true) => {
                                let gid = *gid as u16;
                                self.emit(Op::TakeGArr1V { dst: r, gid, r_i }, place.span);
                                writebacks.push(WriteBack::GlobalArr1V {
                                    gid,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Local(slot), false) => {
                                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                                    unreachable!();
                                };
                                self.emit(
                                    Op::TakeLList1V {
                                        dst: r,
                                        list_v,
                                        r_i,
                                    },
                                    place.span,
                                );
                                writebacks.push(WriteBack::LocalList1V {
                                    list_v,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Global(gid), false) => {
                                let gid = *gid as u16;
                                self.emit(Op::TakeGList1V { dst: r, gid, r_i }, place.span);
                                writebacks.push(WriteBack::GlobalList1V {
                                    gid,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            _ => unreachable!(),
                        }
                        v_args[off] = (r, true);
                    }
                    _ => unreachable!(),
                }
            }
        }
        let arg_base_i = self.next_i;
        for r in i_args {
            let slot = self.alloc_i();
            if slot != r {
                self.emit(Op::MoveI { dst: slot, src: r }, span);
            }
        }
        let arg_base_v = self.next_v;
        for (r, consume) in v_args {
            let slot = self.alloc_v();
            if slot != r {
                if consume {
                    self.emit(Op::MoveVConsume { dst: slot, src: r }, span);
                } else {
                    self.emit(Op::MoveV { dst: slot, src: r }, span);
                }
            }
        }
        let (dst_i, dst_v, dst_some) = match ret_mode {
            CallRet::Unit => (0, 0, 0),
            CallRet::I => (self.alloc_i(), 0, 0),
            CallRet::OptI => (self.alloc_i(), 0, self.alloc_i()),
            CallRet::V => (0, self.alloc_v(), 0),
        };
        self.emit(
            Op::CallVmMix {
                fn_id: fn_id as u16,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            },
            span,
        );
        for wb in writebacks {
            match wb {
                WriteBack::Place { path, arg_off } => {
                    let r_val = arg_base_v + arg_off;
                    self.emit(Op::StorePlaceV { path, r_val }, span);
                    self.emit(Op::ClearV { reg: r_val }, span);
                }
                WriteBack::LocalI { dst, arg_off } => {
                    let src = arg_base_i + arg_off;
                    if dst != src {
                        self.emit(Op::MoveI { dst, src }, span);
                    }
                }
                WriteBack::LocalOptI {
                    dst_val,
                    dst_some,
                    arg_off,
                } => {
                    let sv = arg_base_i + arg_off;
                    let ss = arg_base_i + arg_off + 1;
                    if dst_val != sv {
                        self.emit(
                            Op::MoveI {
                                dst: dst_val,
                                src: sv,
                            },
                            span,
                        );
                    }
                    if dst_some != ss {
                        self.emit(
                            Op::MoveI {
                                dst: dst_some,
                                src: ss,
                            },
                            span,
                        );
                    }
                }
                WriteBack::LocalV { dst, arg_off } => {
                    let src = arg_base_v + arg_off;
                    if dst != src {
                        self.emit(Op::MoveVConsume { dst, src }, span);
                    }
                }
                WriteBack::GlobalI { gid, kind, arg_off } => {
                    self.emit(
                        Op::StoreGlobalI {
                            gid,
                            src: arg_base_i + arg_off,
                            kind,
                        },
                        span,
                    );
                }
                WriteBack::GlobalOptI { gid, kind, arg_off } => {
                    self.emit(
                        Op::StoreGlobalOptI {
                            gid,
                            val: arg_base_i + arg_off,
                            some: arg_base_i + arg_off + 1,
                            kind,
                        },
                        span,
                    );
                }
                WriteBack::GlobalV { gid, arg_off } => {
                    self.emit(
                        Op::StoreGlobalVMove {
                            gid,
                            src: arg_base_v + arg_off,
                        },
                        span,
                    );
                }
                WriteBack::LocalArr1V {
                    arr_v,
                    r_i,
                    arg_off,
                } => {
                    self.emit(
                        Op::StoreLArr1V {
                            arr_v,
                            r_i,
                            r_val: arg_base_v + arg_off,
                        },
                        span,
                    );
                }
                WriteBack::GlobalArr1V { gid, r_i, arg_off } => {
                    self.emit(
                        Op::StoreGArr1V {
                            gid,
                            r_i,
                            r_val: arg_base_v + arg_off,
                        },
                        span,
                    );
                }
                WriteBack::LocalList1V {
                    list_v,
                    r_i,
                    arg_off,
                } => {
                    self.emit(
                        Op::StoreLList1V {
                            list_v,
                            r_i,
                            r_val: arg_base_v + arg_off,
                        },
                        span,
                    );
                }
                WriteBack::GlobalList1V { gid, r_i, arg_off } => {
                    self.emit(
                        Op::StoreGList1V {
                            gid,
                            r_i,
                            r_val: arg_base_v + arg_off,
                        },
                        span,
                    );
                }
            }
        }
        Some((dst_i, dst_v, dst_some))
    }
}
