//! Sorting, domain members, and the seq adapters map, filter and partition.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// `xs.sort()`, `xs.sort(by: x => key)`, `xs.sort(by: (a, b) => a < b)`, `xs.sort(by: ord)`.
    pub(in crate::lower) fn sort(
        &mut self,
        base: &ast::Expr,
        elem: &Ty,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let place = self.mut_receiver(base, "sort");
        self.sort_place(place, elem, args, span)
    }

    pub(in crate::lower) fn sort_place(
        &mut self,
        place: Place,
        elem: &Ty,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
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
            self.report(span, msgs::ir::BAD_OPERANDS_SORT_TAKES_EITHER_BY, &[]);
            return self.err_expr(span);
        }
        let key = match args {
            [] => {
                if !self.ordered(elem) {
                    self.report(
                        span,
                        msgs::ir::BAD_OPERANDS_A_HAS_NO_ORDER_4,
                        &[&self.show(elem)],
                    );
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
                        self.report(
                            value.span,
                            msgs::ir::BAD_OPERANDS_CMP_EXPECTS_A_COMPARATOR,
                            &[],
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
                if key.ty != Ty::Error && (matches!(key.ty, Ty::Opt(_)) || !self.ordered(&key.ty)) {
                    self.report(
                        body.span,
                        msgs::ir::BAD_OPERANDS_A_HAS_NO_ORDER_4,
                        &[&self.show(&key.ty)],
                    );
                    return self.err_expr(span);
                }
                let (key, turned) = match self.reversed_key(&key) {
                    Some(inner) => (inner, true),
                    None => (key, false),
                };
                // `((a, b)) => -b`: the key unpacks the element first.
                let key = if pre.is_empty() {
                    key
                } else {
                    let ty = key.ty.clone();
                    pre.push(Stmt::Give(key));
                    self.mk(ExprKind::Do(Block { stmts: pre }), ty, body.span)
                };
                if turned {
                    let turned = self.mk(ExprKind::Bool(true), Ty::Bool, body.span);
                    let by = SortBy::Key(slot, Box::new(key));
                    let kind = ExprKind::Sort(Box::new(place), by, Some(Box::new(turned)));
                    return self.mk(kind, Ty::Unit, span);
                }
                SortBy::Key(slot, Box::new(key))
            }
            [ast::Arg {
                name: Some(by),
                value:
                    ast::Expr {
                        kind: ast::ExprKind::Lambda { params, .. },
                        ..
                    },
                ..
            }] if by.text == "by" && params.len() > 1 => {
                let (first, last) = (&params[0], &params[params.len() - 1]);
                self.report(
                    first.span.to(last.span),
                    msgs::ir::WRONG_ARG_COUNT_SORT_BY_TAKES_A,
                    &[&params.len()],
                );
                let key = match &elem {
                    Ty::Record(fields) => format!(
                        "x => ({})",
                        fields
                            .iter()
                            .map(|(f, _)| format!("x.{f}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    _ => "x => x".to_string(),
                };
                self.help(format!(
                    "take one element and return its key, e.g. `{key}`; wrap a part in `desc(..)` to reverse it"
                ));
                return self.err_expr(span);
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
                            let lt_fn = self.mk_field(fv, idx, fty, value.span);
                            SortBy::Fn {
                                func: Box::new(lt_fn),
                                is_order: true,
                            }
                        } else {
                            self.report(value.span, msgs::ir::BAD_OPERANDS_BY_EXPECTS_A_KEY, &[]);
                            return self.err_expr(span);
                        }
                    }
                    _ => {
                        self.report(value.span, msgs::ir::BAD_OPERANDS_BY_EXPECTS_A_KEY, &[]);
                        return self.err_expr(span);
                    }
                }
            }
            _ if self.sort_reverse_arg(args) => return self.err_expr(span),
            _ if self.sort_unknown_arg(args) => return self.err_expr(span),
            _ => {
                self.report(span, msgs::ir::WRONG_ARG_COUNT_SORT_TAKES_NOTHING_OR, &[]);
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Sort(Box::new(place), key, None), Ty::Unit, span)
    }
    /// `by: x => w(k)`, where `w` wraps its one argument in a one-field struct whose `Ord` asks
    /// that field the other way round (`lt(a, b) = b.f < a.f`, as std's `desc` does): the same
    /// stable order is `k` sorted descending, without a wrapper per element.
    fn reversed_key(&self, key: &Expr) -> Option<Expr> {
        let ExprKind::Call(fid, args) = &key.kind else {
            return None;
        };
        let [Arg::Value(inner)] = &args[..] else {
            return None;
        };
        let Ty::Struct(sid) = key.ty else {
            return None;
        };
        let gid = self.structs[sid].origin_gen.as_ref()?.0;
        let tpl = &self.gen_structs[gid];
        let [field] = &tpl.decl.fields[..] else {
            return None;
        };
        let wrap = self.sigs[*fid].decl;
        let ([param], ast::FnBody::Expr(made)) = (&wrap.params[..], &wrap.body) else {
            return None;
        };
        let is_name = |e: &ast::Expr, n: &str| matches!(&e.kind, ast::ExprKind::Name(m) if m == n);
        let field_of = |e: &ast::Expr, n: &str| matches!(&e.kind, ast::ExprKind::Field { base, name } if is_name(base, n) && name.text == field.name.text);
        let ast::ExprKind::Call { callee, args } = &made.kind else {
            return None;
        };
        let wraps = is_name(callee, &tpl.decl.name.text)
            && matches!(&args[..], [arg] if arg.name.is_none() && is_name(&arg.value, &param.name.text));
        let reversed = self.impls.iter().any(|imp| {
            imp.mod_id == tpl.mod_id
                && imp.trait_name.as_deref() == Some("Ord")
                && matches!(&imp.target.kind, ast::TypeKind::Named { name, .. } if name.text == tpl.decl.name.text)
                && imp.body.stmts.iter().any(|st| match &st.kind {
                    ast::StmtKind::Fn(lt) => match (&lt.params[..], &lt.body) {
                        ([a, b], ast::FnBody::Expr(e)) if lt.name.text == "lt" => matches!(
                            &e.kind,
                            ast::ExprKind::Compare { first, rest }
                                if field_of(first, &b.name.text)
                                    && matches!(&rest[..], [(ast::CmpOp::Lt, r)] if field_of(r, &a.name.text))
                        ),
                        _ => false,
                    },
                    _ => false,
                })
        });
        (wraps && reversed).then(|| inner.clone())
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
            Some(Ty::Array { dims: want, elem }) if *want == shape => self.expr_as(exprs[0], elem),
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
        self.cx.pending.push_let(slot, k);
        let k = self.local(slot, span);
        let zero = self.int_lit(0, span);
        let below = self.mk_cmp(CmpOp::Lt, k.clone(), zero, Ty::Bool, span);
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        let above = self.mk_cmp(CmpOp::Gt, k.clone(), len, Ty::Bool, span);
        let outside = self.mk_binary(BinOp::Or, below, above, Ty::Bool, span);
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
        let ty = Ty::opt(value.ty.clone());
        let some = self.mk_some(value, ty.clone(), span);
        match empty {
            Some(empty) => {
                let none = self.mk(ExprKind::None, ty.clone(), span);
                self.mk_if(empty, none, some, ty, span)
            }
            None => some,
        }
    }

    pub(in crate::lower) fn domain_field(
        &mut self,
        id: DomainId,
        name: &ast::Name,
        span: Span,
    ) -> Expr {
        if let Some(info) = self.sum_enums.get(&id).cloned() {
            if let Some(tag) = info.variants.iter().position(|(v, _)| *v == name.text) {
                if info.variants[tag].1.is_empty() {
                    return self.encode_sum_variant(id, tag, Vec::new(), span);
                }
                self.report(
                    span,
                    msgs::ir::WRONG_ARG_COUNT_THE_VARIANT_HOLDS_VALUES,
                    &[&name.text, &name.text],
                );
                return self.err_expr(span);
            }
        }
        let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
        if matches!(name.text.as_str(), "first" | "last") {
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
            let empty = self.mk_cmp(CmpOp::Le, len, zero, Ty::Bool, span);
            return self.spec_opt(Some(empty), pos, span);
        }
        // An empty domain has no first or last position: asking stops the
        // program in the built C++ as it does in `cpl run`.
        if matches!(name.text.as_str(), "first" | "last") {
            let zero = self.int_lit(0, span);
            let empty = self.mk_cmp(CmpOp::Le, len.clone(), zero, Ty::Bool, span);
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
            "all" | "range" => self.mk(
                ExprKind::Range {
                    lo: Box::new(self.int_lit(0, span)),
                    hi: Box::new(len),
                },
                Ty::range(Ty::Pos(id)),
                span,
            ),
            "rev" => {
                let range = self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(len),
                    },
                    Ty::range(Ty::Pos(id)),
                    span,
                );
                self.reversed_range(range, Ty::Pos(id), span)
            }
            "bounds" | "cuts" => {
                let hi = self.int_add(len, 1);
                self.mk(
                    ExprKind::Range {
                        lo: Box::new(self.int_lit(0, span)),
                        hi: Box::new(hi),
                    },
                    Ty::range(Ty::Bound(id)),
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
                        self.report(
                            name.span,
                            msgs::ir::UNKNOWN_FIELD_A_DOMAIN_HAS_NO,
                            &[&other],
                        );
                        self.suggest_last(name.span, s);
                    }
                    None => {
                        self.report(
                            name.span,
                            msgs::ir::UNKNOWN_FIELD_A_DOMAIN_HAS_NO,
                            &[&other],
                        );
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
        let is_some = self.mk_is_some(held.clone(), Ty::Bool, span);
        let unwrapped = self.mk_unwrap(held, false, (*inner_ty).clone(), span);
        let val_slot = self.temp((*inner_ty).clone());
        let val_expr = self.local(val_slot, span);

        let mut then_stmts = vec![Stmt::Let(val_slot, Some(unwrapped))];
        let applied = if let ast::ExprKind::Lambda { params, body, .. } = &fn_ast.kind {
            if let Some(param) = ast::Pattern::single_param(params) {
                self.cx.scopes.push(HashMap::new());
                let p_slot = self.loop_var(&param, (*inner_ty).clone(), &mut then_stmts);
                let init_p = self.local(val_slot, span);
                then_stmts.push_let(p_slot, init_p);
                let res = if is_map {
                    self.expr(body, None)
                } else {
                    self.expr_as(body, &Ty::Bool)
                };
                self.cx.scopes.pop();
                res
            } else {
                self.err_expr(span)
            }
        } else {
            let callee_val = self.expr(fn_ast, None);
            match callee_val.ty.clone() {
                Ty::Fn(_, ret_ty) => self.mk_call_expr(
                    callee_val,
                    vec![Arg::Value(val_expr.clone())],
                    *ret_ty,
                    span,
                ),
                Ty::Array { elem, .. } => {
                    self.mk_index(callee_val, vec![val_expr.clone()], *elem, span)
                }
                _ => {
                    self.report(
                        fn_ast.span,
                        msgs::ir::TYPE_MISMATCH_EXPECTS_A_FUNCTION_LAMBDA,
                        &[&if is_map { "map" } else { "filter" }],
                    );
                    self.err_expr(span)
                }
            }
        };

        let out_ty = if is_map {
            Ty::opt(applied.ty.clone())
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
            let some_expr = self.mk_some(applied, out_ty.clone(), span);
            then_stmts.push(Stmt::Assign(res_place, AssignOp::Set, some_expr));
        } else {
            let some_val = self.mk_some(val_expr, out_ty.clone(), span);
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
                let hi = self.mk_binary(BinOp::Add, len, one, Ty::Int, span);
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
                        let len = self.mk_binary(BinOp::Add, diff, one, Ty::Int, span);
                        match lo_e.kind {
                            ExprKind::Int(0) => Some((
                                DimSpec::Int(Box::new(len.clone())),
                                self.int_lit(0, span),
                                len,
                            )),
                            ExprKind::Int(c) => {
                                let hi_excl = self.mk_binary(
                                    BinOp::Add,
                                    self.int_lit(c, span),
                                    len.clone(),
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
        // The source's cells, counted along one line: a symmetric table has none.
        enum Line {
            Pos(DomainId),
            Bound(DomainId),
            Enum(EnumId),
            Int,
            IntFrom(i64),
        }
        let (src_idx_ty, src_line) = match src_dim {
            Dim::Pos(d) => (Ty::Pos(d), Line::Pos(d)),
            Dim::Bound(d) => (Ty::Bound(d), Line::Bound(d)),
            Dim::Enum(e) => (Ty::Enum(e), Line::Enum(e)),
            Dim::Int => (Ty::Int, Line::Int),
            Dim::IntFrom(c) => (Ty::Int, Line::IntFrom(c)),
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
        // A bijection onto an index type is written straight into its result:
        // one below the first index marks a value not yet seen.
        let direct = !is_partial && matches!(src_idx_ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_));
        let src_first = match src_dim {
            Dim::IntFrom(c) => c,
            _ => 0,
        };
        let (table_ty, table_elem) = if direct {
            (ret_ty.clone(), src_idx_ty.clone())
        } else {
            (inv_ty.clone(), Ty::opt(src_idx_ty.clone()))
        };
        self.do_expr(Some(&ret_ty), span, |lw, out| {
            let arr_ty = arr.ty.clone();
            let arr_slot = lw.temp(arr_ty.clone());
            out.push_let(arr_slot, arr);
            let inv_slot = lw.temp(table_ty.clone());
            let unseen = match src_first.checked_sub(1) {
                Some(below) if direct => {
                    let below = lw.int_lit(below, span);
                    Some(match src_idx_ty {
                        Ty::Int => below,
                        _ => lw.cast(Cast::Trust, below, src_idx_ty.clone()),
                    })
                }
                _ => None,
            };
            if direct && src_idx_ty == Ty::Int {
                // Each index stored is below `src_first` plus the target's
                // length: past that, two of them would share an image.
                let target_first = match target_lo.kind {
                    ExprKind::Int(c) => Some(c),
                    _ => None,
                };
                if let Some(shift) =
                    target_first.and_then(|t| src_first.checked_sub(t)?.checked_sub(1))
                {
                    let hi = lw.int_add(target_hi.clone(), shift);
                    let bounds = super::super::range::RangeBounds {
                        lo: None,
                        lo_const: src_first.checked_sub(1),
                        hi_const: const_int_expr(&hi),
                        lo_min: src_first.checked_sub(1),
                        hi_max: lw.bound_limits(&hi).map(|l| l.1),
                        hi: Some(hi),
                        text: "an index".to_string(),
                    };
                    lw.cx.local_ranges.insert(
                        inv_slot,
                        super::super::range::RangeInfo::Elem(Box::new(
                            super::super::range::RangeInfo::Scalar(bounds),
                        )),
                    );
                }
            }
            out.push_let(
                inv_slot,
                lw.mk(
                    ExprKind::Fill(vec![target_dim.clone()], unseen.clone().map(Box::new)),
                    table_ty.clone(),
                    span,
                ),
            );
            let (src_lo, src_hi) = match src_line {
                Line::Pos(d) => (
                    lw.int_lit(0, span),
                    lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                ),
                Line::Bound(d) => {
                    let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let one = lw.int_lit(1, span);
                    (
                        lw.int_lit(0, span),
                        lw.mk_binary(BinOp::Add, len, one, Ty::Int, span),
                    )
                }
                Line::Enum(e) => (
                    lw.int_lit(0, span),
                    lw.int_lit(lw.enums[e].variants.len() as i64, span),
                ),
                Line::Int => {
                    let len =
                        lw.mk_builtin(Builtin::Len, [lw.local(arr_slot, span)], Ty::Int, span);
                    (lw.int_lit(0, span), len)
                }
                Line::IntFrom(c) => {
                    let len =
                        lw.mk_builtin(Builtin::Len, [lw.local(arr_slot, span)], Ty::Int, span);
                    let hi = lw.mk_binary(BinOp::Add, lw.int_lit(c, span), len, Ty::Int, span);
                    (lw.int_lit(c, span), hi)
                }
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
                    Dim::IntFrom(c) => {
                        lw.mk_binary(BinOp::Sub, raw, lw.int_lit(c, span), Ty::Int, span)
                    }
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
                    Dim::IntFrom(c) => {
                        lw.mk_binary(BinOp::Sub, raw, lw.int_lit(c, span), Ty::Int, span)
                    }
                    Dim::Enum(e) => {
                        let ord = lw.cast(Cast::EnumOrd(e), raw, Ty::Int);
                        lw.cast(Cast::Raw, ord, Ty::Int)
                    }
                    _ => raw,
                }
            };
            let i_idx = src_idx_for_arr(lw, i_slot);
            let cell = lw.mk_index(lw.local(arr_slot, span), vec![i_idx], cell_ty.clone(), span);
            let record_value = |lw: &mut Self, v_slot: Slot, stmts: &mut Vec<Stmt>| {
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk_index(
                    lw.local(inv_slot, span),
                    vec![v_idx.clone()],
                    table_elem.clone(),
                    span,
                );
                let is_dup = match &unseen {
                    Some(unseen) => lw.mk_cmp(CmpOp::Ne, existing, unseen.clone(), Ty::Bool, span),
                    None => lw.mk_is_some(existing, Ty::Bool, span),
                };
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
                            ty: table_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![v_idx],
                    ),
                    ty: table_elem.clone(),
                    span: lw.span(span),
                };
                let i = lw.local(i_slot, span);
                let stored = if unseen.is_some() {
                    i
                } else {
                    lw.mk_some(i, Ty::opt(src_idx_ty.clone()), span)
                };
                stmts.push(Stmt::Assign(inv_place, AssignOp::Set, stored));
            };
            if is_partial {
                let cell_slot = lw.temp(cell_ty);
                body.push_let(cell_slot, cell);
                let has_val = lw.mk_is_some(lw.local(cell_slot, span), Ty::Bool, span);
                let mut then_stmts = Vec::new();
                let v_slot = lw.temp(val_ty.clone());
                let unwrapped =
                    lw.mk_unwrap(lw.local(cell_slot, span), false, val_ty.clone(), span);
                then_stmts.push_let(v_slot, unwrapped);
                record_value(lw, v_slot, &mut then_stmts);
                body.push(Stmt::If(
                    has_val,
                    Block { stmts: then_stmts },
                    Block::default(),
                ));
            } else {
                let v_slot = lw.temp(val_ty.clone());
                body.push_let(v_slot, cell);
                record_value(lw, v_slot, &mut body);
            }
            lw.cx.loops.pop();
            let src_len = lw.mk_binary(BinOp::Sub, src_hi.clone(), src_lo.clone(), Ty::Int, span);
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
            if unseen.is_some() {
                // No index repeats an image, so the images are all of the
                // target exactly when there are as many indices as targets.
                let target_len = lw.mk_binary(BinOp::Sub, target_hi, target_lo, Ty::Int, span);
                let short = lw.mk_cmp(CmpOp::Ne, src_len, target_len, Ty::Bool, span);
                out.push(Stmt::If(
                    short,
                    Block {
                        stmts: vec![Stmt::Trap(
                            "inverse requires a bijection (missing image)".to_string(),
                            lw.span(span),
                        )],
                    },
                    Block::default(),
                ));
                out.push_give(lw.local(inv_slot, span));
            } else if is_partial {
                out.push_give(lw.local(inv_slot, span));
            } else {
                let out_slot = lw.temp(ret_ty.clone());
                out.push_let(
                    out_slot,
                    lw.mk(ExprKind::Fill(vec![target_dim], None), ret_ty.clone(), span),
                );
                let v_slot = lw.temp(val_ty.clone());
                let lid2 = lw.open_loop(None);
                let v_idx = tgt_idx_for_arr(lw, v_slot);
                let existing = lw.mk_index(
                    lw.local(inv_slot, span),
                    vec![v_idx.clone()],
                    Ty::opt(src_idx_ty.clone()),
                    span,
                );
                let unwrapped = lw.mk_unwrap(existing, false, src_idx_ty.clone(), span);
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
                out.push_give(lw.local(out_slot, span));
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
                self.report(
                    arg.value.span,
                    msgs::ir::WRONG_ARG_COUNT_REACH_TAKES_NO_MUT,
                    &[],
                );
                return self.err_expr(span);
            }
            match arg.name.as_ref().map(|n| n.text.as_str()) {
                None | Some("while") if pred_ast.is_none() => pred_ast = Some(&arg.value),
                Some("cyclic") if cyclic_ast.is_none() => cyclic_ast = Some(&arg.value),
                Some(other) => {
                    self.report(
                        arg.value.span,
                        msgs::ir::UNKNOWN_ARG_NAME_REACH_HAS_NO_OPTION,
                        &[&other],
                    );
                    return self.err_expr(span);
                }
                None => {
                    self.report(
                        arg.value.span,
                        msgs::ir::WRONG_ARG_COUNT_REACH_TAKES_ONE_WHILE,
                        &[],
                    );
                    return self.err_expr(span);
                }
            }
        }
        let Some(pred_ast) = pred_ast else {
            self.report(span, msgs::ir::WRONG_ARG_COUNT_REACH_REQUIRES_A_WHILE, &[]);
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
                Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => {
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
                    let list =
                        self.walked_list(Iter::RangeVal(v, false).into(), e.clone(), base.span);
                    (list, e)
                }
                Ty::Set(e) | Ty::HashSet(e) => {
                    let e = (**e).clone();
                    let list =
                        self.mk_builtin(Builtin::Sorted, [v], Ty::list(e.clone()), base.span);
                    (list, e)
                }
                other => {
                    self.report(
                        base.span,
                        msgs::ir::TYPE_MISMATCH_REACH_EXPECTS_A_SEQUENCE,
                        &[&self.show(other)],
                    );
                    return self.err_expr(span);
                }
            }
        };
        let fn_hint = Ty::Fn(vec![elem_ty.clone(), elem_ty], Box::new(Ty::Bool));
        let pred = self.expr_as(pred_ast, &fn_hint);
        let cyclic = match cyclic_ast {
            Some(c) => self.expr_as(c, &Ty::Bool),
            None => self.mk(ExprKind::Bool(false), Ty::Bool, span),
        };
        self.mk_builtin(
            Builtin::Reach,
            [seq_val, pred, cyclic],
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
                self.mk_builtin(Builtin::Len, [arr.clone()], Ty::Int, span)
            } else {
                self.mk_builtin(
                    Builtin::AxisLen,
                    [arr.clone(), self.int_lit(axis as i64, span)],
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
        let cell = self.mk_index(arr.clone(), stored.clone(), elem_ty.clone(), span);

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
                        self.report(
                            func_ast.span,
                            msgs::ir::TYPE_MISMATCH_MAP_EXPECTS_A_FUNCTION,
                            &[&self.show(other)],
                        );
                        return self.err_expr(span);
                    }
                };
                let func_slot = self.temp(func.ty.clone());
                pre_stmts.push_let(func_slot, func);
                let func_val = self.local(func_slot, span);
                let mapped =
                    self.mk_call_expr(func_val, vec![Arg::Value(cell)], out_elem.clone(), span);
                (out_elem, Vec::new(), mapped)
            };

        let out_ty = Ty::Array {
            dims,
            elem: Box::new(out_elem.clone()),
        };
        let acc = self.temp(out_ty.clone());
        let blank = self.mk(ExprKind::Fill(dim_specs, None), out_ty.clone(), span);
        pre_stmts.push_let(acc, blank);

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
        pre_stmts.push_give(self.local(acc, span));
        self.mk(ExprKind::Do(Block { stmts: pre_stmts }), out_ty, span)
    }

    pub(in crate::lower) fn lower_map(
        &mut self,
        base: &ast::Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        match iter {
            Head::Iter(Iter::Seq(seq, false)) => self.map_seq(seq, elem_ty, func_ast, span),
            iter => self.lower_map_iter(iter, elem_ty, func_ast, span),
        }
    }

    pub(in crate::lower) fn lower_map_iter(
        &mut self,
        iter: Head,
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
                    out.push_let(acc, empty);
                    let push = Stmt::Expr(lw.mk_mutate(
                        Mutation::Push,
                        Place {
                            kind: PlaceKind::Local(acc),
                            ty: list_ty.clone(),
                            span: lw.span(span),
                        },
                        vec![mapped],
                        Ty::Unit,
                        span,
                    ));
                    let mut stmts = bind_stmts;
                    stmts.push(push);
                    let lid = lw.fresh_loop();
                    lw.push_for(out, lid, item, iter, Block { stmts });
                    out.push_give(lw.local(acc, span));
                });
            }
        }
        let fn_hint = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Error));
        let func = self.expr(func_ast, Some(&fn_hint));
        let out_elem = match &func.ty {
            Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
            Ty::Error => return self.err_expr(span),
            other => {
                self.report(
                    func_ast.span,
                    msgs::ir::TYPE_MISMATCH_MAP_EXPECTS_A_FUNCTION,
                    &[&self.show(other)],
                );
                return self.err_expr(span);
            }
        };
        self.collected(out_elem.clone(), span, |lw, out, push| {
            let func_val = lw.hold(func, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let mapped = lw.mk_call_expr(
                func_val,
                vec![Arg::Value(item_expr)],
                out_elem.clone(),
                span,
            );
            let push_stmt = push(lw, mapped);
            lw.push_for(
                out,
                lid,
                item,
                iter,
                Block {
                    stmts: vec![push_stmt],
                },
            );
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
                    let keep = lw.expr_as(body, &Ty::Bool);
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
                    lw.push_for(out, lid, item, iter, Block { stmts: bind_stmts });
                });
            }
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expr_as(test, &fn_ty);
        self.collected(elem_ty.clone(), span, |lw, out, push| {
            let pred_val = lw.hold(pred, out);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let keep = lw.mk_call_expr(
                pred_val,
                vec![Arg::Value(item_expr.clone())],
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
            lw.push_for(out, lid, item, iter, body);
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
        let pred = self.expr_as(pred_ast, &fn_ty);
        let list_ty = Ty::list(elem_ty.clone());
        let pair_ty = Ty::Tuple(vec![list_ty.clone(), list_ty.clone()]);
        self.do_expr(Some(&pair_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let yes = lw.temp(list_ty.clone());
            let no = lw.temp(list_ty.clone());
            let empty_yes = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            let empty_no = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push_let(yes, empty_yes);
            out.push_let(no, empty_no);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk_call_expr(
                pred_val,
                vec![Arg::Value(item_expr.clone())],
                Ty::Bool,
                span,
            );
            let push_yes = lw.mk_mutate(
                Mutation::Push,
                Place {
                    kind: PlaceKind::Local(yes),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                },
                vec![item_expr.clone()],
                Ty::Unit,
                span,
            );
            let push_no = lw.mk_mutate(
                Mutation::Push,
                Place {
                    kind: PlaceKind::Local(no),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                },
                vec![item_expr],
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
            lw.push_for(out, lid, item, iter, body);
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(yes, span), lw.local(no, span)]),
                pair_ty.clone(),
                span,
            );
            out.push_give(res);
        })
    }
}

