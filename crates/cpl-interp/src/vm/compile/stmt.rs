use super::jump::JumpOp;
use super::seq_load::SeqKind;
use super::{Compiler, GiveTarget, LoopExits, LoopJump, LoopTargets};
use crate::vm::helpers::*;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, Block, Expr, ExprKind, FnId, Iter, LoopId, Mutation, Place, PlaceKind, Slot,
    Stmt, Ty,
};

/// What a traced call calls: a fn, or the fn value an expression gives.
#[derive(Clone, Copy)]
pub(super) enum Callee<'p> {
    Fn(FnId),
    Value(&'p Expr),
}

impl<'p, 'f> Compiler<'p, 'f> {
    /// Compiles `stmts` as the body of loop `id` and gives back the
    /// `break` and `continue` jumps it left for the caller to patch.
    pub(super) fn loop_body(
        &mut self,
        id: LoopId,
        resumable: Option<LoopReg>,
        stmts: &'p [Stmt],
    ) -> LoopExits {
        let (depth, first_jump) = (self.loops.len(), self.jumps.len());
        self.loops.push(LoopTargets {
            id,
            breaks: Vec::new(),
            continues: Vec::new(),
            resumable,
        });
        self.compile_stmts(stmts);
        let mut exits = LoopExits::default();
        let depth = depth.min(self.loops.len());
        for mut lt in self.loops.drain(depth..) {
            exits.breaks.append(&mut lt.breaks);
            exits.continues.append(&mut lt.continues);
        }
        // A jump to a loop further out stays for that loop to claim.
        let jumps: Vec<LoopJump> = self.jumps.drain(first_jump..).collect();
        for j in jumps {
            match j.to {
                Some(to) if to != id => self.jumps.push(j),
                _ if j.brk => exits.breaks.push(j.jump),
                _ => exits.continues.push(j.jump),
            }
        }
        exits
    }

