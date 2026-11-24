//! `for` and `while` loops: loop heads over domains and products, `else`
//! blocks, `for f in fields(x)` unrolling and `while let` clauses.

use super::*;

impl<'a> Lowerer<'a> {
    /// The loop heads and cell value of `for (r, c) in Square` over a
    /// product domain whose factors the pattern names one by one.
    pub(in crate::lower) fn product_heads(
        &mut self,
        pat: &ast::Pattern,
        iter: &ast::Expr,
    ) -> Option<(Vec<LoopHead>, Expr)> {
        let ast::PatternKind::Tuple(pats) = &pat.kind else {
            return None;
        };
        let factors: Vec<ProductFactor> = match (&iter.kind, self.sym(iter)) {
            (_, Some(Sym::Domain(id))) => self
                .product_domains
                .get(&id)?
                .iter()
                .map(|(_, f)| f.clone())
                .collect(),
            (ast::ExprKind::Tuple(items), _) => items
                .iter()
                .map(|it| self.expr_to_product_factor(it))
                .collect::<Option<_>>()?,
            _ => return None,
        };
        if factors.len() != pats.len() {
            return None;
        }
        let (heads, parts) = self.factor_heads(&factors, iter.span);
        let ty = Ty::Tuple(parts.iter().map(|p| p.ty.clone()).collect());
        Some((heads, self.mk(ExprKind::Tuple(parts), ty, iter.span)))
    }

