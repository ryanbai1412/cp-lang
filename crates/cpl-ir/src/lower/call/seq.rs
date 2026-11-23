//! Sorting, domain members, and the lazy seq adapters: map, filter, partition.

use super::*;

impl<'a> Lowerer<'a> {
    /// `xs.sort()`, `xs.sort(by: x => key)`, `xs.sort(by: (a, b) => a < b)`, `xs.sort(by: ord)`.
    pub(in crate::lower) fn sort(
        &mut self,
        base: &ast::Expr,
        elem: &Ty,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let place = self.place(base);
        self.sort_place(place, elem, args, span)
    }

    pub(in crate::lower) fn sort_place(
        &mut self,
        place: Place,
        elem: &Ty,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        // `desc: true`: the same sort asking its comparison the other way
        // round, so the largest comes first and equal keys stay in the
        // order they had.
        if let Some(at) = args
            .iter()
            .position(|arg| arg.name.as_ref().is_some_and(|name| name.text == "desc"))
        {
            let rest: Vec<ast::Arg> = args
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != at)
                .map(|(_, arg)| arg.clone())
                .collect();
            let elem = elem.clone();
            return self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                let desc = lw.expect(&args[at].value, &Ty::Bool);
                let desc = lw.hold(desc, out);
                let place = lw.settle_indexes(place, out);
                let mut sort = lw.sort_place(place, &elem, &rest, span);
                if let ExprKind::Sort(_, _, turned) = &mut sort.kind {
                    *turned = Some(Box::new(desc));
                }
                out.push(Stmt::Expr(sort));
                out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
            });
        }
        // `by: arr` where `arr: [D]T`: an array indexed by the element is the key `x => arr[x]`.
        if let [ast::Arg {
            name: Some(by),
            value,
            mutable,
        }] = args
        {
            if by.text == "by"
                && !matches!(value.kind, ast::ExprKind::Lambda { .. })
                && matches!(
                    self.probe_ty_silent(value),
                    Some(Ty::Array { dims, .. }) if dims.len() == 1
                )
            {
                let at = value.span;
                let idx_expr = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(value.clone()),
                        indices: vec![ast::Expr {
                            kind: ast::ExprKind::Name("__k".to_string()),
                            span: at,
                        }],
                    },
                    span: at,
                };
                let lambda = ast::Expr {
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::Pattern {
                            kind: ast::PatternKind::Name("__k".to_string()),
                            span: at,
                        }],
                        param_tys: vec![None],
                        param_mut: Vec::new(),
                        ret_ty: None,
                        body: Box::new(idx_expr),
                    },
                    span: at,
                };
                let keyed = [ast::Arg {
                    name: Some(by.clone()),
                    mutable: *mutable,
                    value: lambda,
                }];
                return self.sort_place(place, elem, &keyed, span);
            }
        }
        // `by: desc`: a fn by its name is the key `x => desc(x)`.
        if let [ast::Arg {
            name: Some(by),
            value:
                value @ ast::Expr {
                    kind: ast::ExprKind::Name(key),
                    ..
                },
            mutable,
        }] = args
        {
            if by.text == "by" && self.lookup(key).is_none() {
                let at = value.span;
                let name = |text: &str| ast::Expr {
                    kind: ast::ExprKind::Name(text.to_string()),
                    span: at,
                };
                let call = ast::Expr {
                    kind: ast::ExprKind::Call {
                        callee: Box::new(name(key)),
                        args: vec![ast::Arg {
                            name: None,
                            mutable: false,
                            value: name("x"),
                        }],
                    },
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
                        body: Box::new(call),
                    },
                    span: at,
                };
                let keyed = [ast::Arg {
                    name: Some(by.clone()),
                    mutable: *mutable,
                    value: lambda,
                }];
                return self.sort_place(place, elem, &keyed, span);
            }
        }
        let has_by = args
            .iter()
            .any(|a| a.name.as_ref().is_some_and(|n| n.text == "by"));
        let has_cmp = args
            .iter()
            .any(|a| a.name.as_ref().is_some_and(|n| n.text == "cmp"));
        if has_by && has_cmp {
            self.error(
                codes::BAD_OPERANDS,
                span,
                "`sort` takes either `by:` or `cmp:`, not both",
            );
            return self.err_expr(span);
        }
        let key = match args {
            [] => {
                if !self.ordered(elem) {
                    let message = format!("a `{}` has no order to sort by", self.show(elem));
                    self.error(codes::BAD_OPERANDS, span, message);
                    self.help("sort by a key (`xs.sort(by: x => x.w)`) or comparator (`xs.sort(cmp: (a, b) => a < b)`)");
                    return self.err_expr(span);
                }
                SortBy::Natural
            }
            [ast::Arg {
                name: Some(cmp),
                value,
                ..
            }] if cmp.text == "cmp" => {
                let cmp_hint = Ty::Fn(vec![elem.clone(), elem.clone()], Box::new(Ty::Bool));
                let fv = self.expr(value, Some(&cmp_hint));
                match &fv.ty {
                    Ty::Fn(ps, ret) if ps.len() == 2 && matches!(**ret, Ty::Bool | Ty::Error) => {
                        SortBy::Fn {
                            func: Box::new(fv),
                            is_order: true,
                        }
                    }
                    Ty::Error => return self.err_expr(span),
                    _ => {
                        self.error(
                            codes::BAD_OPERANDS,
                            value.span,
                            "`cmp:` expects a comparator `(a, b) -> Bool`",
                        );
                        return self.err_expr(span);
                    }
                }
            }
            [ast::Arg {
                name: Some(by),
                value:
                    ast::Expr {
                        kind: ast::ExprKind::Lambda { params, body, .. },
                        ..
                    },
                ..
            }] if by.text == "by" && params.len() == 1 => {
                self.cx.scopes.push(HashMap::new());
                let mut pre = Vec::new();
                let slot = self.loop_var(&params[0], elem.clone(), &mut pre);
                let key = self.expr(body, None);
                self.cx.scopes.pop();
                if !pre.is_empty() {
                    self.unsupported(params[0].span, "a pattern as the parameter of a sort key");
                    return self.err_expr(span);
                }
                if key.ty != Ty::Error && (matches!(key.ty, Ty::Opt(_)) || !self.ordered(&key.ty)) {
                    let message = format!("a `{}` has no order to sort by", self.show(&key.ty));
                    self.error(codes::BAD_OPERANDS, body.span, message);
                    return self.err_expr(span);
                }
                SortBy::Key(slot, Box::new(key))
            }
            [ast::Arg {
                name: Some(by),
                value,
                ..
            }] if by.text == "by" => {
                let cmp_hint = Ty::Fn(vec![elem.clone(), elem.clone()], Box::new(Ty::Bool));
                let fv = self.expr(value, Some(&cmp_hint));
                match &fv.ty {
                    Ty::Fn(ps, _) if ps.len() == 2 => SortBy::Fn {
                        func: Box::new(fv),
                        is_order: true,
                    },
                    Ty::Fn(ps, _) if ps.len() == 1 => SortBy::Fn {
                        func: Box::new(fv),
                        is_order: false,
                    },
                    Ty::Struct(sid) => {
                        if let Some((idx, fty @ Ty::Fn(..))) =
                            self.field_of(&Ty::Struct(*sid), "lt")
                        {
                            let lt_fn =
                                self.mk(ExprKind::Field(Box::new(fv), idx), fty, value.span);
                            SortBy::Fn {
                                func: Box::new(lt_fn),
                                is_order: true,
                            }
                        } else {
                            self.error(
                                codes::BAD_OPERANDS,
                                value.span,
                                "`by:` expects a key fn or Order",
                            );
                            return self.err_expr(span);
                        }
                    }
                    _ => {
                        self.error(
                            codes::BAD_OPERANDS,
                            value.span,
                            "`by:` expects a key fn or Order",
                        );
                        return self.err_expr(span);
                    }
                }
            }
            _ if self.sort_reverse_arg(args) => return self.err_expr(span),
            _ => {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    span,
                    "`sort` takes nothing, or a key: `xs.sort(by: x => x.w)`",
                );
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Sort(Box::new(place), key, None), Ty::Unit, span)
    }

    pub(in crate::lower) fn array_filled(
        &mut self,
        index: &[ast::Expr],
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let mut dims = Vec::new();
        for e in index {
            match self.dim(&ast::Type::of_index(e), true) {
                Some(d) => dims.push(d),
                None => return self.err_expr(span),
            }
        }
        let Some(exprs) = self.positional_any("filled", args, 1, span) else {
            return self.err_expr(span);
        };
        let shape: Vec<Dim> = dims.iter().map(DimSpec::dim).collect();
        let value = match hint {
            Some(Ty::Array { dims: want, elem }) if *want == shape => self.expect(exprs[0], elem),
            _ => self.expr(exprs[0], None),
        };
        let ty = Ty::Array {
            dims: shape,
            elem: Box::new(value.ty.clone()),
        };
        self.mk(ExprKind::Fill(dims, Some(Box::new(value))), ty, span)
    }

    /// `V.cuts(k)` (trapping) or `V.cuts.at(k)` (`?V.cuts`): the cut with
    /// `k` positions before it.
    pub(in crate::lower) fn domain_cut(
        &mut self,
        id: DomainId,
        k: Expr,
        trapping: bool,
        span: Span,
    ) -> Expr {
        let slot = self.temp(Ty::Int);
        self.cx.pending.push(Stmt::Let(slot, Some(k)));
        let k = self.local(slot, span);
        let zero = self.int_lit(0, span);
        let below = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(k.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        let above = self.mk(
            ExprKind::Cmp(CmpOp::Gt, Box::new(k.clone()), Box::new(len)),
            Ty::Bool,
            span,
        );
        let outside = self.mk(
            ExprKind::Binary(BinOp::Or, Box::new(below), Box::new(above)),
            Ty::Bool,
            span,
        );
        let cut = self.cast(Cast::Trust, k, Ty::Bound(id));
        if !trapping {
            return self.spec_opt(Some(outside), cut, span);
        }
        let message = format!(
            "that cut is not in `{}.cuts`, which is 0..=`{}.len`",
            self.domains[id], self.domains[id]
        );
        let trap = Stmt::Trap(message, self.span(span));
        self.cx.pending.push(Stmt::If(
            outside,
            Block { stmts: vec![trap] },
            Block::default(),
        ));
        cut
    }

    /// The `?T` a v0.3 `first`/`last` getter returns: `None` when `empty` holds.
    pub(in crate::lower) fn spec_opt(
        &mut self,
        empty: Option<Expr>,
        value: Expr,
        span: Span,
    ) -> Expr {
        let ty = Ty::Opt(Box::new(value.ty.clone()));
        let some = self.mk(ExprKind::Some(Box::new(value)), ty.clone(), span);
        match empty {
            Some(empty) => {
                let none = self.mk(ExprKind::None, ty.clone(), span);
                self.mk(
                    ExprKind::If(Box::new(empty), Box::new(none), Box::new(some)),
                    ty,
                    span,
                )
            }
            None => some,
        }
    }

    pub(in crate::lower) fn domain_field(
        &mut self,
        id: DomainId,
        name: &ast::Name,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if let Some(info) = self.sum_enums.get(&id).cloned() {
            if let Some(tag) = info.variants.iter().position(|(v, _)| *v == name.text) {
                if info.variants[tag].1.is_empty() {
                    return self.encode_sum_variant(id, tag, Vec::new(), span);
                }
                let message = format!(
                    "the variant `{}` holds values: write `{}(...)`",
                    name.text, name.text
                );
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
        }
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        if matches!(name.text.as_str(), "first" | "last") && self.subset_base(id).is_none() {
            let ord = if name.text == "first" {
                self.int_lit(0, span)
            } else {
                self.int_add(len.clone(), -1)
            };
            let pos = self.mk(
                ExprKind::Cast(Cast::FromOrd(id), Box::new(ord)),
                Ty::Pos(id),
                span,
            );
            let zero = self.int_lit(0, span);
            let empty = self.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(len), Box::new(zero)),
                Ty::Bool,
                span,
            );
            return self.spec_opt(Some(empty), pos, span);
        }
        // An empty domain has no first or last position: asking stops the
        // program in the built C++ as it does in `cpl run`.
        if matches!(name.text.as_str(), "first" | "last") && self.subset_base(id).is_none() {
            let zero = self.int_lit(0, span);
            let empty = self.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(len.clone()), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let message = format!(
                "`{}` is empty: it has no {} position",
                self.domains[id], name.text
            );
            let trap = Stmt::Trap(message, self.span(span));
            self.cx.pending.push(Stmt::If(
                empty,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        }
        match name.text.as_str() {
            "len" | "size" | "count" => len,
            "empty" if self.subset_base(id).is_some() => {
                let zero = self.int_lit(0, span);
                self.cast(Cast::Trust, zero, Ty::Pos(id))
            }
            "full" if self.subset_base(id).is_some() => {
                let full = self.int_add(len, -1);
                self.cast(Cast::Trust, full, Ty::Pos(id))
            }
            "first" => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cast(Cast::FromOrd(id), Box::new(zero)),
                    Ty::Pos(id),
                    span,
                )
            }
            "pairs" => self.domain_pairs_list(id, span),
            "last" => {
                let last = self.int_add(len, -1);
                self.mk(
                    ExprKind::Cast(Cast::FromOrd(id), Box::new(last)),
                    Ty::Pos(id),
                    span,
                )
            }
            "start" => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cast(Cast::Trust, Box::new(zero)),
                    Ty::Bound(id),
                    span,
                )
            }
            "end" => self.mk(
                ExprKind::Cast(Cast::Trust, Box::new(len)),
                Ty::Bound(id),
                span,
            ),
            "all" | "range" => {
                if let Some(Ty::Struct(sid)) = hint {
                    if self.is_bitset_of(*sid, id) {
                        return self.domain_bitset_full(id, span);
                    }
                }
                if let Some(Ty::Pos(sub)) = hint {
                    if self.subset_base(*sub) == Some(id) {
                        let sub_len = self.mk(ExprKind::DomainLen(*sub), Ty::Int, span);
                        let full = self.int_add(sub_len, -1);
                        return self.cast(Cast::Trust, full, Ty::Pos(*sub));
                    }
                }
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(len),
                    },
                    Ty::Range(Box::new(Ty::Pos(id))),
                    span,
                )
            }
            "rev" => {
                let range = self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(len),
                    },
                    Ty::Range(Box::new(Ty::Pos(id))),
                    span,
                );
                self.reversed_range(range, span)
            }
            "bounds" | "cuts" => {
                let hi = self.int_add(len, 1);
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(hi),
                    },
                    Ty::Range(Box::new(Ty::Bound(id))),
                    span,
                )
            }
            other => {
                const MEMBERS: [&str; 14] = [
                    "len", "size", "count", "empty", "full", "first", "pairs", "last", "start",
                    "end", "all", "rev", "bounds", "range",
                ];
                match cpl_check::closest(other, MEMBERS.into_iter()) {
                    Some(s) => {
                        let message = format!("a domain has no member `{other}`");
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.suggest_last(name.span, s);
                    }
                    None => {
                        self.unsupported(name.span, &format!("`.{other}` of a domain as a value"))
                    }
                }
                self.err_expr(span)
            }
        }
    }

    /// Recognizes `V.pairs` or `V.pairs()` where `V` is a domain, returning its `DomainId`.
    pub(in crate::lower) fn domain_pairs_iter(&self, iter: &ast::Expr) -> Option<DomainId> {
        match &iter.kind {
            ast::ExprKind::Field { base, name } if name.text == "pairs" => {
                if let Some(Sym::Domain(id)) = self.sym(base) {
                    return Some(id);
                }
            }
            ast::ExprKind::Call { callee, args } if args.is_empty() => {
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if name.text == "pairs" {
                        if let Some(Sym::Domain(id)) = self.sym(base) {
                            return Some(id);
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// `V.ranges` in a `for`: the domain whose non-empty ranges it walks.
    pub(in crate::lower) fn domain_ranges_iter(&self, iter: &ast::Expr) -> Option<DomainId> {
        match &iter.kind {
            ast::ExprKind::Field { base, name } if name.text == "ranges" => match self.sym(base) {
                Some(Sym::Domain(id)) => Some(id),
                _ => None,
            },
            _ => None,
        }
    }

    pub(in crate::lower) fn is_bitset_of(&self, sid: StructId, dom: DomainId) -> bool {
        let Some((gid, gargs)) = self.structs[sid].origin_gen.as_ref() else {
            return false;
        };
        self.gen_structs[*gid].decl.name.text == "Bitset"
            && matches!(
                gargs.as_slice(),
                [GenArg::Domain(d) | GenArg::Ty(Ty::Pos(d))] if *d == dom
            )
    }

    pub(in crate::lower) fn instantiate_bitset(
        &mut self,
        dom: DomainId,
        span: Span,
    ) -> Option<StructId> {
        let Some(Sym::GenStruct(gid)) = self.get_sym("Bitset") else {
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                "`Bitset` is not in scope (import `std.bitset.{Bitset}`)",
            );
            return None;
        };
        Some(self.instantiate_struct(gid, vec![GenArg::Domain(dom)]))
    }

    pub(in crate::lower) fn domain_bitset_full(&mut self, id: DomainId, span: Span) -> Expr {
        let Some(sid) = self.instantiate_bitset(id, span) else {
            return self.err_expr(span);
        };
        let Some(&full_fn) = self.structs[sid].methods.get("full") else {
            return self.err_expr(span);
        };
        self.queue_fn(full_fn);
        self.mk(ExprKind::Call(full_fn, Vec::new()), Ty::Struct(sid), span)
    }

    /// Lowers `a.inverse()` on a 1D array:
    /// - `[D]?E` (partial injection) -> `[E]?D`, trapping if two positions map to the same value;
    /// - `[D]E` (total bijection) -> `[E]D`, trapping if a value is repeated or missing.
    pub(in crate::lower) fn lower_opt_map_or_filter(
        &mut self,
        base: &ast::Expr,
        fn_ast: &ast::Expr,
        is_map: bool,
        span: Span,
    ) -> Expr {
        let opt_val = self.expr(base, None);
        let Ty::Opt(inner_ty) = opt_val.ty.clone() else {
            return self.err_expr(span);
        };
        let opt_slot = self.temp(opt_val.ty.clone());
        let held = self.local(opt_slot, span);
        let is_some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
        let unwrapped = self.mk(
            ExprKind::Unwrap(Box::new(held), false),
            (*inner_ty).clone(),
            span,
        );
        let val_slot = self.temp((*inner_ty).clone());
        let val_expr = self.local(val_slot, span);

        let mut then_stmts = vec![Stmt::Let(val_slot, Some(unwrapped))];
        let applied = if let ast::ExprKind::Lambda { params, body, .. } = &fn_ast.kind {
            if let Some(param) = ast::Pattern::single_param(params) {
                self.cx.scopes.push(HashMap::new());
                let p_slot = self.loop_var(&param, (*inner_ty).clone(), &mut then_stmts);
                let init_p = self.local(val_slot, span);
                then_stmts.push(Stmt::Let(p_slot, Some(init_p)));
                let res = if is_map {
                    self.expr(body, None)
                } else {
                    self.expect(body, &Ty::Bool)
                };
                self.cx.scopes.pop();
                res
            } else {
                self.err_expr(span)
            }
        } else {
            let callee_val = self.expr(fn_ast, None);
            match callee_val.ty.clone() {
                Ty::Fn(_, ret_ty) => self.mk(
                    ExprKind::CallExpr(Box::new(callee_val), vec![Arg::Value(val_expr.clone())]),
                    *ret_ty,
                    span,
                ),
                Ty::Array { elem, .. } => self.mk(
                    ExprKind::Index(Box::new(callee_val), vec![val_expr.clone()]),
                    *elem,
                    span,
                ),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        fn_ast.span,
                        format!(
                            "`{}` expects a function, lambda, or array",
                            if is_map { "map" } else { "filter" }
                        ),
                    );
                    self.err_expr(span)
                }
            }
        };

        let out_ty = if is_map {
            Ty::Opt(Box::new(applied.ty.clone()))
        } else {
            Ty::Opt(inner_ty.clone())
        };
        let res_slot = self.temp(out_ty.clone());
        let none_expr = self.mk(ExprKind::None, out_ty.clone(), span);
        let res_place = Place {
            kind: PlaceKind::Local(res_slot),
            ty: out_ty.clone(),
            span,
        };
        if is_map {
            let some_expr = self.mk(ExprKind::Some(Box::new(applied)), out_ty.clone(), span);
            then_stmts.push(Stmt::Assign(res_place, AssignOp::Set, some_expr));
        } else {
            let some_val = self.mk(ExprKind::Some(Box::new(val_expr)), out_ty.clone(), span);
            then_stmts.push(Stmt::If(
                applied,
                Block {
                    stmts: vec![Stmt::Assign(res_place, AssignOp::Set, some_val)],
                },
                Block::default(),
            ));
        }
        let res_expr = self.local(res_slot, span);
        let stmts = vec![
            Stmt::Let(opt_slot, Some(opt_val)),
            Stmt::Let(res_slot, Some(none_expr)),
            Stmt::If(is_some, Block { stmts: then_stmts }, Block::default()),
            Stmt::Give(res_expr),
        ];
        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
    }

    pub(in crate::lower) fn lower_array_inverse(&mut self, base: &ast::Expr, span: Span) -> Expr {
        let base_ty_ast = if let ast::ExprKind::Name(n) = &base.kind {
            if let Some(slot) = self.lookup(n) {
                self.cx.local_specs.get(&slot).cloned()
            } else if let Some(Sym::Global(gid)) = self.sym(base) {
                self.global_specs.get(&gid).cloned()
            } else {
                None
            }
        } else {
            None
        };
        let arr = self.expr(base, None);
        let Ty::Array { dims, elem } = arr.ty.clone() else {
            return self.err_expr(span);
        };
        if dims.len() != 1 {
            return self.err_expr(span);
        }
        let src_dim = dims[0];
        let (is_partial, val_ty) = match *elem {
            Ty::Opt(inner) => (true, *inner),
            other => (false, other),
        };
        let target_dim_info: Option<(DimSpec, Expr, Expr)> = match &val_ty {
            Ty::Pos(d) => Some((
                DimSpec::Pos(*d),
                self.int_lit(0, span),
                self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            )),
            Ty::Bound(d) => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let one = self.int_lit(1, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one)),
                    Ty::Int,
                    span,
                );
                Some((DimSpec::Bound(*d), self.int_lit(0, span), hi))
            }
            Ty::Enum(e) => Some((
                DimSpec::Enum(*e),
                self.int_lit(0, span),
                self.int_lit(self.enums[*e].variants.len() as i64, span),
            )),
            Ty::Int => {
                let base_spec = base_ty_ast
                    .as_ref()
                    .map(|ty_ast| self.resolve(ty_ast, true));
                let range_spec = base_spec.and_then(|s| match s {
                    Spec::Array { elem: es, .. } => match *es {
                        Spec::Opt(inner, _) if is_partial => Some(*inner),
                        other if !is_partial => Some(other),
                        _ => None,
                    },
                    _ => None,
                });
                match range_spec {
                    Some(Spec::Int {
                        lo: Some(lo_e),
                        hi: Some(hi_e),
                        ..
                    }) => {
                        let one = self.int_lit(1, span);
                        let diff = self.mk(
                            ExprKind::Binary(BinOp::Sub, hi_e.clone(), lo_e.clone()),
                            Ty::Int,
                            span,
                        );
                        let len = self.mk(
                            ExprKind::Binary(BinOp::Add, Box::new(diff), Box::new(one)),
                            Ty::Int,
                            span,
                        );
                        match lo_e.kind {
                            ExprKind::Int(0) => Some((
                                DimSpec::Int(Box::new(len.clone())),
                                self.int_lit(0, span),
                                len,
                            )),
                            ExprKind::Int(c) => {
                                let hi_excl = self.mk(
                                    ExprKind::Binary(
                                        BinOp::Add,
                                        Box::new(self.int_lit(c, span)),
                                        Box::new(len.clone()),
                                    ),
                                    Ty::Int,
                                    span,
                                );
                                Some((
                                    DimSpec::IntFrom(c, Box::new(len)),
                                    self.int_lit(c, span),
                                    hi_excl,
                                ))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        let Some((target_dim, target_lo, target_hi)) = target_dim_info else {
            self.unsupported(
                span,
                "`inverse` on an array whose element type is not an index domain or bounded range",
            );
            return self.err_expr(span);
        };
        let src_idx_ty = match &src_dim {
            Dim::Pos(d) => Ty::Pos(*d),
            Dim::Bound(d) => Ty::Bound(*d),
            Dim::Enum(e) => Ty::Enum(*e),
            Dim::Int | Dim::IntFrom(_) => Ty::Int,
            Dim::Sym(_) => {
                self.unsupported(span, "`inverse` on a symmetric array");
                return self.err_expr(span);
            }
        };
        let inv_ty = Ty::Array {
            dims: vec![target_dim.dim()],
            elem: Box::new(Ty::opt(src_idx_ty.clone())),
        };
        let ret_ty = if is_partial {
            inv_ty.clone()
        } else {
            Ty::Array {
                dims: vec![target_dim.dim()],
                elem: Box::new(src_idx_ty.clone()),
            }
        };
        self.do_expr(Some(&ret_ty), span, |lw, out| {
            let arr_ty = arr.ty.clone();
            let arr_slot = lw.temp(arr_ty.clone());
            out.push(Stmt::Let(arr_slot, Some(arr)));
            let inv_slot = lw.temp(inv_ty.clone());
            out.push(Stmt::Let(
                inv_slot,
                Some(lw.mk(
                    ExprKind::Fill(vec![target_dim.clone()], None),
                    inv_ty.clone(),
                    span,
                )),
            ));
            let (src_lo, src_hi) = match &src_dim {
                Dim::Pos(d) => (
                    lw.int_lit(0, span),
                    lw.mk(ExprKind::DomainLen(*d), Ty::Int, span),
                ),
                Dim::Bound(d) => {
                    let len = lw.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                    let one = lw.int_lit(1, span);
                    (
                        lw.int_lit(0, span),
                        lw.mk(
                            ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one)),
                            Ty::Int,
                            span,
                        ),
                    )
                }
                Dim::Enum(e) => (
                    lw.int_lit(0, span),
                    lw.int_lit(lw.enums[*e].variants.len() as i64, span),
                ),
                Dim::Int => {
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![lw.local(arr_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    (lw.int_lit(0, span), len)
                }
                Dim::IntFrom(c) => {
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![lw.local(arr_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(lw.int_lit(*c, span)), Box::new(len)),
                        Ty::Int,
                        span,
                    );
                    (lw.int_lit(*c, span), hi)
                }
                Dim::Sym(_) => unreachable!(),
            };
            let i_slot = lw.temp(src_idx_ty.clone());
            let lid = lw.open_loop(None);
            let mut body = Vec::new();
            let cell_ty = if is_partial {
                Ty::opt(val_ty.clone())
            } else {
                val_ty.clone()
            };
            let src_idx_for_arr = |lw: &mut Self, slot: Slot| {
                let raw = lw.local(slot, span);
                match src_dim {
                    Dim::IntFrom(c) => lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(raw), Box::new(lw.int_lit(c, span))),
                        Ty::Int,
                        span,
                    ),
                    Dim::Enum(e) => {
                        let ord = lw.cast(Cast::EnumOrd(e), raw, Ty::Int);
                        lw.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => raw,
                }
            };
            let target_dim_copy = target_dim.dim();
            let tgt_idx_for_arr = |lw: &mut Self, slot: Slot| {
                let raw = lw.local(slot, span);
                match target_dim_copy {
                    Dim::IntFrom(c) => lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(raw), Box::new(lw.int_lit(c, span))),
                        Ty::Int,
                        span,
                    ),
                    Dim::Enum(e) => {
                        let ord = lw.cast(Cast::EnumOrd(e), raw, Ty::Int);
                        lw.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => raw,
                }
            };
            let i_idx = src_idx_for_arr(lw, i_slot);
            let cell = lw.mk(
                ExprKind::Index(Box::new(lw.local(arr_slot, span)), vec![i_idx]),
                cell_ty.clone(),
                span,
            );
            let record_value = |lw: &mut Self, v_slot: Slot, stmts: &mut Vec<Stmt>| {
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk(
                    ExprKind::Index(Box::new(lw.local(inv_slot, span)), vec![v_idx.clone()]),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                let is_dup = lw.mk(ExprKind::IsSome(Box::new(existing)), Ty::Bool, span);
                let msg = if is_partial {
                    "inverse requires unique values (duplicate image)"
                } else {
                    "inverse requires a bijection (duplicate image)"
                };
                stmts.push(Stmt::If(
                    is_dup,
                    Block {
                        stmts: vec![Stmt::Trap(msg.to_string(), lw.span(span))],
                    },
                    Block::default(),
                ));
                let inv_place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(inv_slot),
                            ty: inv_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![v_idx],
                    ),
                    ty: Ty::opt(src_idx_ty.clone()),
                    span: lw.span(span),
                };
                let some_i = lw.mk(
                    ExprKind::Some(Box::new(lw.local(i_slot, span))),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                stmts.push(Stmt::Assign(inv_place, AssignOp::Set, some_i));
            };
            if is_partial {
                let cell_slot = lw.temp(cell_ty);
                body.push(Stmt::Let(cell_slot, Some(cell)));
                let has_val = lw.mk(
                    ExprKind::IsSome(Box::new(lw.local(cell_slot, span))),
                    Ty::Bool,
                    span,
                );
                let mut then_stmts = Vec::new();
                let v_slot = lw.temp(val_ty.clone());
                let unwrapped = lw.mk(
                    ExprKind::Unwrap(Box::new(lw.local(cell_slot, span)), false),
                    val_ty.clone(),
                    span,
                );
                then_stmts.push(Stmt::Let(v_slot, Some(unwrapped)));
                record_value(lw, v_slot, &mut then_stmts);
                body.push(Stmt::If(
                    has_val,
                    Block { stmts: then_stmts },
                    Block::default(),
                ));
            } else {
                let v_slot = lw.temp(val_ty.clone());
                body.push(Stmt::Let(v_slot, Some(cell)));
                record_value(lw, v_slot, &mut body);
            }
            lw.cx.loops.pop();
            out.push(Stmt::For(
                lid,
                i_slot,
                Iter::Range {
                    lo: src_lo,
                    hi: src_hi,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block { stmts: body },
            ));
            if is_partial {
                out.push(Stmt::Give(lw.local(inv_slot, span)));
            } else {
                let out_slot = lw.temp(ret_ty.clone());
                out.push(Stmt::Let(
                    out_slot,
                    Some(lw.mk(ExprKind::Fill(vec![target_dim], None), ret_ty.clone(), span)),
                ));
                let v_slot = lw.temp(val_ty.clone());
                let lid2 = lw.open_loop(None);
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk(
                    ExprKind::Index(Box::new(lw.local(inv_slot, span)), vec![v_idx.clone()]),
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                let unwrapped = lw.mk(
                    ExprKind::Unwrap(Box::new(existing), false),
                    src_idx_ty.clone(),
                    span,
                );
                let out_place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(out_slot),
                            ty: ret_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![v_idx],
                    ),
                    ty: src_idx_ty.clone(),
                    span: lw.span(span),
                };
                lw.cx.loops.pop();
                out.push(Stmt::For(
                    lid2,
                    v_slot,
                    Iter::Range {
                        lo: target_lo,
                        hi: target_hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Block {
                        stmts: vec![Stmt::Assign(out_place, AssignOp::Set, unwrapped)],
                    },
                ));
                out.push(Stmt::Give(lw.local(out_slot, span)));
            }
        })
    }

    pub(in crate::lower) fn lower_reach(
        &mut self,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let mut pred_ast: Option<&ast::Expr> = None;
        let mut cyclic_ast: Option<&ast::Expr> = None;
        for arg in args {
            if arg.mutable {
                self.error(
                    codes::WRONG_ARG_COUNT,
                    arg.value.span,
                    "`reach` takes no `mut` arguments",
                );
                return self.err_expr(span);
            }
            match arg.name.as_ref().map(|n| n.text.as_str()) {
                None | Some("while") if pred_ast.is_none() => pred_ast = Some(&arg.value),
                Some("cyclic") if cyclic_ast.is_none() => cyclic_ast = Some(&arg.value),
                Some(other) => {
                    self.error(
                        codes::UNKNOWN_ARG_NAME,
                        arg.value.span,
                        format!("`reach` has no `{other}:` option (expected `while:` and optional `cyclic:`)"),
                    );
                    return self.err_expr(span);
                }
                None => {
                    self.error(
                        codes::WRONG_ARG_COUNT,
                        arg.value.span,
                        "`reach` takes one `while:` predicate and optional `cyclic: Bool`",
                    );
                    return self.err_expr(span);
                }
            }
        }
        let Some(pred_ast) = pred_ast else {
            self.error(
                codes::WRONG_ARG_COUNT,
                span,
                "`reach` requires a `while: (T, T) -> Bool` predicate",
            );
            return self.err_expr(span);
        };
        let (seq_val, elem_ty) = if let Some(Sym::Domain(d)) = self.sym(base) {
            (self.positions(d, base.span), Ty::Pos(d))
        } else {
            let v = self.expr(base, None);
            if matches!(v.ty, Ty::Error) {
                return self.err_expr(span);
            }
            match &v.ty {
                Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e) => {
                    let e = (**e).clone();
                    (v, e)
                }
                Ty::Array { dims, elem } if dims.len() == 1 => {
                    let e = (**elem).clone();
                    (v, e)
                }
                Ty::Str | Ty::StrView => (v, Ty::Char),
                Ty::Range(e) => {
                    let e = (**e).clone();
                    let list = self.walked_list(Iter::RangeVal(v, false), e.clone(), base.span);
                    (list, e)
                }
                Ty::Set(e) | Ty::HashSet(e) => {
                    let e = (**e).clone();
                    let list = self.mk(
                        ExprKind::Builtin(Builtin::Sorted, vec![v]),
                        Ty::list(e.clone()),
                        base.span,
                    );
                    (list, e)
                }
                other => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        base.span,
                        format!("`reach` expects a sequence, not `{}`", self.show(other)),
                    );
                    return self.err_expr(span);
                }
            }
        };
        let fn_hint = Ty::Fn(vec![elem_ty.clone(), elem_ty], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_hint);
        let cyclic = match cyclic_ast {
            Some(c) => self.expect(c, &Ty::Bool),
            None => self.mk(ExprKind::Bool(false), Ty::Bool, span),
        };
        self.mk(
            ExprKind::Builtin(Builtin::Reach, vec![seq_val, pred, cyclic]),
            Ty::list(Ty::Int),
            span,
        )
    }

    pub(in crate::lower) fn lower_array_map(
        &mut self,
        base: &ast::Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let arr_val = self.expr(base, None);
        self.lower_array_map_expr(arr_val, func_ast, span)
    }

    pub(in crate::lower) fn lower_array_map_expr(
        &mut self,
        arr_val: Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let Ty::Array { dims, elem } = arr_val.ty.clone() else {
            return self.err_expr(span);
        };
        let elem_ty = (*elem).clone();
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let arr_slot = self.temp(arr_val.ty.clone());
        let arr = self.local(arr_slot, span);

        let mut axis_lens = Vec::with_capacity(dims.len());
        let mut dim_specs = Vec::with_capacity(dims.len());
        for (axis, dim) in dims.iter().enumerate() {
            let len_expr = if dims.len() == 1 {
                self.mk(
                    ExprKind::Builtin(Builtin::Len, vec![arr.clone()]),
                    Ty::Int,
                    span,
                )
            } else {
                self.mk(
                    ExprKind::Builtin(
                        Builtin::AxisLen,
                        vec![arr.clone(), self.int_lit(axis as i64, span)],
                    ),
                    Ty::Int,
                    span,
                )
            };
            let spec = match *dim {
                Dim::Pos(d) => DimSpec::Pos(d),
                Dim::Bound(d) => DimSpec::Bound(d),
                Dim::Sym(d) => DimSpec::Sym(d),
                Dim::Enum(e) => DimSpec::Enum(e),
                Dim::Int => DimSpec::Int(Box::new(len_expr.clone())),
                Dim::IntFrom(lo) => DimSpec::IntFrom(lo, Box::new(len_expr.clone())),
            };
            axis_lens.push(len_expr);
            dim_specs.push(spec);
        }

        let counters: Vec<Slot> = dims.iter().map(|_| self.temp(Ty::Int)).collect();
        let stored: Vec<Expr> = dims
            .iter()
            .zip(&counters)
            .map(|(dim, &k)| {
                let k_expr = self.local(k, span);
                match *dim {
                    Dim::Pos(d) => self.mk(
                        ExprKind::Cast(Cast::FromOrd(d), Box::new(k_expr)),
                        Ty::Pos(d),
                        span,
                    ),
                    _ => k_expr,
                }
            })
            .collect();
        let cell = self.mk(
            ExprKind::Index(Box::new(arr.clone()), stored.clone()),
            elem_ty.clone(),
            span,
        );

        let mut pre_stmts = vec![Stmt::Let(arr_slot, Some(arr_val))];
        let (out_elem, mut inner_stmts, mapped) =
            if let ast::ExprKind::Lambda { params, body, .. } = &func_ast.kind {
                if let Some(param) = ast::Pattern::single_param(params) {
                    self.cx.scopes.push(HashMap::new());
                    let mut bind_stmts = Vec::new();
                    let item = self.loop_var(&param, elem_ty.clone(), &mut bind_stmts);
                    let mut inner = vec![Stmt::Let(item, Some(cell))];
                    inner.extend(bind_stmts);
                    let mapped = self.expr(body, None);
                    self.cx.scopes.pop();
                    if matches!(mapped.ty, Ty::Error) {
                        return self.err_expr(span);
                    }
                    (mapped.ty.clone(), inner, mapped)
                } else {
                    return self.err_expr(span);
                }
            } else {
                let fn_hint = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Error));
                let func = self.expr(func_ast, Some(&fn_hint));
                let out_elem = match &func.ty {
                    Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
                    Ty::Error => return self.err_expr(span),
                    other => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            func_ast.span,
                            format!("`map` expects a function, not `{}`", self.show(other)),
                        );
                        return self.err_expr(span);
                    }
                };
                let func_slot = self.temp(func.ty.clone());
                pre_stmts.push(Stmt::Let(func_slot, Some(func)));
                let func_val = self.local(func_slot, span);
                let mapped = self.mk(
                    ExprKind::CallExpr(Box::new(func_val), vec![Arg::Value(cell)]),
                    out_elem.clone(),
                    span,
                );
                (out_elem, Vec::new(), mapped)
            };

        let out_ty = Ty::Array {
            dims,
            elem: Box::new(out_elem.clone()),
        };
        let acc = self.temp(out_ty.clone());
        let blank = self.mk(ExprKind::Fill(dim_specs, None), out_ty.clone(), span);
        pre_stmts.push(Stmt::Let(acc, Some(blank)));

        let whole = Place {
            kind: PlaceKind::Local(acc),
            ty: out_ty.clone(),
            span: self.span(span),
        };
        let dst = Place {
            kind: PlaceKind::Index(Box::new(whole), stored),
            ty: out_elem,
            span: self.span(span),
        };
        inner_stmts.push(Stmt::Assign(dst, AssignOp::Set, mapped));

        let mut body = inner_stmts;
        for (len_expr, &k) in axis_lens.into_iter().zip(&counters).rev() {
            let iter = Iter::Range {
                lo: self.int_lit(0, span),
                hi: len_expr,
                step: None,
                rev: false,
                closed: false,
            };
            let lid = self.fresh_loop();
            body = vec![Stmt::For(lid, k, iter, Block { stmts: body })];
        }
        pre_stmts.extend(body);
        pre_stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts: pre_stmts }), out_ty, span)
    }

    /// `let name = xs.map(f)` as a lazy view, which calls `f` on every read (R07.4.9); only an
    /// effect-free `f` (see [`effect_free_lambda`]) may be mapped once when `rest` reads it whole.
    pub(in crate::lower) fn map_view_if_indexed(
        &mut self,
        expr: &ast::Expr,
        name: &str,
        rest: &[ast::Stmt],
        out: &mut Vec<Stmt>,
    ) -> Option<MapView> {
        let effect_free = matches!(&expr.kind, ast::ExprKind::Call { args, .. }
            if args.len() == 1 && effect_free_lambda(&args[0].value));
        if effect_free && !only_indexed(name, rest) {
            return None;
        }
        self.map_view_from_expr(expr, out)
    }

    /// `m.drop(k)` on a map view `m = xs.map(f)` is `xs.drop(k).map(f)`, so `f` runs only on
    /// what the piece keeps (R07.4.9): any library fn that returns a `Slice` or `Rev` of `self`.
    pub(in crate::lower) fn map_view_piece(
        &mut self,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let ast::ExprKind::Name(n) = &base.kind else {
            return None;
        };
        let view = self.cx.map_views.get(&self.lookup(n)?)?.clone();
        let Sym::GenFn(gfid) = self
            .get_sym(&cpl_check::std_method(method))
            .or_else(|| self.get_sym(method))?
        else {
            return None;
        };
        if !piece_of_self(self.gen_fns[gfid].decl) {
            return None;
        }
        let hidden = "\u{0}map view base".to_string();
        let name = ast::Name {
            text: method.to_string(),
            span,
        };
        let receiver = ast::Expr {
            kind: ast::ExprKind::Name(hidden.clone()),
            span: base.span,
        };
        let callee = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(receiver),
                name,
            },
            span,
        };
        let call = ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(callee),
                args: args.to_vec(),
            },
            span,
        };
        self.cx
            .scopes
            .push(HashMap::from([(hidden, view.base_slot)]));
        let piece = self.expr(&call, None);
        self.cx.scopes.pop();
        let elem_ty = match &piece.ty {
            Ty::List(e) | Ty::ListView(e, _) | Ty::MutView(e) | Ty::Keyed(e, _) => (**e).clone(),
            Ty::Str | Ty::StrView => Ty::Char,
            _ => return Some(piece),
        };
        Some(self.map_seq(piece, elem_ty, &view.func, span))
    }

    pub(in crate::lower) fn map_view_from_expr(
        &mut self,
        expr: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<MapView> {
        let ast::ExprKind::Call { callee, args } = &expr.kind else {
            return None;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        if name.text != "map" || args.len() != 1 || args[0].name.is_some() || args[0].mutable {
            return None;
        }
        let base_ty = self.probe_ty_silent(base)?;
        if !matches!(
            base_ty,
            Ty::List(_) | Ty::Array { .. } | Ty::ListView(..) | Ty::MutView(_) | Ty::Str
        ) {
            return None;
        }
        let base_val = self.expr(base, None);
        if matches!(base_val.ty, Ty::Error) {
            return None;
        }
        let base_slot = self.temp(base_val.ty.clone());
        out.push(Stmt::Let(base_slot, Some(base_val)));
        Some(MapView {
            base_slot,
            func: args[0].value.clone(),
            span: expr.span,
        })
    }

    pub(in crate::lower) fn lower_map_view_index(
        &mut self,
        base: &ast::Expr,
        indices: &[ast::Expr],
        span: Span,
    ) -> Option<Expr> {
        if let ast::ExprKind::Name(n) = &base.kind {
            if let Some(slot) = self.lookup(n) {
                if let Some(view) = self.cx.map_views.get(&slot).cloned() {
                    let base_expr = self.local(view.base_slot, span);
                    let elem = self.mapped_elem(base_expr, indices, span);
                    return Some(self.apply_map_fn(elem, &view.func, span));
                }
            }
        }
        if let ast::ExprKind::Call { callee, args } = &base.kind {
            if let ast::ExprKind::Field { base: inner, name } = &callee.kind {
                if name.text == "map"
                    && args.len() == 1
                    && args[0].name.is_none()
                    && !args[0].mutable
                {
                    if let Some(inner_elem) = self.lower_map_view_index(inner, indices, span) {
                        return Some(self.apply_map_fn(inner_elem, &args[0].value, span));
                    }
                    if matches!(
                        self.probe_ty_silent(inner),
                        Some(
                            Ty::List(_)
                                | Ty::Array { .. }
                                | Ty::ListView(..)
                                | Ty::MutView(_)
                                | Ty::Str
                        )
                    ) {
                        let base_expr = self.expr(inner, None);
                        let elem = self.mapped_elem(base_expr, indices, span);
                        return Some(self.apply_map_fn(elem, &args[0].value, span));
                    }
                }
            }
        }

        None
    }

    pub(in crate::lower) fn apply_map_fn(
        &mut self,
        elem: Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        if matches!(elem.ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &func_ast.kind {
            if params.len() == 1 {
                let mut out_ty = Ty::Error;
                let expr = self.do_expr(None, span, |lw, out| {
                    lw.cx.scopes.push(HashMap::new());
                    let mut bind_stmts = Vec::new();
                    let item = lw.loop_var(&params[0], elem.ty.clone(), &mut bind_stmts);
                    out.push(Stmt::Let(item, Some(elem)));
                    out.extend(bind_stmts);
                    let mapped = lw.expr(body, None);
                    out_ty = mapped.ty.clone();
                    lw.cx.scopes.pop();
                    out.push(Stmt::Give(mapped));
                });
                return Expr { ty: out_ty, ..expr };
            }
        }
        let fn_hint = Ty::Fn(vec![elem.ty.clone()], Box::new(Ty::Error));
        let func = self.expr(func_ast, Some(&fn_hint));
        let ret = match &func.ty {
            Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    func_ast.span,
                    format!("`map` expects a function, not `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        self.mk(
            ExprKind::CallExpr(Box::new(func), vec![Arg::Value(elem)]),
            ret,
            span,
        )
    }

    pub(in crate::lower) fn lower_map_view_materialize(
        &mut self,
        view: &MapView,
        span: Span,
    ) -> Expr {
        let base_expr = self.local(view.base_slot, view.span);
        if matches!(base_expr.ty, Ty::Array { .. }) {
            return self.lower_array_map_expr(base_expr, &view.func, span);
        }
        let elem_ty = match &base_expr.ty {
            Ty::List(e) | Ty::ListView(e, _) | Ty::MutView(e) => (**e).clone(),
            Ty::Str => Ty::Char,
            _ => return self.err_expr(span),
        };
        self.map_seq(base_expr, elem_ty, &view.func, span)
    }

    pub(in crate::lower) fn lower_map(
        &mut self,
        base: &ast::Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        match iter {
            Iter::Seq(seq, false) => self.map_seq(seq, elem_ty, func_ast, span),
            iter => self.lower_map_iter(iter, elem_ty, func_ast, span),
        }
    }

    pub(in crate::lower) fn lower_map_iter(
        &mut self,
        iter: Iter,
        elem_ty: Ty,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &func_ast.kind {
            if let Some(param) = ast::Pattern::single_param(params) {
                self.cx.scopes.push(HashMap::new());
                let mut bind_stmts = Vec::new();
                let item = self.loop_var(&param, elem_ty.clone(), &mut bind_stmts);
                let mapped = self.expr(body, None);
                self.cx.scopes.pop();
                if matches!(mapped.ty, Ty::Error) {
                    return self.err_expr(span);
                }
                let out_elem = mapped.ty.clone();
                let list_ty = Ty::list(out_elem);
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let acc = lw.temp(list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push(Stmt::Let(acc, Some(empty)));
                    let push = Stmt::Expr(lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![mapped],
                        ),
                        Ty::Unit,
                        span,
                    ));
                    let mut stmts = bind_stmts;
                    stmts.push(push);
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(lid, item, iter, Block { stmts }));
                    out.push(Stmt::Give(lw.local(acc, span)));
                });
            }
        }
        let fn_hint = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Error));
        let func = self.expr(func_ast, Some(&fn_hint));
        let out_elem = match &func.ty {
            Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    func_ast.span,
                    format!("`map` expects a function, not `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        self.collected(out_elem.clone(), span, |lw, out, push| {
            let func_val = lw.hold(func, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let mapped = lw.mk(
                ExprKind::CallExpr(Box::new(func_val), vec![Arg::Value(item_expr)]),
                out_elem.clone(),
                span,
            );
            let push_stmt = push(lw, mapped);
            out.push(Stmt::For(
                lid,
                item,
                iter,
                Block {
                    stmts: vec![push_stmt],
                },
            ));
        })
    }

    pub(in crate::lower) fn lower_filter(
        &mut self,
        base: &ast::Expr,
        test: &ast::Expr,
        span: Span,
    ) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &test.kind {
            if let Some(param) = ast::Pattern::single_param(params) {
                return self.collected(elem_ty.clone(), span, |lw, out, push| {
                    lw.cx.scopes.push(HashMap::new());
                    let mut bind_stmts = Vec::new();
                    let item = lw.loop_var(&param, elem_ty.clone(), &mut bind_stmts);
                    let keep = lw.expect(body, &Ty::Bool);
                    lw.cx.scopes.pop();
                    let lid = lw.fresh_loop();
                    let item_expr = lw.local(item, span);
                    let push_stmt = push(lw, item_expr);
                    bind_stmts.push(Stmt::If(
                        keep,
                        Block {
                            stmts: vec![push_stmt],
                        },
                        Block::default(),
                    ));
                    out.push(Stmt::For(lid, item, iter, Block { stmts: bind_stmts }));
                });
            }
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(test, &fn_ty);
        self.collected(elem_ty.clone(), span, |lw, out, push| {
            let pred_val = lw.hold(pred, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let keep = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let push_stmt = push(lw, item_expr);
            let body = Block {
                stmts: vec![Stmt::If(
                    keep,
                    Block {
                        stmts: vec![push_stmt],
                    },
                    Block::default(),
                )],
            };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    pub(in crate::lower) fn lower_partition(
        &mut self,
        base: &ast::Expr,
        pred_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_ty);
        let list_ty = Ty::list(elem_ty.clone());
        let pair_ty = Ty::Tuple(vec![list_ty.clone(), list_ty.clone()]);
        self.do_expr(Some(&pair_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let yes = lw.temp(list_ty.clone());
            let no = lw.temp(list_ty.clone());
            let empty_yes = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            let empty_no = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(yes, Some(empty_yes)));
            out.push(Stmt::Let(no, Some(empty_no)));
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let push_yes = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(yes),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![item_expr.clone()],
                ),
                Ty::Unit,
                span,
            );
            let push_no = lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(no),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![item_expr],
                ),
                Ty::Unit,
                span,
            );
            let body = Block {
                stmts: vec![Stmt::If(
                    cond,
                    Block {
                        stmts: vec![Stmt::Expr(push_yes)],
                    },
                    Block {
                        stmts: vec![Stmt::Expr(push_no)],
                    },
                )],
            };
            out.push(Stmt::For(lid, item, iter, body));
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(yes, span), lw.local(no, span)]),
                pair_ty.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }
}

