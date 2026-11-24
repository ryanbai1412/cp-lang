//! The value a `let` or `var` binds: its initializer, `read()` forms,
//! filled arrays, literal grids and default zeros.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// `let g = gen(args)` names a generator value, a recipe that runs
    /// afresh on every walk (R06.3.1). It is kept as the call when the
    /// arguments read the same at every walk and no closure or fn captures `g`.
    pub(in crate::lower) fn generator_recipe<'s>(
        &self,
        stmt: &'s ast::Stmt,
        rest: &[ast::Stmt],
    ) -> Option<(String, &'s ast::Expr)> {
        let ast::StmtKind::Let {
            mutable: false,
            pat,
            ty: None,
            init: Some(init),
            otherwise: None,
            guard: None,
        } = &stmt.kind
        else {
            return None;
        };
        let ast::PatternKind::Name(name) = &pat.kind else {
            return None;
        };
        let ast::ExprKind::Call { callee, args } = &init.kind else {
            return None;
        };
        let Some(Sym::Fn(id)) = self.sym(callee) else {
            return None;
        };
        if !self.sigs[id].is_generator {
            return None;
        }
        let steady = args.iter().all(|a| match &a.value.kind {
            ast::ExprKind::Int(_) | ast::ExprKind::Bool(_) | ast::ExprKind::Char(_) => true,
            ast::ExprKind::Name(n) => n != name && (a.mutable || !Self::stmts_mutate_name(rest, n)),
            ast::ExprKind::Mut(inner) => matches!(&inner.kind, ast::ExprKind::Name(n) if n != name),
            _ => false,
        });
        if !steady || Self::stmts_mutate_name(rest, name) || captured(rest, name) {
            return None;
        }
        Some((name.clone(), init))
    }

    pub(in crate::lower) fn let_value_with_rest(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        pat: &ast::Pattern,
        rest: &[ast::Stmt],
        span: Span,
    ) -> Option<Expr> {
        // Handle untyped `var (path, at) = ([], v)` followed by `path.push(elem)`.
        if ty.is_none() {
            if let (
                ast::PatternKind::Tuple(pats),
                Some(ast::Expr {
                    kind: ast::ExprKind::Tuple(exprs),
                    span: tspan,
                }),
            ) = (&pat.kind, init)
            {
                if pats.len() == exprs.len()
                    && exprs
                        .iter()
                        .any(|e| matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()))
                {
                    // Lower non-empty elements first so sibling names (e.g. `at` in `var (path, at) = ([], v)`)
                    // are visible when peeking `path.push(at)` in `rest`.
                    let mut slots: Vec<Option<Expr>> = vec![None; exprs.len()];
                    let mut temp_bindings: Vec<(String, Ty)> = Vec::new();
                    for (i, (p, e)) in pats.iter().zip(exprs).enumerate() {
                        if !matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()) {
                            let v = self.expr(e, None);
                            if let ast::PatternKind::Name(n) = &p.kind {
                                temp_bindings.push((n.clone(), v.ty.clone()));
                            }
                            slots[i] = Some(v);
                        }
                    }
                    // Temporarily bind sibling names in a scratch scope while peeking empty-list element types.
                    let saved_cx = self.cx.clone();
                    self.cx.scopes.push(HashMap::new());
                    for (n, t) in &temp_bindings {
                        let s = self.cx.locals.len();
                        self.cx.locals.push(Local {
                            name: n.clone(),
                            ty: t.clone(),
                            binds: None,
                        });
                        self.cx.scopes.last_mut().unwrap().insert(n.clone(), s);
                    }
                    let mut inferred_elems: Vec<Option<Ty>> = vec![None; exprs.len()];
                    for (i, (p, e)) in pats.iter().zip(exprs).enumerate() {
                        if slots[i].is_none() {
                            if let ast::PatternKind::Name(vname) = &p.kind {
                                inferred_elems[i] = self.peek_list_elem_type(vname, rest);
                            }
                        }
                        let _ = e;
                    }
                    self.cx = saved_cx;
                    let mut lowered = Vec::with_capacity(exprs.len());
                    let mut tys = Vec::with_capacity(exprs.len());
                    for (i, e) in exprs.iter().enumerate() {
                        if let Some(v) = slots[i].take() {
                            tys.push(v.ty.clone());
                            lowered.push(v);
                        } else if let Some(elem_ty) = inferred_elems[i].take() {
                            let lty = Ty::list(elem_ty);
                            lowered.push(self.mk(ExprKind::Tuple(Vec::new()), lty.clone(), e.span));
                            tys.push(lty);
                        } else {
                            let v = self.expr(e, None);
                            tys.push(v.ty.clone());
                            lowered.push(v);
                        }
                    }
                    return Some(self.mk(ExprKind::Tuple(lowered), Ty::Tuple(tys), *tspan));
                }
            }
            if let (
                ast::PatternKind::Name(vname),
                Some(ast::Expr {
                    kind: ast::ExprKind::List(items),
                    ..
                }),
            ) = (&pat.kind, init)
            {
                if items.is_empty() {
                    if let Some(elem_ty) = self.peek_list_elem_type(vname, rest) {
                        return Some(self.mk(ExprKind::Tuple(Vec::new()), Ty::list(elem_ty), span));
                    }
                }
            }
            if let (ast::PatternKind::Name(vname), Some(init_expr)) = (&pat.kind, init) {
                if let Some(depth) = declare::filled_none_depth(init_expr) {
                    if let Some(hint_ty) = self.peek_filled_none_type(vname, depth, rest) {
                        return Some(self.expr(init_expr, Some(&hint_ty)));
                    }
                }
            }
            // Handle untyped `let f = w => ...` followed by `f("abc")` by peeking call argument types in `rest`.
            if let (
                ast::PatternKind::Name(vname),
                Some(ast::Expr {
                    kind:
                        ast::ExprKind::Lambda {
                            params,
                            param_tys: ann_tys,
                            param_mut,
                            body,
                            ..
                        },
                    span: lspan,
                }),
            ) = (&pat.kind, init)
            {
                if let Some(param_tys) = self.peek_fn_arg_types(vname, params.len(), rest) {
                    let hint = Ty::Fn(param_tys, Box::new(Ty::Error));
                    return Some(self.lower_lambda(
                        params,
                        ann_tys,
                        param_mut,
                        body,
                        Some(&hint),
                        *lspan,
                    ));
                }
            }
        }
        let val = self.let_value(ty, init, span)?;
        // Handle `let v = stack.pop()` where `stack: List<T>` (non-Opt elem):
        // if `v` is NOT used as an optional in `rest`, unwrap `Pop` -> `PopVal` so stdlib works,
        // while `collections.cp` (`stack.pop() ?? -1`, `print(stack.pop())`) keeps `Pop` -> `?T`!
        if ty.is_none() {
            if let (ExprKind::Mutate(Mutation::Pop, place, args), Ty::Opt(inner)) =
                (&val.kind, &val.ty)
            {
                let should_unwrap = match &pat.kind {
                    ast::PatternKind::Name(vname) => {
                        !matches!(**inner, Ty::Opt(_)) && !uses_as_optional(vname, rest)
                    }
                    ast::PatternKind::Tuple(_) => true,
                    _ => false,
                };
                if should_unwrap {
                    return Some(Expr {
                        kind: ExprKind::Mutate(Mutation::PopVal, place.clone(), args.clone()),
                        ty: (**inner).clone(),
                        span: val.span,
                    });
                }
            }
        }
        Some(val)
    }

    /// The value a `let` binds: its initializer, read or allocated the way
    /// the declared type says.
    pub(in crate::lower) fn declared_value_inner(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        span: Span,
    ) -> Option<Expr> {
        let Some(ty) = ty else {
            let Some(init) = init else {
                self.report(span, msgs::ir::UNKNOWN_TYPE_A_VARIABLE_NEEDS_A.at(&[]));
                return None;
            };
            return Some(self.expr(init, None));
        };
        let spec = self.resolve(ty, true);
        let want = self.written_as(spec.ty(), ty);
        let Some(init) = init else {
            let z = self.zero(&spec, span);
            if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(&spec) {
                return Some(self.check_range(z, &bounds, span));
            }
            return Some(z);
        };
        if is_read_call(init) {
            let mut spec = spec;
            if let ast::ExprKind::Call { args, .. } = &init.kind {
                if let Some(absent) = args
                    .iter()
                    .find(|a| a.name.as_ref().is_some_and(|n| n.text == "none"))
                    .map(|a| self.expect(&a.value, &Ty::Int))
                {
                    if !crate::lower::call::reads_none_as(&mut spec, &absent) {
                        self.report(
                            init.span,
                            msgs::ir::UNKNOWN_ARG_NAME_NONE_IS_FOR_READING.at(&[&self.show(&want)]),
                        );
                    }
                }
            }
            return Some(self.read(spec, init.span));
        }
        if let Some(none_arg) = read_none_arg(init) {
            let mut spec = spec;
            let absent = self.expect(none_arg, &Ty::Int);
            if !crate::lower::call::reads_none_as(&mut spec, &absent) {
                self.report(
                    init.span,
                    msgs::ir::UNKNOWN_ARG_NAME_NONE_IS_FOR_READING.at(&[&self.show(&want)]),
                );
            }
            return Some(self.read(spec, init.span));
        }
        // `let b: [0..n]Int = read_output()` in a `@check`: the same
        // layout, read from what the solution printed.
        if is_read_output_call(init) && self.aside == Some(Aside::Checker) {
            return Some(self.read_printed(spec, init.span));
        }
        if let Spec::Array { dims, elem } = &spec {
            let elem_ty = elem.ty();
            let elem_range = RangeInfo::from_spec(elem).and_then(|r| match r {
                RangeInfo::Scalar(b) => Some(b),
                _ => None,
            });
            if is_literal(init)
                && (!matches!(&init.kind, ast::ExprKind::List(items) if items.is_empty())
                    || matches!(
                        elem_ty,
                        Ty::List(_)
                            | Ty::Set(_)
                            | Ty::HashSet(_)
                            | Ty::Map(..)
                            | Ty::HashMap(..)
                            | Ty::Struct(_)
                            | Ty::Array { .. }
                    ))
            {
                return Some(self.fill_literal(&spec, init));
            }
            if let Some(table) = self.comprehension_table(init, dims, &elem_ty, elem_range.as_ref())
            {
                return Some(table);
            }
            if let ast::ExprKind::Table {
                domains,
                params,
                body,
            } = &init.kind
            {
                if let Some(bounds) = elem_range.as_ref() {
                    if let Some(tdims) = domains
                        .iter()
                        .map(|d| self.dim(d, true))
                        .collect::<Option<Vec<_>>>()
                    {
                        return Some(self.build_table(
                            tdims,
                            params,
                            body,
                            Some(&elem_ty),
                            Some(bounds),
                            init.span,
                            crate::lower::comprehension::Whole::No,
                        ));
                    }
                }
            }
            // `var ways: [0..n]List<Z> = [Z(1)]`: where a cell is a list, a
            // list of what it holds fills every cell with a copy.
            if let (Ty::List(_), ast::ExprKind::List(items)) = (&elem_ty, &init.kind) {
                let holds = items.first().and_then(|item| self.probe_ty_silent(item));
                if holds.is_some_and(|ty| !matches!(ty, Ty::List(_))) {
                    let value = self.expect(init, &elem_ty);
                    return Some(self.fill_with_index_domains(
                        dims.clone(),
                        Some(Box::new(value)),
                        want,
                        init.span,
                    ));
                }
            }
            // `var dp: [Bool]Z = [1, 0]`: a list of as many values as the
            // array has cells, when that count is written as a number.
            // `let c: [1..=3]Int = [7, 8, 9]` and `[2..n]Int = [5, 6]` too:
            // a count known only as the program runs is checked then.
            // `let g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]`: a list of
            // rows, each as long as the next index says.
            let lens: Option<Vec<&Expr>> = dims
                .iter()
                .map(|dim| match dim {
                    DimSpec::Int(len) | DimSpec::IntFrom(_, len) => Some(&**len),
                    _ => None,
                })
                .collect();
            if let (ast::ExprKind::List(items), Some(lens)) = (&init.kind, lens) {
                let holds_cells = (items.is_empty() && lens.len() == 1)
                    || !items.is_empty()
                        && (lens.len() > 1
                            || !(matches!(elem_ty, Ty::List(_))
                                && items
                                    .first()
                                    .and_then(|item| self.probe_ty_silent(item))
                                    .is_some_and(|ty| !matches!(ty, Ty::List(_)))));
                let shape = if holds_cells {
                    literal_shape(init, lens.len())
                } else {
                    None
                };
                if let (true, Some(Err(at))) = (lens.len() > 1, &shape) {
                    self.report(
                        *at,
                        msgs::ir::TYPE_MISMATCH_A_LIST_FOR_A.at(&[&self.show(&want)]),
                    );
                    return Some(self.err_expr(init.span));
                }
                if let Some(Ok((counts, cells))) = shape {
                    for (len, &count) in lens.iter().zip(&counts) {
                        let Some(cells) = written(len) else { continue };
                        if cells != count as i64 {
                            let message = if lens.len() == 1 {
                                cpl_diag::msgs::ir::TYPE_MISMATCH_THIS_ARRAY_HAS_BUT
                                    .at(&[&plural(cells, "cell"), &plural(count as i64, "value")])
                            } else {
                                cpl_diag::msgs::ir::INDEX_HAS_BUT_LIST
                                    .at(&[&plural(cells, "cell"), &plural(count as i64, "value")])
                            };
                            self.report(init.span, message);
                            return Some(self.err_expr(init.span));
                        }
                    }
                    let unknown: Vec<(Expr, usize)> = lens
                        .iter()
                        .zip(&counts)
                        .filter(|(len, _)| written(len).is_none())
                        .map(|(len, &count)| ((*len).clone(), count))
                        .collect();
                    let dims = dims.clone();
                    let at = init.span;
                    return Some(self.do_expr(Some(&want.clone()), at, |lw, out| {
                        for (len, count) in unknown {
                            let count = lw.int_lit(count as i64, at);
                            let wrong = lw.mk_cmp(CmpOp::Ne, len, count, Ty::Bool, at);
                            let trap = Stmt::Trap(
                                "an array literal needs one value per position".to_string(),
                                lw.span(at),
                            );
                            out.push(Stmt::If(
                                wrong,
                                Block { stmts: vec![trap] },
                                Block::default(),
                            ));
                        }
                        let values: Vec<Expr> = cells
                            .iter()
                            .map(|item| {
                                let v = lw.expect(item, &elem_ty);
                                match elem_range.as_ref() {
                                    Some(b) => lw.check_range(v, b, item.span),
                                    None => v,
                                }
                            })
                            .collect();
                        let mut values = values.into_iter();
                        let first = values.next().map(Box::new);
                        let held = lw.temp(want.clone());
                        let filled = lw.mk(ExprKind::Fill(dims, first), want.clone(), at);
                        out.push_let(held, filled);
                        for (i, value) in values.enumerate() {
                            // The cell's place along each index, last index fastest.
                            let mut rest = i + 1;
                            let mut index = Vec::new();
                            for &count in counts.iter().rev() {
                                index.push(lw.int_lit((rest % count) as i64, at));
                                rest /= count;
                            }
                            index.reverse();
                            let whole = Place {
                                kind: PlaceKind::Local(held),
                                ty: want.clone(),
                                span: at,
                            };
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(whole), index),
                                ty: elem_ty.clone(),
                                span: at,
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, value));
                        }
                        out.push_give(lw.local(held, at));
                    }));
                }
            }
            // `var bags: [R, C]SortedBag<T> = SortedBag()`: a struct built
            // by name is one cell's value, and takes its type from the cell.
            let builds_a_cell = matches!(&init.kind, ast::ExprKind::Call { callee, .. }
                if matches!(&callee.kind, ast::ExprKind::Name(name)
                    if self.lookup(name).is_none()
                        && matches!(self.get_sym(name), Some(Sym::Struct(_) | Sym::GenStruct(_)))));
            // `var adj: [X]List<X> = List()`: an empty list is one cell's
            // value too, and takes its element from the cell.
            let empty_list = matches!(&init.kind, ast::ExprKind::Call { callee, args }
                if args.is_empty()
                    && matches!(&callee.kind, ast::ExprKind::Name(name)
                        if name == "List" && self.lookup(name).is_none()));
            if (builds_a_cell && matches!(elem_ty, Ty::Struct(_)))
                || (empty_list && matches!(elem_ty, Ty::List(_)))
            {
                let value = self.expect(init, &elem_ty);
                return Some(self.fill_with_index_domains(
                    dims.clone(),
                    Some(Box::new(value)),
                    want,
                    init.span,
                ));
            }
            let value = self.expr(init, Some(&want));
            if value.ty == want || value.ty == Ty::Error {
                return Some(value);
            }
            if same_cells(&want, &value.ty) {
                return Some(Expr { ty: want, ..value });
            }
            return Some(match self.try_coerce(value, &elem_ty) {
                Ok(value) => {
                    let value = match elem_range.as_ref() {
                        Some(b) => self.check_range(value, b, init.span),
                        None => value,
                    };
                    self.fill_with_index_domains(
                        dims.clone(),
                        Some(Box::new(value)),
                        want,
                        init.span,
                    )
                }
                Err(value) => {
                    if matches!(elem.as_ref(), Spec::Array { .. })
                        && !matches!(value.ty, Ty::Array { .. } | Ty::List(_))
                    {
                        return Some(self.fill_literal(&spec, init));
                    }
                    self.mismatch(init.span, &want, &value.ty);
                    self.err_expr(init.span)
                }
            });
        }
        if let Ty::Struct(sid) = &want {
            if matches!(&init.kind, ast::ExprKind::List(items) if items.is_empty()) {
                if let Some(new_fn) = self.own_method(*sid, "new") {
                    if self.sigs[new_fn].params.is_empty() {
                        return Some(self.method_call(new_fn, None, &[], init.span));
                    }
                }
            }
        }
        let val = self.expect(init, &want);
        if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(&spec) {
            return Some(self.check_range(val, &bounds, init.span));
        }
        match self.range_info_from_ast(ty) {
            Some(cells @ RangeInfo::Elem(_)) => Some(self.check_fresh_cells(val, &cells, init)),
            _ => Some(val),
        }
    }

    /// `var g: [0..n][0..4]Int = 0`: a literal fills every cell, and a
    /// cell that is itself an array is filled the same way.
    pub(in crate::lower) fn fill_literal(&mut self, spec: &Spec, init: &ast::Expr) -> Expr {
        let Spec::Array { dims, elem } = spec else {
            let v = self.expect(init, &spec.ty());
            if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(spec) {
                return self.check_range(v, &bounds, init.span);
            }
            return v;
        };
        let value = self.fill_literal(elem, init);
        self.fill_with_index_domains(dims.clone(), Some(Box::new(value)), spec.ty(), init.span)
    }

    /// Wraps `ExprKind::Fill` with dynamic `DefDomain` initialization when any dimension
    /// is an `impl Index` struct domain (e.g. `[Tri<D>]Int` where `K = D.K * (D.K + 1) / 2`).
    pub(in crate::lower) fn fill_with_index_domains(
        &mut self,
        dims: Vec<DimSpec>,
        fill: Option<Box<Expr>>,
        want: Ty,
        span: Span,
    ) -> Expr {
        let mut init_pairs: Vec<(DomainId, StructId, StructConst<'a>)> = Vec::new();
        for dim in &dims {
            if let DimSpec::Pos(dom_id) = dim {
                if let Some((sid, sc)) = self.structs.iter().enumerate().find_map(|(sid, s)| {
                    if s.index_domain == Some(*dom_id) {
                        s.consts.get("count").cloned().map(|sc| (sid, sc))
                    } else {
                        None
                    }
                }) {
                    init_pairs.push((*dom_id, sid, sc));
                }
            }
        }
        let fill_expr = self.mk(ExprKind::Fill(dims, fill), want.clone(), span);
        if init_pairs.is_empty() {
            return fill_expr;
        }
        self.do_expr(Some(&want), span, |lw, out| {
            for (dom_id, sid, sc) in init_pairs {
                let k_val = lw.eval_struct_const(sid, sc, span);
                let lo = lw.int_lit(0, span);
                out.push(Stmt::DefDomain {
                    id: dom_id,
                    lo,
                    len: k_val,
                });
            }
            out.push_give(fill_expr);
        })
    }

    /// The value a variable declared without one starts with.
    pub(in crate::lower) fn zero(&mut self, spec: &Spec, span: Span) -> Expr {
        let ty = spec.ty();
        let kind = match spec {
            Spec::Array { dims, .. } => ExprKind::Fill(dims.clone(), None),
            _ => match &ty {
                Ty::Int => ExprKind::Int(0),
                Ty::Float => ExprKind::Float(0.0),
                Ty::Bool => ExprKind::Bool(false),
                Ty::Str => ExprKind::Str(Vec::new()),
                Ty::Opt(_) => ExprKind::None,
                Ty::List(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_) | Ty::HashSet(_) => {
                    ExprKind::Tuple(Vec::new())
                }
                Ty::Error => ExprKind::Error,
                _ => {
                    self.report(
                        span,
                        msgs::ir::UNKNOWN_TYPE_A_HAS_NO_STARTING.at(&[&self.show(&ty)]),
                    );
                    ExprKind::Error
                }
            },
        };
        self.mk(kind, ty, span)
    }
}

