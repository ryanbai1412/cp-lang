use super::ir::*;
use crate::value::Value;
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, CmpOp, Expr, ExprKind, FnId, Iter, Local, LoopId, Place,
    PlaceKind, Program, Slot, Stmt, Ty, UnOp,
};

mod apply;
mod assign;
mod assign_v;
mod call_mix;
mod expr_i;
mod expr_opt_i;
mod expr_v;
mod fed_args;
mod index_nd;
mod range_loop;
mod seq_load;
mod sort;
mod stmt;
mod traced_take;

struct LoopTargets {
    id: LoopId,
    breaks: Vec<usize>,
    continues: Vec<usize>,
    /// How a checkpoint reads this loop, when it can.
    resumable: Option<LoopReg>,
}

enum GiveTarget {
    I {
        dst: u16,
        exits: Vec<usize>,
    },
    OptI {
        dst_val: u16,
        dst_some: u16,
        exits: Vec<usize>,
    },
    V {
        dst: u16,
        exits: Vec<usize>,
    },
    Discard {
        exits: Vec<usize>,
    },
}

impl GiveTarget {
    pub(super) fn exits_mut(&mut self) -> &mut Vec<usize> {
        match self {
            GiveTarget::I { exits, .. }
            | GiveTarget::OptI { exits, .. }
            | GiveTarget::V { exits, .. }
            | GiveTarget::Discard { exits } => exits,
        }
    }
}

#[derive(Clone, Copy)]
enum ModSuperOp {
    Add(u32),
    Sub(u32),
    Mul(u32),
    Neg(u32),
    New(u32),
}

