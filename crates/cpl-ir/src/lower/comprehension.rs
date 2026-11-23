//! Comprehensions and tables: the loops they open, what their parts
//! hoist, and the values they build.

use super::*;

/// One `for` of a comprehension while its body is being lowered.
pub(super) struct LoopFrame {
    id: LoopId,
    slot: Slot,
    iter: Iter,
    /// What working out `iter` hoisted, run just before the loop.
    head: Vec<Stmt>,
    /// Statements that take the loop variable's pattern apart.
    pre: Vec<Stmt>,
    /// The `if`s after the loop, in order: each is the statements that
    /// work out its test, the test, and what it binds when it holds.
    guards: Vec<(Vec<Stmt>, Expr, Vec<Stmt>)>,
    /// On the outermost loop: what the enclosing statement had hoisted
    /// before the loops opened. What the loops' parts hoist runs inside
    /// them, where their variables are; `close_loops` puts it
    /// back.
    outside: Vec<Stmt>,
}

impl Lowerer<'_> {
    /// Opens the loops of a comprehension. Its body is lowered next, with
    /// the loop variables in scope, and [`Self::close_loops`] wraps it.
    pub(super) fn open_loops(&mut self, clauses: &[ast::Clause]) -> Vec<LoopFrame> {
        let mut frames = Vec::new();
        let mut outside = Some(std::mem::take(&mut self.cx.pending));
        for clause in clauses {
            let items = self.items_head(&clause.iter);
            let map_proj = if items.is_some() {
                None
            } else {
                self.map_proj_head(&clause.iter)
            };
            let (iter, elem) = match (&items, &map_proj) {
                (Some(items), _) => (items.range(), Ty::Unit),
                (None, Some(proj)) => (proj.iter(), Ty::Unit),
                (None, None) => self.iter(&clause.iter),
            };
            let head = std::mem::take(&mut self.cx.pending);
            let id = self.next_loop;
            self.next_loop += 1;
            self.cx.scopes.push(HashMap::new());
            let mut pre = Vec::new();
            let slot = match (&items, &map_proj) {
                (Some(items), _) => self.items_bind(&clause.pat, items, &mut pre),
                (None, Some(proj)) => self.map_proj_bind(&clause.pat, proj, &mut pre),
                (None, None) => self.loop_var(&clause.pat, elem, &mut pre),
            };
            pre.append(&mut self.cx.pending);
            let mut guards = Vec::new();
            for cond in &clause.conds {
                self.guard(cond, &mut guards);
            }
            frames.push(LoopFrame {
                id,
                slot,
                iter,
                head,
                pre,
                guards,
                outside: outside.take().unwrap_or_default(),
            });
        }
        if let Some(outside) = outside {
            self.cx.pending = outside;
        }
        frames
    }

    /// One `if` of a comprehension: `if c`, `if let x = opt`, and what
    /// follows its `and`. A name it binds is in scope for the later `if`s
    /// and for the body.
    pub(super) fn guard(
        &mut self,
        cond: &ast::Cond,
        guards: &mut Vec<(Vec<Stmt>, Expr, Vec<Stmt>)>,
    ) {
        match cond {
            ast::Cond::Expr(c) => {
                let test = self.expect(c, &Ty::Bool);
                guards.push((std::mem::take(&mut self.cx.pending), test, Vec::new()));
            }
            ast::Cond::Let { pat, value, and } => {
                let mut before = Vec::new();
                let mut bound = Vec::new();
                let Some(present) = self.cond_let(pat, value, "if let", &mut before, &mut bound)
                else {
                    return;
                };
                before.splice(0..0, std::mem::take(&mut self.cx.pending));
                guards.push((before, present, bound));
                if let Some(and) = and {
                    self.guard(and, guards);
                }
            }
            ast::Cond::Guard { test, and } => {
                let test = self.expect(test, &Ty::Bool);
                guards.push((std::mem::take(&mut self.cx.pending), test, Vec::new()));
                self.guard(and, guards);
            }
            ast::Cond::Const(c) => self.unsupported(c.span, "`if const` in a comprehension"),
        }
    }

    /// One loop over the values of `seq`, shaped like a comprehension's.
    pub(super) fn open_seq(&mut self, seq: Expr, elem: Ty) -> (Vec<LoopFrame>, Expr) {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.scopes.push(HashMap::new());
        let span = seq.span;
        let slot = self.temp(elem.clone());
        let value = Expr {
            kind: ExprKind::Local(slot),
            ty: elem,
            span,
        };
        let iter = if matches!(seq.ty, Ty::Range(_)) {
            Iter::RangeVal(seq, false)
        } else {
            Iter::Seq(seq, false)
        };
        let frame = LoopFrame {
            id,
            slot,
            iter,
            head: Vec::new(),
            pre: Vec::new(),
            guards: Vec::new(),
            outside: std::mem::take(&mut self.cx.pending),
        };
        (vec![frame], value)
    }

    /// Wraps `inner` in the loops; gives the statements and the id of the
    /// outermost loop, for an early exit.
    pub(super) fn close_loops(
        &mut self,
        frames: Vec<LoopFrame>,
        inner: Vec<Stmt>,
    ) -> (Vec<Stmt>, LoopId) {
        let outer = frames.first().map_or(0, |f| f.id);
        let mut stmts = std::mem::take(&mut self.cx.pending);
        stmts.extend(inner);
        for frame in frames.into_iter().rev() {
            self.cx.scopes.pop();
            let mut body = frame.pre;
            for (before, test, mut bound) in frame.guards.into_iter().rev() {
                bound.extend(stmts);
                stmts = before;
                stmts.push(Stmt::If(test, Block { stmts: bound }, Block::default()));
            }
            body.extend(stmts);
            stmts = frame.head;
            stmts.push(Stmt::For(
                frame.id,
                frame.slot,
                frame.iter,
                Block { stmts: body },
            ));
            self.cx.pending.extend(frame.outside);
        }
        (stmts, outer)
    }

    /// `[body for x in xs if cond]`: a list, or a set where one is wanted.
    pub(super) fn comprehension(
        &mut self,
        body: &ast::Expr,
        clauses: &[ast::Clause],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if let Some(filled) = self.comprehension_filled(body, clauses, hint, span) {
            return filled;
        }
        let frames = self.open_loops(clauses);
        let (elem_hint, is_set) = match hint {
            Some(Ty::List(elem)) => (Some((**elem).clone()), false),
            Some(Ty::Set(elem)) => (Some((**elem).clone()), true),
            _ => (None, false),
        };
        let value = match &elem_hint {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        let ty = if is_set {
            Ty::Set(Box::new(value.ty.clone()))
        } else {
            Ty::list(value.ty.clone())
        };
        let acc = self.temp(ty.clone());
        let at = self.span(span);
        let place = Place {
            kind: PlaceKind::Local(acc),
            ty: ty.clone(),
            span: at,
        };
        let op = if is_set {
            Mutation::Insert
        } else {
            Mutation::Push
        };
        let result_ty = if is_set { Ty::Bool } else { Ty::Unit };
        let add = Expr {
            kind: ExprKind::Mutate(op, Box::new(place), vec![value]),
            ty: result_ty,
            span: at,
        };
        let (loops, _) = self.close_loops(frames, vec![Stmt::Expr(add)]);
        let mut stmts = vec![Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
        )];
        stmts.extend(loops);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    /// `[List<V>() for _ in lo..hi]`: the body names no loop variable and
    /// has no effects, so every element is the same fresh value and the list
    /// is `List.filled(max(hi - lo, 0), body)` instead of a push loop.
    pub(super) fn comprehension_filled(
        &mut self,
        body: &ast::Expr,
        clauses: &[ast::Clause],
        hint: Option<&Ty>,
        span: Span,
    ) -> Option<Expr> {
        let [clause] = clauses else { return None };
        if !clause.conds.is_empty() || !matches!(clause.pat.kind, ast::PatternKind::Wildcard) {
            return None;
        }
        let ast::ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive: false,
            step: None,
        } = &clause.iter.kind
        else {
            return None;
        };
        if matches!(hint, Some(Ty::Set(_))) || !fresh_constant(body) {
            return None;
        }
        let lo = self.expect(lo, &Ty::Int);
        let hi = self.expect(hi, &Ty::Int);
        let value = match hint {
            Some(Ty::List(elem)) if !matches!(**elem, Ty::Error) => {
                let elem = (**elem).clone();
                self.expect(body, &elem)
            }
            _ => self.expr(body, None),
        };
        let len = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(hi), Box::new(lo)),
            Ty::Int,
            span,
        );
        let zero = self.int_lit(0, span);
        let count = self.mk(
            ExprKind::Builtin(Builtin::Max, vec![len, zero]),
            Ty::Int,
            span,
        );
        let ty = Ty::list(value.ty.clone());
        Some(self.mk(
            ExprKind::Builtin(Builtin::ListFilled, vec![count, value]),
            ty,
            span,
        ))
    }

    /// `[deg[u] * 2 for u in V]` where a `[V]T` is declared builds the
    /// array, like the table `[V] u => deg[u] * 2`.
    pub(super) fn comprehension_table(
        &mut self,
        init: &ast::Expr,
        dims: &[DimSpec],
        elem: &Ty,
        elem_range: Option<&RangeBounds>,
    ) -> Option<Expr> {
        let ast::ExprKind::Comprehension { body, clauses } = &init.kind else {
            return None;
        };
        let ([clause], [DimSpec::Pos(dim)]) = (clauses.as_slice(), dims) else {
            return None;
        };
        let ast::ExprKind::Name(domain) = &clause.iter.kind else {
            return None;
        };
        if !clause.conds.is_empty() || self.lookup(domain).is_some() {
            return None;
        }
        let dom_matches = match self.gen_env.get(domain) {
            Some(GenArg::Domain(id)) => id == dim,
            _ => matches!(self.get_sym(domain), Some(Sym::Domain(id)) if id == *dim),
        };
        if !dom_matches {
            return None;
        }
        let pats = std::slice::from_ref(&clause.pat);
        Some(self.build_table(
            dims.to_vec(),
            pats,
            body,
            Some(elem),
            elem_range,
            init.span,
            None,
        ))
    }

    pub(super) fn table(
        &mut self,
        domains: &[ast::Type],
        params: &[ast::Pattern],
        body: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let dims: Option<Vec<DimSpec>> = domains.iter().map(|d| self.dim(d, true)).collect();
        let Some(dims) = dims else {
            return self.err_expr(span);
        };
        let elem = match hint {
            Some(Ty::Array { elem, .. }) => Some((**elem).clone()),
            _ => None,
        };
        // `[P] p => f(p)` where an `[A, B]T` is wanted fills the grid.
        if let (Some(Ty::Array { dims: want, .. }), [DimSpec::Pos(p)], [_]) =
            (hint, dims.as_slice(), params)
        {
            let p = *p;
            if self.product_grid(p).as_ref() == Some(want) {
                let grid = want
                    .iter()
                    .map(|d| match d {
                        Dim::Pos(d) => DimSpec::Pos(*d),
                        _ => unreachable!("a product grid has position dimensions"),
                    })
                    .collect();
                return self.build_table(grid, params, body, elem.as_ref(), None, span, Some(p));
            }
        }
        self.build_table(dims, params, body, elem.as_ref(), None, span, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_table(
        &mut self,
        dims: Vec<DimSpec>,
        params: &[ast::Pattern],
        body: &ast::Expr,
        elem: Option<&Ty>,
        elem_range: Option<&RangeBounds>,
        span: Span,
        whole: Option<DomainId>,
    ) -> Expr {
        let named = if whole.is_some() { 1 } else { dims.len() };
        if params.len() != named {
            let message = format!(
                "this table runs over {} domains but names {} positions",
                dims.len(),
                params.len()
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return self.err_expr(span);
        }
        let at = self.span(span);
        let mut stmts = Vec::new();
        // Ensure any `impl Index` struct dimensions have their backing domain initialized (`let K = ...`).
        for dim in &dims {
            if let DimSpec::Pos(dom_id) = dim {
                if let Some((sid, sc)) = self.structs.iter().enumerate().find_map(|(sid, s)| {
                    if s.index_domain == Some(*dom_id) {
                        s.consts.get("K").cloned().map(|sc| (sid, sc))
                    } else {
                        None
                    }
                }) {
                    let k_val = self.eval_struct_const(sid, sc, span);
                    let lo = self.int_lit(0, span);
                    stmts.push(Stmt::DefDomain {
                        id: *dom_id,
                        lo,
                        len: k_val,
                    });
                }
            }
        }
        // A length is used to allocate and again to loop: hold it.
        let dims: Vec<DimSpec> = dims
            .into_iter()
            .map(|dim| match dim {
                DimSpec::Int(len) => DimSpec::Int(Box::new(self.hold(*len, &mut stmts))),
                DimSpec::IntFrom(lo, len) => {
                    DimSpec::IntFrom(lo, Box::new(self.hold(*len, &mut stmts)))
                }
                other => other,
            })
            .collect();
        self.cx.scopes.push(HashMap::new());
        let mut loops = Vec::new();
        let mut indices = Vec::new();
        for (i, dim) in dims.iter().enumerate() {
            let (ty, lo, hi) = self.table_axis(dim, span);
            let mut pre = Vec::new();
            let slot = match whole {
                Some(_) => self.temp(ty.clone()),
                None => self.loop_var(&params[i], ty.clone(), &mut pre),
            };
            let id = self.next_loop;
            self.next_loop += 1;
            let iter = Iter::Range {
                lo: lo.clone(),
                hi,
                step: None,
                rev: false,
                closed: false,
            };
            let index = Expr {
                kind: ExprKind::Local(slot),
                ty,
                span: at,
            };
            indices.push(self.table_cell(dim, index, lo));
            loops.push((id, slot, iter, pre));
        }
        if let Some(p) = whole {
            let parts = loops
                .iter()
                .zip(&dims)
                .map(|((_, slot, _, _), dim)| {
                    let DimSpec::Pos(d) = dim else {
                        unreachable!("a product grid has position dimensions")
                    };
                    self.mk(ExprKind::Local(*slot), Ty::Pos(*d), span)
                })
                .collect();
            let encoded = self.encode_product(p, parts, span);
            let mut pre = Vec::new();
            let slot = self.loop_var(&params[0], Ty::Pos(p), &mut pre);
            if let Some((_, _, _, inner)) = loops.last_mut() {
                inner.push(Stmt::Let(slot, Some(encoded)));
                inner.extend(pre);
            }
        }
        // What the body hoists runs in the loops, where its positions are.
        let outside = std::mem::take(&mut self.cx.pending);
        let value = match elem {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        let value = match elem_range {
            Some(bounds) => self.check_range(value, bounds, body.span),
            None => value,
        };
        let hoisted = std::mem::replace(&mut self.cx.pending, outside);
        self.cx.scopes.pop();
        let ty = Ty::Array {
            dims: dims.iter().map(DimSpec::dim).collect(),
            elem: Box::new(value.ty.clone()),
        };
        let acc = self.temp(ty.clone());
        if dims.len() == 1 {
            if let Some(bounds) = elem_range {
                self.cx.local_ranges.insert(
                    acc,
                    RangeInfo::Elem(Box::new(RangeInfo::Scalar(bounds.clone()))),
                );
            }
        }
        let cell = Place {
            kind: PlaceKind::Index(
                Box::new(Place {
                    kind: PlaceKind::Local(acc),
                    ty: ty.clone(),
                    span: at,
                }),
                indices,
            ),
            ty: value.ty.clone(),
            span: at,
        };
        let mut inner = hoisted;
        inner.push(Stmt::Assign(cell, AssignOp::Set, value));
        for (id, slot, iter, mut pre) in loops.into_iter().rev() {
            pre.extend(inner);
            inner = vec![Stmt::For(id, slot, iter, Block { stmts: pre })];
        }
        stmts.push(Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Fill(dims, None), ty.clone(), span)),
        ));
        stmts.extend(inner);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }
}

/// A literal, an empty collection or a tuple of them: evaluating it once or
/// once per element gives equal values and no effects.
fn fresh_constant(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Int(_)
        | ast::ExprKind::Float(_)
        | ast::ExprKind::Char(_)
        | ast::ExprKind::Bool(_)
        | ast::ExprKind::None => true,
        ast::ExprKind::Str(pieces) => pieces.iter().all(|p| matches!(p, ast::StrPiece::Text(_))),
        ast::ExprKind::Unary { expr, .. } => fresh_constant(expr),
        ast::ExprKind::Tuple(items) => items.iter().all(fresh_constant),
        ast::ExprKind::List(items) => items.is_empty(),
        ast::ExprKind::Call { callee, args } => {
            args.is_empty()
                && match &callee.kind {
                    ast::ExprKind::Generic { base, .. } => {
                        matches!(&base.kind, ast::ExprKind::Name(n) if matches!(n.as_str(), "List" | "Set" | "Map"))
                    }
                    _ => false,
                }
        }
        _ => false,
    }
}
