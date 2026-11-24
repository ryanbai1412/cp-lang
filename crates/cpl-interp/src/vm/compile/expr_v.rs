use super::{Compiler, GiveTarget};
use crate::value::Value;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::Prim;
use cpl_ir::{
    BinOp, Builtin, Cast, Expr, ExprKind, FmtArg, FmtPiece, Mutation, Place, PlaceKind, Stmt, Ty,
};
use std::rc::Rc;

impl<'p, 'f> Compiler<'p, 'f> {
    /// The one scalar kind every field of a 2- to 16-field tuple, record or
    /// struct has, so it builds from Int registers in one op.
    fn uniform_scalar_tuple(&self, expr: &Expr, fields: &[Expr]) -> Option<ScalarKind> {
        if !(2..=16).contains(&fields.len())
            || !matches!(expr.ty, Ty::Tuple(_) | Ty::Record(_) | Ty::Struct(_))
        {
            return None;
        }
        let kind = ScalarKind::of_ty(&fields[0].ty, self.program)?;
        (fields.iter())
            .all(|f| ScalarKind::of_ty(&f.ty, self.program) == Some(kind))
            .then_some(kind)
    }

    fn make_tuple_i(&mut self, fields: &'p [Expr], kind: ScalarKind, span: Span) -> u16 {
        let regs: Vec<u16> = fields.iter().map(|f| self.compile_expr_i(f)).collect();
        let base_i = self.next_i;
        for r in regs {
            let slot = self.alloc_i();
            if slot != r {
                self.emit(Op::MoveI { dst: slot, src: r }, span);
            }
        }
        let dst = self.alloc_v();
        let len = fields.len() as u8;
        self.emit(
            Op::MakeTupleI {
                dst,
                base_i,
                len,
                kind,
            },
            span,
        );
        dst
    }

