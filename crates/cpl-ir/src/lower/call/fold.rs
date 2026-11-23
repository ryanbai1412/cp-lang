//! `read`, and the folds and reductions over a sequence: min/max, sum, count.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn read(&mut self, spec: Spec, span: Span) -> Expr {
        let spec = self.readable(spec);
        if let Some(ty) = super::read_spec::unreadable(&spec) {
            let message = format!("`read()` has no input format for a `{}`", self.show(&ty));
            self.error(codes::UNSUPPORTED, span, message);
            if let Ty::Opt(inner) = &ty {
                if matches!(**inner, Ty::Int | Ty::Pos(_)) {
                    self.help("say which number in the input stands for `none`: `read(none: 0)`");
                }
            }
            return self.err_expr(span);
        }
        if let Spec::Array { dims, elem } = &spec {
            if let [DimSpec::Pos(d)] = dims[..] {
                if self.own_domains.contains(&d) {
                    return self.read_own(d, elem, spec.ty(), span);
                }
            }
        }
        // Deeper in, a domain this `read()` names has no size yet: reading
        // would take no values at all and go on.
        fn names_domain(spec: &Spec, own: &std::collections::HashSet<DomainId>) -> bool {
            match spec {
                Spec::Fields(_, fields) => fields.iter().any(|f| names_domain(f, own)),
                Spec::Opt(inner, _) => names_domain(inner, own),
                Spec::Array { dims, elem } => {
                    dims.iter()
                        .any(|d| matches!(d, DimSpec::Pos(d) if own.contains(d)))
                        || names_domain(elem, own)
                }
                _ => false,
            }
        }
        if names_domain(&spec, &self.own_domains) {
            let shown = self.show(&spec.ty());
            self.error(
                codes::UNSUPPORTED,
                span,
                format!("`read()` can't size a domain named inside a `{shown}`"),
            );
            self.help(
                "read it on its own: `let s: [domain S]Char = read()`, then the rest".to_string(),
            );
            return self.err_expr(span);
        }
        let ty = spec.ty();
        self.mk(ExprKind::Read(Box::new(spec)), ty, span)
    }

    /// `let s: [domain S]Char = read()`: the token read is the array,
    /// and its length is the size of `S`.
    pub(in crate::lower) fn read_own(
        &mut self,
        d: DomainId,
        elem: &Spec,
        ty: Ty,
        span: Span,
    ) -> Expr {
        if elem.ty() != Ty::Char {
            let shown = self.show(&ty);
            self.error(
                codes::UNSUPPORTED,
                span,
                format!("`read()` can't tell how many values a `{shown}` has"),
            );
            self.help(
                "read the count first, name a domain of that size, and read a `[D]T`".to_string(),
            );
            return self.err_expr(span);
        }
        self.do_expr(Some(&ty.clone()), span, |lw, out| {
            let word = lw.temp(Ty::Str);
            let token = lw.mk(
                ExprKind::Read(Box::new(Spec::Plain(Ty::Str))),
                Ty::Str,
                span,
            );
            out.push(Stmt::Let(word, Some(token)));
            let len = |lw: &Self| {
                lw.mk(
                    ExprKind::Builtin(Builtin::Len, vec![lw.local(word, span)]),
                    Ty::Int,
                    span,
                )
            };
            out.push(Stmt::DefDomain {
                id: d,
                lo: lw.int_lit(0, span),
                len: len(lw),
            });
            let cells = lw.temp(ty.clone());
            let blank = lw.mk(
                ExprKind::Fill(vec![DimSpec::Pos(d)], None),
                ty.clone(),
                span,
            );
            out.push(Stmt::Let(cells, Some(blank)));
            let at = lw.temp(Ty::Int);
            let pos = lw.mk(
                ExprKind::Cast(Cast::Trust, Box::new(lw.local(at, span))),
                Ty::Pos(d),
                span,
            );
            let cell = Place {
                kind: PlaceKind::Index(
                    Box::new(Place {
                        kind: PlaceKind::Local(cells),
                        ty: ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![pos],
                ),
                ty: Ty::Char,
                span: lw.span(span),
            };
            let byte = lw.mk(
                ExprKind::Index(Box::new(lw.local(word, span)), vec![lw.local(at, span)]),
                Ty::Char,
                span,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: len(lw),
                step: None,
                rev: false,
                closed: false,
            };
            let copy = Stmt::Assign(cell, AssignOp::Set, byte);
            out.push(Stmt::For(lid, at, iter, Block { stmts: vec![copy] }));
            out.push(Stmt::Give(lw.local(cells, span)));
        })
    }

    pub(in crate::lower) fn apply_key(
        &mut self,
        key_ast: &ast::Expr,
        elem_expr: Expr,
        out: &mut Vec<Stmt>,
        span: Span,
    ) -> Option<Expr> {
        // `by: f` where `f` is a top-level fn / builtin: wrap as `x => f(x)`.
        if let ast::ExprKind::Name(fn_name) = &key_ast.kind {
            if self.lookup(fn_name).is_none() {
                let at = key_ast.span;
                let mk_name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::Pattern {
                            kind: ast::PatternKind::Name("x".to_string()),
                            span: at,
                        }],
                        param_tys: Vec::new(),
                        param_mut: Vec::new(),
                        ret_ty: None,
                        body: Box::new(ast::Expr {
                            kind: ast::ExprKind::Call {
                                callee: Box::new(mk_name(fn_name)),
                                args: vec![ast::Arg {
                                    name: None,
                                    mutable: false,
                                    value: mk_name("x"),
                                }],
                            },
                            span: at,
                        }),
                    },
                    span: at,
                };
                return self.apply_key(&lambda, elem_expr, out, span);
            }
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &key_ast.kind {
            if let Some(param) = ast::Pattern::single_param(params) {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let slot = self.loop_var(&param, elem_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(slot, Some(elem_expr)));
                out.extend(pre);
                let key = self.expr(body, None);
                self.cx.scopes.pop();
                return Some(key);
            }
        }
        let fv = self.expr(key_ast, None);
        match &fv.ty {
            Ty::Fn(ps, ret) if ps.len() == 1 => {
                let ret = (**ret).clone();
                Some(self.mk(
                    ExprKind::CallExpr(Box::new(fv), vec![Arg::Value(elem_expr)]),
                    ret,
                    span,
                ))
            }
            Ty::Error => None,
            _ => {
                self.error(
                    codes::BAD_OPERANDS,
                    key_ast.span,
                    "`by:` expects a one-argument key function",
                );
                None
            }
        }
    }

    pub(in crate::lower) fn min_max_by(
        &mut self,
        is_max: bool,
        seq_ast: &ast::Expr,
        key_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        if let Some(values) = self.walked(seq_ast) {
            return self.min_max_by(is_max, &values, key_ast, span);
        }
        let name = if is_max { "max" } else { "min" };
        let at = self.span(span);
        let (frames, value) = match &seq_ast.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(seq_ast, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, seq_ast.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let elem_ty = value.ty.clone();
        if elem_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let local = |slot: Slot, ty: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let place = |slot: Slot, ty: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let mut inner = Vec::new();
        let x_slot = self.temp(elem_ty.clone());
        inner.push(Stmt::Let(x_slot, Some(value)));
        let Some(key_expr) = self.apply_key(key_ast, local(x_slot, &elem_ty), &mut inner, span)
        else {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        };
        let key_ty = key_expr.ty.clone();
        if key_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        if !self.ordered(&key_ty) {
            let message = format!("a `{}` has no order for `{name}`", self.show(&key_ty));
            self.error(codes::BAD_OPERANDS, key_ast.span, message);
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let out_ty = Ty::opt(elem_ty.clone());
        let opt_key_ty = Ty::opt(key_ty.clone());
        let best_slot = self.temp(out_ty.clone());
        let best_key_slot = self.temp(opt_key_ty.clone());
        let k_slot = self.temp(key_ty.clone());
        inner.push(Stmt::Let(k_slot, Some(key_expr)));

        let bk_is_some = self.mk(
            ExprKind::IsSome(Box::new(local(best_key_slot, &opt_key_ty))),
            Ty::Bool,
            span,
        );
        let bk_none = self.mk(
            ExprKind::Unary(UnOp::Not, Box::new(bk_is_some)),
            Ty::Bool,
            span,
        );
        let bk_val = self.mk(
            ExprKind::Unwrap(Box::new(local(best_key_slot, &opt_key_ty)), false),
            key_ty.clone(),
            span,
        );
        let k_val = local(k_slot, &key_ty);
        let better_cmp = if is_max {
            self.mk(
                ExprKind::Cmp(CmpOp::Lt, Box::new(bk_val), Box::new(k_val)),
                Ty::Bool,
                span,
            )
        } else {
            self.mk(
                ExprKind::Cmp(CmpOp::Lt, Box::new(k_val), Box::new(bk_val)),
                Ty::Bool,
                span,
            )
        };
        let take = self.mk(
            ExprKind::If(
                Box::new(bk_none),
                Box::new(self.mk(ExprKind::Bool(true), Ty::Bool, span)),
                Box::new(better_cmp),
            ),
            Ty::Bool,
            span,
        );
        let set_best = Stmt::Assign(
            place(best_slot, &out_ty),
            AssignOp::Set,
            self.mk(
                ExprKind::Some(Box::new(local(x_slot, &elem_ty))),
                out_ty.clone(),
                span,
            ),
        );
        let set_key = Stmt::Assign(
            place(best_key_slot, &opt_key_ty),
            AssignOp::Set,
            self.mk(
                ExprKind::Some(Box::new(local(k_slot, &key_ty))),
                opt_key_ty.clone(),
                span,
            ),
        );
        inner.push(Stmt::If(
            take,
            Block {
                stmts: vec![set_best, set_key],
            },
            Block::default(),
        ));
        let (loops, _) = self.close_loops(frames, inner);
        let mut stmts = vec![
            Stmt::Let(
                best_slot,
                Some(self.mk(ExprKind::None, out_ty.clone(), span)),
            ),
            Stmt::Let(
                best_key_slot,
                Some(self.mk(ExprKind::None, opt_key_ty, span)),
            ),
        ];
        stmts.extend(loops);
        stmts.push(Stmt::Give(local(best_slot, &out_ty)));
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    pub(in crate::lower) fn min_max(
        &mut self,
        is_max: bool,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let name = if is_max { "max" } else { "min" };
        if let [seq, by_arg] = args {
            if seq.name.is_none()
                && !seq.mutable
                && by_arg.name.as_ref().is_some_and(|n| n.text == "by")
                && !by_arg.mutable
            {
                return self.min_max_by(is_max, &seq.value, &by_arg.value, span);
            }
        }
        if let Some(named) = args.iter().find_map(|a| a.name.as_ref()) {
            let message = format!("`{name}` has no argument named `{}`", named.text);
            self.error(codes::UNKNOWN_ARG_NAME, named.span, message);
            return self.err_expr(span);
        }
        match args {
            [] => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    format!("`{name}` needs something to compare"),
                );
                self.err_expr(span)
            }
            [one] => {
                let generator = one.value.is_generator();
                self.min_max_reduce(is_max, &one.value, generator, span)
            }
            _ => {
                let mut values: Vec<Expr> =
                    args.iter().map(|a| self.expr(&a.value, None)).collect();
                if values.iter().any(|v| v.ty == Ty::Error) {
                    return self.err_expr(span);
                }
                // `min(5, opt)`: a plain value among optionals makes the
                // result plain, and each `none` is skipped.
                let plain = values
                    .iter()
                    .find(|v| !matches!(v.ty, Ty::Opt(_)))
                    .map(|v| v.ty.clone());
                if let Some(plain) = plain.filter(|plain| {
                    self.ordered(plain)
                        && values.iter().any(|v| matches!(v.ty, Ty::Opt(_)))
                        && values
                            .iter()
                            .all(|v| v.ty == *plain || v.ty == Ty::opt(plain.clone()))
                }) {
                    let op = if is_max { Builtin::Max } else { Builtin::Min };
                    return self.do_expr(Some(&plain), span, |lw, out| {
                        let mut slots = Vec::with_capacity(values.len());
                        for value in values {
                            let slot = lw.temp(value.ty.clone());
                            slots.push((slot, value.ty.clone()));
                            out.push(Stmt::Let(slot, Some(value)));
                        }
                        let best = lw.temp(plain.clone());
                        let first = slots
                            .iter()
                            .position(|(_, ty)| *ty == plain)
                            .expect("one value is plain");
                        let start = lw.local(slots[first].0, span);
                        out.push(Stmt::Let(best, Some(start)));
                        for (i, (slot, ty)) in slots.iter().enumerate() {
                            if i == first {
                                continue;
                            }
                            let held = lw.local(*slot, span);
                            let place = Place {
                                kind: PlaceKind::Local(best),
                                ty: plain.clone(),
                                span: lw.span(span),
                            };
                            let so_far = lw.local(best, span);
                            if *ty == plain {
                                let better = lw.mk(
                                    ExprKind::Builtin(op, vec![so_far, held]),
                                    plain.clone(),
                                    span,
                                );
                                out.push(Stmt::Assign(place, AssignOp::Set, better));
                            } else {
                                let some =
                                    lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                                let inside = lw.mk(
                                    ExprKind::Unwrap(Box::new(held), false),
                                    plain.clone(),
                                    span,
                                );
                                let better = lw.mk(
                                    ExprKind::Builtin(op, vec![so_far, inside]),
                                    plain.clone(),
                                    span,
                                );
                                out.push(Stmt::If(
                                    some,
                                    Block {
                                        stmts: vec![Stmt::Assign(place, AssignOp::Set, better)],
                                    },
                                    Block::default(),
                                ));
                            }
                        }
                        let result = lw.local(best, span);
                        out.push(Stmt::Give(result));
                    });
                }
                let ty = if values.iter().any(|v| v.ty == Ty::Float) {
                    Ty::Float
                } else {
                    values[0].ty.clone()
                };
                if let Ty::Opt(inner) = ty.clone() {
                    if !self.ordered(&inner) {
                        let message = format!("a `{}` has no order for `{name}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        return self.err_expr(span);
                    }
                    // Optional min/max: `none` is the identity element.
                    let mut acc_expr = self.coerce(
                        std::mem::replace(&mut values[0], self.err_expr(span)),
                        &ty,
                        args[0].value.span,
                    );
                    for i in 1..values.len() {
                        let rhs_expr = self.coerce(
                            std::mem::replace(&mut values[i], self.err_expr(span)),
                            &ty,
                            args[i].value.span,
                        );
                        let lhs_ty = ty.clone();
                        let inner_c = (*inner).clone();
                        acc_expr = self.do_expr(Some(&ty), span, |lw, out| {
                            let a_slot = lw.temp(lhs_ty.clone());
                            let b_slot = lw.temp(lhs_ty.clone());
                            out.push(Stmt::Let(a_slot, Some(acc_expr)));
                            out.push(Stmt::Let(b_slot, Some(rhs_expr)));
                            let a = lw.local(a_slot, span);
                            let b = lw.local(b_slot, span);
                            let a_some =
                                lw.mk(ExprKind::IsSome(Box::new(a.clone())), Ty::Bool, span);
                            let b_some =
                                lw.mk(ExprKind::IsSome(Box::new(b.clone())), Ty::Bool, span);
                            let av = lw.mk(
                                ExprKind::Unwrap(Box::new(a.clone()), false),
                                inner_c.clone(),
                                span,
                            );
                            let bv = lw.mk(
                                ExprKind::Unwrap(Box::new(b.clone()), false),
                                inner_c.clone(),
                                span,
                            );
                            let op = if is_max { Builtin::Max } else { Builtin::Min };
                            let both = lw.mk(
                                ExprKind::Some(Box::new(lw.mk(
                                    ExprKind::Builtin(op, vec![av, bv]),
                                    inner_c,
                                    span,
                                ))),
                                lhs_ty.clone(),
                                span,
                            );
                            let inner_if = lw.mk(
                                ExprKind::If(Box::new(b_some), Box::new(both), Box::new(a)),
                                lhs_ty.clone(),
                                span,
                            );
                            let res = lw.mk(
                                ExprKind::If(Box::new(a_some), Box::new(inner_if), Box::new(b)),
                                lhs_ty,
                                span,
                            );
                            out.push(Stmt::Give(res));
                        });
                    }
                    return acc_expr;
                }
                if !self.ordered(&ty) {
                    let message = format!("a `{}` has no order for `{name}`", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, span, message);
                    return self.err_expr(span);
                }
                for (i, arg) in args.iter().enumerate() {
                    let value = std::mem::replace(&mut values[i], self.err_expr(span));
                    values[i] = self.coerce(value, &ty, arg.value.span);
                }
                let op = if is_max { Builtin::Max } else { Builtin::Min };
                self.mk(ExprKind::Builtin(op, values), ty, span)
            }
        }
    }

    /// `first(x for x in seq if cond)` or `first(seq)` -> `?T` (first yielded item, or `none`).
    pub(in crate::lower) fn lower_first(&mut self, arg: &ast::Expr, span: Span) -> Expr {
        let at = self.span(span);
        let (frames, value) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(arg, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let elem_ty = value.ty.clone();
        if elem_ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let res_ty = Ty::opt(elem_ty);
        let acc = self.temp(res_ty.clone());
        let mut stmts = vec![Stmt::Let(
            acc,
            Some(Expr {
                kind: ExprKind::None,
                ty: res_ty.clone(),
                span: at,
            }),
        )];
        let inner = vec![
            Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(acc),
                    ty: res_ty.clone(),
                    span: at,
                },
                AssignOp::Set,
                Expr {
                    kind: ExprKind::Some(Box::new(value)),
                    ty: res_ty.clone(),
                    span: at,
                },
            ),
            Stmt::Break(None),
        ];
        let (mut loops, outer) = self.close_loops(frames, inner);
        retarget_break(&mut loops, outer, 0);
        stmts.extend(loops);
        stmts.push(Stmt::Give(Expr {
            kind: ExprKind::Local(acc),
            ty: res_ty.clone(),
            span: at,
        }));
        self.mk(ExprKind::Do(Block { stmts }), res_ty, span)
    }

    /// Lowers `e`, returning `?T` when `e` is a 1-arg `min(...)`, `max(...)`, or
    /// binary-function `xs.fold(f)` reduction (for `... ?? default` and `...!`).
    pub(in crate::lower) fn expr_maybe_opt_reduce(&mut self, e: &ast::Expr) -> Expr {
        if let ast::ExprKind::Call { callee, args } = &e.kind {
            if args.len() == 1 && args[0].name.is_none() {
                if let ast::ExprKind::Name(n) = &callee.kind {
                    if (n == "min" || n == "max") && self.lookup(n).is_none() {
                        return self.min_max_reduce(n == "max", &args[0].value, true, e.span);
                    }
                }
            }
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if args.is_empty()
                    && matches!(name.text.as_str(), "min" | "max")
                    && matches!(
                        self.probe_ty_silent(base),
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_, _)
                                | Ty::Keyed(_, _)
                                | Ty::MutView(_)
                                | Ty::Array { .. }
                                | Ty::Range(_)
                                | Ty::Str
                                | Ty::StrView
                        )
                    )
                {
                    return self.min_max_reduce(name.text == "max", base, true, e.span);
                }
                if args.len() == 1
                    && args[0].name.is_none()
                    && !args[0].mutable
                    && name.text == "fold"
                    && matches!(
                        self.probe_ty_silent(base),
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_, _)
                                | Ty::Keyed(_, _)
                                | Ty::MutView(_)
                                | Ty::Array { .. }
                                | Ty::Range(_)
                                | Ty::Set(_)
                                | Ty::HashSet(_)
                                | Ty::Str
                                | Ty::StrView
                        )
                    )
                {
                    return self.fold_seq(base, &args[0], true, name.span, e.span);
                }
            }
        }
        self.expr(e, None)
    }

    pub(in crate::lower) fn fold_seq(
        &mut self,
        base: &ast::Expr,
        op: &ast::Arg,
        optional: bool,
        name_span: Span,
        span: Span,
    ) -> Expr {
        let fold = |lw: &mut Self| lw.fold_seq(base, op, optional, name_span, span);
        if let Some(checked) = self.fold_checking_laws(base, op, span, fold) {
            return checked;
        }
        if let ast::ExprKind::Name(op_name) = &op.value.kind {
            if matches!(op_name.as_str(), "min" | "max") {
                let elem = self.probe_ty_silent(base).and_then(|t| t.seq_elem());
                if elem
                    .as_ref()
                    .is_some_and(|e| !matches!(e, Ty::Int | Ty::I128 | Ty::Float))
                {
                    return self.min_max_reduce(op_name == "max", base, optional, span);
                }
            }
        }
        let use_monoid = !optional && self.detect_bare_op(&op.value).is_some();
        if !use_monoid {
            let is_binary_fn = match &op.value.kind {
                ast::ExprKind::Lambda { params, .. } => params.len() == 2,
                ast::ExprKind::Name(fn_name) if self.lookup(fn_name).is_none() => true,
                _ => matches!(
                    self.probe_ty_silent(&op.value),
                    Some(Ty::Fn(ps, _)) if ps.len() == 2
                ),
            };
            if is_binary_fn {
                return self.reduce_fold_fn(base, &op.value, optional, span);
            }
            if let Some(Ty::Struct(sid)) = self.probe_ty_silent(&op.value) {
                let s = &self.structs[sid];
                let has_op =
                    s.methods.contains_key("op") || s.trait_methods.keys().any(|(_, m)| m == "op");
                let has_id = s.def.fields.iter().any(|(n, _)| n == "id");
                if has_op && !has_id {
                    let at = op.value.span;
                    let mk_name = |text: &str| ast::Expr {
                        kind: ast::ExprKind::Name(text.to_string()),
                        span: at,
                    };
                    let lambda = ast::Expr {
                        kind: ast::ExprKind::Lambda {
                            params: vec![
                                ast::Pattern {
                                    kind: ast::PatternKind::Name("__fold_a".to_string()),
                                    span: at,
                                },
                                ast::Pattern {
                                    kind: ast::PatternKind::Name("__fold_b".to_string()),
                                    span: at,
                                },
                            ],
                            param_tys: vec![None, None],
                            param_mut: vec![false, false],
                            ret_ty: None,
                            body: Box::new(ast::Expr {
                                kind: ast::ExprKind::Call {
                                    callee: Box::new(ast::Expr {
                                        kind: ast::ExprKind::Field {
                                            base: Box::new(op.value.clone()),
                                            name: ast::Name {
                                                text: "op".to_string(),
                                                span: at,
                                            },
                                        },
                                        span: at,
                                    }),
                                    args: vec![
                                        ast::Arg {
                                            name: None,
                                            mutable: false,
                                            value: mk_name("__fold_a"),
                                        },
                                        ast::Arg {
                                            name: None,
                                            mutable: false,
                                            value: mk_name("__fold_b"),
                                        },
                                    ],
                                },
                                span: at,
                            }),
                        },
                        span: at,
                    };
                    return self.reduce_fold_fn(base, &lambda, optional, span);
                }
            }
        }
        let seq = ast::Arg {
            name: None,
            mutable: false,
            value: base.clone(),
        };
        let callee = ast::Expr {
            kind: ast::ExprKind::Name("fold".into()),
            span: name_span,
        };
        let call_args = vec![op.clone(), seq];
        let whole = ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(callee.clone()),
                args: call_args.clone(),
            },
            span,
        };
        self.call(&whole, &callee, &call_args, None)
    }

    pub(in crate::lower) fn apply_fold_fn(
        &mut self,
        fn_ast: &ast::Expr,
        acc_expr: Expr,
        cur_expr: Expr,
        out: &mut Vec<Stmt>,
        span: Span,
    ) -> Option<Expr> {
        if let ast::ExprKind::Name(fn_name) = &fn_ast.kind {
            if self.lookup(fn_name).is_none() {
                let at = fn_ast.span;
                let mk_name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![
                            ast::Pattern {
                                kind: ast::PatternKind::Name("__fold_a".to_string()),
                                span: at,
                            },
                            ast::Pattern {
                                kind: ast::PatternKind::Name("__fold_b".to_string()),
                                span: at,
                            },
                        ],
                        param_tys: Vec::new(),
                        param_mut: Vec::new(),
                        ret_ty: None,
                        body: Box::new(ast::Expr {
                            kind: ast::ExprKind::Call {
                                callee: Box::new(mk_name(fn_name)),
                                args: vec![
                                    ast::Arg {
                                        name: None,
                                        mutable: false,
                                        value: mk_name("__fold_a"),
                                    },
                                    ast::Arg {
                                        name: None,
                                        mutable: false,
                                        value: mk_name("__fold_b"),
                                    },
                                ],
                            },
                            span: at,
                        }),
                    },
                    span: at,
                };
                return self.apply_fold_fn(&lambda, acc_expr, cur_expr, out, span);
            }
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &fn_ast.kind {
            if params.len() == 2 {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let a_slot = self.loop_var(&params[0], acc_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(a_slot, Some(acc_expr)));
                let b_slot = self.loop_var(&params[1], cur_expr.ty.clone(), &mut pre);
                out.push(Stmt::Let(b_slot, Some(cur_expr)));
                out.extend(pre);
                let res = self.expr(body, None);
                self.cx.scopes.pop();
                return Some(res);
            }
        }
        let fv = self.expr(fn_ast, None);
        match &fv.ty {
            Ty::Fn(ps, ret) if ps.len() == 2 => {
                let ret = (**ret).clone();
                Some(self.mk(
                    ExprKind::CallExpr(
                        Box::new(fv),
                        vec![Arg::Value(acc_expr), Arg::Value(cur_expr)],
                    ),
                    ret,
                    span,
                ))
            }
            Ty::Error => None,
            _ => {
                self.error(
                    codes::BAD_OPERANDS,
                    fn_ast.span,
                    "`fold` expects a monoid or a two-argument function `(T, T) -> T`",
                );
                None
            }
        }
    }

    pub(in crate::lower) fn reduce_fold_fn(
        &mut self,
        base: &ast::Expr,
        fn_ast: &ast::Expr,
        optional: bool,
        span: Span,
    ) -> Expr {
        if let Some(values) = self.walked(base) {
            return self.reduce_fold_fn(&values, fn_ast, optional, span);
        }
        let at = self.span(span);
        let (frames, value) = match &base.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = self.expr(body, None);
                (frames, value)
            }
            _ => {
                let seq = self.expr(base, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, base.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let ty = value.ty.clone();
        if ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        let local = |slot: Slot, t: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: t.clone(),
            span: at,
        };
        let place = |slot: Slot, t: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: t.clone(),
            span: at,
        };
        let lit = |kind: ExprKind, t: Ty| Expr {
            kind,
            ty: t,
            span: at,
        };
        let mut stmts = Vec::new();
        let acc = self.temp(ty.clone());
        let have = self.temp(Ty::Bool);
        let cur = self.temp(ty.clone());
        stmts.push(Stmt::Let(acc, None));
        stmts.push(Stmt::Let(have, Some(lit(ExprKind::Bool(false), Ty::Bool))));

        let mut combine_stmts = Vec::new();
        let Some(combined) = self.apply_fold_fn(
            fn_ast,
            local(acc, &ty),
            local(cur, &ty),
            &mut combine_stmts,
            span,
        ) else {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        };
        if combined.ty == Ty::Error {
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        if combined.ty != ty {
            let message = format!(
                "`fold` expects a function returning `{}`, got `{}`",
                self.show(&ty),
                self.show(&combined.ty)
            );
            self.error(codes::TYPE_MISMATCH, fn_ast.span, message);
            self.close_loops(frames, Vec::new());
            return self.err_expr(span);
        }
        combine_stmts.push(Stmt::Assign(place(acc, &ty), AssignOp::Set, combined));

        let inner = vec![
            Stmt::Let(cur, Some(value)),
            Stmt::If(
                local(have, &Ty::Bool),
                Block {
                    stmts: combine_stmts,
                },
                Block {
                    stmts: vec![
                        Stmt::Assign(place(acc, &ty), AssignOp::Set, local(cur, &ty)),
                        Stmt::Assign(
                            place(have, &Ty::Bool),
                            AssignOp::Set,
                            lit(ExprKind::Bool(true), Ty::Bool),
                        ),
                    ],
                },
            ),
        ];
        let (loops, _) = self.close_loops(frames, inner);
        stmts.extend(loops);

        let (out_slot, out_ty) = if optional {
            let res_ty = Ty::opt(ty.clone());
            let out_slot = self.temp(res_ty.clone());
            stmts.push(Stmt::Let(
                out_slot,
                Some(lit(
                    ExprKind::If(
                        Box::new(local(have, &Ty::Bool)),
                        Box::new(lit(
                            ExprKind::Some(Box::new(local(acc, &ty))),
                            res_ty.clone(),
                        )),
                        Box::new(lit(ExprKind::None, res_ty.clone())),
                    ),
                    res_ty.clone(),
                )),
            ));
            (out_slot, res_ty)
        } else {
            stmts.push(Stmt::If(
                lit(
                    ExprKind::Unary(UnOp::Not, Box::new(local(have, &Ty::Bool))),
                    Ty::Bool,
                ),
                Block {
                    stmts: vec![Stmt::Trap("`fold` of an empty sequence".into(), at)],
                },
                Block::default(),
            ));
            (acc, ty)
        };
        stmts.push(Stmt::Give(local(out_slot, &out_ty)));
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    /// `count(xs, x => x > 2)` is `count(x > 2 for x in xs)`, and
    /// `first(xs, p)` is `first(x for x in xs if p(x))`: a test after the
    /// sequence is the body of the walk, so every fold that takes a walk
    /// takes a test the same way. Only `count`, `all`, `any` and `first`
    /// take one: they fold what the test says, where a `sum` of a
    /// function is `sum(f(x) for x in xs)`.
    pub(in crate::lower) fn tested_walk(
        &mut self,
        name: &str,
        args: &[ast::Arg],
        filter: bool,
    ) -> Option<ast::Expr> {
        if !matches!(name, "count" | "all" | "any" | "first") {
            return None;
        }
        let [seq, test] = args else { return None };
        if seq.name.is_some() || test.name.is_some() || !self.is_test(&test.value) {
            return None;
        }
        let at = test.value.span;
        let mk = |kind: ast::ExprKind| ast::Expr { kind, span: at };
        let (pat, body) = match &test.value.kind {
            ast::ExprKind::Lambda { params, body, .. } if !params.is_empty() => (
                ast::Pattern::single_param(params).expect("a parameter"),
                (**body).clone(),
            ),
            _ => {
                let var = "it\u{b7}".to_string();
                let call = mk(ast::ExprKind::Call {
                    callee: Box::new(test.value.clone()),
                    args: vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: mk(ast::ExprKind::Name(var.clone())),
                    }],
                });
                (
                    ast::Pattern {
                        kind: ast::PatternKind::Name(var),
                        span: at,
                    },
                    call,
                )
            }
        };
        let (body, conds) = if filter {
            let ast::PatternKind::Name(var) = &pat.kind else {
                return None;
            };
            (
                mk(ast::ExprKind::Name(var.clone())),
                vec![ast::Cond::Expr(body)],
            )
        } else {
            (body, Vec::new())
        };
        Some(mk(ast::ExprKind::Comprehension {
            body: Box::new(body),
            clauses: vec![ast::Clause {
                pat,
                iter: seq.value.clone(),
                conds,
            }],
        }))
    }

    /// `sum(a[v] for v in V)`, `max(xs)`, `any(...)`: a loop that folds
    /// the values into one.
    pub(in crate::lower) fn reduce(&mut self, kind: Reduce, arg: &ast::Expr, span: Span) -> Expr {
        self.reduce_opt(kind, arg, false, span)
    }

    /// `argmax(f(v) for v in V)` is the `v` with the largest `f(v)`: `none`
    /// when there is nothing to pick from, the first one on a tie, and a `none`
    /// among the values is skipped as `max` skips it. It is `max_by` of
    /// `std.seq` over the walk, so the two can't drift apart.
    pub(in crate::lower) fn arg_extreme(&mut self, max: bool, arg: &ast::Expr, span: Span) -> Expr {
        let at = arg.span;
        let mk = |kind: ast::ExprKind| ast::Expr { kind, span: at };
        let name = |text: &str| mk(ast::ExprKind::Name(text.to_string()));
        let (body, clauses) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => ((**body).clone(), clauses.clone()),
            _ => {
                // `argmax(a)` of a list or an array is `a.argmax` of
                // `std.seq`: the first index of the largest value.
                let method = if max { "argmax" } else { "argmin" };
                if let Some(found) = self.try_ufcs_call(method, arg, &[], span) {
                    return found;
                }
                self.unsupported(span, "`argmax` / `argmin` of a sequence without a `for`");
                self.help("name the position: `argmax(a[i] for i in D)`");
                return self.err_expr(span);
            }
        };
        let [clause] = clauses.as_slice() else {
            self.unsupported(span, "`argmax` / `argmin` over more than one `for`");
            return self.err_expr(span);
        };
        let ast::PatternKind::Name(var) = &clause.pat.kind else {
            self.unsupported(span, "`argmax` / `argmin` over a pattern");
            return self.err_expr(span);
        };
        // Are the values optionals? Then the ones that are `none` are left out.
        let values = mk(ast::ExprKind::Comprehension {
            body: Box::new(body.clone()),
            clauses: clauses.clone(),
        });
        let (diags, pending) = (self.diags.len(), self.cx.pending.len());
        let optional =
            matches!(self.expr(&values, None).ty, Ty::List(elem) if matches!(*elem, Ty::Opt(_)));
        self.diags.truncate(diags);
        self.cx.pending.truncate(pending);
        let mut clause = clause.clone();
        let mut key = body;
        if optional {
            let test = mk(ast::ExprKind::Compare {
                first: Box::new(key.clone()),
                rest: vec![(CmpOp::Ne, mk(ast::ExprKind::None))],
            });
            clause.conds.push(ast::Cond::Expr(test));
            key = mk(ast::ExprKind::Unwrap(Box::new(key)));
        }
        let picks = mk(ast::ExprKind::Comprehension {
            body: Box::new(name(var)),
            clauses: vec![clause],
        });
        let by = mk(ast::ExprKind::Lambda {
            params: vec![ast::Pattern {
                kind: ast::PatternKind::Name(var.clone()),
                span: at,
            }],
            param_tys: Vec::new(),
            param_mut: Vec::new(),
            ret_ty: None,
            body: Box::new(key),
        });
        let call = mk(ast::ExprKind::Call {
            callee: Box::new(name(if max { "max" } else { "min" })),
            args: vec![
                ast::Arg {
                    name: None,
                    mutable: false,
                    value: picks,
                },
                ast::Arg {
                    name: Some(ast::Name {
                        text: "by".to_string(),
                        span: at,
                    }),
                    mutable: false,
                    value: by,
                },
            ],
        });
        self.expr(&call, None)
    }

    /// O(1) closed-form arithmetic-series sum for `Range<Int>` and
    /// `(Range<Int>).step(k)`. Returns `0` for empty ranges and traps on
    /// `k <= 0` just like `step` iteration.
    pub(in crate::lower) fn try_range_int_sum(
        &mut self,
        arg: &ast::Expr,
        span: Span,
    ) -> Option<Expr> {
        let mut cur = arg;
        while let ast::ExprKind::Paren(inner) = &cur.kind {
            cur = inner;
        }
        if let ast::ExprKind::Call { callee, args } = &cur.kind {
            if args.len() == 1 && args[0].name.is_none() && !args[0].mutable {
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if name.text == "step"
                        && matches!(self.probe_ty_silent(base), Some(Ty::Range(inner)) if *inner == Ty::Int)
                    {
                        let range_val = self.expr(base, None);
                        let step_val = self.expr(&args[0].value, Some(&Ty::Int));
                        let step_val = self.coerce(step_val, &Ty::Int, span);
                        return Some(self.range_int_sum_closed(range_val, Some(step_val), span));
                    }
                }
            }
        }
        if matches!(self.probe_ty_silent(cur), Some(Ty::Range(inner)) if *inner == Ty::Int) {
            let range_val = self.expr(cur, None);
            return Some(self.range_int_sum_closed(range_val, None, span));
        }
        None
    }

    /// An Int range's own `min()`/`max()`: its ends, O(1) (R07.9.6).
    pub(in crate::lower) fn range_min_max(
        &mut self,
        max: bool,
        seq: &ast::Expr,
        optional: bool,
        span: Span,
    ) -> Option<Expr> {
        if !matches!(self.probe_ty_silent(seq), Some(Ty::Range(inner)) if *inner == Ty::Int) {
            return None;
        }
        let range = self.expr(seq, None);
        let ty = if optional { Ty::opt(Ty::Int) } else { Ty::Int };
        Some(self.do_expr(Some(&ty.clone()), span, |lw, out| {
            let r = lw.new_slot("r", range.ty.clone());
            out.push(Stmt::Let(r, Some(range)));
            let (lo, hi) = (lw.new_slot("lo", Ty::Int), lw.new_slot("hi", Ty::Int));
            for (slot, end) in [(lo, Builtin::RangeLo), (hi, Builtin::RangeHi)] {
                let value = lw.local(r, span);
                let value = lw.mk(ExprKind::Builtin(end, vec![value]), Ty::Int, span);
                out.push(Stmt::Let(slot, Some(value)));
            }
            let (l, h) = (lw.local(lo, span), lw.local(hi, span));
            let empty = lw.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(h.clone()), Box::new(l.clone())),
                Ty::Bool,
                span,
            );
            let end = if max { lw.int_add(h, -1) } else { l };
            let result = if optional {
                let some = lw.mk(ExprKind::Some(Box::new(end)), ty.clone(), span);
                let none = lw.mk(ExprKind::None, ty.clone(), span);
                lw.mk(
                    ExprKind::If(Box::new(empty), Box::new(none), Box::new(some)),
                    ty,
                    span,
                )
            } else {
                let word = if max { "max" } else { "min" };
                let trap = Stmt::Trap(format!("`{word}` of an empty sequence"), lw.span(span));
                out.push(Stmt::If(
                    empty,
                    Block { stmts: vec![trap] },
                    Block::default(),
                ));
                end
            };
            out.push(Stmt::Give(result));
        }))
    }

    pub(in crate::lower) fn range_int_sum_closed(
        &mut self,
        range_val: Expr,
        step_val: Option<Expr>,
        span: Span,
    ) -> Expr {
        let mut args = vec![range_val];
        if let Some(sv) = step_val {
            args.push(sv);
        }
        self.mk(ExprKind::Builtin(Builtin::RangeSum, args), Ty::Int, span)
    }

    pub(in crate::lower) fn reduce_opt(
        &mut self,
        kind: Reduce,
        arg: &ast::Expr,
        optional: bool,
        span: Span,
    ) -> Expr {
        if matches!(kind, Reduce::Sum) && !optional {
            if let Some(closed) = self.try_range_int_sum(arg, span) {
                return closed;
            }
        }
        if let Some(values) = self.walked(arg) {
            return self.reduce_opt(kind, &values, optional, span);
        }
        let at = self.span(span);
        let (frames, value) = match &arg.kind {
            ast::ExprKind::Comprehension { body, clauses } => {
                let frames = self.open_loops(clauses);
                let value = match kind {
                    Reduce::Count | Reduce::All | Reduce::Any => self.expect(body, &Ty::Bool),
                    _ => self.expr(body, None),
                };
                (frames, value)
            }
            _ => {
                let seq = self.expr(arg, None);
                let elem = match seq.ty.seq_elem().or_else(|| match &seq.ty {
                    Ty::Array { elem, .. } | Ty::Set(elem) | Ty::HashSet(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                }) {
                    Some(e) => e,
                    None if seq.ty == Ty::Error => return self.err_expr(span),
                    None => {
                        let message = format!(
                            "this needs a sequence of values, not a `{}`",
                            self.show(&seq.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, arg.span, message);
                        return self.err_expr(span);
                    }
                };
                self.open_seq(seq, elem)
            }
        };
        let local = |slot: Slot, ty: &Ty| Expr {
            kind: ExprKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let place = |slot: Slot, ty: &Ty| Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span: at,
        };
        let lit = |kind: ExprKind, ty: Ty| Expr { kind, ty, span: at };
        let mut stmts = Vec::new();
        let (acc, ty, inner, after): (Slot, Ty, Vec<Stmt>, Vec<Stmt>) = match kind {
            Reduce::Sum => 'sum: {
                let ty = value.ty.clone();
                // A struct with a `zero` and a `+` adds with its own `+`,
                // from its own zero: a `Mod<P>`, a matrix, a polynomial.
                if let Ty::Struct(sid) = ty {
                    let zero = self.structs[sid].consts.get("zero").cloned();
                    if let (Some(zero), Some(add)) = (zero, self.struct_op_fn(sid, "add")) {
                        let zero = self.eval_struct_const(sid, zero, span);
                        let zero = self.coerce(zero, &ty, span);
                        let acc = self.temp(ty.clone());
                        stmts.push(Stmt::Let(acc, Some(zero)));
                        let total = lit(ExprKind::Local(acc), ty.clone());
                        let next = lit(
                            ExprKind::Call(add, vec![Arg::Value(total), Arg::Value(value)]),
                            ty.clone(),
                        );
                        let add = Stmt::Assign(place(acc, &ty), AssignOp::Set, next);
                        break 'sum (acc, ty, vec![add], Vec::new());
                    }
                }
                let zero = match ty {
                    Ty::Int => ExprKind::Int(0),
                    Ty::I128 => {
                        ExprKind::Cast(Cast::IntToI128, Box::new(lit(ExprKind::Int(0), Ty::Int)))
                    }
                    Ty::U64 => {
                        ExprKind::Cast(Cast::IntToU64, Box::new(lit(ExprKind::Int(0), Ty::Int)))
                    }
                    Ty::Float => ExprKind::Float(0.0),
                    Ty::Error => {
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                    _ => {
                        let message = format!("`sum` adds numbers, not `{}`", self.show(&ty));
                        self.error(codes::BAD_OPERANDS, arg.span, message);
                        if matches!(ty, Ty::Opt(_)) {
                            self.help("`sum` does not skip or absorb `none`: add `x ?? 0`, or filter with `if let`");
                        }
                        if ty == Ty::Str {
                            self.help("to put the pieces of text one after another, write `xs.join(\"\")`");
                        }
                        self.close_loops(frames, Vec::new());
                        return self.err_expr(span);
                    }
                };
                let acc = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, Some(lit(zero, ty.clone()))));
                let add = Stmt::Assign(place(acc, &ty), AssignOp::Binary(BinOp::Add), value);
                (acc, ty, vec![add], Vec::new())
            }
            Reduce::Count => {
                let acc = self.temp(Ty::Int);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Int(0), Ty::Int))));
                let add = Stmt::Assign(
                    place(acc, &Ty::Int),
                    AssignOp::Binary(BinOp::Add),
                    lit(ExprKind::Int(1), Ty::Int),
                );
                let test = Stmt::If(value, Block { stmts: vec![add] }, Block::default());
                (acc, Ty::Int, vec![test], Vec::new())
            }
            Reduce::All | Reduce::Any => {
                let all = kind == Reduce::All;
                let acc = self.temp(Ty::Bool);
                stmts.push(Stmt::Let(acc, Some(lit(ExprKind::Bool(all), Ty::Bool))));
                let set = Stmt::Assign(
                    place(acc, &Ty::Bool),
                    AssignOp::Set,
                    lit(ExprKind::Bool(!all), Ty::Bool),
                );
                // Leave every loop at the first value that decides it.
                let test = if all {
                    lit(ExprKind::Unary(UnOp::Not, Box::new(value)), Ty::Bool)
                } else {
                    value
                };
                (
                    acc,
                    Ty::Bool,
                    vec![Stmt::If(
                        test,
                        Block {
                            stmts: vec![set, Stmt::Break(None)],
                        },
                        Block::default(),
                    )],
                    Vec::new(),
                )
            }
            Reduce::Min | Reduce::Max => {
                let ty = value.ty.clone();
                if ty == Ty::Error {
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                // Over optionals `none` is skipped, as `max=` skips it, and
                // the answer is `none` when nothing is left.
                let (ty, held) = match ty {
                    Ty::Opt(inner) => {
                        let held = self.temp(Ty::Opt(inner.clone()));
                        (*inner, Some(held))
                    }
                    ty => (ty, None),
                };
                let optional = optional || held.is_some();
                if matches!(ty, Ty::Opt(_)) || !self.ordered(&ty) {
                    let message = format!("a `{}` has no order to compare by", self.show(&ty));
                    self.error(codes::BAD_OPERANDS, arg.span, message);
                    self.close_loops(frames, Vec::new());
                    return self.err_expr(span);
                }
                let acc = self.temp(ty.clone());
                let have = self.temp(Ty::Bool);
                let cur = self.temp(ty.clone());
                stmts.push(Stmt::Let(acc, None));
                stmts.push(Stmt::Let(have, Some(lit(ExprKind::Bool(false), Ty::Bool))));
                let op = if kind == Reduce::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                let (read, value) = match held {
                    Some(held) => {
                        let opt = Ty::opt(ty.clone());
                        let inside = ExprKind::Unwrap(Box::new(local(held, &opt)), false);
                        (Some((held, value, opt)), lit(inside, ty.clone()))
                    }
                    None => (None, value),
                };
                let inner = vec![
                    Stmt::Let(cur, Some(value)),
                    Stmt::If(
                        local(have, &Ty::Bool),
                        Block {
                            stmts: vec![Stmt::Assign(place(acc, &ty), op, local(cur, &ty))],
                        },
                        Block {
                            stmts: vec![
                                Stmt::Assign(place(acc, &ty), AssignOp::Set, local(cur, &ty)),
                                Stmt::Assign(
                                    place(have, &Ty::Bool),
                                    AssignOp::Set,
                                    lit(ExprKind::Bool(true), Ty::Bool),
                                ),
                            ],
                        },
                    ),
                ];
                let inner = match read {
                    Some((held, value, opt)) => vec![
                        Stmt::Let(held, Some(value)),
                        Stmt::If(
                            lit(ExprKind::IsSome(Box::new(local(held, &opt))), Ty::Bool),
                            Block { stmts: inner },
                            Block::default(),
                        ),
                    ],
                    None => inner,
                };
                if optional {
                    let res_ty = Ty::opt(ty.clone());
                    let out_slot = self.temp(res_ty.clone());
                    let choose = Stmt::Let(
                        out_slot,
                        Some(lit(
                            ExprKind::If(
                                Box::new(local(have, &Ty::Bool)),
                                Box::new(lit(
                                    ExprKind::Some(Box::new(local(acc, &ty))),
                                    res_ty.clone(),
                                )),
                                Box::new(lit(ExprKind::None, res_ty.clone())),
                            ),
                            res_ty.clone(),
                        )),
                    );
                    (out_slot, res_ty, inner, vec![choose])
                } else {
                    let word = if kind == Reduce::Max { "max" } else { "min" };
                    let empty = Stmt::If(
                        lit(
                            ExprKind::Unary(UnOp::Not, Box::new(local(have, &Ty::Bool))),
                            Ty::Bool,
                        ),
                        Block {
                            stmts: vec![Stmt::Trap(format!("`{word}` of an empty sequence"), at)],
                        },
                        Block::default(),
                    );
                    (acc, ty, inner, vec![empty])
                }
            }
        };
        let (mut loops, outer) = self.close_loops(frames, inner);
        if matches!(kind, Reduce::All | Reduce::Any) {
            retarget_break(&mut loops, outer, 0);
        }
        stmts.extend(loops);
        stmts.extend(after);
        stmts.push(Stmt::Give(local(acc, &ty)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    // ----- members -----

    /// `base.name` that is not being called.
    /// A number a generic struct was given, read off the type by the parameter's name: with
    /// `struct Mod<P>` and `type Z = Mod<7>`, `Z.P` is 7.
    pub(in crate::lower) fn struct_number_param(
        &mut self,
        sid: StructId,
        name: &str,
        span: Span,
    ) -> Option<Expr> {
        let (gid, gargs) = self.structs[sid].origin_gen.clone()?;
        let at = self.gen_structs[gid]
            .decl
            .generics
            .iter()
            .position(|p| p.text == name)?;
        match gargs.get(at)? {
            GenArg::Int(value) => Some(self.int_lit(*value, span)),
            GenArg::RuntimeInt(id) => Some(self.mk(ExprKind::Global(*id), Ty::Int, span)),
            _ => None,
        }
    }
}