impl Lowerer<'_> {
    /// The zero-arg `each(self)` (such as `Bitset<D>`'s), or the `iter(self)` of its
    /// `impl Seq`, that walks a struct, and the element it yields.
    pub(in crate::lower) fn struct_walk(
        &mut self,
        sid: StructId,
        span: Span,
    ) -> Option<(FnId, Ty)> {
        let f = ["each", "iter"].into_iter().find_map(|m| {
            let f = self.own_method(sid, m)?;
            self.sigs[f].params.is_empty().then_some(f)
        })?;
        self.queue_fn(f);
        match self.ret_of(f, span) {
            Ty::List(elem) => Some((f, *elem)),
            _ => None,
        }
    }

    /// What a `for` over a `ty` binds: a built-in sequence's or set's element, or
    /// what a struct's walk yields.
    pub(in crate::lower) fn walk_elem(&mut self, ty: &Ty, span: Span) -> Option<Ty> {
        match ty {
            Ty::Set(elem) | Ty::HashSet(elem) => Some((**elem).clone()),
            Ty::Struct(sid) => self.struct_walk(*sid, span).map(|(_, elem)| elem),
            _ => ty.seq_elem(),
        }
    }

    /// `seq` as a built-in sequence a loop walks: a struct becomes the call of its walk.
    pub(in crate::lower) fn walked_struct(&mut self, seq: Expr) -> Expr {
        let Ty::Struct(sid) = seq.ty else {
            return seq;
        };
        let span = seq.span;
        match self.struct_walk(sid, span) {
            Some((f, elem)) => self.mk(
                ExprKind::Call(f, vec![Arg::Value(seq)]),
                Ty::list(elem),
                span,
            ),
            None => seq,
        }
    }
}

impl Lowerer<'_> {
    /// The element of the first bound generic that is walkable.
    pub(in crate::lower) fn inferred_elem(&mut self, inferred: &GenEnv, span: Span) -> Option<Ty> {
        let tys: Vec<Ty> = (inferred.values())
            .filter_map(|ga| match ga {
                GenArg::Ty(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        tys.iter().find_map(|t| self.walk_elem(t, span))
    }
}