    pub(super) fn compile_expr_v(&mut self, expr: &'p Expr) -> u16 {
        if let Some(place) = self.reported_write(expr) {
            return self.reporting_write(expr, place, |c| c.compile_expr_v(expr));
        }
        let span = expr.span;
        if let Some((fn_id, args)) = self.traced_call(expr) {
            if ScalarKind::of_ty(&expr.ty, self.program).is_none() {
                return self.emit_traced_call(expr.span, fn_id, args, false);
            }
        }
        if let Some(kind) = ScalarKind::of_ty(&expr.ty, self.program) {
            let src_i = self.compile_expr_i(expr);
            let dst_v = self.alloc_v();
            self.emit(Op::BoxI { dst_v, src_i, kind }, span);
            return dst_v;
        }
        if let Some(kind) = ScalarKind::of_opt_ty(&expr.ty, self.program) {
            let (val, some) = self.compile_expr_opt_i(expr);
            let dst_v = self.alloc_v();
            self.emit(
                Op::BoxOptI {
                    dst_v,
                    val,
                    some,
                    kind,
                },
                span,
            );
            return dst_v;
        }
        if let ExprKind::Tuple(fields) = &expr.kind {
            if let Some(kind) = self.uniform_scalar_tuple(expr, fields) {
                return self.make_tuple_i(fields, kind, span);
            }
        }
        match &expr.kind {
            ExprKind::Format(pieces) => self.compile_format(pieces, span),
            ExprKind::Str(bytes) => {
                let idx = self.consts_v.len() as u32;
                self.consts_v.push(Value::Str(Rc::new(bytes.clone())));
                let dst = self.alloc_v();
                self.emit(Op::ConstV { dst, idx }, span);
                dst
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::V(r) => r,
                SlotLoc::I(src_i, kind) => {
                    let dst_v = self.alloc_v();
                    self.emit(Op::BoxI { dst_v, src_i, kind }, span);
                    dst_v
                }
                SlotLoc::OptI { val, some, kind } => {
                    let dst_v = self.alloc_v();
                    self.emit(
                        Op::BoxOptI {
                            dst_v,
                            val,
                            some,
                            kind,
                        },
                        span,
                    );
                    dst_v
                }
                SlotLoc::SmallTuple { base, len, kinds } => {
                    let dst_v = self.alloc_v();
                    self.emit(
                        Op::BoxSmallTuple {
                            dst_v,
                            src_base: base,
                            len,
                            kinds,
                        },
                        span,
                    );
                    dst_v
                }
            },
            ExprKind::Global(gid) => {
                let dst = self.alloc_v();
                self.emit(
                    Op::LoadGlobalV {
                        dst,
                        gid: *gid as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::Field(base, idx) => {
                let obj_v = self.compile_expr_v(base);
                let dst = self.alloc_v();
                self.emit(
                    Op::FieldV {
                        dst,
                        obj_v,
                        field: *idx as u16,
                    },
                    span,
                );
                if obj_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: obj_v }, span);
                }
                dst
            }
            ExprKind::Index(base, indices) if Self::indexes_nd_cell(&base.ty, indices) => {
                self.compile_index_nd_v(base, indices, span)
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let dst = self.alloc_v();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadGArr2V {
                                    dst,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1V {
                                    dst,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                span,
                            );
                        }
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let r_j =
                                (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                            let dst = self.alloc_v();
                            if let Some(r_j) = r_j {
                                self.emit(
                                    Op::LoadLArr2V {
                                        dst,
                                        arr_v,
                                        r_i,
                                        r_j,
                                    },
                                    span,
                                );
                            } else {
                                self.emit(Op::LoadLArr1V { dst, arr_v, r_i }, span);
                            }
                            dst
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let dst = self.alloc_v();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadLArr2V {
                                    dst,
                                    arr_v,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(Op::LoadLArr1V { dst, arr_v, r_i }, span);
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Float(f) => {
                let idx = self.consts_i.len() as u16;
                self.consts_i.push(f.to_bits() as i64);
                let dst = self.alloc_v();
                self.emit(Op::ConstF64 { dst, idx }, span);
                dst
            }
            ExprKind::Binary(op, a, b)
                if matches!(a.ty, Ty::Float)
                    && matches!(b.ty, Ty::Float)
                    && matches!(
                        op,
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow
                    ) =>
            {
                let ra = self.compile_expr_v(a);
                let rb = self.compile_expr_v(b);
                let dst = self.alloc_v();
                self.emit(
                    Op::FloatBin {
                        dst,
                        a: ra,
                        b: rb,
                        op: *op,
                    },
                    span,
                );
                dst
            }
            ExprKind::Builtin(b @ (Builtin::Min | Builtin::Max), args)
                if args.len() == 2 && matches!(expr.ty, Ty::Float) =>
            {
                let ra = self.compile_expr_v(&args[0]);
                let rb = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(
                    Op::FloatMinMax {
                        dst,
                        a: ra,
                        b: rb,
                        is_max: matches!(b, Builtin::Max),
                    },
                    span,
                );
                dst
            }
            ExprKind::If(cond, then_e, else_e) => {
                let dst = self.alloc_v();
                let rc = self.compile_expr_i(cond);
                let j_else = self.emit(Op::JmpIfZero { reg: rc, target: 0 }, span);
                let rt = self.compile_expr_v(then_e);
                if rt != dst {
                    if rt >= self.num_local_v {
                        self.emit(Op::MoveVConsume { dst, src: rt }, span);
                    } else {
                        self.emit(Op::MoveV { dst, src: rt }, span);
                    }
                }
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let re = self.compile_expr_v(else_e);
                if re != dst {
                    if re >= self.num_local_v {
                        self.emit(Op::MoveVConsume { dst, src: re }, span);
                    } else {
                        self.emit(Op::MoveV { dst, src: re }, span);
                    }
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) if matches!(base.ty, Ty::List(_)) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_v();
                        self.emit(
                            Op::LoadGList1V {
                                dst,
                                gid: *gid as u16,
                                r_i,
                            },
                            span,
                        );
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_v();
                            self.emit(Op::LoadLList1V { dst, list_v, r_i }, span);
                            dst
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    _ => {
                        let list_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_v();
                        self.emit(Op::LoadLList1V { dst, list_v, r_i }, span);
                        if list_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: list_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Tuple(fields)
                if fields.len() <= 16
                    && matches!(expr.ty, Ty::Tuple(_) | Ty::Record(_) | Ty::Struct(_)) =>
            {
                let mut regs = Vec::with_capacity(fields.len());
                for f in fields {
                    regs.push(self.compile_expr_v(f));
                }
                let base_v = self.next_v;
                for r in regs {
                    let slot = self.alloc_v();
                    if slot != r {
                        self.emit(Op::MoveV { dst: slot, src: r }, span);
                    }
                }
                let dst = self.alloc_v();
                self.emit(
                    Op::MakeTuple {
                        dst,
                        base_v,
                        len: fields.len() as u8,
                    },
                    span,
                );
                dst
            }
            ExprKind::Tuple(fields) if fields.len() <= 16 && matches!(expr.ty, Ty::List(_)) => {
                let mut regs = Vec::with_capacity(fields.len());
                for f in fields {
                    regs.push(self.compile_expr_v(f));
                }
                let base_v = self.next_v;
                for r in regs {
                    let slot = self.alloc_v();
                    if slot != r {
                        self.emit(Op::MoveV { dst: slot, src: r }, span);
                    }
                }
                let dst = self.alloc_v();
                self.emit(
                    Op::MakeList {
                        dst,
                        base_v,
                        len: fields.len() as u8,
                    },
                    span,
                );
                dst
            }
            ExprKind::Builtin(Builtin::ListFilled, args) if args.len() == 2 => {
                let r_n = self.compile_expr_i(&args[0]);
                let val_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(Op::ListFilled { dst, r_n, val_v }, span);
                if val_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: val_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::ArrayToList | Builtin::ToList, args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(Op::ToList { dst, src_v }, span);
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::ListSlice, args) if args.len() == 3 => {
                let src_v = self.compile_expr_v(&args[0]);
                let r_lo = self.compile_expr_i(&args[1]);
                let r_hi = self.compile_expr_i(&args[2]);
                let dst = self.alloc_v();
                self.emit(
                    Op::ListSlice {
                        dst,
                        src_v,
                        r_lo,
                        r_hi,
                    },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::StrSlice, args) if args.len() == 3 => {
                let src_v = self.compile_expr_v(&args[0]);
                let r_lo = self.compile_expr_i(&args[1]);
                let r_hi = self.compile_expr_i(&args[2]);
                let dst = self.alloc_v();
                self.emit(
                    Op::StrSlice {
                        dst,
                        src_v,
                        r_lo,
                        r_hi,
                    },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Fill(dims, value) if dims.len() <= 4 => {
                let mut dim_regs: Vec<u16> = Vec::with_capacity(dims.len());
                for dim in dims {
                    let r = match dim {
                        cpl_ir::DimSpec::Pos(d) => {
                            let r = self.alloc_i();
                            self.emit(
                                Op::DomainLen {
                                    dst: r,
                                    dom: *d as u16,
                                },
                                span,
                            );
                            r
                        }
                        cpl_ir::DimSpec::Bound(d) => {
                            let r = self.alloc_i();
                            self.emit(
                                Op::DomainLen {
                                    dst: r,
                                    dom: *d as u16,
                                },
                                span,
                            );
                            let one = self.alloc_i();
                            self.emit_const_i(one, 1, span);
                            let out = self.alloc_i();
                            self.emit(
                                Op::AddI {
                                    dst: out,
                                    a: r,
                                    b: one,
                                },
                                span,
                            );
                            out
                        }
                        cpl_ir::DimSpec::Sym(d) => {
                            let n = self.alloc_i();
                            self.emit(
                                Op::DomainLen {
                                    dst: n,
                                    dom: *d as u16,
                                },
                                span,
                            );
                            let one = self.alloc_i();
                            self.emit_const_i(one, 1, span);
                            let np1 = self.alloc_i();
                            self.emit(
                                Op::AddI {
                                    dst: np1,
                                    a: n,
                                    b: one,
                                },
                                span,
                            );
                            let prod = self.alloc_i();
                            self.emit(
                                Op::MulI {
                                    dst: prod,
                                    a: n,
                                    b: np1,
                                },
                                span,
                            );
                            let two = self.alloc_i();
                            self.emit_const_i(two, 2, span);
                            let out = self.alloc_i();
                            self.emit(
                                Op::DivI {
                                    dst: out,
                                    a: prod,
                                    b: two,
                                },
                                span,
                            );
                            out
                        }
                        cpl_ir::DimSpec::Enum(e) => {
                            let r = self.alloc_i();
                            self.emit_const_i(
                                r,
                                self.program.enums[*e].variants.len() as i64,
                                span,
                            );
                            r
                        }
                        cpl_ir::DimSpec::Int(len) | cpl_ir::DimSpec::IntFrom(_, len) => {
                            self.compile_expr_i(len)
                        }
                    };
                    dim_regs.push(r);
                }
                let dims_base = self.next_i;
                for r in dim_regs {
                    let slot = self.alloc_i();
                    if slot != r {
                        self.emit(Op::MoveI { dst: slot, src: r }, span);
                    }
                }
                let (fill_v, fill_ty) = match value {
                    Some(v) => (self.compile_expr_v(v), u16::MAX),
                    None => match &expr.ty {
                        Ty::Array { elem, .. } => {
                            let idx = self.fill_tys.len() as u16;
                            self.fill_tys.push(elem.as_ref());
                            (u16::MAX, idx)
                        }
                        _ => (u16::MAX, u16::MAX),
                    },
                };
                let dst = self.alloc_v();
                self.emit(
                    Op::MakeFill {
                        dst,
                        dims_base,
                        ndims: dims.len() as u8,
                        fill_v,
                        fill_ty,
                    },
                    span,
                );
                if fill_v != u16::MAX && fill_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: fill_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::Copy, args) if args.len() == 1 => {
                self.compile_expr_v(&args[0])
            }
            ExprKind::None => {
                let dst_v = self.alloc_v();
                self.emit(Op::ConstNoneV { dst_v }, span);
                dst_v
            }
            ExprKind::Some(inner) => {
                let src_v = self.compile_expr_v(inner);
                let dst_v = self.alloc_v();
                self.emit(Op::WrapSomeV { dst_v, src_v }, span);
                dst_v
            }
            ExprKind::Unwrap(inner, _) => {
                let src_v = self.compile_expr_v(inner);
                let dst_v = self.alloc_v();
                self.emit(Op::UnwrapV { dst_v, src_v }, span);
                dst_v
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs) => {
                let lv = self.compile_expr_v(lhs);
                let is_some = self.alloc_i();
                self.emit(
                    Op::IsSomeV {
                        dst_i: is_some,
                        src_v: lv,
                    },
                    lhs.span,
                );
                let dst = self.alloc_v();
                let j_none = self.emit(
                    Op::JmpIfZero {
                        reg: is_some,
                        target: 0,
                    },
                    span,
                );
                // An optional default keeps the optional; a plain one gives what is inside.
                if lhs.ty == expr.ty {
                    self.emit(Op::MoveV { dst, src: lv }, lhs.span);
                } else {
                    self.emit(
                        Op::UnwrapV {
                            dst_v: dst,
                            src_v: lv,
                        },
                        lhs.span,
                    );
                }
                let j_done = self.emit(Op::Jmp { target: 0 }, span);
                let none_pc = self.ops.len() as u32;
                self.patch(j_none, none_pc);
                let rv = self.compile_expr_v(rhs);
                if rv != dst {
                    self.emit(Op::MoveV { dst, src: rv }, rhs.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_done, end_pc);
                dst
            }
            ExprKind::Builtin(Builtin::Last, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_v = self.alloc_v();
                self.emit(Op::SeqLastV { dst_v, seq_v }, span);
                dst_v
            }
            ExprKind::Builtin(Builtin::First, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_v = self.alloc_v();
                self.emit(Op::SeqFirstV { dst_v, seq_v }, span);
                dst_v
            }
            ExprKind::Mutate(Mutation::Pop, place, args)
                if args.is_empty() && matches!(place.ty, Ty::List(_)) =>
            {
                match &place.kind {
                    PlaceKind::Local(slot) => {
                        if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                            let dst_v = self.alloc_v();
                            self.emit(Op::ListPopLV { dst_v, list_v }, span);
                            dst_v
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    PlaceKind::Global(gid) => {
                        let dst_v = self.alloc_v();
                        self.emit(
                            Op::ListPopGV {
                                dst_v,
                                gid: *gid as u16,
                            },
                            span,
                        );
                        dst_v
                    }
                    PlaceKind::Field(obj_place, field) => {
                        if let PlaceKind::Local(slot) = obj_place.kind {
                            if let SlotLoc::V(obj_v) = self.slot_loc(slot) {
                                let dst_v = self.alloc_v();
                                self.emit(
                                    Op::ListPopFieldLV {
                                        dst_v,
                                        obj_v,
                                        field: *field as u16,
                                    },
                                    span,
                                );
                                return dst_v;
                            }
                        }
                        self.emit_fallback_expr_v(expr)
                    }
                    _ => self.emit_fallback_expr_v(expr),
                }
            }
            ExprKind::Mutate(Mutation::Take, place, args) if args.is_empty() => {
                if let PlaceKind::Local(slot) = place.kind {
                    if let SlotLoc::V(src) = self.slot_loc(slot) {
                        let dst = self.alloc_v();
                        self.emit(Op::TakeLocalV { dst, src }, span);
                        return dst;
                    }
                }
                if let Some(path) = self.compile_place_path(place) {
                    let dst = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst, path }, span);
                    return dst;
                }
                self.emit_fallback_expr_v(expr)
            }
            ExprKind::Closure(cid) => {
                let def = &self.program.closures[*cid];
                {
                    let mut caps = Vec::with_capacity(def.captures.len());
                    for cap in &def.captures {
                        if cap.is_global {
                            if cap.by_ref {
                                caps.push(CapSrc::GlobalRef(cap.outer as u16));
                            } else {
                                caps.push(CapSrc::GlobalVal(cap.outer as u16));
                            }
                        } else if cap.by_ref {
                            // Kept sorted and unique as it is built.
                            let slot = cap.outer as u16;
                            if let Err(at) = self.ref_slots.binary_search(&slot) {
                                self.ref_slots.insert(at, slot);
                            }
                            let v = match self.slot_loc(cap.outer) {
                                SlotLoc::V(r) => r,
                                _ => u16::MAX,
                            };
                            caps.push(CapSrc::LocalRef {
                                slot: cap.outer as u16,
                                v,
                            });
                        } else {
                            match self.slot_loc(cap.outer) {
                                SlotLoc::I(r, kind) => caps.push(CapSrc::LocalI(r, kind)),
                                SlotLoc::OptI { val, some, kind } => {
                                    caps.push(CapSrc::LocalOptI { val, some, kind });
                                }
                                SlotLoc::SmallTuple { base, len, kinds } => {
                                    caps.push(CapSrc::LocalSmallTuple { base, len, kinds });
                                }
                                SlotLoc::V(r) => caps.push(CapSrc::LocalV(r)),
                            }
                        }
                    }
                    let spec = self.closure_specs.len() as u16;
                    self.closure_specs.push(ClosureMakeSpec {
                        cid: *cid as u32,
                        caps,
                    });
                    let dst = self.alloc_v();
                    self.emit(Op::MakeClosure { dst, spec }, span);
                    dst
                }
            }
            ExprKind::Range { lo, hi }
                if ScalarKind::of_ty(&lo.ty, self.program).is_some()
                    && ScalarKind::of_ty(&hi.ty, self.program).is_some() =>
            {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let dst = self.alloc_v();
                self.emit(Op::MakeRange { dst, r_lo, r_hi }, span);
                dst
            }
            ExprKind::DomainVal(d) => {
                let dst = self.alloc_v();
                let dom = *d as u16;
                self.emit(Op::DomainValV { dst, dom }, span);
                dst
            }
            ExprKind::Range { lo, hi } if matches!(&expr.ty, Ty::Range(inner) if matches!(**inner, Ty::Float | Ty::I128)) =>
            {
                let float = matches!(&expr.ty, Ty::Range(inner) if **inner == Ty::Float);
                let lo_v = self.compile_expr_v(lo);
                let hi_v = self.compile_expr_v(hi);
                let dst = self.alloc_v();
                self.emit(
                    Op::MakeRangeV {
                        dst,
                        lo: lo_v,
                        hi: hi_v,
                        float,
                    },
                    span,
                );
                for reg in [lo_v, hi_v] {
                    if reg >= self.num_local_v {
                        self.emit(Op::ClearV { reg }, span);
                    }
                }
                dst
            }
            ExprKind::Do(block) => {
                let dst = self.alloc_v();
                self.give_body(
                    GiveTarget::V {
                        dst,
                        exits: Vec::new(),
                    },
                    &block.stmts,
                    Self::compile_do_stmts,
                );
                dst
            }
            ExprKind::Call(fn_id, args) => {
                if let Some((_, dst_v, _)) =
                    self.try_compile_call_mix(*fn_id, args, CallRet::V, span)
                {
                    dst_v
                } else {
                    self.emit_fallback_expr_v(expr)
                }
            }
            ExprKind::Cast(Cast::IntToFloat, inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_v();
                self.emit(Op::IntToFloat { dst, src }, span);
                dst
            }
            ExprKind::Intrinsic(Prim::Sqrt, args) if args.len() == 1 && args[0].ty == Ty::Float => {
                let src = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(Op::FloatSqrt { dst, src }, span);
                if src >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src }, span);
                }
                dst
            }
            ExprKind::Unit => {
                let dst = self.alloc_v();
                self.emit(Op::ConstUnit { dst }, span);
                dst
            }
            ExprKind::Builtin(b @ (Builtin::SetMin | Builtin::SetMax), args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(
                    Op::SetMinMaxV {
                        dst,
                        src_v,
                        is_max: matches!(b, Builtin::SetMax),
                    },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::MapGet, args)
                if args.len() == 2
                    && matches!(&args[0].ty, Ty::Map(kty, _) | Ty::HashMap(kty, _) if !crate::value::ty_has_cached(kty, self.program) && !crate::value::keys::user_ordered(kty, self.program)) =>
            {
                let map_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(Op::MapGetV { dst, map_v, key_v }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                dst
            }
            ExprKind::Builtin(
                b @ (Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt),
                args,
            ) if args.len() == 2
                && matches!(
                    args[0].ty,
                    Ty::Set(_) | Ty::HashSet(_) | Ty::Map(_, _) | Ty::HashMap(_, _)
                )
                && !crate::value::ty_has_cached(&args[1].ty, self.program)
                && !crate::value::keys::user_ordered(&args[1].ty, self.program) =>
            {
                // 0 to 3: `first_ge`, `first_gt`, `last_le`, `last_lt`.
                let mode = 2 * u8::from(matches!(b, Builtin::LastLe | Builtin::LastLt))
                    + u8::from(matches!(b, Builtin::FirstGt | Builtin::LastLt));
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(
                    Op::SetMapBoundV {
                        dst,
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
                dst
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
                    let dst = self.alloc_v();
                    self.emit(
                        Op::SetMapPopEndPlace {
                            dst,
                            path,
                            is_last: matches!(m, Mutation::PopLast),
                        },
                        span,
                    );
                    dst
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    self.emit_fallback_expr_v(expr)
                }
            }
            ExprKind::Index(base, indices)
                if matches!(&base.ty, Ty::Map(kty, _) | Ty::HashMap(kty, _) if !crate::value::ty_has_cached(kty, self.program) && !crate::value::keys::user_ordered(kty, self.program))
                    && indices.len() == 1 =>
            {
                let (Ty::Map(kty, _) | Ty::HashMap(kty, _)) = &base.ty else {
                    return self.emit_fallback_expr_v(expr);
                };
                let map_v = self.compile_expr_v(base);
                let key_v = self.compile_expr_v(&indices[0]);
                let key_ty = self.fill_tys.len() as u16;
                self.fill_tys.push(kty);
                let dst = self.alloc_v();
                self.emit(
                    Op::MapIndexV {
                        dst,
                        map_v,
                        key_v,
                        key_ty,
                    },
                    span,
                );
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                dst
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((_, dst_v, _)) =
                    self.try_compile_call_closure(callee, args, CallRet::V, span)
                {
                    dst_v
                } else {
                    self.emit_fallback_expr_v(expr)
                }
            }
            _ => self.emit_fallback_expr_v(expr),
        }
    }

    pub(super) fn try_compile_mutate_stmt(
        &mut self,
        op: Mutation,
        place: &'p Place,
        args: &'p [Expr],
        span: Span,
    ) -> bool {
        if op == Mutation::Reverse
            && args.is_empty()
            && matches!(place.ty, Ty::List(_) | Ty::Array { .. } | Ty::Str)
        {
            let saved_ops = self.ops.len();
            let saved_spans = self.spans.len();
            let saved_places = self.places.len();
            if let Some(path) = self.compile_place_path(place) {
                self.emit(Op::ReversePlace { path }, span);
                return true;
            }
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
        }
        if matches!(op, Mutation::PopFirst | Mutation::PopLast)
            && args.is_empty()
            && matches!(
                place.ty,
                Ty::Set(_) | Ty::HashSet(_) | Ty::Map(_, _) | Ty::HashMap(_, _)
            )
        {
            let saved_ops = self.ops.len();
            let saved_spans = self.spans.len();
            let saved_places = self.places.len();
            if let Some(path) = self.compile_place_path(place) {
                let dst = self.alloc_v();
                self.emit(
                    Op::SetMapPopEndPlace {
                        dst,
                        path,
                        is_last: matches!(op, Mutation::PopLast),
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: dst }, span);
                return true;
            }
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
        }
        if !matches!(place.ty, Ty::List(_)) {
            return false;
        }
        match (op, &place.kind) {
            (Mutation::Push, PlaceKind::Local(slot)) if args.len() == 1 => {
                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                    return false;
                };
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(
                        Op::ListPushLI {
                            list_v,
                            val_i,
                            kind,
                        },
                        span,
                    );
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(Op::ListPushLV { list_v, val_v }, span);
                }
                true
            }
            (Mutation::Push, PlaceKind::Global(gid)) if args.len() == 1 => {
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(
                        Op::ListPushGI {
                            gid: *gid as u16,
                            val_i,
                            kind,
                        },
                        span,
                    );
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(
                        Op::ListPushGV {
                            gid: *gid as u16,
                            val_v,
                        },
                        span,
                    );
                }
                true
            }
            (Mutation::Pop, PlaceKind::Local(slot)) if args.is_empty() => {
                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                    return false;
                };
                self.emit(Op::ListPopDiscardL { list_v }, span);
                true
            }
            (Mutation::Pop, PlaceKind::Global(gid)) if args.is_empty() => {
                self.emit(Op::ListPopDiscardG { gid: *gid as u16 }, span);
                true
            }
            (Mutation::Push, PlaceKind::Field(obj_place, field)) if args.len() == 1 => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(
                        Op::ListPushFieldLI {
                            obj_v,
                            field: *field as u16,
                            val_i,
                            kind,
                        },
                        span,
                    );
                } else if let Some((len, kinds)) = small_tuple_shape(&args[0].ty, self.program)
                    .filter(|_| self.small_tuple_is_cheap(&args[0]))
                {
                    let src_base = self.small_tuple_operand(&args[0], len);
                    let push = Op::ListPushFieldSmallTuple {
                        obj_v,
                        field: *field as u16,
                        src_base,
                        len,
                        kinds,
                    };
                    self.emit(push, span);
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(
                        Op::ListPushFieldLV {
                            obj_v,
                            field: *field as u16,
                            val_v,
                        },
                        span,
                    );
                }
                true
            }
            (Mutation::Pop, PlaceKind::Field(obj_place, field)) if args.is_empty() => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                self.emit(
                    Op::ListPopDiscardFieldL {
                        obj_v,
                        field: *field as u16,
                    },
                    span,
                );
                true
            }
            (Mutation::Push, PlaceKind::Index(base, indices))
                if matches!(base.ty, Ty::Array { .. }) && indices.len() == 1 && args.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                            let val_i = self.compile_expr_i(&args[0]);
                            self.emit(
                                Op::ListPushGArr1I {
                                    gid: *gid as u16,
                                    r_i,
                                    val_i,
                                    kind,
                                },
                                span,
                            );
                        } else {
                            let val_v = self.compile_expr_v(&args[0]);
                            self.emit(
                                Op::ListPushGArr1V {
                                    gid: *gid as u16,
                                    r_i,
                                    val_v,
                                },
                                span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                            let val_i = self.compile_expr_i(&args[0]);
                            self.emit(
                                Op::ListPushLArr1I {
                                    arr_v,
                                    r_i,
                                    val_i,
                                    kind,
                                },
                                span,
                            );
                        } else {
                            let val_v = self.compile_expr_v(&args[0]);
                            self.emit(Op::ListPushLArr1V { arr_v, r_i, val_v }, span);
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => match op {
                Mutation::Pop if args.is_empty() => {
                    let saved_ops = self.ops.len();
                    let saved_spans = self.spans.len();
                    let saved_places = self.places.len();
                    let Some(path) = self.compile_place_path(place) else {
                        self.ops.truncate(saved_ops);
                        self.spans.truncate(saved_spans);
                        self.places.truncate(saved_places);
                        return false;
                    };
                    self.emit(Op::ListPopDiscardPlace { path }, span);
                    true
                }
                Mutation::Push if args.len() == 1 => {
                    let saved_ops = self.ops.len();
                    let saved_spans = self.spans.len();
                    let saved_places = self.places.len();
                    let Some(path) = self.compile_place_path(place) else {
                        self.ops.truncate(saved_ops);
                        self.spans.truncate(saved_spans);
                        self.places.truncate(saved_places);
                        return false;
                    };
                    if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                        let val_i = self.compile_expr_i(&args[0]);
                        self.emit(Op::ListPushPlaceI { path, val_i, kind }, span);
                    } else {
                        let val_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::ListPushPlaceV { path, val_v }, span);
                    }
                    true
                }
                _ => false,
            },
        }
    }