    /// The loop heads and cell of `for p in Square` or `for p in g.keys` over a
    /// grid: one loop per factor, with `p` built from them.
    pub(in crate::lower) fn product_name_heads(
        &mut self,
        pat: &ast::Pattern,
        iter: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<(Vec<LoopHead>, Expr, Vec<Expr>)> {
        let ast::PatternKind::Name(_) = &pat.kind else {
            return None;
        };
        let id = match (&iter.kind, self.sym(iter)) {
            (_, Some(Sym::Domain(id))) if self.product_domains.contains_key(&id) => id,
            (ast::ExprKind::Field { base, name }, _) if name.text == "keys" => {
                let Some(Ty::Array { dims, .. }) = self.probe_ty_silent(base) else {
                    return None;
                };
                let factors = (dims.len() > 1)
                    .then(|| {
                        dims.iter()
                            .map(|d| match d {
                                Dim::Pos(d) => Some((None, ProductFactor::Domain(*d))),
                                _ => None,
                            })
                            .collect::<Option<Vec<_>>>()
                    })
                    .flatten()?;
                let id = self.intern_anon_product_domain(factors);
                self.def_anon_product_domain(id, iter.span, out);
                id
            }
            _ => return None,
        };
        let factors: Vec<ProductFactor> = self.product_domains[&id]
            .iter()
            .map(|(_, f)| f.clone())
            .collect();
        let (heads, parts) = self.factor_heads(&factors, iter.span);
        let cell = self.encode_product(id, parts.clone(), iter.span);
        Some((heads, cell, parts))
    }

    /// One loop head per factor, and the factor's value from its loop variable.
    fn factor_heads(
        &mut self,
        factors: &[ProductFactor],
        span: Span,
    ) -> (Vec<LoopHead>, Vec<Expr>) {
        let (mut heads, mut parts) = (Vec::new(), Vec::new());
        for f in factors {
            let ty = match f {
                ProductFactor::Domain(d) => Ty::Pos(*d),
                _ => Ty::Int,
            };
            let slot = self.temp(ty.clone());
            let var = self.mk(ExprKind::Local(slot), ty, span);
            parts.push(match f {
                ProductFactor::Domain(_) => var,
                _ => self.product_factor_from_ord(f, var, span),
            });
            heads.push((
                slot,
                self.int_lit(0, span),
                self.product_factor_len(f, span),
            ));
        }
        (heads, parts)
    }

    /// Runs `body` with `pat` bound to `value` inside one loop per head,
    /// outermost first. `break` leaves them all; `continue` steps the
    /// innermost, so together they act as the one loop the user wrote.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::lower) fn nested_for(
        &mut self,
        label: Option<&ast::Name>,
        heads: Vec<LoopHead>,
        pat: &ast::Pattern,
        (value, rinfo): (Expr, Option<RangeInfo>),
        body: &ast::Block,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        let mut ids: Vec<LoopId> = heads[1..].iter().map(|_| self.fresh_loop()).collect();
        let inner = self.open_loop(label.map(|l| l.text.clone()));
        let break_id = ids.first().copied().unwrap_or(inner);
        if let Some(&outer) = ids.first() {
            self.cx.break_out.insert(inner, outer);
        }
        let else_flag = self.begin_loop_else(break_id, else_body, out);
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let mut stmts = Vec::new();
        let mut product_pos = None;
        match (&pat.kind, value.kind) {
            (ast::PatternKind::Tuple(pats), ExprKind::Tuple(parts))
                if pats.len() == parts.len() =>
            {
                for (p, part) in pats.iter().zip(parts) {
                    match (&p.kind, &part.kind) {
                        (ast::PatternKind::Name(name), ExprKind::Local(slot))
                            if heads.iter().any(|h| h.0 == *slot) =>
                        {
                            self.cx.locals[*slot].name = name.clone();
                            self.cx
                                .scopes
                                .last_mut()
                                .expect("a scope")
                                .insert(name.clone(), *slot);
                        }
                        _ => self.bind(p, part, &mut stmts),
                    }
                }
            }
            (_, kind) => {
                let value = Expr { kind, ..value };
                self.bind(pat, value, &mut stmts);
                if let (Some(parts), ast::PatternKind::Name(name)) =
                    (self.cx.loop_parts.take(), &pat.kind)
                {
                    if let Some(&slot) = self.cx.scopes.last().and_then(|s| s.get(name)) {
                        self.cx.product_parts.insert(slot, parts);
                        product_pos = Some(slot);
                    }
                }
                if let Some(rinfo) = &rinfo {
                    self.bind_range_info(pat, rinfo);
                }
            }
        }
        stmts.extend(self.block(body, Tail::Discard).stmts);
        if let Some(slot) = product_pos {
            drop_unread_let(slot, &mut stmts);
        }
        self.cx.local_syms.pop();
        self.cx.scopes.pop();
        self.cx.loops.pop();
        self.cx.break_out.remove(&inner);
        ids.push(inner);
        for ((slot, lo, hi), id) in heads.into_iter().zip(ids).rev() {
            let iter = Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            };
            stmts = vec![Stmt::For(id, slot, iter, Block { stmts })];
        }
        out.extend(stmts);
        self.end_loop_else(break_id, else_flag, else_body, out);
    }

    pub(in crate::lower) fn begin_loop_else(
        &mut self,
        loop_id: LoopId,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let eb = else_body?;
        let flag = self.temp(Ty::Bool);
        let init = self.mk(ExprKind::Bool(true), Ty::Bool, eb.span);
        out.push(Stmt::Let(flag, Some(init)));
        self.cx.loop_else.insert(loop_id, flag);
        Some(flag)
    }

    pub(in crate::lower) fn end_loop_else(
        &mut self,
        loop_id: LoopId,
        flag: Option<Slot>,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        let (Some(flag), Some(eb)) = (flag, else_body) else {
            return;
        };
        self.cx.loop_else.remove(&loop_id);
        let cond = self.local(flag, eb.span);
        let then = self.block(eb, Tail::Discard);
        out.push(Stmt::If(cond, then, Block::default()));
    }

    pub(in crate::lower) fn open_loop(&mut self, label: Option<String>) -> LoopId {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.loops.push((label, id));
        id
    }

    /// The slot a `for` fills each round. A pattern other than a name is
    /// taken apart at the top of the body.
    pub(in crate::lower) fn loop_var(
        &mut self,
        pat: &ast::Pattern,
        elem: Ty,
        body: &mut Vec<Stmt>,
    ) -> Slot {
        match &pat.kind {
            ast::PatternKind::Name(name) => {
                let slot = self.new_slot(name, elem);
                self.cx
                    .scopes
                    .last_mut()
                    .expect("a scope")
                    .insert(name.clone(), slot);
                slot
            }
            ast::PatternKind::Wildcard => self.temp(elem),
            _ => {
                let slot = self.temp(elem);
                let value = self.local(slot, pat.span);
                self.bind(pat, value, body);
                slot
            }
        }
    }

    /// The values of `fields(a, b, ..)` in a `for` head (decision 28.1).
    pub(in crate::lower) fn reflected_values<'e>(
        &self,
        iter: &'e ast::Expr,
    ) -> Option<&'e [ast::Arg]> {
        let ast::ExprKind::Call { callee, args } = &iter.kind else {
            return None;
        };
        let ast::ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        (name == "fields"
            && self.lookup(name).is_none()
            && self.get_sym(name).is_none()
            && !args.is_empty()
            && args.iter().all(|a| a.name.is_none()))
        .then_some(args.as_slice())
    }

    /// `for f in fields(x):`, unrolled: the body once per field of `x`, in
    /// declaration order, with `f` that field. With several values, `f` is
    /// the tuple of their fields at that position.
    pub(in crate::lower) fn fields_loop(
        &mut self,
        pat: &ast::Pattern,
        values: &[ast::Arg],
        body: &ast::Block,
        out: &mut Vec<Stmt>,
    ) {
        let mut temps = Vec::new();
        for a in values {
            let v = self.expr(&a.value, None);
            let ty = v.ty.clone();
            let slot = self.temp(ty.clone());
            out.push(Stmt::Let(slot, Some(v)));
            temps.push((slot, ty, a.value.span));
        }
        if let Ty::Enum(eid) = temps[0].1 {
            if let Some((_, ty, span)) = temps.iter().find(|(_, ty, _)| *ty != temps[0].1) {
                let message = format!(
                    "`fields` walks values of one type together; this is a `{}`",
                    self.show(ty)
                );
                self.error(codes::TYPE_MISMATCH, *span, message);
                return;
            }
            // Each value is the same variant, or the body doesn't run.
            let variants = self.enums[eid].variants.clone();
            for (tag, (_, payload)) in variants.iter().enumerate() {
                let mut stmts = Vec::new();
                for (i, fty) in payload.iter().enumerate() {
                    let mut items: Vec<Expr> = temps
                        .iter()
                        .map(|(slot, _, span)| {
                            let base = self.local(*slot, *span);
                            self.mk(
                                ExprKind::Payload(Box::new(base), tag, i),
                                fty.clone(),
                                *span,
                            )
                        })
                        .collect();
                    let value = if items.len() == 1 {
                        items.pop().unwrap()
                    } else {
                        let ty = Ty::Tuple(vec![fty.clone(); items.len()]);
                        self.mk(ExprKind::Tuple(items), ty, temps[0].2)
                    };
                    self.cx.scopes.push(HashMap::new());
                    self.cx.local_syms.push(HashMap::new());
                    self.bind(pat, value, &mut stmts);
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                    self.cx.local_syms.pop();
                    self.cx.scopes.pop();
                }
                if stmts.is_empty() {
                    continue;
                }
                for (slot, _, span) in temps.iter().rev() {
                    let base = self.local(*slot, *span);
                    let cond = self.mk(ExprKind::IsVariant(Box::new(base), tag), Ty::Bool, *span);
                    stmts = vec![Stmt::If(cond, Block { stmts }, Block { stmts: Vec::new() })];
                }
                out.extend(stmts);
            }
            return;
        }
        // Values of different types walk together when they have as many
        // fields: `fields(ops, a, b)` pairs the i-th op with the i-th parts.
        let mut shapes: Vec<Vec<Ty>> = Vec::new();
        for (_, ty, span) in &temps {
            if *ty == Ty::Error {
                return;
            }
            let Some(tys) = self.field_tys(ty) else {
                let message = format!(
                    "`fields` walks the fields of a struct or a tuple, not of a `{}`",
                    self.show(ty)
                );
                self.error(codes::TYPE_MISMATCH, *span, message);
                return;
            };
            if let Some(first) = shapes.first().filter(|f| f.len() != tys.len()) {
                let message = format!(
                    "`fields` walks values with as many fields together; this has {}, the first {}",
                    tys.len(),
                    first.len()
                );
                self.error(codes::TYPE_MISMATCH, *span, message);
                return;
            }
            shapes.push(tys);
        }
        let field_tys = shapes[0].clone();
        if field_tys.is_empty() {
            let at = out.len() - temps.len();
            let lets: Vec<Stmt> = out.drain(at..).collect();
            for stmt in lets {
                if let Stmt::Let(_, Some(v)) = stmt {
                    let pure = match &v.kind {
                        ExprKind::Unit | ExprKind::Local(_) => true,
                        ExprKind::Tuple(items) => items.is_empty(),
                        _ => false,
                    };
                    if !pure {
                        out.push(Stmt::Expr(v));
                    }
                }
            }
        }
        for i in 0..field_tys.len() {
            let mut items: Vec<Expr> = temps
                .iter()
                .zip(&shapes)
                .map(|((slot, _, span), tys)| {
                    let base = self.local(*slot, *span);
                    self.mk_field(base, i, tys[i].clone(), *span)
                })
                .collect();
            let value = if items.len() == 1 {
                items.pop().unwrap()
            } else {
                let ty = Ty::Tuple(items.iter().map(|e| e.ty.clone()).collect());
                let span = temps[0].2;
                self.mk(ExprKind::Tuple(items), ty, span)
            };
            self.cx.scopes.push(HashMap::new());
            self.cx.local_syms.push(HashMap::new());
            self.bind(pat, value, out);
            out.extend(self.block(body, Tail::Discard).stmts);
            self.cx.local_syms.pop();
            self.cx.scopes.pop();
        }
    }

    /// The clauses of a `while let`, at the top of the loop's body: the
    /// first that fails leaves the loop. False when one could not be read.
    pub(in crate::lower) fn while_clauses(
        &mut self,
        cond: &ast::Cond,
        stmts: &mut Vec<Stmt>,
    ) -> bool {
        let leave_unless = |lw: &mut Self, holds: Expr, stmts: &mut Vec<Stmt>| {
            let span = holds.span;
            let fails = lw.mk_unary(UnOp::Not, holds, Ty::Bool, Span::new(0, 0));
            let fails = Expr { span, ..fails };
            stmts.push(Stmt::If(
                fails,
                Block {
                    stmts: vec![Stmt::Break(None)],
                },
                Block::default(),
            ));
        };
        match cond {
            ast::Cond::Expr(test) => {
                let test = self.expect(test, &Ty::Bool);
                leave_unless(self, test, stmts);
                true
            }
            ast::Cond::Guard { test, and } => {
                let test = self.expect(test, &Ty::Bool);
                leave_unless(self, test, stmts);
                self.while_clauses(and, stmts)
            }
            ast::Cond::Let { pat, value, and } => {
                let mut binds = Vec::new();
                let Some(present) = self.cond_let(pat, value, "while let", stmts, &mut binds)
                else {
                    return false;
                };
                leave_unless(self, present, stmts);
                stmts.extend(binds);
                and.as_ref()
                    .is_none_or(|and| self.while_clauses(and, stmts))
            }
            ast::Cond::Const(c) => {
                self.unsupported(c.span, "`const` in a `while` condition");
                false
            }
        }
    }

    pub(in crate::lower) fn while_stmt(
        &mut self,
        cond: &ast::Cond,
        body: &ast::Block,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.worked_out_here(cond.span, |lw| lw.expect(cond, &Ty::Bool));
                let id = self.open_loop(None);
                let else_flag = self.begin_loop_else(id, else_body, out);
                let body = self.block(body, Tail::Discard);
                self.cx.loops.pop();
                out.push(Stmt::While(id, cond, body));
                self.end_loop_else(id, else_flag, else_body, out);
            }
            ast::Cond::Let { .. } | ast::Cond::Guard { .. } => {
                let id = self.open_loop(None);
                let else_flag = self.begin_loop_else(id, else_body, out);
                self.cx.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                if self.while_clauses(cond, &mut stmts) {
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                }
                self.cx.scopes.pop();
                self.cx.loops.pop();
                let always = self.mk(ExprKind::Bool(true), Ty::Bool, body.span);
                out.push(Stmt::While(id, always, Block { stmts }));
                self.end_loop_else(id, else_flag, else_body, out);
            }
            ast::Cond::Const(cond) => self.unsupported(cond.span, "`while const`"),
        }
    }
}

