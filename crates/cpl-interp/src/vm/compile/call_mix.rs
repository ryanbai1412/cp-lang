//! Calls of compiled fns from compiled code: value args first, then each
//! `mut` arg moved out of its place, written back after the call.

use super::stmt::Callee;
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
        if self.program.judge_fns.contains(&fn_id) || args.len() != f.params.len() {
            return None;
        }
        if (args.iter().zip(&f.params)).any(|(a, p)| matches!(a, Arg::Mut(_)) != p.mutable) {
            return None;
        }
        if self.call_must_lend(fn_id, args) {
            return self.compile_call_lending(fn_id, args, ret_mode, span);
        }
        let tys: Vec<&'p Ty> = (f.locals.iter().take(f.params.len()))
            .map(|l| &l.ty)
            .collect();
        let mix = self.compile_mix_args(&tys, args, Some(f), span)?;
        let (arg_base_i, arg_base_v) = (mix.arg_base_i, mix.arg_base_v);
        let (dst_i, dst_v, dst_some) = match ret_mode {
            CallRet::Unit => (0, 0, 0),
            CallRet::I => (self.alloc_i(), 0, 0),
            CallRet::OptI => (self.alloc_i(), 0, self.alloc_i()),
            CallRet::V => (0, self.alloc_v(), 0),
            CallRet::Tuple(len) => {
                let base = self.next_i;
                for _ in 0..len {
                    self.alloc_i();
                }
                (base, 0, 0)
            }
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
        self.emit_mix_writebacks(mix, span);
        Some((dst_i, dst_v, dst_some))
    }

    /// A call whose fn values passed beside its `mut` args may reach their
    /// places: the places are lent, as the tree-walker lends them, and the
    /// callee puts them back around each closure it calls (`show_lent`).
    fn compile_call_lending(
        &mut self,
        fn_id: FnId,
        args: &'p [Arg],
        ret_mode: CallRet,
        span: Span,
    ) -> Option<(u16, u16, u16)> {
        let scalar = match ret_mode {
            CallRet::Unit | CallRet::V => false,
            CallRet::I => true,
            CallRet::OptI | CallRet::Tuple(_) => return None,
        };
        let dst = self.emit_traced_call(span, Callee::Fn(fn_id), args, scalar);
        Some(match scalar {
            true => (dst, 0, 0),
            false => (0, dst, 0),
        })
    }

    /// Whether [`Self::compile_mix_args`] takes `args` for params of types
    /// `tys`: which `mut` args go by their place path, and how many int
    /// and value arg slots they fill.
    pub(super) fn mix_arg_plan(
        &self,
        tys: &[&'p Ty],
        args: &'p [Arg],
        callee: Option<&'p cpl_ir::Function>,
    ) -> Option<MixPlan<'p>> {
        // The checker proves several `mut` args never overlap (E0601), so
        // each is taken in turn and written back after the call.
        let multi = args.iter().filter(|a| matches!(a, Arg::Mut(_))).count() > 1;
        // A mut arg is a local or global, a 1D indexed V place, or any other
        // place walked by path.
        let mut takes = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let Arg::Mut(place) = a else {
                takes.push(None);
                continue;
            };
            let take = match &place.kind {
                PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                    SlotLoc::I(r, _) => MutTake::LocalI(r),
                    SlotLoc::OptI { val, some, .. } => MutTake::LocalOptI { val, some },
                    SlotLoc::V(r) => MutTake::LocalV(r),
                    SlotLoc::SmallTuple { base, len, kinds } => {
                        MutTake::LocalTuple { base, len, kinds }
                    }
                },
                PlaceKind::Global(gid) => MutTake::Global(*gid as u16),
                PlaceKind::Index(base, indices)
                    if !multi
                        && indices.len() == 1
                        && ScalarKind::of_ty(&indices[0].ty, self.program).is_some()
                        && ScalarKind::of_ty(tys[i], self.program).is_none()
                        && ScalarKind::of_opt_ty(tys[i], self.program).is_none()
                        && self.elem_1d_v(base).is_some() =>
                {
                    let (root, is_arr) = self.elem_1d_v(base)?;
                    MutTake::Elem {
                        root,
                        is_arr,
                        index: &indices[0],
                    }
                }
                PlaceKind::Index(..) | PlaceKind::Field(..) | PlaceKind::Some(_) => MutTake::Path,
            };
            takes.push(Some((place, take)));
        }
        let mut ni = 0usize;
        let mut nv = 0usize;
        for (k, &ty) in tys.iter().enumerate() {
            if let Some(len) = arg_tuple_len(self.program, callee, ty, &args[k], k) {
                ni += len as usize;
            } else if ScalarKind::of_ty(ty, self.program).is_some() {
                ni += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                ni += 2;
            } else {
                nv += 1;
            }
        }
        Some((takes, ni, nv))
    }

    /// The args of a call to params of types `tys`: value args first, then
    /// each `mut` arg moved out of its place, all into consecutive arg
    /// slots.
    pub(super) fn compile_mix_args(
        &mut self,
        tys: &[&'p Ty],
        args: &'p [Arg],
        callee: Option<&'p cpl_ir::Function>,
        span: Span,
    ) -> Option<MixArgs> {
        let (takes, ni, nv) = self.mix_arg_plan(tys, args, callee)?;
        let program = self.program;
        let tuple_len = |k: usize| arg_tuple_len(program, callee, tys[k], &args[k], k);
        let mut i_args: Vec<u16> = vec![0; ni];
        let mut v_args: Vec<(u16, bool)> = vec![(0, false); nv];
        let mut writebacks: Vec<WriteBack> = Vec::new();
        // Precompute argument slot offsets so all Arg::Value expressions are evaluated
        // before any Arg::Mut place is moved out (e.g. `a.union(a.shifted(w))`).
        let mut param_offs: Vec<usize> = Vec::with_capacity(args.len());
        let mut cur_i = 0usize;
        let mut cur_v = 0usize;
        for (k, &ty) in tys.iter().enumerate() {
            if let Some(len) = tuple_len(k) {
                param_offs.push(cur_i);
                cur_i += len as usize;
            } else if ScalarKind::of_ty(ty, self.program).is_some() {
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
        let marks = self.code_mark();
        // Pass 1: evaluate all value arguments first.
        for (i, a) in args.iter().enumerate() {
            if let Arg::Value(ae) = a {
                let ty = tys[i];
                let off = param_offs[i];
                if let Some(len) = tuple_len(i) {
                    let base = self.small_tuple_operand(ae, len);
                    for k in 0..len as usize {
                        i_args[off + k] = base + k as u16;
                    }
                } else if ScalarKind::of_ty(ty, self.program).is_some() {
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
        // Pass 2: walk to every path place before any is moved out of.
        let mut paths = vec![0; takes.len()];
        for (i, take) in takes.iter().enumerate() {
            if let Some((place, MutTake::Path)) = *take {
                let Some(path) = self.compile_place_path_to(place, true) else {
                    self.rewind(marks);
                    return None;
                };
                paths[i] = path;
            }
        }
        // Pass 3: move out all mut arguments after value arguments are evaluated.
        for (i, take) in takes.iter().enumerate() {
            let Some((place, take)) = *take else {
                continue;
            };
            let ty = tys[i];
            let off = param_offs[i];
            match take {
                MutTake::Path => {
                    let path = paths[i];
                    let r = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst: r, path }, place.span);
                    if let Some(kind) = ScalarKind::of_opt_ty(ty, self.program) {
                        let (dst_val, dst_some) = (self.alloc_i(), self.alloc_i());
                        let unbox = Op::UnboxOptI {
                            dst_val,
                            dst_some,
                            src_v: r,
                        };
                        self.emit(unbox, place.span);
                        i_args[off] = dst_val;
                        i_args[off + 1] = dst_some;
                        writebacks.push(WriteBack::PlaceOptI {
                            path,
                            kind,
                            arg_off: off as u16,
                        });
                        continue;
                    }
                    if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
                        let dst_i = self.alloc_i();
                        self.emit(Op::UnboxI { dst_i, src_v: r }, place.span);
                        i_args[off] = dst_i;
                        writebacks.push(WriteBack::PlaceI {
                            path,
                            kind,
                            arg_off: off as u16,
                        });
                        continue;
                    }
                    writebacks.push(WriteBack::Place {
                        path,
                        arg_off: off as u16,
                    });
                    v_args[off] = (r, true);
                    continue;
                }
                MutTake::LocalI(r) => {
                    i_args[off] = r;
                    writebacks.push(WriteBack::LocalI {
                        dst: r,
                        arg_off: off as u16,
                    });
                }
                MutTake::LocalOptI { val, some } => {
                    i_args[off] = val;
                    i_args[off + 1] = some;
                    writebacks.push(WriteBack::LocalOptI {
                        dst_val: val,
                        dst_some: some,
                        arg_off: off as u16,
                    });
                }
                MutTake::LocalV(r) => {
                    v_args[off] = (r, true);
                    writebacks.push(WriteBack::LocalV {
                        dst: r,
                        arg_off: off as u16,
                    });
                }
                MutTake::LocalTuple { base, len, kinds } => {
                    let dst_v = self.alloc_v();
                    let src_base = base;
                    let op = Op::BoxSmallTuple {
                        dst_v,
                        src_base,
                        len,
                        kinds,
                    };
                    self.emit(op, place.span);
                    v_args[off] = (dst_v, true);
                    let arg_off = off as u16;
                    writebacks.push(WriteBack::LocalTuple { base, len, arg_off });
                }
                MutTake::Global(gid) => {
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
                MutTake::Elem {
                    root,
                    is_arr,
                    index,
                } => {
                    let r_i = self.compile_expr_i(index);
                    let r = self.alloc_v();
                    match (root, is_arr) {
                        (ElemRoot::Local(arr_v), true) => {
                            self.emit(Op::TakeLArr1V { dst: r, arr_v, r_i }, place.span);
                            writebacks.push(WriteBack::LocalArr1V {
                                arr_v,
                                r_i,
                                arg_off: off as u16,
                            });
                        }
                        (ElemRoot::Global(gid), true) => {
                            self.emit(Op::TakeGArr1V { dst: r, gid, r_i }, place.span);
                            writebacks.push(WriteBack::GlobalArr1V {
                                gid,
                                r_i,
                                arg_off: off as u16,
                            });
                        }
                        (ElemRoot::Local(list_v), false) => {
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
                        (ElemRoot::Global(gid), false) => {
                            self.emit(Op::TakeGList1V { dst: r, gid, r_i }, place.span);
                            writebacks.push(WriteBack::GlobalList1V {
                                gid,
                                r_i,
                                arg_off: off as u16,
                            });
                        }
                    }
                    v_args[off] = (r, true);
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
        Some(MixArgs {
            arg_base_i,
            arg_base_v,
            writebacks,
        })
    }

    /// Puts each `mut` arg of `mix` back in its place after the call.
    pub(super) fn emit_mix_writebacks(&mut self, mix: MixArgs, span: Span) {
        let MixArgs {
            arg_base_i,
            arg_base_v,
            writebacks,
        } = mix;
        for wb in writebacks {
            match wb {
                WriteBack::Place { path, arg_off } => {
                    let r_val = arg_base_v + arg_off;
                    self.emit(Op::StorePlaceV { path, r_val }, span);
                    self.emit(Op::ClearV { reg: r_val }, span);
                }
                WriteBack::PlaceI {
                    path,
                    kind,
                    arg_off,
                } => {
                    let r_val = arg_base_i + arg_off;
                    self.emit(Op::StorePlaceI { path, r_val, kind }, span);
                }
                WriteBack::PlaceOptI {
                    path,
                    kind,
                    arg_off,
                } => {
                    let r_val = self.alloc_v();
                    let boxed = Op::BoxOptI {
                        dst_v: r_val,
                        val: arg_base_i + arg_off,
                        some: arg_base_i + arg_off + 1,
                        kind,
                    };
                    self.emit(boxed, span);
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
                WriteBack::LocalTuple { base, len, arg_off } => {
                    let src_v = arg_base_v + arg_off;
                    let op = Op::UnboxSmallTuple {
                        dst_base: base,
                        len,
                        src_v,
                    };
                    self.emit(op, span);
                    self.emit(Op::ClearV { reg: src_v }, span);
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
    }

    /// Whether `base[i]` is a 1D list or array the VM indexes in a value
    /// register or global directly.
    fn elem_1d_v(&self, base: &Place) -> Option<(ElemRoot, bool)> {
        let root = match &base.kind {
            PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::V(r) => ElemRoot::Local(r),
                _ => return None,
            },
            PlaceKind::Global(gid) => ElemRoot::Global(*gid as u16),
            _ => return None,
        };
        match &base.ty {
            Ty::Array { dims, .. }
                if dims.len() == 1
                    && !matches!(dims[0], cpl_ir::Dim::IntFrom(_) | cpl_ir::Dim::Sym(_)) =>
            {
                Some((root, true))
            }
            Ty::List(_) => Some((root, false)),
            _ => None,
        }
    }
}

/// The arg slots [`Compiler::compile_mix_args`] filled, and how each `mut`
/// arg goes back.
pub(super) struct MixArgs {
    pub arg_base_i: u16,
    pub arg_base_v: u16,
    writebacks: Vec<WriteBack>,
}

/// Where a `mut` arg goes back after the call, from its arg slot.
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
    LocalTuple {
        base: u16,
        len: u8,
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
    PlaceOptI {
        path: u16,
        kind: ScalarKind,
        arg_off: u16,
    },
    PlaceI {
        path: u16,
        kind: ScalarKind,
        arg_off: u16,
    },
}

/// Per arg, the `mut` place and how it is taken (`None` for a value arg),
/// and the int and value arg slots the call needs.
pub(super) type MixPlan<'p> = (Vec<Option<(&'p Place, MutTake<'p>)>>, usize, usize);

/// How [`Compiler::compile_mix_args`] moves one `mut` arg out of its place.
#[derive(Clone, Copy)]
pub(super) enum MutTake<'p> {
    LocalI(u16),
    LocalOptI {
        val: u16,
        some: u16,
    },
    LocalV(u16),
    /// A tuple of scalars held one field per int register.
    LocalTuple {
        base: u16,
        len: u8,
        kinds: [crate::value::SmallFieldKind; 5],
    },
    Global(u16),
    /// One scalar-indexed element of a 1D array (`is_arr`) or list of values.
    Elem {
        root: ElemRoot,
        is_arr: bool,
        index: &'p Expr,
    },
    /// Any other place, walked by its path.
    Path,
}

/// The local V register or global a 1D element is taken from.
#[derive(Clone, Copy)]
pub(super) enum ElemRoot {
    Local(u16),
    Global(u16),
}

impl Compiler<'_, '_> {
    /// The shape `RetSmallTuple` returns `e` in, when this body returns small tuples of `e`'s.
    pub(super) fn tuple_ret_of(&self, e: &Expr) -> Option<(u8, [crate::value::SmallFieldKind; 5])> {
        let (len, kinds) = self.tuple_ret?;
        let (l, k) = small_tuple_shape(&e.ty, self.program)?;
        (l == len && k == kinds).then_some((len, kinds))
    }

    /// Whether the call `e` returns a small tuple of `len` fields.
    pub(super) fn call_returns_small_tuple(&self, e: &Expr, len: u8) -> bool {
        let ret = match &e.kind {
            ExprKind::Call(fn_id, _) => &self.program.fns[*fn_id].ret,
            ExprKind::CallExpr(callee, _) => match &callee.ty {
                Ty::Fn(_, ret) => &**ret,
                _ => return false,
            },
            _ => return false,
        };
        small_tuple_shape(ret, self.program).is_some_and(|(l, _)| l == len)
    }
}

/// The Int registers a small tuple arg takes: as the fn's param holds it, or
/// for a closure (whose params all hold them so) any arg that isn't `mut`.
fn arg_tuple_len(
    program: &cpl_ir::Program,
    callee: Option<&cpl_ir::Function>,
    ty: &cpl_ir::Ty,
    arg: &Arg,
    k: usize,
) -> Option<u8> {
    match callee {
        Some(f) => tuple_param_len(program, f, k),
        None if matches!(arg, Arg::Value(_)) => small_tuple_shape(ty, program).map(|(l, _)| l),
        None => None,
    }
}