    pub(super) fn compile_print(&mut self, args: &'p [FmtArg], end: Option<&'p Expr>, span: Span) {
        let regs: Vec<u16> = args.iter().map(|a| self.compile_expr_v(&a.value)).collect();
        let end = end.map(|e| self.compile_expr_v(e));
        let idx = self.prints.len() as u32;
        self.prints.push(PrintSpec {
            args,
            regs: regs.clone(),
            end,
        });
        self.emit(Op::Print { idx }, span);
        for reg in regs.into_iter().chain(end) {
            if reg >= self.num_local_v {
                self.emit(Op::ClearV { reg }, span);
            }
        }
    }

    pub(super) fn compile_format(&mut self, pieces: &'p [FmtPiece], span: Span) -> u16 {
        let regs: Vec<u16> = pieces
            .iter()
            .filter_map(|p| match p {
                FmtPiece::Text(_) => None,
                FmtPiece::Value(a) => Some(&a.value),
                FmtPiece::Number(e, _) => Some(e),
            })
            .map(|e| self.compile_expr_v(e))
            .collect();
        let dst = self.alloc_v();
        let idx = self.formats.len() as u32;
        self.formats.push(FormatSpec {
            pieces,
            regs: regs.clone(),
        });
        self.emit(Op::Format { idx, dst }, span);
        for reg in regs {
            if reg >= self.num_local_v {
                self.emit(Op::ClearV { reg }, span);
            }
        }
        dst
    }

