use super::seq_load::SeqKind;
use super::sort::sort_key_of;
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
        match stmt {
            Stmt::Let(slot, Some(init))
                if self.feed_acc != Some(*slot) && self.traced_quiet(init) =>
            {
                self.compile_untraced_stmt(stmt);
                if !self.fell_back(stmt) {
                    self.emit_trace(TracePoint::Let(*slot, init), init.span);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None) => return false,
            Stmt::Assign(place, _, value)
                if quiet_place(place, cpl_ir::walk::root(place)) && self.traced_quiet(value) =>
            {
                self.compile_untraced_stmt(stmt);
                if !self.fell_back(stmt) {
                    let point = self.cell_point(place).unwrap_or(TracePoint::Assign(place));
                    self.emit_trace(point, place.span);
                }
            }
            Stmt::If(cond, ..) | Stmt::While(_, cond, _) if self.traced_quiet(cond) => {
                return false
            }
            // Only the condition is left to the tree-walker; the branches stay here.
            Stmt::If(cond, then_b, else_b) if cond_writes(cond).is_some() => {
                let reg = self.emit_fallback_expr_i(cond);
                for (place, span) in cond_writes(cond).unwrap() {
                    self.emit_trace(TracePoint::Mutated(place, span), span);
                }
                let j_else = self.emit(Op::JmpIfZero { reg, target: 0 }, cond.span);
                self.compile_stmts(&then_b.stmts);
                let j_end = self.emit(Op::Jmp { target: 0 }, cond.span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                self.compile_stmts(&else_b.stmts);
                let end = self.ops.len() as u32;
                self.patch(j_end, end);
            }
            Stmt::Assert { cond, .. } if quiet(cond) => return false,
            Stmt::Return(Some(value)) | Stmt::Give(value) if self.traced_quiet(value) => {
                return false
            }
            Stmt::Expr(e) if self.traced_call(e).is_some() => {
                let dst = self.compile_expr_v(e);
                if dst >= self.num_local_v {
                    self.emit(Op::ClearV { reg: dst }, e.span);
                }
            }
            Stmt::For(id, _, Iter::Seq(seq, false), body) if self.fed_body_ok(*id, seq, body) => {
                self.emit_fallback_stmt(stmt);
                self.emit_escapes();
                self.compile_fed_body(*id, body);
            }
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
            Stmt::Expr(e) if self.traced_yield(e).is_some() => {
                let (acc, place, arg) = self.traced_yield(e).unwrap();
                let back = match arg.kind {
                    ExprKind::Local(slot) => slot as u16 + 1,
                    _ => 0,
                };
                let src = self.compile_expr_v(arg);
                let point = self.trace_points.len() as u32;
                self.trace_points.push(TracePoint::Assign(place));
                let at = self.emit(
                    Op::FeedTree {
                        src,
                        back,
                        acc: acc as u16,
                        point,
                        skip: 0,
                    },
                    e.span,
                );
                if src >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src }, e.span);
                }
                self.emit_fallback_stmt(stmt);
                self.emit_escapes();
                let skip = self.ops.len() as u32;
                if let Op::FeedTree { skip: s, .. } = &mut self.ops[at] {
                    *s = skip;
                }
            }
            Stmt::Expr(
                e @ Expr {
                    kind: ExprKind::Mutate(_, place, args),
                    ..
                },
            ) if quiet_place(place, cpl_ir::walk::root(place))
                && !self.yields_into(place)
                && args.iter().all(|a| self.traced_quiet(a)) =>
            {
                self.compile_untraced_stmt(stmt);
                let whole = self
                    .fallback_exprs
                    .last()
                    .is_some_and(|f| std::ptr::eq(*f, e));
                if !self.fell_back(stmt) && !whole {
                    self.emit_trace(TracePoint::Mutated(place, e.span), e.span);
                }
            }
            Stmt::Let(
                slot,
                Some(
                    init @ Expr {
                        kind: ExprKind::Mutate(_, place, args),
                        ..
                    },
                ),
            ) if self.feed_acc != Some(*slot)
                && quiet_place(place, cpl_ir::walk::root(place))
                && !self.yields_into(place)
                && args.iter().all(|a| self.traced_quiet(a)) =>
            {
                self.compile_untraced_stmt(stmt);
                if !self.fell_back(stmt) {
                    self.emit_trace(TracePoint::Mutated(place, init.span), init.span);
                    self.emit_trace(TracePoint::Let(*slot, init), init.span);
                }
            }
            // `print_values` tells the tracer what it printed, as in the tree-walker.
            Stmt::Expr(e) if Self::quiet_print(e) => self.compile_untraced_stmt(stmt),
            Stmt::Expr(e) if self.traced_quiet(e) => self.compile_untraced_stmt(stmt),
            _ => {
                self.emit_fallback_stmt(stmt);
                self.emit_escapes();
            }
        }
        true
    }

    /// A call a traced chunk makes itself: by-value arguments, and a
    /// result kept in one register.
    pub(super) fn traced_call(&self, e: &'p Expr) -> Option<(FnId, &'p [Arg])> {
        let ExprKind::Call(fn_id, args) = &e.kind else {
            return None;
        };
        let f = &self.program.fns[*fn_id];
        let ok = self.traced
            && args.len() == f.params.len()
            && (args.iter().zip(&f.params)).all(|(a, p)| matches!(a, Arg::Mut(_)) == p.mutable)
            && !self.program.judge_fns.contains(fn_id);
        ok.then_some((*fn_id, args.as_slice()))
    }

    /// [`quiet`], but for the calls a traced chunk makes itself, and a
    /// `do` block, whose statements each compile as traced statements.
    pub(super) fn traced_quiet(&self, e: &'p Expr) -> bool {
        let ok = match &e.kind {
            ExprKind::Call(..) => self.traced_call(e).is_some(),
            ExprKind::Do(block) => self.gives_natively(&block.stmts),
            // Made on the VM as `MakeClosure`, which writes nothing.
            ExprKind::Closure(cid) => (self.program.closures[*cid].captures.iter())
                .all(|cap| cap.is_global || !cap.by_ref),
            _ => quiet_node(e),
        };
        let mut children = ok;
        if !matches!(e.kind, ExprKind::Do(_)) {
            cpl_ir::walk::children(e, &mut |child| children &= self.traced_quiet(child));
        }
        children
    }

    /// Whether each `give` in `stmts` compiles on the VM, so none is left
    /// to a tree-walker statement, which could not reach the `do`'s end.
    fn gives_natively(&self, stmts: &'p [Stmt]) -> bool {
        stmts.iter().all(|stmt| match stmt {
            Stmt::Give(value) => self.traced_quiet(value),
            Stmt::If(cond, then, otherwise) => {
                !cpl_ir::walk::has_give(std::slice::from_ref(stmt))
                    || (self.traced_quiet(cond)
                        && self.gives_natively(&then.stmts)
                        && self.gives_natively(&otherwise.stmts))
            }
            _ => !cpl_ir::walk::has_give(std::slice::from_ref(stmt)),
        })
    }

    /// Whether `stmt` itself was left to the tree-walker, which reports it.
    fn fell_back(&self, stmt: &Stmt) -> bool {
        self.fallback_stmts
            .last()
            .is_some_and(|f| std::ptr::eq(*f, stmt))
    }

    pub(super) fn emit_traced_call(
        &mut self,
        e: &'p Expr,
        fn_id: FnId,
        args: &'p [Arg],
        scalar: bool,
    ) -> u16 {
        let regs: Vec<u16> = args.iter().map(|_| self.alloc_v()).collect();
        let mut indices = Vec::new();
        for arg in args {
            if let Arg::Mut(place) = arg {
                place_indices(place, &mut indices);
            }
        }
        let index_regs: Vec<u16> = indices.iter().map(|_| self.alloc_v()).collect();
        // Values first, then the places' indices, as the tree-walker lends.
        let values = args.iter().zip(&regs).filter_map(|(arg, &dst)| match arg {
            Arg::Value(arg) => Some((arg, dst)),
            Arg::Mut(_) => None,
        });
        for (e, dst) in values
            .collect::<Vec<_>>()
            .into_iter()
            .chain(indices.into_iter().zip(index_regs))
        {
            let src = self.compile_expr_v(e);
            self.emit(Op::MoveV { dst, src }, e.span);
        }
        let lends = match args.iter().any(|a| matches!(a, Arg::Mut(_))) {
            true => {
                self.lent_calls.push(args);
                (self.lent_calls.len() - 1) as u16
            }
            false => u16::MAX,
        };
        let dst_v = self.alloc_v();
        let dst_i = if scalar { self.alloc_i() } else { u16::MAX };
        self.emit(
            Op::CallTraced {
                fn_id: fn_id as u16,
                arg_base_v: regs.first().copied().unwrap_or(0),
                argc: args.len() as u8,
                dst_v,
                dst_i,
                lends,
            },
            e.span,
        );
        if scalar {
            dst_i
        } else {
            dst_v
        }
    }

    fn quiet_print(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Print(args, end) => {
                args.iter().all(|a| quiet(&a.value)) && end.as_deref().is_none_or(quiet)
            }
            _ => false,
        }
    }

    /// A `yield` of the `yields` fn this chunk is: its list, the place
    /// and the value.
    /// Whether `place` is this generator's yield list, which the tree-walker
    /// drives while a fed loop runs it.
    fn yields_into(&self, place: &Place) -> bool {
        let acc = self.cur_fn.and_then(|f| self.feeds.pushers.get(&f));
        acc.is_some_and(|&acc| cpl_ir::walk::root(place) == (false, acc))
    }

    fn traced_yield(&self, e: &'p Expr) -> Option<(usize, &'p Place, &'p Expr)> {
        let acc = *self.feeds.pushers.get(&self.cur_fn?)?;
        match &e.kind {
            ExprKind::Mutate(Mutation::Push, place, args) => match (&place.kind, args.as_slice()) {
                (PlaceKind::Local(slot), [arg]) if *slot == acc && quiet(arg) => {
                    Some((acc, place, arg))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether the tree-walker can run the body of the fed loop `id` from
    /// this traced chunk.
    fn fed_body_ok(&self, id: LoopId, seq: &'p Expr, body: &'p Block) -> bool {
        self.allow_fed_loops
            && self.feed_acc.is_none()
            && self.inline_slots.is_none()
            && self.inline_ret.is_none()
            && self.give_targets.is_empty()
            && self.feeds.feeds(id, seq, body).is_some()
            && !Self::block_has_outer_loop_jump(body, id)
    }

    /// The body of fed loop `id`, out of the way of the chunk's own flow:
    /// a `break` ends at `FeedBodyBreak`, the end and `continue` at
    /// `FeedBodyNext`.
    fn compile_fed_body(&mut self, id: LoopId, body: &'p Block) {
        let j_over = self.emit(Op::Jmp { target: 0 }, Span::default());
        self.fed_bodies.push((id, self.ops.len() as u32));
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
        let after = self.ops.len() as u32;
        self.patch(j_over, after);
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
                            let start = self.ops.len();
                            let r = self.compile_expr_i(expr);
                            let fresh = self.ops.len() > start;
                            match self.ops.last_mut() {
                                Some(Op::FromOrd { dst: last, .. }) if fresh && *last == r => {
                                    *last = dst
                                }
                                _ if r != dst => {
                                    self.emit(Op::MoveI { dst, src: r }, expr.span);
                                }
                                _ => {}
                            }
                            self.in_dom.retain(|f| f.0 != dst);
                            let dom = self.in_dom.iter().find(|f| f.0 == r && r != dst);
                            if let Some(&(_, dom, _)) = dom {
                                self.in_dom.push((dst, dom, Some(*slot)));
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
                                let back = match arg.kind {
                                    ExprKind::Local(slot) => slot as u16 + 1,
                                    _ => 0,
                                };
                                if let Some(kind) = ScalarKind::of_ty(&arg.ty, self.program) {
                                    let src = self.compile_expr_i(arg);
                                    self.emit(Op::FeedYieldI { src, kind, back }, expr.span);
                                } else {
                                    let src = self.compile_expr_v(arg);
                                    self.emit(Op::FeedYieldV { src, back }, expr.span);
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
                    if self.try_compile_mutate_stmt(*op, place, args, expr.span) {
                    } else if let Some(dst) = self.try_compile_mutate_place(expr) {
                        self.emit(Op::ClearV { reg: dst }, expr.span);
                    } else {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Sort(place, cpl_ir::SortBy::Natural, desc) = &expr.kind {
                    if !self.try_compile_sort_stmt(place, desc.as_deref(), expr.span) {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let Some((place, by, desc)) = sort_key_of(&expr.kind) {
                    if !self.try_compile_sort_key_stmt(place, by, desc, expr.span) {
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
                self.in_dom
                    .retain(|f| f.2.is_none_or(|s| !Self::place_mentions_slot(place, s)));
                if let (false, slot) = cpl_ir::walk::root(place) {
                    self.assigned_locals.push(slot);
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
                self.in_dom.clear();
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
                self.in_dom.clear();
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
            Stmt::DefDomain { id, lo, len } => {
                let (lo_r, len_r) = (self.compile_expr_i(lo), self.compile_expr_i(len));
                let dom = *id as u16;
                self.emit(
                    Op::DefDomain {
                        dom,
                        lo: lo_r,
                        len: len_r,
                    },
                    len.span,
                );
            }
            Stmt::DefDomainFromVal { id, val } => {
                let src = self.compile_expr_v(val);
                self.emit(
                    Op::DefDomainFromVal {
                        dom: *id as u16,
                        src,
                    },
                    val.span,
                );
                if src >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src }, val.span);
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
            || f.locals.len() > 512
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || f.params.iter().any(|p| p.mutable)
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

    /// The variable of `for k in 0..len(d)` is a position of `d` for the
    /// whole body unless the body might resize `d` or assigns `k`; then each
    /// `FromOrd` of it in the body is a plain move.
    fn drop_checks_of_positions(
        &mut self,
        r_var: u16,
        dom: u16,
        slot: Slot,
        body_pc: u32,
        (clears, assigned): (u32, usize),
    ) {
        if self.dom_clears != clears || self.assigned_locals[assigned..].contains(&slot) {
            return;
        }
        for op in &mut self.ops[body_pc as usize..] {
            if let Op::FromOrd { dst, src, dom: d } = *op {
                if src == r_var && d == dom {
                    *op = Op::MoveI { dst, src };
                }
            }
        }
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
                let marks = (self.dom_clears, self.assigned_locals.len());
                self.loops.push(LoopTargets {
                    id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                    resumable: Some(LoopReg::Range { id, r_var, r_hi }),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                if let (ExprKind::Int(0), ExprKind::DomainLen(d), false) =
                    (&lo.kind, &hi.kind, *closed)
                {
                    self.drop_checks_of_positions(r_var, *d as u16, slot, body_pc, marks);
                }
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
            Iter::Range {
                lo,
                hi,
                step: Some(step),
                rev: false,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                // The body may change a variable the bounds were read from.
                let [r_hi, r_step] = [hi, step].map(|e| {
                    let src = self.compile_expr_i(e);
                    let dst = self.alloc_i();
                    self.emit(Op::MoveI { dst, src }, e.span);
                    dst
                });
                let enter = self.emit(
                    Op::ForStepEnter {
                        r_lo,
                        r_hi,
                        r_step,
                        closed: *closed,
                        skip: 0,
                    },
                    step.span,
                );
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
                    resumable: None,
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                for c in lt.continues {
                    self.patch(c, step_pc);
                }
                self.emit(
                    Op::ForStepNext {
                        r_i: r_var,
                        r_hi,
                        r_step,
                        closed: *closed,
                        target: body_pc,
                    },
                    hi.span,
                );
                let end_pc = self.ops.len() as u32;
                self.patch(enter, end_pc);
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
        let seq_kind = match &seq_expr.ty {
            Ty::Array { dims, .. } if dims.len() == 1 => SeqKind::Array,
            Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) => SeqKind::List,
            Ty::Str | Ty::StrView => SeqKind::Str,
            Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) => SeqKind::List,
            _ => return false,
        };
        let items = matches!(
            seq_expr.ty,
            Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..)
        );
        let slot_loc = self.slot_loc(slot);
        match (seq_kind, slot_loc) {
            (SeqKind::Str, SlotLoc::I(..)) => {}
            (SeqKind::Str, _) => return false,
            (_, SlotLoc::I(..) | SlotLoc::V(_) | SlotLoc::SmallTuple { .. }) => {}
            (_, SlotLoc::OptI { .. }) => {}
        }
        let r_idx = self.alloc_i();
        let r_len = self.alloc_i();
        let global_id = match &seq_expr.kind {
            ExprKind::Global(gid) if matches!(seq_expr.ty, Ty::Array { .. } | Ty::List(_)) => {
                Some(*gid as u16)
            }
            _ => None,
        };
        let local_v = if items {
            let src = self.compile_expr_v(seq_expr);
            let dst = self.alloc_v();
            self.emit(Op::SeqItems { dst, src }, seq_expr.span);
            Some(dst)
        } else if global_id.is_none() {
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
        self.emit_seq_load(slot_loc, seq_kind, global_id, local_v, r_idx, seq_expr.span);
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

/// The places a condition the tree-walker evaluates changes in place, in
/// order, when those are its only writes: calls without `mut` arguments
/// report their own steps. `None` for any other write.
fn cond_writes(cond: &Expr) -> Option<Vec<(&Place, Span)>> {
    let mut writes = Some(vec![]);
    cpl_ir::walk::deep_expr(cond, &mut |e| match &e.kind {
        ExprKind::Mutate(_, place, _) => {
            if let Some(w) = &mut writes {
                w.push((&**place, e.span));
            }
        }
        ExprKind::Call(_, args) if args.iter().all(|a| !matches!(a, Arg::Mut(_))) => {}
        ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Do(_) | ExprKind::Closure(_) => {
            writes = None
        }
        _ => {}
    });
    writes
}