/// `read_output()`.
pub(in crate::lower) fn is_read_output_call(e: &ast::Expr) -> bool {
    matches!(&e.kind, ast::ExprKind::Call { callee, args }
        if args.is_empty() && matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read_output"))
}

/// `read(none: sentinel)` (optionally with `gen: ...`).
pub(in crate::lower) fn read_none_arg(e: &ast::Expr) -> Option<&ast::Expr> {
    let ast::ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    if !matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read") {
        return None;
    }
    let mut none_val = None;
    for a in args {
        match a.name.as_ref().map(|n| n.text.as_str()) {
            Some("none") => none_val = Some(&a.value),
            Some("gen") => {}
            _ => return None,
        }
    }
    none_val
}

pub(in crate::lower) fn uses_as_optional(name: &str, stmts: &[ast::Stmt]) -> bool {
    fn in_expr(name: &str, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Unwrap(inner) | ast::ExprKind::Try(inner) => {
                matches!(&inner.kind, ast::ExprKind::Name(n) if n == name) || in_expr(name, inner)
            }
            ast::ExprKind::Binary {
                op: BinOp::Coalesce,
                lhs,
                rhs,
                ..
            } => {
                matches!(&lhs.kind, ast::ExprKind::Name(n) if n == name)
                    || in_expr(name, lhs)
                    || in_expr(name, rhs)
            }
            ast::ExprKind::Compare { first, rest } => {
                let first_is_name = matches!(&first.kind, ast::ExprKind::Name(n) if n == name);
                let any_none = rest
                    .iter()
                    .any(|(_, r)| matches!(r.kind, ast::ExprKind::None));
                (first_is_name && any_none)
                    || in_expr(name, first)
                    || rest.iter().any(|(_, r)| in_expr(name, r))
            }
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches
                    .iter()
                    .any(|(c, b)| in_cond(name, c) || uses_as_optional(name, &b.stmts))
                    || otherwise
                        .as_ref()
                        .is_some_and(|b| uses_as_optional(name, &b.stmts))
            }
            _ => false,
        }
    }
    fn in_cond(name: &str, c: &ast::Cond) -> bool {
        match c {
            ast::Cond::Let { value, and, .. } => {
                matches!(&value.kind, ast::ExprKind::Name(n) if n == name)
                    || in_expr(name, value)
                    || and.as_ref().is_some_and(|a| in_cond(name, a))
            }
            ast::Cond::Guard { test, and } => in_expr(name, test) || in_cond(name, and),
            ast::Cond::Expr(e) | ast::Cond::Const(e) => in_expr(name, e),
        }
    }
    stmts.iter().any(|s| match &s.kind {
        ast::StmtKind::Expr(e) => in_expr(name, e),
        ast::StmtKind::Let { init, .. } => init.as_ref().is_some_and(|e| in_expr(name, e)),
        ast::StmtKind::Assign { value, .. } => in_expr(name, value),
        ast::StmtKind::While {
            cond,
            body,
            else_body,
        } => {
            in_cond(name, cond)
                || uses_as_optional(name, &body.stmts)
                || else_body
                    .as_ref()
                    .is_some_and(|eb| uses_as_optional(name, &eb.stmts))
        }
        ast::StmtKind::For {
            body, else_body, ..
        } => {
            uses_as_optional(name, &body.stmts)
                || else_body
                    .as_ref()
                    .is_some_and(|eb| uses_as_optional(name, &eb.stmts))
        }
        ast::StmtKind::Return(Some(e)) => in_expr(name, e),
        _ => false,
    })
}