    pub(super) fn emit_fallback_stmt(&mut self, stmt: &'p Stmt) {
        self.ref_all |= Self::stmt_makes_closure(stmt);
        let idx = self.fallback_stmts.len() as u32;
        self.fallback_stmts.push(stmt);
        let in_loop = !self.loops.is_empty();
        self.emit(
            Op::FallbackStmt {
                idx,
                clear_from_v: self.pinned_v,
                in_loop,
            },
            stmt.span().unwrap_or_default(),
        );
    }

    /// Stops the run with `stmt`'s report: a failed `assert` or a `Trap`.
    pub(super) fn emit_fail(&mut self, stmt: &'p Stmt, msg_v: u16, span: Span) {
        let idx = self.fails.len() as u32;
        self.fails.push(stmt);
        self.emit(Op::Fail { idx, msg_v }, span);
    }

    pub(super) fn emit_fallback_expr_i(&mut self, expr: &'p Expr) -> u16 {
        self.ref_all |= Self::expr_makes_closure(expr);
        if let Some(dst) = self.try_compile_apply_i(expr) {
            return dst;
        }
        let idx = self.fallback_exprs.len() as u32;
        self.fallback_exprs.push(expr);
        let dst = self.alloc_i();
        let in_loop = !self.loops.is_empty();
        self.emit(Op::FallbackExprI { idx, dst, in_loop }, expr.span);
        dst
    }

