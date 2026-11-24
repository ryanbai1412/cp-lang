//! Escapes: where a closure leaves its home, and which captures nothing uses.

use super::*;

impl<'p> Devirt<'p> {
    pub(super) fn find_escapes(&self, home: Option<FnId>, block: &Block, out: &mut Vec<Home>) {
        for stmt in &block.stmts {
            self.find_escapes_stmt(home, stmt, out);
        }
    }

    pub(super) fn find_escapes_stmt(&self, home: Option<FnId>, stmt: &Stmt, out: &mut Vec<Home>) {
        match stmt {
            Stmt::Let(_, init) => {
                if let Some(e) = init {
                    self.find_escapes_expr(home, e, false, out);
                }
            }
            Stmt::Assign(place, _, value) => {
                walk::place_exprs(place, &mut |e| self.find_escapes_expr(home, e, false, out));
                self.find_escapes_expr(home, value, false, out);
            }
            Stmt::Expr(e) | Stmt::Give(e) | Stmt::DefDomainFromVal { val: e, .. } => {
                self.find_escapes_expr(home, e, false, out);
            }
            Stmt::Return(value) => {
                if let Some(e) = value {
                    self.find_escapes_expr(home, e, false, out);
                }
            }
            Stmt::If(cond, then, otherwise) => {
                self.find_escapes_expr(home, cond, false, out);
                self.find_escapes(home, then, out);
                self.find_escapes(home, otherwise, out);
            }
            Stmt::While(_, cond, body) => {
                self.find_escapes_expr(home, cond, false, out);
                self.find_escapes(home, body, out);
            }
            Stmt::For(_, _, iter, body) => {
                match iter {
                    cpl_ir::Iter::Walk { .. } => {
                        unreachable!("Iter::Walk is expanded before codegen")
                    }
                    cpl_ir::Iter::Range { lo, hi, step, .. } => {
                        self.find_escapes_expr(home, lo, false, out);
                        self.find_escapes_expr(home, hi, false, out);
                        if let Some(step) = step {
                            self.find_escapes_expr(home, step, false, out);
                        }
                    }
                    cpl_ir::Iter::RangeVal(e, _) | cpl_ir::Iter::Seq(e, _) => {
                        self.find_escapes_expr(home, e, false, out);
                    }
                    cpl_ir::Iter::TreeRange {
                        container, lo, hi, ..
                    } => {
                        self.find_escapes_expr(home, container, false, out);
                        if let Some((e, _)) = lo {
                            self.find_escapes_expr(home, e, false, out);
                        }
                        if let Some((e, _)) = hi {
                            self.find_escapes_expr(home, e, false, out);
                        }
                    }
                }
                self.find_escapes(home, body, out);
            }
            Stmt::Assert { cond, message, .. } => {
                self.find_escapes_expr(home, cond, false, out);
                if let Some(msg) = message {
                    self.find_escapes_expr(home, msg, false, out);
                }
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.find_escapes_expr(home, lo, false, out);
                self.find_escapes_expr(home, len, false, out);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
        }
    }

    pub(super) fn find_escapes_expr(
        &self,
        home: Option<FnId>,
        e: &Expr,
        elided_ctx: bool,
        out: &mut Vec<Home>,
    ) {
        match &e.kind {
            ExprKind::Local(slot) => {
                if !elided_ctx {
                    if let Some(id) = home {
                        if *slot < self.program.fns[id].params.len() {
                            let h = Home::Param(id, *slot);
                            if self.elidable.contains(&h) {
                                out.push(h);
                            }
                        }
                    }
                }
            }
            ExprKind::Field(base, index) => {
                if !elided_ctx {
                    if let Ty::Struct(s) = base.ty {
                        let h = Home::Field(s, *index);
                        if self.elidable.contains(&h) {
                            out.push(h);
                        }
                    }
                }
                self.find_escapes_expr(home, base, false, out);
            }
            ExprKind::CallExpr(callee, args) => {
                let direct = self.target(home, callee).is_some() && direct_callee(callee);
                self.find_escapes_expr(home, callee, direct, out);
                for arg in args {
                    match arg {
                        Arg::Value(v) => self.find_escapes_expr(home, v, false, out),
                        Arg::Mut(p) => walk::place_exprs(p, &mut |x| {
                            self.find_escapes_expr(home, x, false, out)
                        }),
                    }
                }
            }
            ExprKind::Call(callee, args) => {
                for (slot, arg) in args.iter().enumerate() {
                    match arg {
                        Arg::Value(v) => {
                            let ok = self.elidable.contains(&Home::Param(*callee, slot))
                                && self.skippable_value(v);
                            self.find_escapes_expr(home, v, ok, out);
                        }
                        Arg::Mut(p) => walk::place_exprs(p, &mut |x| {
                            self.find_escapes_expr(home, x, false, out)
                        }),
                    }
                }
            }
            ExprKind::Tuple(items) => {
                let s = match e.ty {
                    Ty::Struct(s) => Some(s),
                    _ => None,
                };
                for (i, item) in items.iter().enumerate() {
                    let ok = s.is_some_and(|s| {
                        self.elidable.contains(&Home::Field(s, i)) && self.skippable_value(item)
                    });
                    self.find_escapes_expr(home, item, ok, out);
                }
            }
            ExprKind::Do(block) => {
                self.find_escapes(home, block, out);
            }
            _ => {
                walk::children(e, &mut |child| {
                    self.find_escapes_expr(home, child, false, out)
                });
            }
        }
    }

    pub(super) fn block_captures_unused(&self, block: &Block, cap_slots: &BTreeSet<usize>) -> bool {
        block
            .stmts
            .iter()
            .all(|s| self.stmt_captures_unused(s, cap_slots))
    }

    pub(super) fn place_captures_unused(
        &self,
        place: &cpl_ir::Place,
        cap_slots: &BTreeSet<usize>,
    ) -> bool {
        match &place.kind {
            PlaceKind::Local(slot) => !cap_slots.contains(slot),
            PlaceKind::Global(_) => true,
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                self.place_captures_unused(base, cap_slots)
            }
            PlaceKind::Index(base, idxs) => {
                self.place_captures_unused(base, cap_slots)
                    && idxs.iter().all(|i| self.expr_captures_unused(i, cap_slots))
            }
        }
    }