/// One integer loop: its variable, first value and end.
pub(in crate::lower) type LoopHead = (Slot, Expr, Expr);

/// Drops `let slot = ..` from `stmts` when nothing after it reads `slot`: a
/// product position whose every use read a loop variable instead.
fn drop_unread_let(slot: usize, stmts: &mut Vec<Stmt>) {
    let Some(at) = stmts
        .iter()
        .position(|s| matches!(s, Stmt::Let(s, _) if *s == slot))
    else {
        return;
    };
    let rest = Block {
        stmts: stmts[at + 1..].to_vec(),
    };
    let mut read = false;
    crate::walk::deep_exprs(&rest, &mut |e| {
        read |= matches!(e.kind, ExprKind::Local(s) if s == slot)
            || matches!(e.kind, ExprKind::Closure(_));
    });
    if !read {
        stmts.remove(at);
    }
}

/// `for x in { let a = ..; f(a) }`, as a method view's walk lowers, walks
/// `f(a)` after the lets: a loop over a `yields` call is fed by it instead
/// of walking the list it would build.
pub(in crate::lower) fn hoist_call_args(iter: Iter, out: &mut Vec<Stmt>) -> Iter {
    let Iter::Seq(mut seq, rev) = iter else {
        return iter;
    };
    let ExprKind::Do(block) = &mut seq.kind else {
        return Iter::Seq(seq, rev);
    };
    let lets = block.stmts.len().saturating_sub(1);
    if !block.stmts[..lets]
        .iter()
        .all(|s| matches!(s, Stmt::Let(_, Some(_))))
    {
        return Iter::Seq(seq, rev);
    }
    match block.stmts.pop() {
        Some(Stmt::Give(call)) if matches!(call.kind, ExprKind::Call(..)) => {
            out.append(&mut block.stmts);
            Iter::Seq(call, rev)
        }
        last => {
            block.stmts.extend(last);
            Iter::Seq(seq, rev)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            ty: Ty::Int,
            span: Span::default(),
        }
    }

    fn walk(stmts: Vec<Stmt>) -> Iter {
        Iter::Seq(expr(ExprKind::Do(Block { stmts })), false)
    }

    #[test]
    fn a_walk_ending_in_a_call_after_lets_is_fed_by_the_call() {
        let call = expr(ExprKind::Call(0, Vec::new()));
        let lets = vec![Stmt::Let(0, Some(expr(ExprKind::Int(1))))];
        let mut out = Vec::new();
        let iter = hoist_call_args(
            walk([lets.clone(), vec![Stmt::Give(call.clone())]].concat()),
            &mut out,
        );
        assert_eq!(iter, Iter::Seq(call, false));
        assert_eq!(out, lets);
    }

    #[test]
    fn any_other_walk_is_kept_whole() {
        let call = || Stmt::Give(expr(ExprKind::Call(0, Vec::new())));
        for stmts in [
            vec![],
            vec![Stmt::Give(expr(ExprKind::Int(1)))],
            vec![call(), Stmt::Let(0, Some(expr(ExprKind::Int(1))))],
            vec![Stmt::Let(0, None), call()],
        ] {
            let mut out = Vec::new();
            assert_eq!(hoist_call_args(walk(stmts.clone()), &mut out), walk(stmts));
            assert!(out.is_empty());
        }
    }
}