    /// Compiles `stmts` with `compile` as a `do` block's body, `target`
    /// open for its `give`s, then sends every `give` to the block's end.
    pub(super) fn give_body(
        &mut self,
        target: GiveTarget,
        stmts: &'p [Stmt],
        compile: fn(&mut Self, &'p [Stmt]),
    ) {
        let prev_pinned = self.pinned_v;
        self.pinned_v = self.next_v;
        let depth = self.give_targets.len();
        self.give_targets.push(target);
        compile(self, stmts);
        self.pinned_v = prev_pinned;
        let end_pc = self.ops.len() as u32;
        let depth = depth.min(self.give_targets.len());
        while self.give_targets.len() > depth {
            let Some(mut t) = self.give_targets.pop() else {
                break;
            };
            self.patch_all(std::mem::take(t.exits_mut()), end_pc);
        }
    }

    pub(super) fn compile_stmts(&mut self, stmts: &'p [Stmt]) {
        if let Some((slot, span)) = self.traced_loop_var.take() {
            self.emit_trace(TracePoint::Loop(slot, span), span);
        }
        let has_closure = Self::stmts_have_closure(stmts);
        let mut declared = Vec::new();
        let mut i = 0;
        let temps = self.temps_mark();
        while i < stmts.len() {
            self.release_temps(temps);
            if self.traced || self.profiled {
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
        self.release_temps(temps);
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

    /// In a profiled chunk, a step for `stmt` starting: its op.
    pub(super) fn emit_step(&mut self, stmt: &'p Stmt) -> Option<usize> {
        self.profiled.then(|| {
            let idx = self.steps.len() as u32;
            self.steps.push(stmt);
            self.emit(Op::Step { idx }, stmt.span().unwrap_or_default())
        })
    }

    pub(super) fn compile_stmt(&mut self, stmt: &'p Stmt) {
        let step = self.emit_step(stmt);
        let start = self.ops.len();
        if !self.traced || !self.compile_traced_stmt(stmt) {
            self.compile_untraced_stmt(stmt);
        }
        // The tree-walker steps a statement it runs itself.
        if let Some(step) = step {
            let falls_back = (start..self.ops.len()).any(|pc| {
                matches!(self.ops[pc], Op::FallbackStmt { idx, .. }
                    if std::ptr::eq(self.fallback_stmts[idx as usize], stmt))
            });
            if falls_back {
                self.ops[step] = Op::Jmp {
                    target: start as u32,
                };
            }
        }
        self.note_call_site(start, stmt);
    }

    /// A write to `place` the VM reports with raw values: a global, or a
    /// cell of one whose `Int` indices it evaluates again.
    pub(super) fn cell_point(&mut self, place: &'p Place) -> Option<TracePoint<'p>> {
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
        if let Stmt::For(id, slot, Iter::Seq(seq, false), body) = stmt {
            if let Some((fn_id, feed, args)) = self.traced_feed(*id, *slot, seq, body) {
                let eager = self.emit_jump(JumpOp::JmpEager, seq.span);
                let dst = self.emit_traced_call(seq.span, Callee::Fn(fn_id), args, false);
                // `emit_traced_call` ends with its `CallTraced`.
                let call = self.ops.len() - 1;
                self.traced_feeds.push((call as u32, feed));
                self.note_fed_site(call, stmt, seq);
                self.free_temp_v(dst, seq.span);
                let j_end = self.emit_jump(JumpOp::Jmp, seq.span);
                self.patch(eager, self.ops.len() as u32);
                self.compile_eager_for(*id, *slot, seq, body);
                self.patch(j_end, self.ops.len() as u32);
                self.compile_fed_body(*id, body);
                return true;
            }
        }
        match stmt {
            Stmt::Let(slot, Some(init)) => {
                self.compile_reporting_stmt(stmt, init);
                if !self.fell_back(stmt) {
                    self.emit_trace(TracePoint::Let(*slot, init), init.span);
                }
            }
            Stmt::Let(slot, None) => {
                self.compile_untraced_stmt(stmt);
                if !self.fell_back(stmt) {
                    self.emit_trace(TracePoint::Declared(*slot), stmt.span().unwrap_or_default());
                }
            }
            // A trap ends the run: the tracer takes it from the failure. A
            // domain's bounds are no value the trace shows. Each call, write
            // and print in a value reports itself as it runs.
            Stmt::Break(_)
            | Stmt::Continue(_)
            | Stmt::Return(_)
            | Stmt::Give(_)
            | Stmt::Trap(..)
            | Stmt::Assert { .. }
            | Stmt::DefDomain { .. }
            | Stmt::DefDomainFromVal { .. } => return false,
            Stmt::Assign(place, _, value) => {
                self.compile_reporting_stmt(stmt, value);
                if !self.fell_back(stmt) {
                    // Indices the write ran once are not run again for the
                    // step: it shows the root changed.
                    let point = match quiet_place(place, cpl_ir::walk::root(place)) {
                        true => self.cell_point(place).unwrap_or(TracePoint::Assign(place)),
                        false => TracePoint::Mutated(place, place.span),
                    };
                    self.emit_trace(point, place.span);
                }
            }
            Stmt::If(cond, ..) | Stmt::While(_, cond, _) if self.traced_quiet(cond) => {
                return false
            }
            // Only the condition is left to the tree-walker; the branches stay here.
            Stmt::If(cond, then_b, else_b) if cond_writes(cond).is_some() => {
                let Some(writes) = cond_writes(cond) else {
                    return false;
                };
                let reg = self.emit_fallback_expr_i(cond);
                for (place, span) in writes {
                    self.emit_trace(TracePoint::Mutated(place, span), span);
                }
                let j_else = self.emit_jump(JumpOp::JmpIfZero { reg }, cond.span);
                self.compile_stmts(&then_b.stmts);
                let j_end = self.emit_jump(JumpOp::Jmp, cond.span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                self.compile_stmts(&else_b.stmts);
                let end = self.ops.len() as u32;
                self.patch(j_end, end);
            }
            Stmt::If(..) | Stmt::While(..) => return false,
            Stmt::Expr(e) if self.traced_call(e).is_some() => {
                let dst = self.compile_expr_v(e);
                self.free_temp_v(dst, e.span);
            }
            Stmt::For(_, slot, iter, _) => match self.traced_quiet_iter(iter) {
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
                let Some((acc, list_v, place, arg, each)) = self.traced_yield(e) else {
                    return false;
                };
                let back = match arg.kind {
                    ExprKind::Local(slot) if !each => slot as u16 + 1,
                    _ => 0,
                };
                let path = match each {
                    true => match self.compile_place_path_to(place, true) {
                        Some(path) => Some(path),
                        None => return false,
                    },
                    false => None,
                };
                let src = self.compile_expr_v(arg);
                let point = self.trace_points.len() as u32;
                self.trace_points.push(TracePoint::Assign(place));
                let at = self.emit(
                    Op::FeedTree {
                        src,
                        back,
                        each,
                        acc: acc as u16,
                        point,
                        skip: 0,
                    },
                    e.span,
                );
                // Run eagerly, as by a comprehension, the yield keeps its value.
                match path {
                    Some(path) => self.emit_extend_from(e, path, src),
                    None => {
                        self.emit(Op::ListPushLV { list_v, val_v: src }, e.span);
                    }
                }
                self.emit_trace(TracePoint::Mutated(place, e.span), e.span);
                let skip = self.ops.len() as u32;
                if let Op::FeedTree { skip: s, .. } = &mut self.ops[at] {
                    *s = skip;
                }
                self.free_temp_v(src, e.span);
            }
            Stmt::Expr(Expr {
                kind: ExprKind::Sort(place, ..),
                ..
            }) => {
                self.compile_untraced_stmt(stmt);
                self.trace_sorted(stmt, place);
            }
            Stmt::Expr(e) => self.compile_reporting_stmt(stmt, e),
        }
        true
    }

    /// A call a traced chunk makes itself: by-value arguments, and a
    /// result kept in one register.
    pub(super) fn traced_call(&self, e: &'p Expr) -> Option<(Callee<'p>, &'p [Arg])> {
        if let ExprKind::CallExpr(callee, args) = &e.kind {
            let ok = self.traced && matches!(callee.ty, Ty::Fn(..));
            return ok.then_some((Callee::Value(callee), args.as_slice()));
        }
        let ExprKind::Call(fn_id, args) = &e.kind else {
            return None;
        };
        let f = &self.program.fns[*fn_id];
        let ok = self.traced
            && args.len() == f.params.len()
            && (args.iter().zip(&f.params)).all(|(a, p)| matches!(a, Arg::Mut(_)) == p.mutable)
            && !self.program.judge_fns.contains(fn_id);
        ok.then_some((Callee::Fn(*fn_id), args.as_slice()))
    }

    /// [`quiet`], but for the calls a traced chunk makes itself, and a
    /// `do` block, whose statements each compile as traced statements.
    pub(super) fn traced_quiet(&self, e: &'p Expr) -> bool {
        let ok = match &e.kind {
            ExprKind::Call(..) | ExprKind::CallExpr(..) => self.traced_call(e).is_some(),
            ExprKind::Do(block) => self.gives_natively(&block.stmts),
            // Making a closure writes nothing: `MakeClosure` on the VM, or
            // the tree-walker for one that captures a local by reference.
            ExprKind::Closure(_) => true,
            // Reported as each runs: a write by `reporting_write`, a sort
            // by `trace_sorted`, a print or debug as it runs.
            ExprKind::Mutate(_, place, _) => {
                quiet_place(place, cpl_ir::walk::root(place)) && !self.yields_into(place)
            }
            ExprKind::Print(..) | ExprKind::Debug(..) => true,
            // A view writes nothing until written through.
            ExprKind::MutSlice(..) | ExprKind::MutLine(..) => true,
            ExprKind::Sort(place, ..) => {
                quiet_place(place, cpl_ir::walk::root(place)) && !self.yields_into(place)
            }
            _ => quiet_node(e),
        };
        let mut children = ok;
        if !matches!(e.kind, ExprKind::Do(_)) {
            cpl_ir::walk::children(e, &mut |child| children &= self.traced_quiet(child));
        }
        children
    }

    /// Where a traced `for` reports its variable, when its iterable is
    /// [`Self::traced_quiet`]: evaluated once before the loop, so a `do`
    /// in it traces its own lets as a `let` initializer's would.
    fn traced_quiet_iter(&self, iter: &'p Iter) -> Option<Span> {
        match iter {
            Iter::Range { lo, hi, step, .. } => (self.traced_quiet(lo)
                && self.traced_quiet(hi)
                && step.as_ref().is_none_or(|s| self.traced_quiet(s)))
            .then_some(lo.span),
            Iter::RangeVal(e, _) | Iter::Seq(e, _) => self.traced_quiet(e).then_some(e.span),
            Iter::TreeRange {
                container, lo, hi, ..
            } => (self.traced_quiet(container)
                && [lo, hi]
                    .iter()
                    .all(|b| b.as_ref().is_none_or(|(e, _)| self.traced_quiet(e))))
            .then_some(container.span),
        }
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
    pub(super) fn fell_back(&self, stmt: &Stmt) -> bool {
        self.fallback_stmts
            .last()
            .is_some_and(|f| std::ptr::eq(*f, stmt))
    }

    pub(super) fn emit_traced_call(
        &mut self,
        span: Span,
        callee: Callee<'p>,
        args: &'p [Arg],
        scalar: bool,
    ) -> u16 {
        let (fn_id, callee_v) = match callee {
            Callee::Fn(fn_id) => (fn_id as u16, 0),
            Callee::Value(callee) => (u16::MAX, self.compile_expr_v(callee)),
        };
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
                fn_id,
                callee_v,
                arg_base_v: regs.first().copied().unwrap_or(0),
                argc: args.len() as u8,
                dst_v,
                dst_i,
                lends,
            },
            span,
        );
        if scalar {
            dst_i
        } else {
            dst_v
        }
    }

    /// Whether `place` is this generator's yield list, which the tree-walker
    /// drives while a fed loop runs it.
    pub(super) fn yields_into(&self, place: &Place) -> bool {
        let acc = self.cur_fn.and_then(|f| self.feeds.pushers.get(&f));
        acc.is_some_and(|&acc| cpl_ir::walk::root(place) == (false, acc))
    }

    /// A `yield` of the `yields` fn this chunk is: its list's slot and
    /// register, the place and the value.
    fn traced_yield(&self, e: &'p Expr) -> Option<(usize, u16, &'p Place, &'p Expr, bool)> {
        let acc = *self.feeds.pushers.get(&self.cur_fn?)?;
        let SlotLoc::V(list_v) = self.slot_loc(acc) else {
            return None;
        };
        match &e.kind {
            ExprKind::Mutate(op @ (Mutation::Push | Mutation::Extend), place, args) => {
                match (&place.kind, args.as_slice()) {
                    (PlaceKind::Local(slot), [arg]) if *slot == acc && self.traced_quiet(arg) => {
                        Some((acc, list_v, place, arg, *op == Mutation::Extend))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// The eager `yield from` `e`: extends the list at `path` by the
    /// sequence already in `src`.
    fn emit_extend_from(&mut self, e: &'p Expr, path: u16, src: u16) {
        let idx = self.applies.len() as u32;
        self.applies.push(ApplySpec {
            expr: e,
            regs: vec![src],
            view: None,
        });
        let dst = self.alloc_v();
        self.emit(Op::MutatePlace { idx, path, dst }, e.span);
        self.emit(Op::ClearV { reg: dst }, e.span);
    }

    /// The list the fed loop `id` is fed into, when this traced chunk can
    /// run its body.
    fn fed_acc(&self, id: LoopId, seq: &'p Expr, body: &'p Block) -> Option<usize> {
        let ok = self.allow_fed_loops
            && self.feed_acc.is_none()
            && self.inline_slots.is_none()
            && self.inline_ret.is_none()
            && (self.give_targets.is_empty() || !cpl_ir::walk::has_give(&body.stmts))
            && !Self::block_has_outer_loop_jump(body, id);
        ok.then(|| self.feeds.feeds(id, seq, body))
            .flatten()
            .map(|(_, acc)| acc)
    }

    /// The fed loop `id`'s call, when this chunk makes it itself.
    fn traced_feed(
        &self,
        id: LoopId,
        slot: Slot,
        seq: &'p Expr,
        body: &'p Block,
    ) -> Option<(FnId, TracedFeed<'p>, &'p [Arg])> {
        // A judge's or `@cpp` fn traps as `CallTraced` makes it, as any call.
        let ExprKind::Call(fn_id, args) = &seq.kind else {
            return None;
        };
        let acc = self.fed_acc(id, seq, body)?;
        Some((
            *fn_id,
            TracedFeed {
                id,
                slot,
                acc,
                body,
            },
            args,
        ))
    }

    /// The fed loop `id` run eagerly where it feeds no loop: in a
    /// closure, or in a generator a loop is being fed from.
    fn compile_eager_for(&mut self, id: LoopId, slot: Slot, seq: &'p Expr, body: &'p Block) {
        self.traced_loop_var = Some((slot, seq.span));
        self.in_dom.clear();
        self.compile_for_seq(id, slot, seq, None, false, body);
        self.traced_loop_var = None;
        self.emit_trace(TracePoint::LoopEnd(slot), seq.span);
    }

    /// The body of fed loop `id`, out of the way of the chunk's own flow:
    /// a `break` ends at `FeedBodyBreak`, the end and `continue` at
    /// `FeedBodyNext`.
    fn compile_fed_body(&mut self, id: LoopId, body: &'p Block) {
        let j_over = self.emit_jump(JumpOp::Jmp, Span::default());
        self.fed_bodies.push((id, self.ops.len() as u32));
        let lt = self.loop_body(id, None, &body.stmts);
        let next_pc = self.ops.len() as u32;
        self.patch_all(lt.continues, next_pc);
        self.emit(Op::FeedBodyNext, Span::default());
        let break_pc = self.ops.len() as u32;
        self.patch_all(lt.breaks, break_pc);
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
            let pc = self.emit_jump(
                JumpOp::JmpEscape {
                    brk: true,
                    id,
                    innermost,
                },
                Span::default(),
            );
            self.loops[pos].breaks.push(pc);
            let pc = self.emit_jump(
                JumpOp::JmpEscape {
                    brk: false,
                    id,
                    innermost,
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
                            let ty = self.fill_tys.len() as u16;
                            self.fill_tys.push(&self.locals[*slot].ty);
                            self.emit(Op::ZeroV { dst, ty }, Span::default());
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
                                    self.free_temp_v(src, expr.span);
                                }
                                return;
                            }
                            (Mutation::Extend, [arg]) => {
                                let src = self.compile_expr_v(arg);
                                self.emit(Op::FeedYieldExtend { src }, expr.span);
                                self.free_temp_v(src, expr.span);
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                if ScalarKind::of_ty(&expr.ty, self.program).is_some() {
                    let _ = self.compile_expr_i(expr);
                } else if let ExprKind::Mutate(op, place, args) = &expr.kind {
                    if !super::apply::through_view(place)
                        && self.try_compile_mutate_stmt(*op, place, args, expr.span)
                    {
                    } else if let Some(dst) = self.try_compile_apply_v(expr) {
                        self.emit(Op::ClearV { reg: dst }, expr.span);
                    } else {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Sort(place, by, desc) = &expr.kind {
                    self.compile_sort(place, by, desc.as_deref(), expr);
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
                    self.give_body(
                        GiveTarget::Discard { exits: Vec::new() },
                        &block.stmts,
                        Self::compile_stmts,
                    );
                } else if let Some(dst) = self.try_compile_apply_v(expr) {
                    self.emit(Op::ClearV { reg: dst }, expr.span);
                } else if let ExprKind::Local(_) = expr.kind {
                    // Reading a local for nothing does nothing.
                } else {
                    let dst = self.compile_expr_v(expr);
                    self.free_temp_v(dst, expr.span);
                }
            }
            Stmt::Assert {
                cond, kept: false, ..
            } if !crate::release::keeps(cpl_ir::checks::Check::Asserts) => {
                self.compile_expr_i(cond);
            }
            Stmt::Assert {
                cond,
                message,
                span,
                ..
            } => {
                let r_cond = self.compile_expr_i(cond);
                let j_ok = self.emit_jump(JumpOp::JmpIfNonZero { reg: r_cond }, *span);
                let msg_v = message
                    .as_ref()
                    .map_or(u16::MAX, |m| self.compile_expr_v(m));
                self.emit_fail(stmt, msg_v, *span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_ok, end_pc);
            }
            Stmt::Assign(place, op, value) => {
                if super::apply::through_view(place) {
                    let write = ViewOp::Write(place, *op);
                    let reg = self.compile_view(write, place, &[value], true, value);
                    self.emit(Op::ClearV { reg }, place.span);
                } else if !self.try_compile_assign(place, *op, value) {
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
                    self.patch_all(j_else, end);
                } else {
                    let j_end = self.emit_jump(JumpOp::Jmp, cond.span);
                    let else_pc = self.ops.len() as u32;
                    self.patch_all(j_else, else_pc);
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
                let lt = self.loop_body(*id, Some(LoopReg::While(*id)), &body.stmts);
                self.patch_all(lt.continues, head);
                if self.traced {
                    self.emit_trace(TracePoint::Tick, cond.span);
                }
                let back = self.ops.len();
                self.note_back_edge(back, Some(LoopReg::While(*id)));
                self.emit(Op::Jmp { target: head }, cond.span);
                let end = self.ops.len() as u32;
                self.patch_all(j_exit, end);
                self.patch_all(lt.breaks, end);
            }
            Stmt::For(id, slot, iter, body) => {
                self.in_dom.clear();
                if !self.try_compile_for(stmt, *id, *slot, iter, body) {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Break(to) | Stmt::Continue(to) => {
                let jump = self.emit_jump(JumpOp::Jmp, Span::default());
                let brk = matches!(stmt, Stmt::Break(_));
                self.jumps.push(LoopJump { to: *to, brk, jump });
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
                    let pc = self.emit_jump(JumpOp::Jmp, Span::default());
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
                    let pc = self.emit_jump(JumpOp::Jmp, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if self.cur_fn.is_none() {
                    // The program's own block: its value ends the program.
                    let _ = self.compile_expr_v(e);
                    self.emit(Op::RetUnit, e.span);
                } else if let Some((len, kinds)) = self.tuple_ret_of(e) {
                    let base = self.small_tuple_operand(e, len);
                    self.emit(Op::RetSmallTuple { base, len, kinds }, e.span);
                } else if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                    let r = self.compile_expr_i(e);
                    self.emit(Op::RetI { src: r }, e.span);
                } else {
                    let r = self.compile_expr_v(e);
                    let r = self.ret_v(r, e.span);
                    self.emit(Op::RetV { src: r }, e.span);
                }
            }
            Stmt::Return(val) => {
                // An inlined fn's `return` leaves that fn, never the feed it's inlined into.
                if let Some((ret_dst, _)) = self.inline_ret {
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
                    let pc = self.emit_jump(JumpOp::Jmp, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if self.feed_acc.is_some() {
                    self.emit(Op::FeedEnd, Span::default());
                } else if let Some(e) = val {
                    if let Some((len, kinds)) = self.tuple_ret_of(e) {
                        let base = self.small_tuple_operand(e, len);
                        self.emit(Op::RetSmallTuple { base, len, kinds }, e.span);
                    } else if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                        let r = self.compile_expr_i(e);
                        self.emit(Op::RetI { src: r }, e.span);
                    } else {
                        let r = self.compile_expr_v(e);
                        let r = self.ret_v(r, e.span);
                        self.emit(Op::RetV { src: r }, e.span);
                    }
                } else {
                    self.emit(Op::RetUnit, Span::default());
                }
            }
            Stmt::Trap(_, span) => self.emit_fail(stmt, u16::MAX, *span),
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
                self.free_temp_v(src, val.span);
            }
        }
    }

    pub(super) fn block_has_outer_loop_jump(block: &Block, fed_id: LoopId) -> bool {
        fn check_block(b: &Block, fed_id: LoopId, inner: &mut Vec<LoopId>, in_do: bool) -> bool {
            for s in &b.stmts {
                match s {
                    Stmt::Break(Some(id)) | Stmt::Continue(Some(id)) => {
                        if *id != fed_id && !inner.contains(id) {
                            return true;
                        }
                    }
                    Stmt::If(_, t, e) => {
                        if check_block(t, fed_id, inner, in_do)
                            || check_block(e, fed_id, inner, in_do)
                        {
                            return true;
                        }
                    }
                    Stmt::While(lid, _, body) | Stmt::For(lid, _, _, body) => {
                        inner.push(*lid);
                        let bad = check_block(body, fed_id, inner, in_do);
                        inner.pop();
                        if bad {
                            return true;
                        }
                    }
                    Stmt::Expr(e) if matches!(e.kind, ExprKind::Do(_)) => {
                        if let ExprKind::Do(b2) = &e.kind {
                            if check_block(b2, fed_id, inner, true) {
                                return true;
                            }
                        }
                    }
                    // A `give` leaves the innermost `do`, so only one outside it leaves the loop.
                    Stmt::Give(_) if !in_do => return true,
                    _ => {}
                }
            }
            false
        }
        check_block(block, fed_id, &mut Vec::new(), false)
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
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || self.inline_ret.is_some()
            || Self::block_has_outer_loop_jump(body, id)
        {
            return false;
        }
        let Some((args_i, args_v, muts)) = self.compile_fed_args(fn_id, args) else {
            return false;
        };
        let var_loc = self.slot_loc(slot);
        let spec_idx = self.feed_calls.len() as u16;
        self.feed_calls.push(FeedCallSpec {
            fn_id,
            args_i,
            args_v,
            body_pc: 0,
            var_loc,
            muts,
        });
        let call_pc = self.emit(Op::CallVmFeed { spec: spec_idx }, seq_expr.span);
        self.note_fed_site(call_pc, stmt, seq_expr);
        let j_after = self.emit_jump(JumpOp::Jmp, seq_expr.span);

        let body_pc = self.ops.len();
        self.feed_calls[spec_idx as usize].body_pc = body_pc;
        let lt = self.loop_body(id, None, &body.stmts);
        let next_pc = self.ops.len() as u32;
        self.patch_all(lt.continues, next_pc);
        self.emit(Op::FeedBodyNext, Span::default());
        let break_pc = self.ops.len() as u32;
        self.patch_all(lt.breaks, break_pc);
        self.emit(Op::FeedBodyBreak, Span::default());
        let after_pc = self.ops.len() as u32;
        self.patch(j_after, after_pc);
        true
    }

    /// The variable of `for k in 0..len(d)` is a position of `d` for the
    /// whole body unless the body might resize `d` or assigns `k`; then each
    /// `FromOrd` of it in the body is a plain move.
    pub(super) fn drop_checks_of_positions(
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
            // Inside a fed fn, the body's yields feed on out; one that may
            // stop the loop early has no such form.
            let fed_here =
                self.allow_fed_loops && (self.feed_acc.is_none() || !cpl_ir::pushed::escapes(body));
            if !*rev && fed_here {
                if let Some((fn_id, _acc_slot)) = self.feeds.feeds(id, seq_expr, body) {
                    return self.try_compile_for_fed(stmt, id, slot, fn_id, seq_expr, body);
                }
            }
            if !*rev && self.feeds.feeds(id, seq_expr, body).is_some() {
                return false;
            }
            self.compile_for_seq(id, slot, seq_expr, None, *rev, body);
            return true;
        }
        let in_i = matches!(self.slot_loc(slot), SlotLoc::I(..));
        match iter {
            // An `I128` or `U64` range, or one whose items aren't ints, walks its items.
            Iter::RangeVal(e, rev)
                if !in_i || crate::value::WideElem::of_range(&e.ty).is_some() =>
            {
                self.compile_for_seq(id, slot, e, None, *rev, body);
                return true;
            }
            Iter::TreeRange {
                container,
                lo,
                hi,
                rev,
            } => {
                let src = self.compile_expr_v(container);
                let [lo, hi] = [lo, hi]
                    .map(|b| (b.as_ref()).map(|(e, closed)| (self.compile_expr_v(e), *closed)));
                let dst = self.alloc_v();
                self.emit(Op::TreeRangeItems { dst, src, lo, hi }, container.span);
                self.compile_for_seq(id, slot, container, Some(dst), *rev, body);
                return true;
            }
            _ => {}
        }
        // A range's ints count in a register; an enum variable is tagged from it.
        let (r_var, tag_v) = match self.slot_loc(slot) {
            SlotLoc::I(r_var, _) => (r_var, None),
            SlotLoc::V(v) if matches!(iter, Iter::Range { .. }) => (self.alloc_i(), Some(v)),
            _ => return false,
        };
        match iter {
            Iter::RangeVal(e, rev) => {
                let src_v = self.compile_expr_v(e);
                let r_lo = self.alloc_i();
                let r_hi = self.alloc_i();
                self.emit(
                    Op::RangeBoundsI {
                        lo: r_lo,
                        hi: r_hi,
                        src_v,
                    },
                    e.span,
                );
                if *rev {
                    self.emit_down_loop(id, r_var, tag_v, r_lo, r_hi, false, body, e.span, e.span);
                } else {
                    self.emit_up_loop(
                        id, slot, r_var, tag_v, r_lo, r_hi, false, None, false, body, e.span,
                        e.span,
                    );
                }
                true
            }
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let positions_of = match (&lo.kind, &hi.kind, *closed) {
                    (ExprKind::Int(0), ExprKind::DomainLen(d), false) => Some(*d as u16),
                    _ => None,
                };
                self.emit_up_loop(
                    id,
                    slot,
                    r_var,
                    tag_v,
                    r_lo,
                    r_hi,
                    *closed,
                    positions_of,
                    true,
                    body,
                    lo.span,
                    hi.span,
                );
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
                self.emit_down_loop(
                    id, r_var, tag_v, r_lo, r_hi, *closed, body, lo.span, hi.span,
                );
                true
            }
            Iter::Range {
                lo,
                hi,
                step: Some(step),
                rev,
                closed,
            } => {
                let mut copy = |e: &'p Expr| {
                    let src = self.compile_expr_i(e);
                    let dst = self.alloc_i();
                    self.emit(Op::MoveI { dst, src }, e.span);
                    dst
                };
                // The body may change a variable the bounds were read from;
                // walked backward, the loop ends at `lo` and starts at `r_hi`.
                let r_lo = if *rev {
                    copy(lo)
                } else {
                    self.compile_expr_i(lo)
                };
                let [r_hi, r_step] = [hi, step].map(|e| {
                    let src = self.compile_expr_i(e);
                    let dst = self.alloc_i();
                    self.emit(Op::MoveI { dst, src }, e.span);
                    dst
                });
                let enter = self.emit_jump(
                    JumpOp::ForStepEnter {
                        r_lo,
                        r_hi,
                        r_step,
                        closed: *closed,
                        rev: *rev,
                    },
                    step.span,
                );
                let (first, r_end) = if *rev { (r_hi, r_lo) } else { (r_lo, r_hi) };
                if r_var != first {
                    self.emit(
                        Op::MoveI {
                            dst: r_var,
                            src: first,
                        },
                        lo.span,
                    );
                }
                let body_pc = self.ops.len() as u32;
                self.emit_loop_tag(tag_v, r_var, lo.span);
                let lt = self.loop_body(id, None, &body.stmts);
                let step_pc = self.ops.len() as u32;
                self.patch_all(lt.continues, step_pc);
                self.emit(
                    Op::ForStepNext {
                        r_i: r_var,
                        r_end,
                        r_step,
                        closed: *closed,
                        rev: *rev,
                        target: body_pc,
                    },
                    hi.span,
                );
                let end_pc = self.ops.len() as u32;
                self.patch(enter, end_pc);
                self.patch_all(lt.breaks, end_pc);
                true
            }
            _ => false,
        }
    }

    /// `for slot in seq_expr`, or over the list already in `listed`.
    pub(super) fn compile_for_seq(
        &mut self,
        id: LoopId,
        slot: Slot,
        seq_expr: &'p Expr,
        listed: Option<u16>,
        rev: bool,
        body: &'p Block,
    ) {
        let slot_loc = self.slot_loc(slot);
        let reads = match (&seq_expr.ty, slot_loc) {
            _ if listed.is_some() => SeqReads::Elems(SeqKind::List),
            (Ty::Array { dims, .. }, _) if dims.len() == 1 => SeqReads::Elems(SeqKind::Array),
            (Ty::Str | Ty::StrView, SlotLoc::I(var, _)) => SeqReads::Chars(var),
            _ => SeqReads::Elems(SeqKind::List),
        };
        // Any other sequence walks the list of its items; a table its cells
        // in row-major order.
        let items = listed.is_none()
            && match &seq_expr.ty {
                Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) => false,
                Ty::Array { dims, .. } => dims.len() > 1,
                Ty::Str | Ty::StrView => !matches!(reads, SeqReads::Chars(_)),
                _ => true,
            };
        let r_idx = self.alloc_i();
        let r_len = self.alloc_i();
        let walk = match reads {
            SeqReads::Chars(var) => SeqWalk::Chars {
                var,
                str_v: self.compile_expr_v(seq_expr),
            },
            SeqReads::Elems(kind) => SeqWalk::Elems(
                kind,
                match &seq_expr.kind {
                    _ if listed.is_some() => SeqHeld::Local(listed.unwrap_or_default()),
                    ExprKind::Global(gid)
                        if !items && matches!(seq_expr.ty, Ty::Array { .. } | Ty::List(_)) =>
                    {
                        SeqHeld::Global(*gid as u16)
                    }
                    _ if items => {
                        let src = self.compile_expr_v(seq_expr);
                        let dst = self.alloc_v();
                        self.emit(Op::SeqItems { dst, src }, seq_expr.span);
                        SeqHeld::Local(dst)
                    }
                    _ => SeqHeld::Local(self.compile_expr_v(seq_expr)),
                },
            ),
        };
        let held = match walk {
            SeqWalk::Elems(_, held) => held,
            SeqWalk::Chars { str_v, .. } => SeqHeld::Local(str_v),
        };
        let local_v = match held {
            SeqHeld::Local(v) => Some(v),
            SeqHeld::Global(_) => None,
        };
        let len = match walk {
            SeqWalk::Elems(SeqKind::Array, SeqHeld::Global(gid)) => Op::GArrLen { dst: r_len, gid },
            SeqWalk::Elems(SeqKind::Array, SeqHeld::Local(arr_v)) => {
                Op::LArrLen { dst: r_len, arr_v }
            }
            SeqWalk::Elems(SeqKind::List, SeqHeld::Global(gid)) => Op::GListLen { dst: r_len, gid },
            SeqWalk::Elems(SeqKind::List, SeqHeld::Local(list_v)) => {
                Op::LListLen { dst: r_len, list_v }
            }
            SeqWalk::Chars { str_v, .. } => Op::LStrLen { dst: r_len, str_v },
        };
        self.emit(len, seq_expr.span);
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
            self.emit_jump(
                JumpOp::JmpLtI {
                    a: r_idx,
                    b: r_zero,
                },
                seq_expr.span,
            )
        } else {
            self.emit(Op::ConstImmI { dst: r_idx, val: 0 }, seq_expr.span);
            self.emit_jump(JumpOp::JmpGeI { a: r_idx, b: r_len }, seq_expr.span)
        };
        let body_pc = self.ops.len() as u32;
        match walk {
            SeqWalk::Elems(kind, held) => {
                self.emit_seq_load(slot_loc, kind, held, r_idx, seq_expr.span)
            }
            SeqWalk::Chars { var, str_v } => {
                self.emit(
                    Op::StrGetI {
                        dst: var,
                        str_v,
                        r_i: r_idx,
                    },
                    seq_expr.span,
                );
            }
        }
        let own = LoopReg::Seq {
            id,
            r_idx,
            r_len,
            held,
        };
        let saved_pinned = self.pinned_v;
        if let Some(lv) = local_v {
            self.pinned_v = self.pinned_v.max(lv + 1);
        }
        let lt = self.loop_body(id, Some(own), &body.stmts);
        self.pinned_v = saved_pinned;
        let step_pc = self.ops.len() as u32;
        self.patch_all(lt.continues, step_pc);
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
        self.patch_all(lt.breaks, end_pc);
        if let Some(lv) = local_v {
            self.free_temp_v(lv, seq_expr.span);
        }
    }
}

/// The places a condition the tree-walker evaluates changes in place, in
/// order, when those are its only writes: calls without `mut` arguments
/// report their own steps. `None` for any other write.
pub(super) fn cond_writes(cond: &Expr) -> Option<Vec<(&Place, Span)>> {
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

/// What a `for` over a sequence reads each step, before it holds the
/// sequence.
#[derive(Clone, Copy)]
enum SeqReads {
    Elems(SeqKind),
    /// The string's chars, into the scalar register `.0`.
    Chars(u16),
}

/// What a `for` over a sequence reads each step, and where it holds it.
#[derive(Clone, Copy)]
enum SeqWalk {
    Elems(SeqKind, SeqHeld),
    Chars { var: u16, str_v: u16 },
}