pub(super) struct Compiler<'p, 'f> {
    program: &'p Program,
    feeds: &'f cpl_ir::pushed::Feeds,
    fed_ok: &'f [bool],
    ops: Vec<Op>,
    spans: Vec<Span>,
    consts_i: Vec<i64>,
    range_texts: Vec<&'p str>,
    places: Vec<PlacePath>,
    closure_specs: Vec<ClosureMakeSpec>,
    /// The slots a closure made here captures by reference; `ref_all` when a
    /// closure the tree-walker makes may capture any of them.
    ref_slots: Vec<u16>,
    ref_all: bool,
    feed_calls: Vec<FeedCallSpec>,
    fed_bodies: Vec<(LoopId, u32)>,
    consts_v: Vec<Value>,
    prints: Vec<PrintSpec<'p>>,
    formats: Vec<FormatSpec<'p>>,
    applies: Vec<ApplySpec<'p>>,
    applies_sync: bool,
    fallback_stmts: Vec<&'p Stmt>,
    fails: Vec<&'p Stmt>,
    fallback_exprs: Vec<&'p Expr>,
    pub(super) lent_calls: Vec<&'p [Arg]>,
    fill_tys: Vec<&'p Ty>,
    locals: &'p [Local],
    slot_map: Vec<SlotLoc>,
    next_i: u16,
    next_v: u16,
    num_local_v: u16,
    pinned_v: u16,
    loops: Vec<LoopTargets>,
    give_targets: Vec<GiveTarget>,
    cur_fn: Option<FnId>,
    pub(in crate::vm) feed_acc: Option<usize>,
    pub(in crate::vm) allow_fed_loops: bool,
    inline_slots: Option<Vec<SlotLoc>>,
    inline_ret: Option<(u16, Vec<usize>)>,
    /// Registers known to hold a position of a domain, so a `FromOrd` of
    /// one needs no check: the register, the domain, and the local it holds
    /// if any. Cleared wherever a domain could be resized (a call, a
    /// fallback) and at each loop, whose back edge could bring a resize.
    in_dom: Vec<(u16, u16, Option<Slot>)>,
    /// How many ops so far could resize a domain, and the locals assigned
    /// so far: a range loop over `0..len(d)` whose body adds to neither
    /// keeps its variable a position of `d`.
    dom_clears: u32,
    assigned_locals: Vec<Slot>,
    pub(in crate::vm) traced: bool,
    trace_points: Vec<TracePoint<'p>>,
    /// The `for` whose body is compiled next, whose variable each round reports.
    traced_loop_var: Option<(Slot, Span)>,
    /// Whether a `take` compiled next reports its place, as it runs.
    trace_takes: bool,
    back_edges: Vec<BackEdge>,
    call_sites: Vec<CallSite<'p>>,
    keeps: Vec<(u32, Option<SlotLoc>, usize)>,
}

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn new(
        program: &'p Program,
        feeds: &'f cpl_ir::pushed::Feeds,
        fed_ok: &'f [bool],
        locals: &'p [Local],
        cur_fn: Option<FnId>,
        param_count: usize,
    ) -> Self {
        let mut slot_map = Vec::with_capacity(locals.len());
        let mut next_i: u16 = 0;
        let mut next_v: u16 = 0;
        for (idx, local) in locals.iter().enumerate() {
            if let Some(kind) = ScalarKind::of_ty(&local.ty, program) {
                slot_map.push(SlotLoc::I(next_i, kind));
                next_i += 1;
            } else if let Some(kind) = ScalarKind::of_opt_ty(&local.ty, program) {
                let val = next_i;
                let some = next_i + 1;
                next_i += 2;
                slot_map.push(SlotLoc::OptI { val, some, kind });
            } else if idx >= param_count {
                if let Some((len, kinds)) = small_tuple_shape(&local.ty, program) {
                    let base = next_i;
                    next_i += len as u16;
                    slot_map.push(SlotLoc::SmallTuple { base, len, kinds });
                } else {
                    slot_map.push(SlotLoc::V(next_v));
                    next_v += 1;
                }
            } else {
                slot_map.push(SlotLoc::V(next_v));
                next_v += 1;
            }
        }
        Self {
            program,
            feeds,
            fed_ok,
            ops: Vec::new(),
            spans: Vec::new(),
            consts_i: Vec::new(),
            range_texts: Vec::new(),
            places: Vec::new(),
            closure_specs: Vec::new(),
            ref_slots: Vec::new(),
            ref_all: false,
            feed_calls: Vec::new(),
            fed_bodies: Vec::new(),
            consts_v: Vec::new(),
            prints: Vec::new(),
            formats: Vec::new(),
            applies: Vec::new(),
            applies_sync: false,
            fallback_stmts: Vec::new(),
            fails: Vec::new(),
            fallback_exprs: Vec::new(),
            lent_calls: Vec::new(),
            fill_tys: Vec::new(),
            locals,
            slot_map,
            next_i,
            next_v,
            num_local_v: next_v,
            pinned_v: next_v,
            loops: Vec::new(),
            give_targets: Vec::new(),
            cur_fn,
            feed_acc: None,
            allow_fed_loops: true,
            inline_slots: None,
            inline_ret: None,
            in_dom: Vec::new(),
            dom_clears: 0,
            assigned_locals: Vec::new(),
            traced: false,
            trace_points: Vec::new(),
            traced_loop_var: None,
            trace_takes: false,
            back_edges: Vec::new(),
            call_sites: Vec::new(),
            keeps: Vec::new(),
        }
    }

    pub(super) fn finish(self) -> Chunk<'p> {
        let needs_stack = !self.fallback_stmts.is_empty()
            || !self.fallback_exprs.is_empty()
            || self.applies_sync
            || !self.trace_points.is_empty()
            || (self.ops.iter()).any(|op| matches!(op, Op::CallTraced { .. }));
        Chunk {
            ops: self.ops,
            spans: self.spans,
            consts_i: self.consts_i,
            range_texts: self.range_texts,
            places: self.places,
            closure_specs: self.closure_specs,
            feed_calls: self.feed_calls,
            fed_bodies: self.fed_bodies,
            consts_v: self.consts_v,
            prints: self.prints,
            formats: self.formats,
            applies: self.applies,
            fallback_stmts: self.fallback_stmts,
            fails: self.fails,
            fallback_exprs: self.fallback_exprs,
            lent_calls: self.lent_calls,
            trace_points: self.trace_points,
            traced: self.traced,
            fill_tys: self.fill_tys,
            num_i: self.next_i as usize,
            num_v: self.next_v as usize,
            num_local_v: self.num_local_v as usize,
            slot_map: self.slot_map,
            param_count: 0,
            ret_kind: None,
            guarded_domains: Vec::new(),
            needs_stack,
            has_by_ref_closure: self.ref_all || !self.ref_slots.is_empty(),
            ref_slots: (!self.ref_all).then(|| {
                let mut slots = self.ref_slots;
                slots.sort_unstable();
                slots.dedup();
                slots
            }),
            back_edges: self.back_edges,
            call_sites: self.call_sites,
            keeps: {
                let mut keeps = self.keeps;
                keeps.sort_by_key(|k| k.0);
                keeps.dedup_by_key(|k| k.0);
                keeps
            },
        }
    }

    /// The loops around the code compiled next, then `own`, when a
    /// checkpoint can read every one of them.
    fn resumable_loops(&self, own: Option<Option<LoopReg>>) -> Option<Vec<LoopReg>> {
        if self.inline_slots.is_some() {
            return None;
        }
        (self.loops.iter().map(|l| l.resumable))
            .chain(own)
            .collect()
    }

    /// Notes the jump back at `pc` of a loop that `own` reads, when a
    /// checkpoint can read it and every loop around it.
    pub(super) fn note_back_edge(&mut self, pc: usize, own: Option<LoopReg>) {
        if let Some(loops) = self.resumable_loops(Some(own)) {
            self.back_edges.push(BackEdge {
                pc: pc as u32,
                loops,
            });
        }
    }

    /// Notes each call `stmt` just compiled, from `start`, that a run from a
    /// checkpoint inside the callee can go on from: one `stmt` evaluates
    /// before any other effect, but maybe after other calls. Such a run
    /// evaluates `stmt` again up to the call, takes what the calls before it
    /// returned, and goes on with what the callee returns. When `stmt` runs
    /// on the tree-walker, every call in it waits at its fallback op.
    pub(super) fn note_call_site(&mut self, start: usize, stmt: &'p Stmt) {
        let head = match stmt {
            Stmt::Let(_, Some(e)) | Stmt::Expr(e) | Stmt::Return(Some(e)) => e,
            Stmt::If(e, ..) | Stmt::While(_, e, _) => e,
            // A traced run's loop fed by a `yields` fn: the tree-walker runs it.
            Stmt::For(_, _, Iter::Seq(e, _), _) if matches!(e.kind, ExprKind::Call(..)) => {
                let falls_back = (start..self.ops.len()).any(|pc| {
                    matches!(self.ops[pc], Op::FallbackStmt { idx, .. }
                        if std::ptr::eq(self.fallback_stmts[idx as usize], stmt))
                });
                if !falls_back {
                    return;
                }
                e
            }
            Stmt::Assign(place, _, e) => {
                let mut effects = Effects::default();
                cpl_ir::walk::place_exprs(place, &mut |i| effects.walk(i));
                if effects.count > 0 {
                    return;
                }
                e
            }
            _ => return,
        };
        let mut order = Order::default();
        order.walk(head);
        let Some(loops) = self.resumable_loops(None) else {
            return;
        };
        let fallback = (start..self.ops.len()).find(|&pc| {
            matches!(self.ops[pc], Op::FallbackStmt { idx, .. }
                if std::ptr::eq(self.fallback_stmts[idx as usize], stmt))
        });
        let mut before: Vec<&'p Expr> = Vec::new();
        let mut last = None;
        for call in order.calls {
            let Some(call) = call.filter(|c| c.span != Span::default()) else {
                break;
            };
            // A call the VM makes, or one a statement of a `do` in `stmt`
            // makes on the tree-walker, which waits at that fallback op.
            let at = (self.call_op(start, call).map(|pc| (pc, true)))
                .or_else(|| self.inner_fallback(start, call).map(|pc| (pc, false)));
            let (pc, native) = match (fallback, at) {
                (Some(pc), _) => (pc as u32, false),
                (None, Some((pc, native)))
                    if last.is_none_or(|last| last < pc || (!native && last == pc)) =>
                {
                    (pc, native)
                }
                _ => break,
            };
            // A statement inside `stmt` that noted this call first is one a
            // run from a checkpoint cannot find: `stmt` is.
            self.call_sites
                .retain(|c| !(c.pc == pc && std::ptr::eq(c.call, call)));
            self.call_sites.push(CallSite {
                pc,
                loops: loops.clone(),
                stmt,
                call,
                before: before.clone(),
            });
            if native {
                let Some(result) = self.call_result(pc, call) else {
                    break;
                };
                self.keeps.push((pc, result, call as *const Expr as usize));
            }
            last = Some(pc);
            before.push(call);
        }
    }

    /// The fallback op from `start` on whose statement makes `call`.
    fn inner_fallback(&self, start: usize, call: &Expr) -> Option<u32> {
        let pc = (start..self.ops.len()).find(|&pc| {
            let mut found = false;
            let mut look = |e: &'p Expr| {
                cpl_ir::walk::deep_expr(e, &mut |inner| found |= std::ptr::eq(inner, call))
            };
            match self.ops[pc] {
                Op::FallbackStmt { idx, .. } => {
                    cpl_ir::walk::stmt_exprs(self.fallback_stmts[idx as usize], &mut look)
                }
                Op::FallbackExprI { idx, .. } | Op::FallbackExprV { idx, .. } => {
                    look(self.fallback_exprs[idx as usize])
                }
                _ => {}
            }
            found
        })?;
        Some(pc as u32)
    }

    /// Notes the call op at `pc` of the `yields` fn that feeds the loop
    /// `stmt`: a run from a checkpoint inside the fn makes the loop again
    /// and goes on in the fn.
    pub(super) fn note_fed_site(&mut self, pc: usize, stmt: &'p Stmt, call: &'p Expr) {
        let mut order = Order::default();
        order.walk(call);
        if !matches!(order.calls[..], [Some(only)] if std::ptr::eq(only, call)) {
            return;
        }
        let Some(loops) = self.resumable_loops(None) else {
            return;
        };
        self.call_sites.push(CallSite {
            pc: pc as u32,
            loops,
            stmt,
            call,
            before: Vec::new(),
        });
    }

    /// The one call op from `start` on that `call` compiled to.
    fn call_op(&self, start: usize, call: &Expr) -> Option<u32> {
        let mut calls = (start..self.ops.len()).filter(|&pc| {
            self.spans[pc] == call.span
                && matches!(
                    self.ops[pc],
                    Op::CallVmI { .. }
                        | Op::CallVmMix { .. }
                        | Op::CallTraced { .. }
                        | Op::CallClosure { .. }
                )
        });
        match (calls.next(), calls.next()) {
            (Some(pc), None) => Some(pc as u32),
            _ => None,
        }
    }

    /// Where the call op at `pc` puts what it returns (`None` for `Unit`).
    fn call_result(&self, pc: u32, call: &Expr) -> Option<Option<SlotLoc>> {
        let ret = |fn_id: u16| &self.program.fns[usize::from(fn_id)].ret;
        Some(match self.ops[pc as usize] {
            Op::CallVmI { fn_id, dst, .. } => Some(SlotLoc::I(
                dst,
                ScalarKind::of_ty(ret(fn_id), self.program)?,
            )),
            Op::CallTraced {
                fn_id: u16::MAX, ..
            } => return None,
            Op::CallTraced {
                fn_id,
                dst_i,
                dst_v,
                ..
            } => match ret(fn_id) {
                Ty::Unit => None,
                ty if dst_i != u16::MAX => {
                    Some(SlotLoc::I(dst_i, ScalarKind::of_ty(ty, self.program)?))
                }
                _ => Some(SlotLoc::V(dst_v)),
            },
            Op::CallVmMix {
                fn_id,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
                ..
            } => match ret_mode {
                CallRet::Unit => None,
                CallRet::I => Some(SlotLoc::I(
                    dst_i,
                    ScalarKind::of_ty(ret(fn_id), self.program)?,
                )),
                CallRet::OptI => Some(SlotLoc::OptI {
                    val: dst_i,
                    some: dst_some,
                    kind: ScalarKind::of_opt_ty(ret(fn_id), self.program)?,
                }),
                CallRet::V => Some(SlotLoc::V(dst_v)),
            },
            Op::CallClosure {
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
                ..
            } => match ret_mode {
                CallRet::Unit => None,
                CallRet::I => Some(SlotLoc::I(
                    dst_i,
                    ScalarKind::of_ty(&call.ty, self.program)?,
                )),
                CallRet::OptI => Some(SlotLoc::OptI {
                    val: dst_i,
                    some: dst_some,
                    kind: ScalarKind::of_opt_ty(&call.ty, self.program)?,
                }),
                CallRet::V => Some(SlotLoc::V(dst_v)),
            },
            _ => return None,
        })
    }

    pub(super) fn const_bool(e: &Expr) -> Option<bool> {
        match &e.kind {
            ExprKind::Bool(b) => Some(*b),
            ExprKind::Unary(UnOp::Not, a) => Self::const_bool(a).map(|b| !b),
            ExprKind::Binary(BinOp::And, a, b) => {
                Some(Self::const_bool(a)? && Self::const_bool(b)?)
            }
            ExprKind::Binary(BinOp::Or, a, b) => Some(Self::const_bool(a)? || Self::const_bool(b)?),
            ExprKind::Cmp(op, a, b) => {
                if let (ExprKind::Int(av), ExprKind::Int(bv)) = (&a.kind, &b.kind) {
                    Some(match op {
                        CmpOp::Eq => av == bv,
                        CmpOp::Ne => av != bv,
                        CmpOp::Lt => av < bv,
                        CmpOp::Le => av <= bv,
                        CmpOp::Gt => av > bv,
                        CmpOp::Ge => av >= bv,
                        _ => return None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(super) fn mut_fn_args_alias_free(&self, fn_id: FnId, args: &'p [Arg]) -> bool {
        let f = &self.program.fns[fn_id];
        fn place_root_global(p: &Place) -> Option<cpl_ir::GlobalId> {
            match &p.kind {
                PlaceKind::Global(g) => Some(*g),
                PlaceKind::Local(_) => None,
                PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    place_root_global(base)
                }
            }
        }
        fn place_may_touch_globals(p: &Place, globals: &[cpl_ir::GlobalId]) -> bool {
            match &p.kind {
                PlaceKind::Global(g) => globals.contains(g),
                PlaceKind::Local(_) => false,
                PlaceKind::Index(base, idxs) => {
                    place_may_touch_globals(base, globals)
                        || idxs.iter().any(|e| expr_may_touch_globals(e, globals))
                }
                PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    place_may_touch_globals(base, globals)
                }
            }
        }
        fn expr_may_touch_globals(e: &Expr, globals: &[cpl_ir::GlobalId]) -> bool {
            match &e.kind {
                ExprKind::Unit
                | ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Str(_)
                | ExprKind::None
                | ExprKind::Local(_)
                | ExprKind::DomainVal(_)
                | ExprKind::DomainLen(_)
                | ExprKind::DomainLo(_)
                | ExprKind::Error => false,
                ExprKind::Global(g) => globals.contains(g),
                ExprKind::Some(x)
                | ExprKind::Unary(_, x)
                | ExprKind::Cast(_, x)
                | ExprKind::Field(x, _)
                | ExprKind::IsVariant(x, _)
                | ExprKind::Payload(x, _, _)
                | ExprKind::IsSome(x)
                | ExprKind::Unwrap(x, _) => expr_may_touch_globals(x, globals),
                ExprKind::Binary(_, a, b)
                | ExprKind::Cmp(_, a, b)
                | ExprKind::Range { lo: a, hi: b } => {
                    expr_may_touch_globals(a, globals) || expr_may_touch_globals(b, globals)
                }
                ExprKind::RangeCheck { value, lo, hi, .. } => {
                    expr_may_touch_globals(value, globals)
                        || lo
                            .as_deref()
                            .is_some_and(|x| expr_may_touch_globals(x, globals))
                        || hi
                            .as_deref()
                            .is_some_and(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::If(c, t, f) => {
                    expr_may_touch_globals(c, globals)
                        || expr_may_touch_globals(t, globals)
                        || expr_may_touch_globals(f, globals)
                }
                ExprKind::Index(b, idxs) => {
                    expr_may_touch_globals(b, globals)
                        || idxs.iter().any(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::Tuple(xs) | ExprKind::Variant(_, xs) | ExprKind::Builtin(_, xs) => {
                    xs.iter().any(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::Do(blk) => block_may_touch_globals(blk, globals),
                ExprKind::Call(..)
                | ExprKind::CallExpr(..)
                | ExprKind::Closure(..)
                | ExprKind::Mutate(..)
                | ExprKind::Sort(..)
                | ExprKind::Read(..)
                | ExprKind::Parse(..)
                | ExprKind::Print(..)
                | ExprKind::Debug(..)
                | ExprKind::Format(..)
                | ExprKind::Fill(..)
                | ExprKind::MutSlice(..) => true,
            }
        }
        fn block_may_touch_globals(b: &Block, globals: &[cpl_ir::GlobalId]) -> bool {
            b.stmts.iter().any(|s| match s {
                Stmt::Let(_, init) => init
                    .as_ref()
                    .is_some_and(|e| expr_may_touch_globals(e, globals)),
                Stmt::Assign(place, _, rhs) => {
                    place_may_touch_globals(place, globals) || expr_may_touch_globals(rhs, globals)
                }
                Stmt::Expr(e) | Stmt::Give(e) => expr_may_touch_globals(e, globals),
                Stmt::Return(e) => e
                    .as_ref()
                    .is_some_and(|e| expr_may_touch_globals(e, globals)),
                Stmt::If(c, t, f) => {
                    expr_may_touch_globals(c, globals)
                        || block_may_touch_globals(t, globals)
                        || block_may_touch_globals(f, globals)
                }
                Stmt::While(_, c, body) => {
                    expr_may_touch_globals(c, globals) || block_may_touch_globals(body, globals)
                }
                Stmt::Assert { cond, message, .. } => {
                    expr_may_touch_globals(cond, globals)
                        || message
                            .as_ref()
                            .is_some_and(|e| expr_may_touch_globals(e, globals))
                }
                Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => false,
                Stmt::For(..) | Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. } => true,
            })
        }

        let mut mut_globals = Vec::new();
        for arg in args {
            if let Arg::Mut(place) = arg {
                if let Some(g) = place_root_global(place) {
                    mut_globals.push(g);
                }
            }
        }
        for (i, arg) in args.iter().enumerate() {
            if !matches!(f.locals[i].ty, Ty::Fn(..)) {
                continue;
            }
            let Arg::Value(ae) = arg else {
                return false;
            };
            let ExprKind::Closure(cid) = &ae.kind else {
                return false;
            };
            let cdef = &self.program.closures[*cid];
            if cdef.captures.iter().any(|c| c.by_ref) {
                return false;
            }
            if !mut_globals.is_empty() && block_may_touch_globals(&cdef.body, &mut_globals) {
                return false;
            }
        }
        true
    }

    pub(super) fn try_compile_call_closure(
        &mut self,
        callee: &'p Expr,
        args: &'p [Arg],
        ret_mode: CallRet,
        span: Span,
    ) -> Option<(u16, u16, u16)> {
        let Ty::Fn(params, _) = &callee.ty else {
            return None;
        };
        if args.len() != params.len() || !args.iter().all(|a| matches!(a, Arg::Value(_))) {
            return None;
        }
        let mut ni = 0usize;
        let mut nv = 0usize;
        for ty in params {
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
        let callee_v = self.compile_expr_v(callee);
        let mut i_args = Vec::with_capacity(ni);
        let mut v_args = Vec::with_capacity(nv);
        for (ty, a) in params.iter().zip(args.iter()) {
            let Arg::Value(ae) = a else { unreachable!() };
            if ScalarKind::of_ty(ty, self.program).is_some() {
                i_args.push(self.compile_expr_i(ae));
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                let (v, s) = self.compile_expr_opt_i(ae);
                i_args.push(v);
                i_args.push(s);
            } else {
                v_args.push(self.compile_expr_v(ae));
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
        for r in v_args {
            let slot = self.alloc_v();
            if slot != r {
                self.emit(Op::MoveV { dst: slot, src: r }, span);
            }
        }
        let (dst_i, dst_v, dst_some) = match ret_mode {
            CallRet::Unit => (0, 0, 0),
            CallRet::I => (self.alloc_i(), 0, 0),
            CallRet::OptI => (self.alloc_i(), 0, self.alloc_i()),
            CallRet::V => (0, self.alloc_v(), 0),
        };
        self.emit(
            Op::CallClosure {
                callee_v,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            },
            span,
        );
        Some((dst_i, dst_v, dst_some))
    }

    pub(super) fn alloc_i(&mut self) -> u16 {
        let r = self.next_i;
        self.next_i += 1;
        r
    }

    pub(super) fn alloc_v(&mut self) -> u16 {
        let r = self.next_v;
        self.next_v += 1;
        r
    }

    pub(super) fn slot_loc(&self, slot: Slot) -> SlotLoc {
        if let Some(map) = &self.inline_slots {
            map[slot]
        } else {
            self.slot_map[slot]
        }
    }

    pub(in crate::vm) fn emit(&mut self, op: Op, span: Span) -> usize {
        if matches!(
            op,
            Op::CallVmI { .. }
                | Op::CallVmMix { .. }
                | Op::CallTraced { .. }
                | Op::CallClosure { .. }
                | Op::CallVmFeed { .. }
                | Op::FeedYieldI { .. }
                | Op::FeedYieldV { .. }
                | Op::FeedYieldExtend { .. }
                | Op::FeedTree { .. }
                | Op::FallbackStmt { .. }
                | Op::FallbackExprI { .. }
                | Op::FallbackExprV { .. }
                | Op::ApplyV { sync: true, .. }
                | Op::DefDomain { .. }
                | Op::DefDomainFromVal { .. }
        ) {
            self.in_dom.clear();
            self.dom_clears += 1;
        }
        let pc = self.ops.len();
        self.ops.push(op);
        self.spans.push(span);
        pc
    }

    pub(super) fn patch(&mut self, pc: usize, target: u32) {
        match &mut self.ops[pc] {
            Op::Jmp { target: t }
            | Op::JmpEscape { target: t, .. }
            | Op::JmpIfZero { target: t, .. }
            | Op::JmpIfNonZero { target: t, .. }
            | Op::JmpLtI { target: t, .. }
            | Op::JmpLeI { target: t, .. }
            | Op::JmpGtI { target: t, .. }
            | Op::JmpGeI { target: t, .. }
            | Op::JmpEqI { target: t, .. }
            | Op::JmpNeI { target: t, .. }
            | Op::ForInc { target: t, .. }
            | Op::ForIncClosed { target: t, .. }
            | Op::ForStepEnter { skip: t, .. }
            | Op::ForStepNext { target: t, .. }
            | Op::ForDec { target: t, .. } => *t = target,
            _ => unreachable!("patch on non-jump op"),
        }
    }

    pub(super) fn emit_const_i(&mut self, dst: u16, val: i64, span: Span) {
        if let Ok(imm) = i32::try_from(val) {
            self.emit(Op::ConstImmI { dst, val: imm }, span);
        } else {
            let idx = self.consts_i.len() as u32;
            self.consts_i.push(val);
            self.emit(Op::ConstI { dst, idx }, span);
        }
    }

    fn recognize_mod_op(program: &Program, fn_id: FnId) -> Option<ModSuperOp> {
        let f = &program.fns[fn_id];
        if f.params.iter().any(|p| p.mutable) || f.body.stmts.len() != 1 {
            return None;
        }
        let ret_expr = match &f.body.stmts[0] {
            Stmt::Return(Some(e)) | Stmt::Give(e) => e,
            _ => return None,
        };
        let inner = match &ret_expr.kind {
            ExprKind::Tuple(fs) if fs.len() == 1 => &fs[0],
            _ => ret_expr,
        };
        let is_param_field = |e: &Expr, p: Slot| -> bool {
            match &e.kind {
                ExprKind::Field(base, 0) => matches!(base.kind, ExprKind::Local(s) if s == p),
                ExprKind::Local(s) => *s == p,
                _ => false,
            }
        };
        if f.params.len() == 1 {
            if let ExprKind::Binary(BinOp::Rem, a, m_e) = &inner.kind {
                if let ExprKind::Int(m) = m_e.kind {
                    if m > 0 && m <= u32::MAX as i64 && is_param_field(a, 0) {
                        return Some(ModSuperOp::New(m as u32));
                    }
                }
            }
        }
        let ExprKind::If(cond, then_e, else_e) = &inner.kind else {
            return None;
        };
        if f.params.len() == 2 {
            if let (
                ExprKind::Cmp(CmpOp::Ge, lhs, rhs),
                ExprKind::Binary(BinOp::Sub, t_lhs, t_rhs),
                ExprKind::Binary(BinOp::Add, e_a, e_b),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (
                    ExprKind::Binary(BinOp::Add, c_a, c_b),
                    ExprKind::Int(m1),
                    ExprKind::Binary(BinOp::Add, t_a, t_b),
                    ExprKind::Int(m2),
                ) = (&lhs.kind, &rhs.kind, &t_lhs.kind, &t_rhs.kind)
                {
                    if m1 == m2
                        && *m1 > 0
                        && *m1 <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(c_b, 1)
                        && is_param_field(t_a, 0)
                        && is_param_field(t_b, 1)
                        && is_param_field(e_a, 0)
                        && is_param_field(e_b, 1)
                    {
                        return Some(ModSuperOp::Add(*m1 as u32));
                    }
                }
            }
            if let (
                ExprKind::Cmp(CmpOp::Ge, c_a, c_b),
                ExprKind::Binary(BinOp::Sub, t_a, t_b),
                ExprKind::Binary(BinOp::Add, e_lhs, e_rhs),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (ExprKind::Binary(BinOp::Sub, e_a, e_b), ExprKind::Int(m)) =
                    (&e_lhs.kind, &e_rhs.kind)
                {
                    if *m > 0
                        && *m <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(c_b, 1)
                        && is_param_field(t_a, 0)
                        && is_param_field(t_b, 1)
                        && is_param_field(e_a, 0)
                        && is_param_field(e_b, 1)
                    {
                        return Some(ModSuperOp::Sub(*m as u32));
                    }
                }
            }
            if Self::const_bool(cond) == Some(true) {
                if let ExprKind::Binary(BinOp::Rem, mul_e, mod_e) = &then_e.kind {
                    if let (ExprKind::Binary(BinOp::Mul, m_a, m_b), ExprKind::Int(m)) =
                        (&mul_e.kind, &mod_e.kind)
                    {
                        if *m > 0
                            && *m <= 3_037_000_499
                            && is_param_field(m_a, 0)
                            && is_param_field(m_b, 1)
                        {
                            return Some(ModSuperOp::Mul(*m as u32));
                        }
                    }
                }
            }
        } else if f.params.len() == 1 {
            if let (
                ExprKind::Cmp(CmpOp::Eq, c_a, c_z),
                ExprKind::Int(0),
                ExprKind::Binary(BinOp::Sub, e_m, e_a),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (ExprKind::Int(0), ExprKind::Int(m)) = (&c_z.kind, &e_m.kind) {
                    if *m > 0
                        && *m <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(e_a, 0)
                    {
                        return Some(ModSuperOp::Neg(*m as u32));
                    }
                }
            }
        }
        None
    }

    pub(super) fn is_pure_leaf_fn(&self, fn_id: FnId) -> bool {
        if self.cur_fn == Some(fn_id)
            || self.inline_slots.is_some()
            || self.program.judge_fns.contains(&fn_id)
        {
            return false;
        }
        let f = &self.program.fns[fn_id];
        if f.params.iter().any(|p| p.mutable) {
            return false;
        }
        if f.locals.len() > 24 || f.body.stmts.len() > 12 {
            return false;
        }
        if !self.program.fn_guarded_domains(&f.ret, &f.body).is_empty() {
            return false;
        }
        if ScalarKind::of_ty(&f.ret, self.program).is_none() {
            return false;
        }
        if !f
            .locals
            .iter()
            .skip(f.params.len())
            .all(|l| ScalarKind::of_ty(&l.ty, self.program).is_some())
        {
            return false;
        }
        fn simple_block(b: &Block, f: &cpl_ir::Function, program: &Program) -> bool {
            b.stmts.iter().all(|s| simple_stmt(s, f, program))
        }
        fn simple_stmt(s: &Stmt, f: &cpl_ir::Function, program: &Program) -> bool {
            match s {
                Stmt::Let(slot, e) => {
                    ScalarKind::of_ty(&f.locals[*slot].ty, program).is_some()
                        && e.as_ref().is_none_or(|e| simple_expr(e, f, program))
                }
                Stmt::Assign(p, _, e) => {
                    matches!(p.kind, PlaceKind::Local(slot) if ScalarKind::of_ty(&f.locals[slot].ty, program).is_some())
                        && simple_expr(e, f, program)
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Expr(e) => simple_expr(e, f, program),
                Stmt::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b {
                            simple_block(t, f, program)
                        } else {
                            simple_block(e, f, program)
                        }
                    } else {
                        simple_expr(c, f, program)
                            && simple_block(t, f, program)
                            && simple_block(e, f, program)
                    }
                }
                _ => false,
            }
        }
        fn simple_v_base(e: &Expr, f: &cpl_ir::Function) -> bool {
            match &e.kind {
                ExprKind::Global(_) => true,
                ExprKind::Local(slot) => *slot < f.params.len(),
                ExprKind::Field(inner, _) => simple_v_base(inner, f),
                _ => false,
            }
        }
        fn simple_expr(e: &Expr, f: &cpl_ir::Function, program: &Program) -> bool {
            if ScalarKind::of_ty(&e.ty, program).is_none() {
                return false;
            }
            match &e.kind {
                ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Local(_)
                | ExprKind::Global(_) => true,
                ExprKind::Unary(_, a) | ExprKind::Cast(_, a) => simple_expr(a, f, program),
                ExprKind::Field(a, _) => simple_expr(a, f, program) || simple_v_base(a, f),
                ExprKind::Binary(op, a, b) => {
                    let safe_op = match op {
                        BinOp::Div | BinOp::Rem => matches!(b.kind, ExprKind::Int(v) if v != 0),
                        BinOp::Shl | BinOp::Shr => {
                            matches!(b.kind, ExprKind::Int(v) if (0..64).contains(&v))
                        }
                        _ => true,
                    };
                    safe_op && simple_expr(a, f, program) && simple_expr(b, f, program)
                }
                ExprKind::Cmp(_, a, b) => simple_expr(a, f, program) && simple_expr(b, f, program),
                ExprKind::Tuple(fs) => fs.len() == 1 && simple_expr(&fs[0], f, program),
                ExprKind::Index(base, indices) => {
                    let container_ok = match &base.ty {
                        Ty::Array { dims, .. } => {
                            (indices.len() == 1 || indices.len() == 2)
                                && dims.len() == indices.len()
                        }
                        Ty::List(_) => indices.len() == 1,
                        _ => false,
                    };
                    container_ok
                        && simple_v_base(base, f)
                        && indices.iter().all(|i| simple_expr(i, f, program))
                }
                ExprKind::Call(callee, args) => {
                    Compiler::recognize_mod_op(program, *callee).is_some()
                        && args
                            .iter()
                            .all(|a| matches!(a, Arg::Value(e) if simple_expr(e, f, program)))
                }
                ExprKind::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b {
                            simple_expr(t, f, program)
                        } else {
                            simple_expr(e, f, program)
                        }
                    } else {
                        simple_expr(c, f, program)
                            && simple_expr(t, f, program)
                            && simple_expr(e, f, program)
                    }
                }
                ExprKind::Do(b) => simple_block(b, f, program),
                _ => false,
            }
        }
        simple_block(&f.body, f, self.program)
    }

    pub(super) fn stmt_mentions_slot(stmt: &Stmt, target: Slot) -> bool {
        match stmt {
            Stmt::Let(s, init) => {
                *s == target
                    || init
                        .as_ref()
                        .is_some_and(|e| Self::expr_mentions_slot(e, target))
            }
            Stmt::Assign(p, _, e) => {
                Self::place_mentions_slot(p, target) || Self::expr_mentions_slot(e, target)
            }
            Stmt::Expr(e) | Stmt::Give(e) => Self::expr_mentions_slot(e, target),
            Stmt::Return(e) => e
                .as_ref()
                .is_some_and(|e| Self::expr_mentions_slot(e, target)),
            Stmt::If(c, t, e) => {
                Self::expr_mentions_slot(c, target)
                    || t.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
                    || e.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::While(_, c, b) => {
                Self::expr_mentions_slot(c, target)
                    || b.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::For(_, var, iter, body) => {
                *var == target
                    || Self::iter_mentions_slot(iter, target)
                    || body
                        .stmts
                        .iter()
                        .any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::Assert { cond, message, .. } => {
                Self::expr_mentions_slot(cond, target)
                    || message
                        .as_ref()
                        .is_some_and(|m| Self::expr_mentions_slot(m, target))
            }
            Stmt::DefDomain { lo, len, .. } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(len, target)
            }
            Stmt::DefDomainFromVal { val, .. } => Self::expr_mentions_slot(val, target),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(_, _) => false,
        }
    }

    pub(super) fn iter_mentions_slot(iter: &Iter, target: Slot) -> bool {
        match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range { lo, hi, .. } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(hi, target)
            }
            Iter::RangeVal(e, _) | Iter::Seq(e, _) => Self::expr_mentions_slot(e, target),
            Iter::TreeRange {
                container, lo, hi, ..
            } => {
                Self::expr_mentions_slot(container, target)
                    || lo
                        .as_ref()
                        .is_some_and(|(e, _)| Self::expr_mentions_slot(e, target))
                    || hi
                        .as_ref()
                        .is_some_and(|(e, _)| Self::expr_mentions_slot(e, target))
            }
        }
    }

    pub(super) fn place_mentions_slot(p: &Place, target: Slot) -> bool {
        match &p.kind {
            PlaceKind::Local(s) => *s == target,
            PlaceKind::Global(_) => false,
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                Self::place_mentions_slot(base, target)
            }
            PlaceKind::Index(base, idxs) => {
                Self::place_mentions_slot(base, target)
                    || idxs.iter().any(|i| Self::expr_mentions_slot(i, target))
            }
        }
    }

    pub(super) fn expr_mentions_slot(e: &Expr, target: Slot) -> bool {
        match &e.kind {
            ExprKind::Local(s) => *s == target,
            ExprKind::Closure(_) => true,
            ExprKind::Unary(_, a)
            | ExprKind::Cast(_, a)
            | ExprKind::Field(a, _)
            | ExprKind::Some(a)
            | ExprKind::IsSome(a)
            | ExprKind::IsVariant(a, _)
            | ExprKind::Payload(a, _, _)
            | ExprKind::Unwrap(a, _) => Self::expr_mentions_slot(a, target),
            ExprKind::Binary(_, a, b) | ExprKind::Cmp(_, a, b) => {
                Self::expr_mentions_slot(a, target) || Self::expr_mentions_slot(b, target)
            }
            ExprKind::Range { lo, hi } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(hi, target)
            }
            ExprKind::RangeCheck { value, lo, hi, .. } => {
                Self::expr_mentions_slot(value, target)
                    || lo
                        .as_ref()
                        .is_some_and(|e| Self::expr_mentions_slot(e, target))
                    || hi
                        .as_ref()
                        .is_some_and(|e| Self::expr_mentions_slot(e, target))
            }
            ExprKind::If(c, t, f) => {
                Self::expr_mentions_slot(c, target)
                    || Self::expr_mentions_slot(t, target)
                    || Self::expr_mentions_slot(f, target)
            }
            ExprKind::Index(b, idxs) => {
                Self::expr_mentions_slot(b, target)
                    || idxs.iter().any(|i| Self::expr_mentions_slot(i, target))
            }
            ExprKind::Tuple(es) | ExprKind::Variant(_, es) | ExprKind::Builtin(_, es) => {
                es.iter().any(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Call(_, args) => args.iter().any(|a| match a {
                Arg::Value(x) => Self::expr_mentions_slot(x, target),
                Arg::Mut(p) => Self::place_mentions_slot(p, target),
            }),
            ExprKind::CallExpr(f, args) => {
                Self::expr_mentions_slot(f, target)
                    || args.iter().any(|a| match a {
                        Arg::Value(x) => Self::expr_mentions_slot(x, target),
                        Arg::Mut(p) => Self::place_mentions_slot(p, target),
                    })
            }
            ExprKind::MutSlice(p, lo, hi) => {
                Self::place_mentions_slot(p, target)
                    || Self::expr_mentions_slot(lo, target)
                    || Self::expr_mentions_slot(hi, target)
            }
            ExprKind::Mutate(_, p, args) => {
                Self::place_mentions_slot(p, target)
                    || args.iter().any(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Sort(p, _, desc) => {
                Self::place_mentions_slot(p, target)
                    || desc
                        .as_ref()
                        .is_some_and(|x| Self::expr_mentions_slot(x, target))
                    || true
            }
            ExprKind::Do(b) => b.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target)),
            ExprKind::Fill(_, fill) => fill
                .as_ref()
                .is_some_and(|x| Self::expr_mentions_slot(x, target)),
            ExprKind::Print(args, end) => {
                args.iter()
                    .any(|a| Self::expr_mentions_slot(&a.value, target))
                    || end
                        .as_ref()
                        .is_some_and(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Debug(_, args) => args
                .iter()
                .any(|a| Self::expr_mentions_slot(&a.value, target)),
            ExprKind::Format(pieces) => pieces.iter().any(|p| match p {
                cpl_ir::FmtPiece::Text(_) => false,
                cpl_ir::FmtPiece::Value(a) => Self::expr_mentions_slot(&a.value, target),
                cpl_ir::FmtPiece::Number(e, _) => Self::expr_mentions_slot(e, target),
            }),
            ExprKind::Read(_) | ExprKind::Parse(..) => true,
            ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::Str(_)
            | ExprKind::None
            | ExprKind::Global(_)
            | ExprKind::DomainVal(_)
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_)
            | ExprKind::Error => false,
        }
    }

    pub(in crate::vm) fn stmts_have_closure(stmts: &[Stmt]) -> bool {
        stmts
            .iter()
            .any(|s| Self::stmt_mentions_slot(s, usize::MAX))
    }

    pub(super) fn expr_pure_eq(a: &Expr, b: &Expr) -> bool {
        match (&a.kind, &b.kind) {
            (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
            (ExprKind::Bool(x), ExprKind::Bool(y)) => x == y,
            (ExprKind::Local(x), ExprKind::Local(y)) => x == y,
            (ExprKind::Global(x), ExprKind::Global(y)) => x == y,
            (ExprKind::Binary(op1, l1, r1), ExprKind::Binary(op2, l2, r2)) => {
                op1 == op2 && Self::expr_pure_eq(l1, l2) && Self::expr_pure_eq(r1, r2)
            }
            (ExprKind::Unary(op1, x1), ExprKind::Unary(op2, x2)) => {
                op1 == op2 && Self::expr_pure_eq(x1, x2)
            }
            (ExprKind::Field(e1, f1), ExprKind::Field(e2, f2)) => {
                f1 == f2 && Self::expr_pure_eq(e1, e2)
            }
            (ExprKind::Cast(c1, e1), ExprKind::Cast(c2, e2)) => {
                c1 == c2 && Self::expr_pure_eq(e1, e2)
            }
            _ => false,
        }
    }

    pub(super) fn extract_arr_place_parts(place: &Place) -> Option<(&Place, &Expr, Option<&Expr>)> {
        let PlaceKind::Index(base, indices) = &place.kind else {
            return None;
        };
        if matches!(base.ty, Ty::Array { .. }) {
            if indices.len() == 2 {
                return Some((base, &indices[0], Some(&indices[1])));
            }
            if indices.len() == 1 {
                if let PlaceKind::Index(outer, outer_idxs) = &base.kind {
                    if matches!(outer.ty, Ty::Array { .. }) && outer_idxs.len() == 1 {
                        return Some((outer, &outer_idxs[0], Some(&indices[0])));
                    }
                }
                return Some((base, &indices[0], None));
            }
        }
        None
    }

    pub(super) fn extract_arr_expr_parts(expr: &Expr) -> Option<(&Expr, &Expr, Option<&Expr>)> {
        let ExprKind::Index(base, indices) = &expr.kind else {
            return None;
        };
        if matches!(base.ty, Ty::Array { .. }) {
            if indices.len() == 2 {
                return Some((base, &indices[0], Some(&indices[1])));
            }
            if indices.len() == 1 {
                if let ExprKind::Index(outer, outer_idxs) = &base.kind {
                    if matches!(outer.ty, Ty::Array { .. }) && outer_idxs.len() == 1 {
                        return Some((outer, &outer_idxs[0], Some(&indices[0])));
                    }
                }
                return Some((base, &indices[0], None));
            }
        }
        None
    }

    pub(super) fn expr_matches_arr_place(expr: &Expr, place: &Place) -> bool {
        let (Some((eb, ei0, ei1)), Some((pb, pi0, pi1))) = (
            Self::extract_arr_expr_parts(expr),
            Self::extract_arr_place_parts(place),
        ) else {
            return false;
        };
        let base_ok = match (&eb.kind, &pb.kind) {
            (ExprKind::Global(g1), PlaceKind::Global(g2)) => g1 == g2,
            (ExprKind::Local(l1), PlaceKind::Local(l2)) => l1 == l2,
            _ => false,
        };
        if !base_ok || !Self::expr_pure_eq(ei0, pi0) {
            return false;
        }
        match (ei1, pi1) {
            (Some(a), Some(b)) => Self::expr_pure_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }

    pub(super) fn try_compile_opt_minmax_pair(
        &mut self,
        s0: &'p Stmt,
        s1: &'p Stmt,
        rest: &'p [Stmt],
    ) -> bool {
        let Stmt::Let(r_slot, Some(rhs)) = s0 else {
            return false;
        };
        let Stmt::If(outer_cond, outer_then, outer_else) = s1 else {
            return false;
        };
        if !outer_else.stmts.is_empty() || outer_then.stmts.len() != 1 {
            return false;
        }
        let ExprKind::IsSome(c_inner) = &outer_cond.kind else {
            return false;
        };
        if !matches!(c_inner.kind, ExprKind::Local(s) if s == *r_slot) {
            return false;
        }
        let Stmt::If(inner_cond, inner_then, inner_else) = &outer_then.stmts[0] else {
            return false;
        };
        if !inner_else.stmts.is_empty() || inner_then.stmts.len() != 1 {
            return false;
        }
        let Stmt::Assign(place, AssignOp::Set, set_val) = &inner_then.stmts[0] else {
            return false;
        };
        let ExprKind::Some(set_inner) = &set_val.kind else {
            return false;
        };
        if !matches!(&set_inner.kind, ExprKind::Unwrap(u, false) if matches!(u.kind, ExprKind::Local(s) if s == *r_slot))
        {
            return false;
        }
        let ExprKind::Binary(BinOp::Or, p_none, better) = &inner_cond.kind else {
            return false;
        };
        let ExprKind::Unary(UnOp::Not, p_some) = &p_none.kind else {
            return false;
        };
        let ExprKind::IsSome(p_expr1) = &p_some.kind else {
            return false;
        };
        let ExprKind::Cmp(cmp_op @ (CmpOp::Lt | CmpOp::Gt), cmp_lhs, cmp_rhs) = &better.kind else {
            return false;
        };
        if !matches!(&cmp_lhs.kind, ExprKind::Unwrap(u, false) if matches!(u.kind, ExprKind::Local(s) if s == *r_slot))
        {
            return false;
        }
        let ExprKind::Unwrap(p_expr2, false) = &cmp_rhs.kind else {
            return false;
        };
        if !Self::expr_matches_arr_place(p_expr1, place)
            || !Self::expr_matches_arr_place(p_expr2, place)
        {
            return false;
        }
        if ScalarKind::of_opt_ty(&place.ty, self.program) != Some(ScalarKind::Int) {
            return false;
        }
        let Some((base, idx0, idx1)) = Self::extract_arr_place_parts(place) else {
            return false;
        };
        if rest.iter().any(|s| Self::stmt_mentions_slot(s, *r_slot)) {
            return false;
        }
        let (root, is_global) = match &base.kind {
            PlaceKind::Global(gid) => (*gid as u16, true),
            PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::V(v) => (v, false),
                _ => return false,
            },
            _ => return false,
        };
        let is_2d = idx1.is_some();
        let is_max = *cmp_op == CmpOp::Gt;
        let (r_val, r_some) = self.compile_expr_opt_i(rhs);
        let j_skip = self.emit(
            Op::JmpIfZero {
                reg: r_some,
                target: 0,
            },
            place.span,
        );
        let r_i = self.compile_expr_i(idx0);
        let r_j = match idx1 {
            Some(e) => self.compile_expr_i(e),
            None => 0,
        };
        self.emit(
            Op::MinMaxArrOptI {
                root,
                is_global,
                is_2d,
                is_max,
                r_i,
                r_j,
                r_val,
            },
            place.span,
        );
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        true
    }
}

/// The calls an expression makes, in the order it makes them, with `None`
/// for any other effect, which a call after it cannot be gone on from.
#[derive(Default)]
struct Order<'p> {
    calls: Vec<Option<&'p Expr>>,
}

impl<'p> Order<'p> {
    fn walk(&mut self, e: &'p Expr) {
        match &e.kind {
            // What these evaluate more than once, or runs code no call op
            // shows, is one effect.
            ExprKind::Fill(..) | ExprKind::Sort(..) => self.calls.push(None),
            // What `print` and the like lower to: values bound, then used.
            ExprKind::Do(block) => {
                for stmt in &block.stmts {
                    match stmt {
                        Stmt::Let(_, None) => {}
                        Stmt::Let(_, Some(e)) | Stmt::Expr(e) | Stmt::Give(e) => self.walk(e),
                        _ => return self.calls.push(None),
                    }
                }
            }
            _ => {
                cpl_ir::walk::children(e, &mut |c| self.walk(c));
                let mut effects = Effects::default();
                effects.walk_one(e);
                match &e.kind {
                    ExprKind::Call(..) | ExprKind::CallExpr(..) => self.calls.push(Some(e)),
                    _ if effects.count > 0 => self.calls.push(None),
                    _ => {}
                }
            }
        }
    }
}

/// The calls and other effects in an expression: what evaluating it again
/// could do twice.
#[derive(Default)]
struct Effects<'p> {
    count: usize,
    call: Option<&'p Expr>,
}

impl<'p> Effects<'p> {
    /// `e` itself, not what is inside it.
    fn walk_one(&mut self, e: &'p Expr) {
        match &e.kind {
            ExprKind::Call(..) => {
                self.count += 1;
                self.call = Some(e);
            }
            ExprKind::CallExpr(..)
            | ExprKind::Read(_)
            | ExprKind::Parse(..)
            | ExprKind::Print(..)
            | ExprKind::Mutate(..)
            | ExprKind::Sort(..)
            | ExprKind::MutSlice(..)
            | ExprKind::Debug(..)
            | ExprKind::Do(_)
            | ExprKind::Builtin(
                cpl_ir::Builtin::Rand
                | cpl_ir::Builtin::Flush
                | cpl_ir::Builtin::CountQuery
                | cpl_ir::Builtin::SwapInput,
                _,
            ) => self.count += 1,
            _ => {}
        }
    }

    fn walk(&mut self, e: &'p Expr) {
        self.walk_one(e);
        cpl_ir::walk::children(e, &mut |c| self.walk(c));
    }
}
