use super::{Compiler, GiveTarget, ModSuperOp};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use cpl_ir::{
    Arg, BinOp, Builtin, Cast, CmpOp, Expr, ExprKind, Mutation, PlaceKind, Stmt, Ty, UnOp,
};

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn compile_expr_i(&mut self, expr: &'p Expr) -> u16 {
        let span = expr.span;
        if let Some((fn_id, args)) = self.traced_call(expr) {
            return self.emit_traced_call(expr, fn_id, args, true);
        }
        match &expr.kind {
            ExprKind::Int(v) => {
                let dst = self.alloc_i();
                self.emit_const_i(dst, *v, span);
                dst
            }
            ExprKind::Bool(b) => {
                let dst = self.alloc_i();
                self.emit(
                    Op::ConstImmI {
                        dst,
                        val: i32::from(*b),
                    },
                    span,
                );
                dst
            }
            ExprKind::Char(c) => {
                let dst = self.alloc_i();
                self.emit(
                    Op::ConstImmI {
                        dst,
                        val: i32::from(*c),
                    },
                    span,
                );
                dst
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::I(r, _) => r,
                SlotLoc::V(v) => {
                    let dst = self.alloc_i();
                    self.emit(
                        Op::UnboxI {
                            dst_i: dst,
                            src_v: v,
                        },
                        span,
                    );
                    dst
                }
                SlotLoc::OptI { val, .. } => val,
                SlotLoc::SmallTuple { base, .. } => base,
            },
            ExprKind::Global(gid) => {
                let dst = self.alloc_i();
                self.emit(
                    Op::LoadGlobalI {
                        dst,
                        gid: *gid as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::DomainLen(d) => {
                let dst = self.alloc_i();
                self.emit(
                    Op::DomainLen {
                        dst,
                        dom: *d as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::DomainLo(d) => {
                let dst = self.alloc_i();
                self.emit(
                    Op::DomainLo {
                        dst,
                        dom: *d as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::IsSome(inner) => {
                if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                    let (_, s) = self.compile_expr_opt_i(inner);
                    s
                } else {
                    let src_v = self.compile_expr_v(inner);
                    let dst_i = self.alloc_i();
                    self.emit(Op::IsSomeV { dst_i, src_v }, span);
                    dst_i
                }
            }
            ExprKind::Unwrap(inner, _) => {
                if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                    let (val, some) = self.compile_expr_opt_i(inner);
                    let dst = self.alloc_i();
                    self.emit(Op::UnwrapOptI { dst, val, some }, span);
                    dst
                } else {
                    let src_v = self.compile_expr_v(inner);
                    let dst_i = self.alloc_i();
                    self.emit(Op::UnwrapVI { dst_i, src_v }, span);
                    dst_i
                }
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs)
                if ScalarKind::of_opt_ty(&lhs.ty, self.program).is_some() =>
            {
                let (val, some) = self.compile_expr_opt_i(lhs);
                let dst = self.alloc_i();
                self.emit(Op::MoveI { dst, src: val }, lhs.span);
                let j_done = self.emit(
                    Op::JmpIfNonZero {
                        reg: some,
                        target: 0,
                    },
                    span,
                );
                let rv = self.compile_expr_i(rhs);
                if rv != dst {
                    self.emit(Op::MoveI { dst, src: rv }, rhs.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_done, end_pc);
                dst
            }
            ExprKind::Tuple(fields) if fields.len() == 1 => self.compile_expr_i(&fields[0]),
            ExprKind::Field(base, 0)
                if matches!(
                    ScalarKind::of_ty(&base.ty, self.program),
                    Some(ScalarKind::OneIntStruct)
                ) =>
            {
                self.compile_expr_i(base)
            }
            ExprKind::Field(base, idx) => {
                if let ExprKind::Local(slot) = &base.kind {
                    if let SlotLoc::SmallTuple { base: b, len, .. } = self.slot_loc(*slot) {
                        if *idx < len as usize {
                            return b + *idx as u16;
                        }
                    }
                }
                if let ExprKind::Tuple(fs) = &base.kind {
                    if *idx < fs.len() {
                        return self.compile_expr_i(&fs[*idx]);
                    }
                }
                let obj_v = self.compile_expr_v(base);
                let dst = self.alloc_i();
                self.emit(
                    Op::FieldI {
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
            ExprKind::RangeCheck {
                value,
                lo,
                hi,
                text,
            } => {
                let val = self.compile_expr_i(value);
                let r_lo = match lo {
                    Some(lo) => self.compile_expr_i(lo),
                    None => {
                        let r = self.alloc_i();
                        self.emit_const_i(r, cpl_ir::INT_MIN, span);
                        r
                    }
                };
                let r_hi = match hi {
                    Some(hi) => self.compile_expr_i(hi),
                    None => {
                        let r = self.alloc_i();
                        self.emit_const_i(r, i64::MAX, span);
                        r
                    }
                };
                let text_idx = self.range_texts.len() as u16;
                self.range_texts.push(text.as_str());
                let dst = self.alloc_i();
                self.emit(
                    Op::RangeCheckI {
                        dst,
                        val,
                        lo: r_lo,
                        hi: r_hi,
                        text_idx,
                    },
                    span,
                );
                dst
            }
            ExprKind::Unary(UnOp::Not, inner) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NotBool { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Neg, inner) if matches!(expr.ty, Ty::Int) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NegI { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Neg, inner) if expr.ty == Ty::U64 => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NegU64 { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::BitNot, inner) if matches!(expr.ty, Ty::Int | Ty::U64) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::BitNotI { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Pos, inner) => self.compile_expr_i(inner),
            ExprKind::Binary(op, lhs, rhs)
                if expr.ty == Ty::U64 && lhs.ty == Ty::U64 && rhs.ty == Ty::U64 =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                let inst = match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow => {
                        Op::BinU64 { dst, a, b, op: *op }
                    }
                    BinOp::BitAnd => Op::BitAndU64 { dst, a, b },
                    BinOp::BitOr => Op::BitOrU64 { dst, a, b },
                    BinOp::BitXor => Op::BitXorU64 { dst, a, b },
                    _ => return self.emit_fallback_expr_i(expr),
                };
                self.emit(inst, span);
                dst
            }
            ExprKind::Binary(op, lhs, rhs)
                if expr.ty == Ty::U64
                    && lhs.ty == Ty::U64
                    && matches!(op, BinOp::Shl | BinOp::Shr) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::BinU64 { dst, a, b, op: *op }, span);
                dst
            }
            ExprKind::Binary(op, lhs, rhs)
                if matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                    && matches!(lhs.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                    && matches!(rhs.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(lhs);
                if *op == BinOp::Add {
                    if let ExprKind::Int(v) = rhs.kind {
                        if let Ok(imm) = i32::try_from(v) {
                            let dst = self.alloc_i();
                            self.emit(Op::AddImmI { dst, a, imm }, span);
                            return dst;
                        }
                    }
                }
                if *op == BinOp::Sub {
                    if let ExprKind::Int(v) = rhs.kind {
                        if let Some(neg) = v.checked_neg() {
                            if let Ok(imm) = i32::try_from(neg) {
                                let dst = self.alloc_i();
                                self.emit(Op::AddImmI { dst, a, imm }, span);
                                return dst;
                            }
                        }
                    }
                }
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                let inst = match op {
                    BinOp::Add => Op::AddI { dst, a, b },
                    BinOp::Sub => Op::SubI { dst, a, b },
                    BinOp::Mul => Op::MulI { dst, a, b },
                    BinOp::Div => Op::DivI { dst, a, b },
                    BinOp::Rem => Op::RemI { dst, a, b },
                    BinOp::BitAnd => Op::BitAndI { dst, a, b },
                    BinOp::BitOr => Op::BitOrI { dst, a, b },
                    BinOp::BitXor => Op::BitXorI { dst, a, b },
                    BinOp::Shl => Op::ShlI { dst, a, b },
                    BinOp::Shr => Op::ShrI { dst, a, b },
                    _ => return self.emit_fallback_expr_i(expr),
                };
                self.emit(inst, span);
                dst
            }
            ExprKind::Binary(BinOp::And, lhs, rhs) => {
                let dst = self.alloc_i();
                let j_false = self.emit_branch_if_false(lhs);
                let r = self.compile_expr_i(rhs);
                self.emit(Op::MoveI { dst, src: r }, rhs.span);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let false_pc = self.ops.len() as u32;
                self.patch(j_false, false_pc);
                self.emit(Op::ConstImmI { dst, val: 0 }, span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Binary(BinOp::Or, lhs, rhs) => {
                let dst = self.alloc_i();
                let l = self.compile_expr_i(lhs);
                let j_true = self.emit(Op::JmpIfNonZero { reg: l, target: 0 }, lhs.span);
                let r = self.compile_expr_i(rhs);
                self.emit(Op::MoveI { dst, src: r }, rhs.span);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let true_pc = self.ops.len() as u32;
                self.patch(j_true, true_pc);
                self.emit(Op::ConstImmI { dst, val: 1 }, span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if lhs.ty == Ty::U64
                    && rhs.ty == Ty::U64
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::CmpU64 { dst, a, b, op: *op }, span);
                dst
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
                let dst = self.alloc_i();
                self.emit(Op::CmpI { dst, a, b, op: *op }, span);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if matches!(
                    ScalarKind::of_opt_ty(&lhs.ty, self.program),
                    Some(k) if k != ScalarKind::U64
                ) && matches!(
                    ScalarKind::of_opt_ty(&rhs.ty, self.program),
                    Some(k) if k != ScalarKind::U64
                ) && ty_uses_structural_cmp(&lhs.ty, self.program)
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let (a_val, a_some) = self.compile_expr_opt_i(lhs);
                let (b_val, b_some) = self.compile_expr_opt_i(rhs);
                let dst = self.alloc_i();
                self.emit(
                    Op::CmpOptI {
                        dst,
                        a_val,
                        a_some,
                        b_val,
                        b_some,
                        op: *op,
                    },
                    span,
                );
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if matches!(
                    op,
                    CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                ) && ty_uses_structural_cmp(&lhs.ty, self.program) =>
            {
                let a_v = self.compile_expr_v(lhs);
                let b_v = self.compile_expr_v(rhs);
                let dst = self.alloc_i();
                self.emit(
                    Op::CmpV {
                        dst,
                        a_v,
                        b_v,
                        op: *op,
                    },
                    span,
                );
                if a_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: a_v }, span);
                }
                if b_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: b_v }, span);
                }
                dst
            }
            ExprKind::Do(block) => {
                let dst = self.alloc_i();
                let prev_pinned = self.pinned_v;
                self.pinned_v = self.next_v;
                self.give_targets.push(GiveTarget::I {
                    dst,
                    exits: Vec::new(),
                });
                self.compile_stmts(&block.stmts);
                let mut target = self.give_targets.pop().unwrap();
                self.pinned_v = prev_pinned;
                let end_pc = self.ops.len() as u32;
                for j in target.exits_mut().drain(..) {
                    self.patch(j, end_pc);
                }
                dst
            }
            ExprKind::Cast(
                Cast::Ord | Cast::Trust | Cast::Raw | Cast::CharToInt | Cast::IntToU64,
                inner,
            ) => self.compile_expr_i(inner),
            ExprKind::Cast(Cast::U64ToInt, inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::U64ToInt { dst, src }, span);
                dst
            }
            ExprKind::Cast(Cast::Label(d), inner) => {
                let a = self.compile_expr_i(inner);
                let lo = self.alloc_i();
                self.emit(
                    Op::DomainLo {
                        dst: lo,
                        dom: *d as u16,
                    },
                    span,
                );
                let dst = self.alloc_i();
                self.emit(Op::AddI { dst, a, b: lo }, span);
                dst
            }
            ExprKind::Cast(Cast::FromLabel(d), inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(
                    Op::FromLabel {
                        dst,
                        src,
                        dom: *d as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::Cast(Cast::FromOrd(d), inner) => {
                let src = self.compile_expr_i(inner);
                let dom = *d as u16;
                if self.in_dom.iter().any(|f| (f.0, f.1) == (src, dom)) {
                    return src;
                }
                let dst = self.alloc_i();
                self.emit(Op::FromOrd { dst, src, dom }, span);
                self.in_dom.push((dst, dom, None));
                let held = match inner.kind {
                    ExprKind::Local(slot) => Some(slot),
                    _ => None,
                };
                self.in_dom.push((src, dom, held));
                dst
            }
            ExprKind::Builtin(Builtin::BoundStepL(d), args) if args.len() == 2 => {
                let ord = self.compile_expr_i(&args[0]);
                let k = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(
                    Op::BoundStepL {
                        dst,
                        ord,
                        k,
                        dom: *d as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::Builtin(Builtin::BoundStepR(d), args) if args.len() == 2 => {
                let ord = self.compile_expr_i(&args[0]);
                let k = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(
                    Op::BoundStepR {
                        dst,
                        ord,
                        k,
                        dom: *d as u16,
                    },
                    span,
                );
                dst
            }
            ExprKind::Builtin(Builtin::Min, args)
                if args.len() == 2 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::MinI { dst, a, b }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Max, args)
                if args.len() == 2 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::MaxI { dst, a, b }, span);
                dst
            }
            ExprKind::Builtin(
                op @ (Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul),
                args,
            ) if args.len() == 2 => {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                let op = match op {
                    Builtin::WrapAdd => Op::WrapAddI { dst, a, b },
                    Builtin::WrapSub => Op::WrapSubI { dst, a, b },
                    _ => Op::WrapMulI { dst, a, b },
                };
                self.emit(op, span);
                dst
            }
            ExprKind::Builtin(Builtin::Abs, args)
                if args.len() == 1 && matches!(expr.ty, Ty::Int) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::AbsI { dst, a }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Len, args) if args.len() == 1 => match &args[0].ty {
                Ty::Array { .. } => {
                    let dst = self.alloc_i();
                    if let ExprKind::Global(gid) = args[0].kind {
                        self.emit(
                            Op::GArrLen {
                                dst,
                                gid: gid as u16,
                            },
                            span,
                        );
                    } else if let ExprKind::Field(obj, fidx) = &args[0].kind {
                        let obj_v = self.compile_expr_v(obj);
                        self.emit(
                            Op::FieldArrLen {
                                dst,
                                obj_v,
                                field: *fidx as u16,
                            },
                            span,
                        );
                    } else {
                        let arr_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::LArrLen { dst, arr_v }, span);
                    }
                    dst
                }
                Ty::List(_) => {
                    let dst = self.alloc_i();
                    if let ExprKind::Global(gid) = args[0].kind {
                        self.emit(
                            Op::GListLen {
                                dst,
                                gid: gid as u16,
                            },
                            span,
                        );
                    } else if let ExprKind::Field(obj, fidx) = &args[0].kind {
                        let obj_v = self.compile_expr_v(obj);
                        self.emit(
                            Op::FieldListLen {
                                dst,
                                obj_v,
                                field: *fidx as u16,
                            },
                            span,
                        );
                    } else {
                        let list_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::LListLen { dst, list_v }, span);
                    }
                    dst
                }
                _ => {
                    let seq_v = self.compile_expr_v(&args[0]);
                    let dst = self.alloc_i();
                    self.emit(Op::SeqLenI { dst, seq_v }, span);
                    dst
                }
            },
            ExprKind::Builtin(Builtin::RangeLo, args)
                if args.len() == 1 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::RangeLoI { dst, src_v }, span);
                dst
            }
            ExprKind::Builtin(Builtin::RangeHi, args)
                if args.len() == 1 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::RangeHiI { dst, src_v }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BitCeil, args)
                if args.len() == 1 && matches!(expr.ty, Ty::Int) =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::BitCeilI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Popcount, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::PopcountI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Ctz, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::CtzI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Clz, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::ClzI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BitWidth, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::BitWidthI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Contains, args)
                if args.len() == 2
                    && matches!(
                        args[0].ty,
                        Ty::Set(_)
                            | Ty::HashSet(_)
                            | Ty::Map(_, _)
                            | Ty::HashMap(_, _)
                            | Ty::List(_)
                            | Ty::Array { .. }
                    )
                    && !crate::value::ty_has_cached(&args[1].ty, self.program) =>
            {
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::ContainsV { dst, src_v, key_v }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Mutate(m @ (Mutation::Insert | Mutation::Remove), place, args)
                if args.len() == 1
                    && !crate::value::ty_has_cached(&args[0].ty, self.program)
                    && ((matches!(m, Mutation::Insert)
                        && matches!(place.ty, Ty::Set(_) | Ty::HashSet(_)))
                        || (matches!(m, Mutation::Remove)
                            && matches!(
                                place.ty,
                                Ty::Set(_) | Ty::HashSet(_) | Ty::Map(_, _) | Ty::HashMap(_, _)
                            ))) =>
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                if let Some(path) = self.compile_place_path(place) {
                    let val_v = self.compile_expr_v(&args[0]);
                    let dst = self.alloc_i();
                    self.emit(
                        Op::SetMapMutPlace {
                            dst,
                            path,
                            val_v,
                            is_remove: matches!(m, Mutation::Remove),
                        },
                        span,
                    );
                    if val_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: val_v }, span);
                    }
                    dst
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    self.emit_fallback_expr_i(expr)
                }
            }
            ExprKind::Index(base, indices)
                if matches!(&base.ty, Ty::Map(kty, _) | Ty::HashMap(kty, _) if !crate::value::ty_has_cached(kty, self.program))
                    && indices.len() == 1 =>
            {
                let (Ty::Map(kty, _) | Ty::HashMap(kty, _)) = &base.ty else {
                    unreachable!()
                };
                let map_v = self.compile_expr_v(base);
                let key_v = self.compile_expr_v(&indices[0]);
                let key_ty = self.fill_tys.len() as u16;
                self.fill_tys.push(kty);
                let tmp_v = self.alloc_v();
                self.emit(
                    Op::MapIndexV {
                        dst: tmp_v,
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
                let dst = self.alloc_i();
                self.emit(
                    Op::UnboxI {
                        dst_i: dst,
                        src_v: tmp_v,
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: tmp_v }, span);
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2I {
                                    dst,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1I {
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
                            let dst = self.alloc_i();
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(
                                    Op::LoadLArr2I {
                                        dst,
                                        arr_v,
                                        r_i,
                                        r_j,
                                    },
                                    span,
                                );
                            } else {
                                self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                            }
                            dst
                        } else {
                            self.emit_fallback_expr_i(expr)
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
                            let dst = self.alloc_i();
                            self.emit(
                                Op::LoadNestedArrI {
                                    dst,
                                    root,
                                    is_global,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                            return dst;
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                    ExprKind::Field(obj, fidx) if indices.len() == 1 => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldArr1I {
                                dst,
                                obj_v,
                                field: *fidx as u16,
                                r_i,
                            },
                            span,
                        );
                        if obj_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: obj_v }, span);
                        }
                        dst
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadLArr2I {
                                    dst,
                                    arr_v,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) if matches!(base.ty, Ty::List(_)) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadGList1I {
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
                            let dst = self.alloc_i();
                            self.emit(Op::LoadLList1I { dst, list_v, r_i }, span);
                            dst
                        } else {
                            self.emit_fallback_expr_i(expr)
                        }
                    }
                    ExprKind::Field(obj, fidx) if matches!(base.ty, Ty::List(_)) => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldList1I {
                                dst,
                                obj_v,
                                field: *fidx as u16,
                                r_i,
                            },
                            span,
                        );
                        dst
                    }
                    _ => {
                        let list_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(Op::LoadLList1I { dst, list_v, r_i }, span);
                        if list_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: list_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Str | Ty::StrView) && indices.len() == 1 =>
            {
                let r_i = self.compile_expr_i(&indices[0]);
                let dst = self.alloc_i();
                match (&base.kind, &base.ty) {
                    (ExprKind::Global(gid), Ty::Str) => {
                        self.emit(
                            Op::GStrGetI {
                                dst,
                                gid: *gid as u16,
                                r_i,
                            },
                            span,
                        );
                    }
                    _ => {
                        let str_v = self.compile_expr_v(base);
                        self.emit(Op::StrGetI { dst, str_v, r_i }, span);
                        if str_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: str_v }, span);
                        }
                    }
                }
                dst
            }
            ExprKind::Mutate(Mutation::Take, place, args) if args.is_empty() => {
                match &place.kind {
                    PlaceKind::Local(slot) => {
                        if let SlotLoc::I(src, _) = self.slot_loc(*slot) {
                            let dst = self.alloc_i();
                            self.emit(Op::MoveI { dst, src }, span);
                            return dst;
                        }
                    }
                    PlaceKind::Global(gid) => {
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadGlobalI {
                                dst,
                                gid: *gid as u16,
                            },
                            span,
                        );
                        return dst;
                    }
                    _ => {}
                }
                // Taking a scalar leaves it in place: a read of the place.
                if let Some(path) = self.compile_place_path(place) {
                    let src_v = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst: src_v, path }, span);
                    let dst_i = self.alloc_i();
                    self.emit(Op::UnboxI { dst_i, src_v }, span);
                    self.emit(Op::ClearV { reg: src_v }, span);
                    return dst_i;
                }
                self.emit_fallback_expr_i(expr)
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs)
                if ScalarKind::of_opt_ty(&lhs.ty, self.program).is_some() =>
            {
                let (lv, ls) = self.compile_expr_opt_i(lhs);
                let dst = self.alloc_i();
                self.emit(Op::MoveI { dst, src: lv }, span);
                let j_done = self.emit(Op::JmpIfNonZero { reg: ls, target: 0 }, span);
                let rv = self.compile_expr_i(rhs);
                if rv != dst {
                    self.emit(Op::MoveI { dst, src: rv }, span);
                }
                let end = self.ops.len() as u32;
                self.patch(j_done, end);
                dst
            }
            ExprKind::If(cond, then_e, else_e) => {
                if let Some(b) = Self::const_bool(cond) {
                    return if b {
                        self.compile_expr_i(then_e)
                    } else {
                        self.compile_expr_i(else_e)
                    };
                }
                let dst = self.alloc_i();
                let j_else = self.emit_branch_if_false(cond);
                let rt = self.compile_expr_i(then_e);
                if rt != dst {
                    self.emit(Op::MoveI { dst, src: rt }, then_e.span);
                }
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let re = self.compile_expr_i(else_e);
                if re != dst {
                    self.emit(Op::MoveI { dst, src: re }, else_e.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Call(fn_id, args) => {
                if let Some(mop) = Self::recognize_mod_op(self.program, *fn_id) {
                    if args.iter().all(|a| matches!(a, Arg::Value(_))) {
                        let dst = self.alloc_i();
                        match (mop, args.as_slice()) {
                            (ModSuperOp::Add(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(
                                    Op::ModAddI {
                                        dst,
                                        a: ra,
                                        b: rb,
                                        modulus,
                                    },
                                    span,
                                );
                                return dst;
                            }
                            (ModSuperOp::Sub(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(
                                    Op::ModSubI {
                                        dst,
                                        a: ra,
                                        b: rb,
                                        modulus,
                                    },
                                    span,
                                );
                                return dst;
                            }
                            (ModSuperOp::Mul(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(
                                    Op::ModMulI {
                                        dst,
                                        a: ra,
                                        b: rb,
                                        modulus,
                                    },
                                    span,
                                );
                                return dst;
                            }
                            (ModSuperOp::Neg(modulus), [Arg::Value(a)]) => {
                                let ra = self.compile_expr_i(a);
                                self.emit(
                                    Op::ModNegI {
                                        dst,
                                        a: ra,
                                        modulus,
                                    },
                                    span,
                                );
                                return dst;
                            }
                            (ModSuperOp::New(modulus), [Arg::Value(a)]) => {
                                if let ExprKind::Int(v) = a.kind {
                                    self.emit_const_i(dst, v.rem_euclid(modulus as i64), span);
                                    return dst;
                                }
                                let ra = self.compile_expr_i(a);
                                let rm = self.alloc_i();
                                self.emit_const_i(rm, modulus as i64, span);
                                self.emit(Op::RemI { dst, a: ra, b: rm }, span);
                                return dst;
                            }
                            _ => {}
                        }
                    }
                }
                let can_inline_pure_leaf =
                    self.is_pure_leaf_fn(*fn_id) && {
                        let f = &self.program.fns[*fn_id];
                        args.iter().enumerate().all(|(i, a)| {
                        let Arg::Value(ae) = a else { return false; };
                        if ScalarKind::of_ty(&f.locals[i].ty, self.program).is_some() {
                            true
                        } else if let ExprKind::Local(slot) = &ae.kind {
                            matches!(self.slot_loc(*slot), SlotLoc::V(r) if r < self.num_local_v)
                        } else {
                            false
                        }
                    })
                    };
                let inlined = if can_inline_pure_leaf {
                    let f = &self.program.fns[*fn_id];
                    // A fallback reads its locals from the tree-walker's frame,
                    // which an inlined body does not have: if the body needs
                    // one, undo the inlining and call the function instead.
                    let mark = (
                        self.ops.len(),
                        self.fallback_stmts.len(),
                        self.fallback_exprs.len(),
                        self.trace_points.len(),
                    );
                    let mut map = Vec::with_capacity(f.locals.len());
                    for (i, a) in args.iter().enumerate() {
                        let Arg::Value(ae) = a else { unreachable!() };
                        if let Some(kind) = ScalarKind::of_ty(&f.locals[i].ty, self.program) {
                            let fresh = self.next_i;
                            let src = self.compile_expr_i(ae);
                            // A parameter is never assigned, so it reads its
                            // argument where it already is, unless a later
                            // argument could still change that register.
                            if src >= fresh || i + 1 == args.len() {
                                map.push(SlotLoc::I(src, kind));
                            } else {
                                let reg = self.alloc_i();
                                self.emit(Op::MoveI { dst: reg, src }, span);
                                map.push(SlotLoc::I(reg, kind));
                            }
                        } else if let ExprKind::Local(slot) = &ae.kind {
                            let SlotLoc::V(r) = self.slot_loc(*slot) else {
                                unreachable!()
                            };
                            map.push(SlotLoc::V(r));
                        } else {
                            unreachable!()
                        }
                    }
                    for l in f.locals.iter().skip(args.len()) {
                        let kind = ScalarKind::of_ty(&l.ty, self.program).unwrap();
                        let reg = self.alloc_i();
                        self.emit(Op::ConstImmI { dst: reg, val: 0 }, span);
                        map.push(SlotLoc::I(reg, kind));
                    }
                    let dst = self.alloc_i();
                    let prev_slots = self.inline_slots.replace(map);
                    let prev_ret = self.inline_ret.replace((dst, Vec::new()));
                    let (tail, body) = match f.body.stmts.split_last() {
                        Some((Stmt::Return(Some(e)) | Stmt::Give(e), rest)) => (Some(e), rest),
                        _ => (None, &f.body.stmts[..]),
                    };
                    for s in body {
                        self.compile_stmt(s);
                    }
                    // The last statement's value falls through to the end, so
                    // it needs no jump, nor a move when nothing else returns.
                    let tail = tail.map(|e| self.compile_expr_i(e));
                    let (_, jumps) = self.inline_ret.take().unwrap();
                    let dst = match tail {
                        Some(r) if jumps.is_empty() => r,
                        Some(r) => {
                            if r != dst {
                                self.emit(Op::MoveI { dst, src: r }, span);
                            }
                            dst
                        }
                        None => dst,
                    };
                    let end_pc = self.ops.len() as u32;
                    self.inline_ret = prev_ret;
                    self.inline_slots = prev_slots;
                    for j in jumps {
                        self.patch(j, end_pc);
                    }
                    if (self.fallback_stmts.len(), self.fallback_exprs.len()) == (mark.1, mark.2) {
                        Some(dst)
                    } else {
                        self.ops.truncate(mark.0);
                        self.spans.truncate(mark.0);
                        self.fallback_stmts.truncate(mark.1);
                        self.fallback_exprs.truncate(mark.2);
                        self.trace_points.truncate(mark.3);
                        None
                    }
                } else {
                    None
                };
                if let Some(dst) = inlined {
                    dst
                } else {
                    let f = &self.program.fns[*fn_id];
                    let can_vm = !self.program.judge_fns.contains(fn_id)
                        && !f.params.iter().any(|p| p.mutable)
                        && f.locals.len() <= 512
                        && self.program.fn_guarded_domains(&f.ret, &f.body).is_empty()
                        && (0..f.params.len())
                            .all(|i| ScalarKind::of_ty(&f.locals[i].ty, self.program).is_some())
                        && ScalarKind::of_ty(&f.ret, self.program).is_some()
                        && args.len() <= 8
                        && args.iter().all(|a| matches!(a, Arg::Value(_)));
                    if can_vm {
                        let mut evaluated = Vec::with_capacity(args.len());
                        for a in args {
                            let Arg::Value(ae) = a else { unreachable!() };
                            evaluated.push(self.compile_expr_i(ae));
                        }
                        let arg_base = self.next_i;
                        for r in evaluated {
                            let slot = self.alloc_i();
                            if slot != r {
                                self.emit(Op::MoveI { dst: slot, src: r }, span);
                            }
                        }
                        let dst = self.alloc_i();
                        self.emit(
                            Op::CallVmI {
                                fn_id: *fn_id as u16,
                                arg_base,
                                argc: args.len() as u8,
                                dst,
                            },
                            span,
                        );
                        dst
                    } else if let Some((dst, _, _)) =
                        self.try_compile_call_mix(*fn_id, args, CallRet::I, span)
                    {
                        dst
                    } else {
                        self.emit_fallback_expr_i(expr)
                    }
                }
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((dst, _, _)) =
                    self.try_compile_call_closure(callee, args, CallRet::I, span)
                {
                    dst
                } else {
                    self.emit_fallback_expr_i(expr)
                }
            }
            _ => self.emit_fallback_expr_i(expr),
        }
    }
}