/// A literal: in `var a: [V]Int = 0` it is the value of every cell.
pub(in crate::lower) fn is_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Int(_)
        | ast::ExprKind::Float(_)
        | ast::ExprKind::Char(_)
        | ast::ExprKind::Bool(_)
        | ast::ExprKind::None => true,
        ast::ExprKind::Str(pieces) => pieces.iter().all(|p| matches!(p, ast::StrPiece::Text(_))),
        ast::ExprKind::Unary { expr, .. } | ast::ExprKind::Paren(expr) => is_literal(expr),
        ast::ExprKind::Tuple(items) => items.iter().all(is_literal),
        ast::ExprKind::List(items) => items.is_empty(),
        _ => false,
    }
}

/// The number `e` is written as, when it is one.
/// The count along each index, and the cells in order.
pub(in crate::lower) type Rows<'a> = (Vec<usize>, Vec<&'a ast::Expr>);

/// How many values a list written for an array of `depth` indices has
/// along each, and its cells in order; `Err` is where a row is not a
/// list or is not as long as the first.
pub(in crate::lower) fn literal_shape(
    list: &ast::Expr,
    depth: usize,
) -> Option<Result<Rows<'_>, Span>> {
    let ast::ExprKind::List(items) = &list.kind else {
        return None;
    };
    if depth == 1 {
        return Some(Ok((vec![items.len()], items.iter().collect())));
    }
    let mut counts = vec![items.len()];
    let mut cells = Vec::new();
    for (k, row) in items.iter().enumerate() {
        match literal_shape(row, depth - 1) {
            None => return Some(Err(row.span)),
            Some(Err(at)) => return Some(Err(at)),
            Some(Ok((inner, _))) if inner.contains(&0) => return Some(Err(row.span)),
            Some(Ok((inner, row_cells))) => {
                if k == 0 {
                    counts.extend(inner);
                } else if counts[1..] != inner[..] {
                    return Some(Err(row.span));
                }
                cells.extend(row_cells);
            }
        }
    }
    Some(Ok((counts, cells)))
}