    pub(super) fn stmt_captures_unused(&self, stmt: &Stmt, cap_slots: &BTreeSet<usize>) -> bool {
        match stmt {
            Stmt::Let(_, init) => init
                .as_ref()
                .is_none_or(|e| self.expr_captures_unused(e, cap_slots)),
            Stmt::Assign(place, _, value) => {
                self.place_captures_unused(place, cap_slots)
                    && self.expr_captures_unused(value, cap_slots)
            }
            Stmt::Expr(e) | Stmt::Give(e) | Stmt::DefDomainFromVal { val: e, .. } => {
                self.expr_captures_unused(e, cap_slots)
            }
            Stmt::Return(value) => value
                .as_ref()
                .is_none_or(|e| self.expr_captures_unused(e, cap_slots)),
            Stmt::If(cond, then, otherwise) => {
                self.expr_captures_unused(cond, cap_slots)
                    && self.block_captures_unused(then, cap_slots)
                    && self.block_captures_unused(otherwise, cap_slots)
            }
            Stmt::While(_, cond, body) => {
                self.expr_captures_unused(cond, cap_slots)
                    && self.block_captures_unused(body, cap_slots)
            }
            Stmt::For(_, _, iter, body) => {
                let iter_ok = match iter {
                    cpl_ir::Iter::Walk { .. } => {
                        unreachable!("Iter::Walk is expanded before codegen")
                    }
                    cpl_ir::Iter::Range { lo, hi, step, .. } => {
                        self.expr_captures_unused(lo, cap_slots)
                            && self.expr_captures_unused(hi, cap_slots)
                            && step
                                .as_ref()
                                .is_none_or(|s| self.expr_captures_unused(s, cap_slots))
                    }
                    cpl_ir::Iter::RangeVal(e, _) | cpl_ir::Iter::Seq(e, _) => {
                        self.expr_captures_unused(e, cap_slots)
                    }
                    cpl_ir::Iter::TreeRange {
                        container, lo, hi, ..
                    } => {
                        self.expr_captures_unused(container, cap_slots)
                            && lo
                                .as_ref()
                                .is_none_or(|(e, _)| self.expr_captures_unused(e, cap_slots))
                            && hi
                                .as_ref()
                                .is_none_or(|(e, _)| self.expr_captures_unused(e, cap_slots))
                    }
                };
                iter_ok && self.block_captures_unused(body, cap_slots)
            }
            Stmt::Assert { cond, message, .. } => {
                self.expr_captures_unused(cond, cap_slots)
                    && message
                        .as_ref()
                        .is_none_or(|m| self.expr_captures_unused(m, cap_slots))
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.expr_captures_unused(lo, cap_slots)
                    && self.expr_captures_unused(len, cap_slots)
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => true,
        }
    }

    pub(super) fn expr_captures_unused(&self, e: &Expr, cap_slots: &BTreeSet<usize>) -> bool {
        match &e.kind {
            ExprKind::Local(slot) => !cap_slots.contains(slot),
            ExprKind::CallExpr(callee, args) => {
                let callee_ok = self.target(None, callee).is_some_and(|_| skippable(callee))
                    || self.expr_captures_unused(callee, cap_slots);
                callee_ok
                    && args.iter().all(|a| match a {
                        Arg::Value(v) => self.expr_captures_unused(v, cap_slots),
                        Arg::Mut(p) => self.place_captures_unused(p, cap_slots),
                    })
            }
            ExprKind::Closure(inner_cid) => {
                let inner = &self.program.closures[*inner_cid];
                !inner
                    .captures
                    .iter()
                    .any(|c| !c.is_global && cap_slots.contains(&c.outer))
            }
            ExprKind::Do(b) => self.block_captures_unused(b, cap_slots),
            ExprKind::Call(_, args) => args.iter().all(|a| match a {
                Arg::Value(v) => self.expr_captures_unused(v, cap_slots),
                Arg::Mut(p) => self.place_captures_unused(p, cap_slots),
            }),
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _)
                if !self.place_captures_unused(place, cap_slots) =>
            {
                false
            }
            _ => {
                let mut ok = true;
                walk::children(e, &mut |child| {
                    if ok && !self.expr_captures_unused(child, cap_slots) {
                        ok = false;
                    }
                });
                ok
            }
        }
    }
}
