use super::{Compiler, GiveTarget, LoopTargets};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, Block, Expr, ExprKind, FnId, Iter, LoopId, Mutation, Place, PlaceKind, Slot,
    Stmt, Ty,
};

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn compile_stmts(&mut self, stmts: &'p [Stmt]) {
        if let Some((slot, span)) = self.traced_loop_var.take() {
            self.emit_trace(TracePoint::Loop(slot, span), span);
        }
        let has_closure = Self::stmts_have_closure(stmts);
        let mut declared = Vec::new();
        let mut i = 0;
        while i < stmts.len() {
            if self.traced {
                self.compile_stmt(&stmts[i]);
                i += 1;
                continue;
            }
            if i + 1 < stmts.len()
                && self.try_compile_opt_minmax_pair(&stmts[i], &stmts[i + 1], &stmts[i + 2..])
            {
                i += 2;
                continue;
            }
            let stmt = &stmts[i];
            if let Stmt::Let(slot, _) = stmt {
                declared.push(*slot);
            }
            if !has_closure {
                if let Stmt::Assign(place, AssignOp::Set, rhs) = stmt {
                    if let ExprKind::Local(src_slot) = rhs.kind {
                        if declared.contains(&src_slot)
                            && !stmts[i + 1..]
                                .iter()
                                .any(|s| Self::stmt_mentions_slot(s, src_slot))
                        {
                            if let SlotLoc::V(src_reg) = self.slot_loc(src_slot) {
                                match &place.kind {
                                    PlaceKind::Global(gid) => {
                                        self.emit(
                                            Op::StoreGlobalVMove {
                                                gid: *gid as u16,
                                                src: src_reg,
                                            },
                                            rhs.span,
                                        );
                                        i += 1;
                                        continue;
                                    }
                                    PlaceKind::Local(dst_slot) => {
                                        if let SlotLoc::V(dst_reg) = self.slot_loc(*dst_slot) {
                                            if dst_reg != src_reg {
                                                self.emit(
                                                    Op::MoveVConsume {
                                                        dst: dst_reg,
                                                        src: src_reg,
                                                    },
                                                    rhs.span,
                                                );
                                            }
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            self.compile_stmt(stmt);
            i += 1;
        }
    }

    pub(in crate::vm) fn compile_block(
        &mut self,
        block: &'p Block,
        ret_kind: Option<ScalarKind>,
        emit_ret: bool,
    ) {
        self.compile_stmts(&block.stmts);
        if emit_ret {
            if ret_kind.is_some() {
                let r = self.alloc_i();
                self.emit(Op::ConstImmI { dst: r, val: 0 }, Span::default());
                self.emit(Op::RetI { src: r }, Span::default());
            } else {
                self.emit(Op::RetUnit, Span::default());
            }
        }
    }

    pub(super) fn compile_stmt(&mut self, stmt: &'p Stmt) {
        let start = self.ops.len();
        if !self.traced || !self.compile_traced_stmt(stmt) {
            self.compile_untraced_stmt(stmt);
        }
        self.note_call_site(start, stmt);
    }

    /// A write to `place` the VM reports with raw values: a global, or a
    /// cell of one whose `Int` indices it evaluates again.
    fn cell_point(&mut self, place: &'p Place) -> Option<TracePoint<'p>> {
        let mut indices: Vec<&'p [Expr]> = Vec::new();
        let mut at: &'p Place = place;
        loop {
            match &at.kind {
                PlaceKind::Global(_) => break,
                PlaceKind::Index(base, index) => {
                    indices.push(index);
                    at = base;
                }
                _ => return None,
            }
        }
        let indices: Vec<&'p Expr> = indices.into_iter().rev().flatten().collect();
        let int =
            |e: &Expr| ScalarKind::of_ty(&e.ty, self.program) == Some(ScalarKind::Int) && quiet(e);
        if !indices.iter().all(|e| int(e)) {
            return None;
        }
        let regs = indices
            .into_iter()
            .map(|e| self.compile_expr_i(e))
            .collect();
        Some(TracePoint::Cell(place, regs))
    }

    pub(super) fn emit_trace(&mut self, point: TracePoint<'p>, span: Span) {
        let idx = self.trace_points.len() as u32;
        self.trace_points.push(point);
        self.emit(Op::Trace { idx }, span);
    }

    /// In a traced run the VM runs control flow and expressions that
    /// neither write nor call; a write it does itself is followed by a
    /// `Trace`, and anything else is left to the tree-walker, which
    /// reports it. Returns whether `stmt` was compiled.
    pub(super) fn compile_traced_stmt(&mut self, stmt: &'p Stmt) -> bool {
        let fallbacks = self.fallback_stmts.len();
        match stmt {
            Stmt::Let(slot, Some(init)) if self.feed_acc != Some(*slot) && quiet(init) => {
                self.compile_untraced_stmt(stmt);
                if self.fallback_stmts.len() == fallbacks {
                    self.emit_trace(TracePoint::Let(*slot, init), init.span);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None) => return false,
            Stmt::Assign(place, _, value)
                if quiet_place(place, cpl_ir::walk::root(place)) && quiet(value) =>
            {
                self.compile_untraced_stmt(stmt);
                if self.fallback_stmts.len() == fallbacks {
                    let point = self.cell_point(place).unwrap_or(TracePoint::Assign(place));
                    self.emit_trace(point, place.span);
                }
            }
            Stmt::If(cond, ..) | Stmt::While(_, cond, _) | Stmt::Assert { cond, .. }
                if quiet(cond) =>
            {
                return false
            }
            Stmt::Return(Some(value)) | Stmt::Give(value) if quiet(value) => return false,
            Stmt::For(_, slot, iter, _) if self.feed_acc.is_none() => match quiet_iter(iter) {
                Some(span) => {
                    self.traced_loop_var = Some((*slot, span));
                    self.compile_untraced_stmt(stmt);
                    self.traced_loop_var = None;
                    self.emit_trace(TracePoint::LoopEnd(*slot), span);
                }
                None => {
                    self.emit_fallback_stmt(stmt);
                    self.emit_escapes();
                }
            },
            _ => {
                self.emit_fallback_stmt(stmt);
                self.emit_escapes();
            }
        }
        true
    }

    /// After a fallback statement inside loops: where its `break` or
    /// `continue`, run by the tree-walker, goes.
    pub(super) fn emit_escapes(&mut self) {
        let last = self.loops.len().wrapping_sub(1);
        for pos in (0..self.loops.len()).rev() {
            let (id, innermost) = (self.loops[pos].id, pos == last);
            let pc = self.emit(
                Op::JmpEscape {
                    brk: true,
                    id,
                    innermost,
                    target: 0,
                },
                Span::default(),
            );
            self.loops[pos].breaks.push(pc);
            let pc = self.emit(
                Op::JmpEscape {
                    brk: false,
                    id,
                    innermost,
                    target: 0,
                },
                Span::default(),
            );
            self.loops[pos].continues.push(pc);
        }
    }

    pub(super) fn compile_untraced_stmt(&mut self, stmt: &'p Stmt) {
        match stmt {
            Stmt::Let(slot, init) => {
                if self.feed_acc == Some(*slot) {
                    return;
                }
                match self.slot_loc(*slot) {
                    SlotLoc::I(dst, _) => {
                        if let Some(expr) = init {
                            let r = self.compile_expr_i(expr);
                            if r != dst {
                                self.emit(Op::MoveI { dst, src: r }, expr.span);
                            }
                        } else {
                            self.emit(Op::ConstImmI { dst, val: 0 }, Span::default());
                        }
                    }
                    SlotLoc::OptI { val, some, .. } => {
                        if let Some(expr) = init {
                            let (rv, rs) = self.compile_expr_opt_i(expr);
                            if rv != val {
                                self.emit(Op::MoveI { dst: val, src: rv }, expr.span);
                            }
                            if rs != some {
                                self.emit(Op::MoveI { dst: some, src: rs }, expr.span);
                            }
                        } else {
                            self.emit(Op::ConstImmI { dst: some, val: 0 }, Span::default());
                        }
                    }
                    SlotLoc::SmallTuple { base, len, .. } => {
                        if let Some(expr) = init {
                            self.compile_expr_small_tuple(expr, base, len);
                        } else {
                            for k in 0..len as u16 {
                                self.emit(
                                    Op::ConstImmI {
                                        dst: base + k,
                                        val: 0,
                                    },
                                    Span::default(),
                                );
                            }
                        }
                    }
                    SlotLoc::V(dst) => {
                        if let Some(expr) = init {
                            let r = self.compile_expr_v(expr);
                            if r != dst {
                                self.emit(Op::MoveV { dst, src: r }, expr.span);
                            }
                        } else {
                            self.emit_fallback_stmt(stmt);
                        }
                    }
                }
            }
            Stmt::Expr(expr) => {
                if let (Some(acc), ExprKind::Mutate(op, place, args)) = (self.feed_acc, &expr.kind)
                {
                    if matches!(place.kind, PlaceKind::Local(s) if s == acc) {
                        match (*op, args.as_slice()) {
                            (Mutation::Push, [arg]) => {
                                if let Some(kind) = ScalarKind::of_ty(&arg.ty, self.program) {
                                    let src = self.compile_expr_i(arg);
                                    self.emit(Op::FeedYieldI { src, kind }, expr.span);
                                } else {
                                    let src = self.compile_expr_v(arg);
                                    self.emit(Op::FeedYieldV { src }, expr.span);
                                    if src >= self.num_local_v {
                                        self.emit(Op::ClearV { reg: src }, expr.span);
                                    }
                                }
                                return;
                            }
                            (Mutation::Extend, [arg]) => {
                                let src = self.compile_expr_v(arg);
                                self.emit(Op::FeedYieldExtend { src }, expr.span);
                                if src >= self.num_local_v {
                                    self.emit(Op::ClearV { reg: src }, expr.span);
                                }
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                if ScalarKind::of_ty(&expr.ty, self.program).is_some() {
                    let _ = self.compile_expr_i(expr);
                } else if let ExprKind::Print(args, end) = &expr.kind {
                    self.compile_print(args, end.as_deref(), expr.span);
                } else if let ExprKind::Mutate(op, place, args) = &expr.kind {
                    if !self.try_compile_mutate_stmt(*op, place, args, expr.span) {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Sort(place, cpl_ir::SortBy::Natural, desc) = &expr.kind {
                    if !self.try_compile_sort_stmt(place, desc.as_deref(), expr.span) {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Call(fn_id, args) = &expr.kind {
                    if self
                        .try_compile_call_mix(*fn_id, args, CallRet::Unit, expr.span)
                        .is_none()
                    {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::CallExpr(callee, args) = &expr.kind {
                    if self
                        .try_compile_call_closure(callee, args, CallRet::Unit, expr.span)
                        .is_none()
                    {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Do(block) = &expr.kind {
                    let prev_pinned = self.pinned_v;
                    self.pinned_v = self.next_v;
                    self.give_targets
                        .push(GiveTarget::Discard { exits: Vec::new() });
                    self.compile_stmts(&block.stmts);
                    let mut target = self.give_targets.pop().unwrap();
                    self.pinned_v = prev_pinned;
                    let end_pc = self.ops.len() as u32;
                    for j in target.exits_mut().drain(..) {
                        self.patch(j, end_pc);
                    }
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Assert { cond, span, .. } => {
                let r_cond = self.compile_expr_i(cond);
                let j_ok = self.emit(
                    Op::JmpIfNonZero {
                        reg: r_cond,
                        target: 0,
                    },
                    *span,
                );
                self.emit_fallback_stmt(stmt);
                let end_pc = self.ops.len() as u32;
                self.patch(j_ok, end_pc);
            }
            Stmt::Assign(place, op, value) => {
                if !self.try_compile_assign(place, *op, value) {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::If(cond, then_b, else_b) => {
                if let Some(b) = Self::const_bool(cond) {
                    let branch = if b { then_b } else { else_b };
                    for s in &branch.stmts {
                        self.compile_stmt(s);
                    }
                    return;
                }
                let j_else = self.emit_branch_if_false(cond);
                for s in &then_b.stmts {
                    self.compile_stmt(s);
                }
                if else_b.stmts.is_empty() {
                    let end = self.ops.len() as u32;
                    self.patch(j_else, end);
                } else {
                    let j_end = self.emit(Op::Jmp { target: 0 }, cond.span);
                    let else_pc = self.ops.len() as u32;
                    self.patch(j_else, else_pc);
                    for s in &else_b.stmts {
                        self.compile_stmt(s);
                    }
                    let end = self.ops.len() as u32;
                    self.patch(j_end, end);
                }
            }
            Stmt::While(id, cond, body) => {
                let head = self.ops.len() as u32;
                let j_exit = self.emit_branch_if_false(cond);
                self.loops.push(LoopTargets {
                    id: *id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    resumable: Some(LoopReg::While(*id)),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                for c in lt.continues {
                    self.patch(c, head);
                }
                if self.traced {
                    self.emit_trace(TracePoint::Tick, cond.span);
                }
                let back = self.ops.len();
                self.note_back_edge(back, Some(LoopReg::While(*id)));
                self.emit(Op::Jmp { target: head }, cond.span);
                let end = self.ops.len() as u32;
                self.patch(j_exit, end);
                for b in lt.breaks {
                    self.patch(b, end);
                }
            }
            Stmt::For(id, slot, iter, body) => {
                if !self.try_compile_for(stmt, *id, *slot, iter, body) {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Break(id) => {
                let target_pos = match id {
                    Some(id) => self.loops.iter().rposition(|l| l.id == *id),
                    None => self.loops.len().checked_sub(1),
                };
                if let Some(pos) = target_pos {
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    self.loops[pos].breaks.push(pc);
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Continue(id) => {
                let target_pos = match id {
                    Some(id) => self.loops.iter().rposition(|l| l.id == *id),
                    None => self.loops.len().checked_sub(1),
                };
                if let Some(pos) = target_pos {
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    self.loops[pos].continues.push(pc);
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Give(e) => {
                if let Some(target) = self.give_targets.last() {
                    match *target {
                        GiveTarget::I { dst, .. } => {
                            let r = self.compile_expr_i(e);
                            if r != dst {
                                self.emit(Op::MoveI { dst, src: r }, e.span);
                            }
                        }
                        GiveTarget::OptI {
                            dst_val, dst_some, ..
                        } => {
                            let (rv, rs) = self.compile_expr_opt_i(e);
                            if rv != dst_val {
                                self.emit(
                                    Op::MoveI {
                                        dst: dst_val,
                                        src: rv,
                                    },
                                    e.span,
                                );
                            }
                            if rs != dst_some {
                                self.emit(
                                    Op::MoveI {
                                        dst: dst_some,
                                        src: rs,
                                    },
                                    e.span,
                                );
                            }
                        }
                        GiveTarget::V { dst, .. } => {
                            let r = self.compile_expr_v(e);
                            if r != dst {
                                self.emit(Op::MoveV { dst, src: r }, e.span);
                            }
                        }
                        GiveTarget::Discard { .. } => {
                            if !matches!(
                                e.kind,
                                ExprKind::Unit
                                    | ExprKind::Int(_)
                                    | ExprKind::Bool(_)
                                    | ExprKind::Char(_)
                                    | ExprKind::None
                                    | ExprKind::Local(_)
                                    | ExprKind::Global(_)
                            ) {
                                let _ = self.compile_expr_v(e);
                            }
                        }
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some(t) = self.give_targets.last_mut() {
                        t.exits_mut().push(pc);
                    }
                } else if let Some((ret_dst, _)) = self.inline_ret {
                    let r = self.compile_expr_i(e);
                    if r != ret_dst {
                        self.emit(
                            Op::MoveI {
                                dst: ret_dst,
                                src: r,
                            },
                            e.span,
                        );
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if self.cur_fn.is_some() {
                    if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                        let r = self.compile_expr_i(e);
                        self.emit(Op::RetI { src: r }, e.span);
                    } else {
                        let r = self.compile_expr_v(e);
                        self.emit(Op::RetV { src: r }, e.span);
                    }
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Return(val) => {
                if self.feed_acc.is_some() {
                    self.emit(Op::FeedEnd, Span::default());
                } else if let Some((ret_dst, _)) = self.inline_ret {
                    if let Some(e) = val {
                        let r = self.compile_expr_i(e);
                        if r != ret_dst {
                            self.emit(
                                Op::MoveI {
                                    dst: ret_dst,
                                    src: r,
                                },
                                e.span,
                            );
                        }
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if let Some(e) = val {
                    if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                        let r = self.compile_expr_i(e);
                        self.emit(Op::RetI { src: r }, e.span);
                    } else {
                        let r = self.compile_expr_v(e);
                        self.emit(Op::RetV { src: r }, e.span);
                    }
                } else {
                    self.emit(Op::RetUnit, Span::default());
                }
            }
            _ => self.emit_fallback_stmt(stmt),
        }
    }

    pub(super) fn block_has_outer_loop_jump(block: &Block, fed_id: LoopId) -> bool {
        fn check_block(b: &Block, fed_id: LoopId, inner: &mut Vec<LoopId>) -> bool {
            for s in &b.stmts {
                match s {
                    Stmt::Break(Some(id)) | Stmt::Continue(Some(id)) => {
                        if *id != fed_id && !inner.contains(id) {
                            return true;
                        }
                    }
                    Stmt::If(_, t, e) => {
                        if check_block(t, fed_id, inner) || check_block(e, fed_id, inner) {
                            return true;
                        }
                    }
                    Stmt::While(lid, _, body) | Stmt::For(lid, _, _, body) => {
                        inner.push(*lid);
                        let bad = check_block(body, fed_id, inner);
                        inner.pop();
                        if bad {
                            return true;
                        }
                    }
                    Stmt::Expr(e) if matches!(e.kind, ExprKind::Do(_)) => {
                        if let ExprKind::Do(b2) = &e.kind {
                            if check_block(b2, fed_id, inner) {
                                return true;
                            }
                        }
                    }
                    Stmt::Give(_) => return true,
                    _ => {}
                }
            }
            false
        }
        check_block(block, fed_id, &mut Vec::new())
    }

    pub(super) fn try_compile_for_fed(
        &mut self,
        stmt: &'p Stmt,
        id: LoopId,
        slot: Slot,
        fn_id: FnId,
        seq_expr: &'p Expr,
        body: &'p Block,
    ) -> bool {
        let ExprKind::Call(cid, args) = &seq_expr.kind else {
            return false;
        };
        let f = &self.program.fns[fn_id];
        if *cid != fn_id
            || !self.fed_ok[fn_id]
            || matches!(&f.ret, Ty::List(elem) if matches!(**elem, Ty::Struct(_)))
            || f.locals.len() > 512
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || f.params.iter().any(|p| p.mutable)
            || !self.give_targets.is_empty()
            || self.inline_ret.is_some()
            || Self::block_has_outer_loop_jump(body, id)
        {
            return false;
        }
        if !args.iter().all(|a| matches!(a, Arg::Value(_))) {
            return false;
        }
        let mut args_i = Vec::new();
        let mut args_v = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let Arg::Value(e) = arg else { unreachable!() };
            let pty = &f.locals[i].ty;
            if ScalarKind::of_ty(pty, self.program).is_some() {
                args_i.push(self.compile_expr_i(e));
            } else if ScalarKind::of_opt_ty(pty, self.program).is_some() {
                let (rv, rs) = self.compile_expr_opt_i(e);
                args_i.push(rv);
                args_i.push(rs);
            } else {
                args_v.push(self.compile_expr_v(e));
            }
        }
        let var_loc = self.slot_loc(slot);
        let spec_idx = self.feed_calls.len() as u16;
        self.feed_calls.push(FeedCallSpec {
            fn_id,
            args_i,
            args_v,
            body_pc: 0,
            var_loc,
        });
        let call_pc = self.emit(Op::CallVmFeed { spec: spec_idx }, seq_expr.span);
        self.note_fed_site(call_pc, stmt, seq_expr);
        let j_after = self.emit(Op::Jmp { target: 0 }, seq_expr.span);

        let body_pc = self.ops.len();
        self.feed_calls[spec_idx as usize].body_pc = body_pc;
        self.loops.push(LoopTargets {
            id,
            breaks: Vec::new(),
            continues: Vec::new(),
            resumable: None,
        });
        self.compile_stmts(&body.stmts);
        let lt = self.loops.pop().unwrap();
        let next_pc = self.ops.len() as u32;
        for c in lt.continues {
            self.patch(c, next_pc);
        }
        self.emit(Op::FeedBodyNext, Span::default());
        let break_pc = self.ops.len() as u32;
        for b in lt.breaks {
            self.patch(b, break_pc);
        }
        self.emit(Op::FeedBodyBreak, Span::default());
        let after_pc = self.ops.len() as u32;
        self.patch(j_after, after_pc);
        true
    }

    pub(super) fn try_compile_for(
        &mut self,
        stmt: &'p Stmt,
        id: LoopId,
        slot: Slot,
        iter: &'p Iter,
        body: &'p Block,
    ) -> bool {
        if let Iter::Seq(seq_expr, rev) = iter {
            if !*rev && self.allow_fed_loops {
                if let Some((fn_id, _acc_slot)) = self.feeds.feeds(id, seq_expr, body) {
                    return self.try_compile_for_fed(stmt, id, slot, fn_id, seq_expr, body);
                }
            }
            if !*rev && self.feeds.feeds(id, seq_expr, body).is_some() {
                return false;
            }
            return self.try_compile_for_seq(id, slot, seq_expr, *rev, body);
        }
        let SlotLoc::I(r_var, _) = self.slot_loc(slot) else {
            return false;
        };
        match iter {
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let j_skip = if *closed {
                    self.emit(
                        Op::JmpGtI {
                            a: r_lo,
                            b: r_hi,
                            target: 0,
                        },
                        lo.span,
                    )
                } else {
                    self.emit(
                        Op::JmpGeI {
                            a: r_lo,
                            b: r_hi,
                            target: 0,
                        },
                        lo.span,
                    )
                };
                if r_var != r_lo {
                    self.emit(
                        Op::MoveI {
                            dst: r_var,
                            src: r_lo,
                        },
                        lo.span,
                    );
                }
                let body_pc = self.ops.len() as u32;
                self.loops.push(LoopTargets {
                    id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    resumable: Some(LoopReg::Range { id, r_var, r_hi }),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                for c in lt.continues {
                    self.patch(c, step_pc);
                }
                let back = self.ops.len();
                self.note_back_edge(back, Some(LoopReg::Range { id, r_var, r_hi }));
                if *closed {
                    self.emit(
                        Op::ForIncClosed {
                            r_i: r_var,
                            r_hi,
                            target: body_pc,
                        },
                        hi.span,
                    );
                } else {
                    self.emit(
                        Op::ForInc {
                            r_i: r_var,
                            r_hi,
                            target: body_pc,
                        },
                        hi.span,
                    );
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_skip, end_pc);
                for b in lt.breaks {
                    self.patch(b, end_pc);
                }
                true
            }
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: true,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let j_skip = if *closed {
                    self.emit(
                        Op::JmpGtI {
                            a: r_lo,
                            b: r_hi,
                            target: 0,
                        },
                        lo.span,
                    )
                } else {
                    self.emit(
                        Op::JmpGeI {
                            a: r_lo,
                            b: r_hi,
                            target: 0,
                        },
                        lo.span,
                    )
                };
                if *closed {
                    if r_var != r_hi {
                        self.emit(
                            Op::MoveI {
                                dst: r_var,
                                src: r_hi,
                            },
                            hi.span,
                        );
                    }
                } else {
                    self.emit(
                        Op::AddImmI {
                            dst: r_var,
                            a: r_hi,
                            imm: -1,
                        },
                        hi.span,
                    );
                }
                let body_pc = self.ops.len() as u32;
                self.loops.push(LoopTargets {
                    id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    resumable: None,
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                for c in lt.continues {
                    self.patch(c, step_pc);
                }
                self.emit(
                    Op::ForDec {
                        r_i: r_var,
                        r_lo,
                        target: body_pc,
                    },
                    lo.span,
                );
                let end_pc = self.ops.len() as u32;
                self.patch(j_skip, end_pc);
                for b in lt.breaks {
                    self.patch(b, end_pc);
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn try_compile_for_seq(
        &mut self,
        id: LoopId,
        slot: Slot,
        seq_expr: &'p Expr,
        rev: bool,
        body: &'p Block,
    ) -> bool {
        #[derive(Clone, Copy)]
        enum SeqKind {
            Array,
            List,
            Str,
        }
        let seq_kind = match &seq_expr.ty {
            Ty::Array { dims, .. } if dims.len() == 1 => SeqKind::Array,
            Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) => SeqKind::List,
            Ty::Str | Ty::StrView => SeqKind::Str,
            _ => return false,
        };
        let slot_loc = self.slot_loc(slot);
        match (seq_kind, slot_loc) {
            (SeqKind::Str, SlotLoc::I(..)) => {}
            (SeqKind::Str, _) => return false,
            (_, SlotLoc::I(..) | SlotLoc::V(_) | SlotLoc::SmallTuple { .. }) => {}
            (_, SlotLoc::OptI { .. }) => return false,
        }
        let r_idx = self.alloc_i();
        let r_len = self.alloc_i();
        let global_id = match &seq_expr.kind {
            ExprKind::Global(gid) if matches!(seq_expr.ty, Ty::Array { .. } | Ty::List(_)) => {
                Some(*gid as u16)
            }
            _ => None,
        };
        let local_v = if global_id.is_none() {
            Some(self.compile_expr_v(seq_expr))
        } else {
            None
        };
        match (seq_kind, global_id) {
            (SeqKind::Array, Some(gid)) => {
                self.emit(Op::GArrLen { dst: r_len, gid }, seq_expr.span)
            }
            (SeqKind::Array, None) => self.emit(
                Op::LArrLen {
                    dst: r_len,
                    arr_v: local_v.unwrap(),
                },
                seq_expr.span,
            ),
            (SeqKind::List, Some(gid)) => {
                self.emit(Op::GListLen { dst: r_len, gid }, seq_expr.span)
            }
            (SeqKind::List, None) => self.emit(
                Op::LListLen {
                    dst: r_len,
                    list_v: local_v.unwrap(),
                },
                seq_expr.span,
            ),
            (SeqKind::Str, _) => self.emit(
                Op::LStrLen {
                    dst: r_len,
                    str_v: local_v.unwrap(),
                },
                seq_expr.span,
            ),
        };
        let r_zero = self.alloc_i();
        let j_skip = if rev {
            let r_one = self.alloc_i();
            self.emit(
                Op::ConstImmI {
                    dst: r_zero,
                    val: 0,
                },
                seq_expr.span,
            );
            self.emit(Op::ConstImmI { dst: r_one, val: 1 }, seq_expr.span);
            self.emit(
                Op::SubI {
                    dst: r_idx,
                    a: r_len,
                    b: r_one,
                },
                seq_expr.span,
            );
            self.emit(
                Op::JmpLtI {
                    a: r_idx,
                    b: r_zero,
                    target: 0,
                },
                seq_expr.span,
            )
        } else {
            self.emit(Op::ConstImmI { dst: r_idx, val: 0 }, seq_expr.span);
            self.emit(
                Op::JmpGeI {
                    a: r_idx,
                    b: r_len,
                    target: 0,
                },
                seq_expr.span,
            )
        };
        let body_pc = self.ops.len() as u32;
        match slot_loc {
            SlotLoc::I(r_var, _) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1I {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1I {
                            dst: r_var,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadGList1I {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1I {
                            dst: r_var,
                            list_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, Some(gid)) => {
                    self.emit(
                        Op::GStrGetI {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, None) => {
                    self.emit(
                        Op::StrGetI {
                            dst: r_var,
                            str_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
            },
            SlotLoc::V(r_var) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1V {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1V {
                            dst: r_var,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadGList1V {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1V {
                            dst: r_var,
                            list_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::SmallTuple { base, len, .. } => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1SmallTuple {
                            dst_base: base,
                            len,
                            gid,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1SmallTuple {
                            dst_base: base,
                            len,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadList1SmallTuple {
                            dst_base: base,
                            len,
                            root: gid,
                            is_global: true,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadList1SmallTuple {
                            dst_base: base,
                            len,
                            root: local_v.unwrap(),
                            is_global: false,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::OptI { .. } => unreachable!(),
        }
        let own = LoopReg::Seq {
            id,
            r_idx,
            held: match global_id {
                Some(gid) => SeqHeld::Global(gid),
                None => SeqHeld::Local(local_v.unwrap()),
            },
        };
        self.loops.push(LoopTargets {
            id,
            breaks: Vec::new(),
            continues: Vec::new(),
            resumable: Some(own),
        });
        let saved_pinned = self.pinned_v;
        if let Some(lv) = local_v {
            self.pinned_v = self.pinned_v.max(lv + 1);
        }
        self.compile_stmts(&body.stmts);
        self.pinned_v = saved_pinned;
        let lt = self.loops.pop().unwrap();
        let step_pc = self.ops.len() as u32;
        for c in lt.continues {
            self.patch(c, step_pc);
        }
        self.note_back_edge(self.ops.len(), Some(own));
        if rev {
            self.emit(
                Op::ForDec {
                    r_i: r_idx,
                    r_lo: r_zero,
                    target: body_pc,
                },
                seq_expr.span,
            );
        } else {
            self.emit(
                Op::ForInc {
                    r_i: r_idx,
                    r_hi: r_len,
                    target: body_pc,
                },
                seq_expr.span,
            );
        }
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        for b in lt.breaks {
            self.patch(b, end_pc);
        }
        if let Some(lv) = local_v {
            if lv >= self.num_local_v {
                self.emit(Op::ClearV { reg: lv }, seq_expr.span);
            }
        }
        true
    }
}