    /// Whether `expr` reaches Int registers without boxing: a small tuple
    /// local, a tuple of its fields, or an element of a list.
    pub(super) fn small_tuple_is_cheap(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Local(slot) => matches!(self.slot_loc(*slot), SlotLoc::SmallTuple { .. }),
            ExprKind::Tuple(_) => true,
            ExprKind::Index(base, indices) => {
                matches!(base.ty, Ty::List(_)) && indices.len() == 1 && indices[0].ty == Ty::Int
            }
            _ => false,
        }
    }

    /// The first of `len` Int registers holding the small tuple `expr`: its
    /// local's own when it is one, else fresh ones it is loaded into.
    pub(super) fn small_tuple_operand(&mut self, expr: &'p Expr, len: u8) -> u16 {
        if let ExprKind::Local(slot) = &expr.kind {
            if let SlotLoc::SmallTuple { base, len: l, .. } = self.slot_loc(*slot) {
                if l == len {
                    return base;
                }
            }
        }
        let base = self.next_i;
        for _ in 0..len {
            self.alloc_i();
        }
        self.compile_expr_small_tuple(expr, base, len);
        base
    }

    pub(super) fn compile_expr_small_tuple(&mut self, expr: &'p Expr, dst_base: u16, len: u8) {
        if let Some((fn_id, args)) = self.traced_call(expr) {
            let src_v = self.emit_traced_call(expr.span, fn_id, args, false);
            let unbox = Op::UnboxSmallTuple {
                dst_base,
                len,
                src_v,
            };
            self.emit(unbox, expr.span);
            return;
        }
        let span = expr.span;
        match &expr.kind {
            ExprKind::Tuple(fs) if fs.len() == len as usize => {
                let mut regs = [0u16; 4];
                for (k, f) in fs.iter().enumerate() {
                    regs[k] = self.compile_expr_i(f);
                }
                for (k, &r) in regs.iter().enumerate().take(len as usize) {
                    let d = dst_base + k as u16;
                    if r != d {
                        self.emit(Op::MoveI { dst: d, src: r }, span);
                    }
                }
            }
            ExprKind::If(cond, then_e, else_e) => {
                let j_else = self.emit_branch_if_false(cond);
                self.compile_expr_small_tuple(then_e, dst_base, len);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                self.compile_expr_small_tuple(else_e, dst_base, len);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::SmallTuple {
                    base, len: slen, ..
                } if slen == len => {
                    for k in 0..len as u16 {
                        if dst_base + k != base + k {
                            self.emit(
                                Op::MoveI {
                                    dst: dst_base + k,
                                    src: base + k,
                                },
                                span,
                            );
                        }
                    }
                }
                SlotLoc::V(src_v) => {
                    self.emit(
                        Op::UnboxSmallTuple {
                            dst_base,
                            len,
                            src_v,
                        },
                        span,
                    );
                }
                _ => {
                    let src_v = self.emit_fallback_expr_v(expr);
                    self.emit(
                        Op::UnboxSmallTuple {
                            dst_base,
                            len,
                            src_v,
                        },
                        span,
                    );
                }
            },
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_))
                    && indices.len() == 1
                    && indices[0].ty == Ty::Int =>
            {
                if let ExprKind::Field(obj, fidx) = &base.kind {
                    let obj_v = self.compile_expr_v(obj);
                    let r_i = self.compile_expr_i(&indices[0]);
                    let load = Op::LoadFieldList1SmallTuple {
                        dst_base,
                        len,
                        obj_v,
                        field: *fidx as u16,
                        r_i,
                    };
                    self.emit(load, span);
                    if obj_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: obj_v }, span);
                    }
                    return;
                }
                let root = self.compile_expr_v(base);
                let r_i = self.compile_expr_i(&indices[0]);
                let load = Op::LoadList1SmallTuple {
                    dst_base,
                    len,
                    root,
                    is_global: false,
                    r_i,
                };
                self.emit(load, span);
                if root >= self.num_local_v {
                    self.emit(Op::ClearV { reg: root }, span);
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2SmallTuple {
                                    dst_base,
                                    len,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1SmallTuple {
                                    dst_base,
                                    len,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                span,
                            );
                        }
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(
                                    Op::LoadLArr2SmallTuple {
                                        dst_base,
                                        len,
                                        arr_v,
                                        r_i,
                                        r_j,
                                    },
                                    span,
                                );
                            } else {
                                self.emit(
                                    Op::LoadLArr1SmallTuple {
                                        dst_base,
                                        len,
                                        arr_v,
                                        r_i,
                                    },
                                    span,
                                );
                            }
                        } else {
                            let src_v = self.emit_fallback_expr_v(expr);
                            self.emit(
                                Op::UnboxSmallTuple {
                                    dst_base,
                                    len,
                                    src_v,
                                },
                                span,
                            );
                        }
                    }
                    _ => {
                        let src_v = self.emit_fallback_expr_v(expr);
                        self.emit(
                            Op::UnboxSmallTuple {
                                dst_base,
                                len,
                                src_v,
                            },
                            span,
                        );
                    }
                }
            }
            _ => {
                let src_v = self.compile_expr_v(expr);
                self.emit(
                    Op::UnboxSmallTuple {
                        dst_base,
                        len,
                        src_v,
                    },
                    span,
                );
            }
        }
    }

    pub(super) fn emit_fallback_expr_v(&mut self, expr: &'p Expr) -> u16 {
        self.ref_all |= Self::expr_makes_closure(expr);
        if let Some(dst) = self.try_compile_apply_v(expr) {
            return dst;
        }
        let idx = self.fallback_exprs.len() as u32;
        self.fallback_exprs.push(expr);
        let dst = self.alloc_v();
        let in_loop = !self.loops.is_empty();
        self.emit(Op::FallbackExprV { idx, dst, in_loop }, expr.span);
        dst
    }
}