/// Every read of `name` in `rest` is `name[i]`, `name.len` or `name.is_empty`, which a lazy
/// `MapView` answers in O(1); any other read (`.first`, a loop, a call) would map the whole
/// list again each time, so the `let` maps it once instead.
fn only_indexed(name: &str, rest: &[ast::Stmt]) -> bool {
    struct Reads<'n> {
        name: &'n str,
        all: usize,
        cheap: usize,
    }
    impl cpl_syntax::visit::Visitor for Reads<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            let named = |b: &ast::Expr| matches!(&b.kind, ast::ExprKind::Name(n) if n == self.name);
            match &e.kind {
                ast::ExprKind::Name(n) if n == self.name => self.all += 1,
                ast::ExprKind::Index { base, .. } if named(base) => self.cheap += 1,
                ast::ExprKind::Field { base, name }
                    if named(base) && matches!(name.text.as_str(), "len" | "is_empty") =>
                {
                    self.cheap += 1
                }
                _ => {}
            }
        }
    }
    let mut reads = Reads {
        name,
        all: 0,
        cheap: 0,
    };
    for stmt in rest {
        cpl_syntax::visit::walk_stmt(stmt, &mut reads);
    }
    reads.all == reads.cheap
}

/// A lambda whose body has no calls, writes or I/O and cannot trap (no arithmetic, index or
/// `!`), so calling it once per element or once per read gives the same run.
fn effect_free_lambda(func: &ast::Expr) -> bool {
    struct Quiet(bool);
    impl cpl_syntax::visit::Visitor for Quiet {
        fn expr(&mut self, e: &ast::Expr) {
            self.0 &= matches!(
                e.kind,
                ast::ExprKind::Int(_)
                    | ast::ExprKind::Char(_)
                    | ast::ExprKind::Bool(_)
                    | ast::ExprKind::None
                    | ast::ExprKind::Name(_)
                    | ast::ExprKind::SelfValue
                    | ast::ExprKind::Paren(_)
                    | ast::ExprKind::Tuple(_)
                    | ast::ExprKind::Field { .. }
                    | ast::ExprKind::Compare { .. }
            );
        }
    }
    let ast::ExprKind::Lambda { body, .. } = &func.kind else {
        return false;
    };
    let mut quiet = Quiet(true);
    cpl_syntax::visit::walk_expr(body, &mut quiet);
    quiet.0
}

/// `fn drop<S: Indexed>(self: S, ..) -> Slice<S>`: a piece of the receiver, in some order.
fn piece_of_self(decl: &ast::FnDecl) -> bool {
    fn named(ty: Option<&ast::Type>) -> Option<(&str, &[ast::Type])> {
        match ty.map(|t| &t.kind) {
            Some(ast::TypeKind::Named { name, args }) => Some((name.text.as_str(), args)),
            _ => None,
        }
    }
    let Some(first) = decl.params.first().filter(|p| p.name.text == "self") else {
        return false;
    };
    let Some((s, _)) = named(first.ty.as_ref()) else {
        return false;
    };
    matches!(named(decl.ret.as_ref()), Some(("Slice" | "Rev", args))
        if args.len() == 1 && named(Some(&args[0])).is_some_and(|(a, _)| a == s)
            && decl.generics.iter().any(|g| g.text == s))
}
