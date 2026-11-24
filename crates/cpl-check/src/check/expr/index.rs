//! Indexing: element types, typed and double indices, and walks over slices.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// What a `for` over a value of type `ty` yields.
    pub(crate) fn element_ty(&mut self, ty: &Ty) -> Ty {
        match ty {
            Ty::Domain(name) => Ty::Pos(name.clone()),
            Ty::TypeName(name) if name == "Bool" => Ty::Bool,
            // `type Group = Subset<R>`: walking `Group` walks the aliased type.
            Ty::TypeName(name) => match self.type_def(name).as_deref() {
                Some(Def::Alias { ty, .. }) => ty.clone(),
                _ => Ty::Named {
                    name: name.clone(),
                    args: Vec::new(),
                },
            },
            Ty::Tuple(items)
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|t| matches!(t, Ty::Domain(_) | Ty::TypeName(_))) =>
            {
                let elems = items.iter().map(|t| self.element_ty(t)).collect();
                Ty::Tuple(elems)
            }
            Ty::Record(fields)
                if !fields.is_empty()
                    && fields
                        .iter()
                        .all(|(_, t)| matches!(t, Ty::Domain(_) | Ty::TypeName(_))) =>
            {
                let elems = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.element_ty(t)))
                    .collect();
                Ty::Record(elems)
            }
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Str => Ty::Char,
            Ty::Named { name, args }
                if matches!(
                    name.as_str(),
                    "List" | "Set" | "HashSet" | "Range" | "SortedRange" | "Gen"
                ) && args.len() == 1 =>
            {
                args[0].clone()
            }
            Ty::Named { name, args } => {
                if let Some(members) = self.members_of(name) {
                    let bindings = members.bindings(args);
                    if let Some((_, each_ty)) = members.members.iter().find(|(m, _)| m == "each") {
                        if let Ty::Fn { ret, .. } = each_ty.subst(&bindings) {
                            if let Ty::Named {
                                name: rname,
                                args: rargs,
                            } = ret.as_ref()
                            {
                                if matches!(rname.as_str(), "Gen" | "List") && rargs.len() == 1 {
                                    return self.settle(&rargs[0]);
                                }
                            }
                        }
                    }
                }
                ty.seq_elem().unwrap_or(Ty::Unknown)
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }

    pub(crate) fn array_inverse_ret(&mut self, index: &[Ty], elem: &Ty) -> Option<Ty> {
        if index.len() != 1 {
            return None;
        }
        let mut is_index_ty = |ty: &Ty| match ty {
            Ty::Pos(_) | Ty::Range { .. } | Ty::Param(_) => true,
            Ty::Named { name, args } if args.is_empty() => {
                self.finite_index_enum_size(name).is_some()
            }
            _ => false,
        };
        let d = index[0].clone();
        match elem {
            Ty::Optional(inner) if is_index_ty(inner) => Some(Ty::Array {
                index: vec![(**inner).clone()],
                elem: Box::new(Ty::Optional(Box::new(d))),
            }),
            other if is_index_ty(other) => Some(Ty::Array {
                index: vec![other.clone()],
                elem: Box::new(d),
            }),
            _ => None,
        }
    }

    /// Records `expr` as in bounds when each index has exactly the domain
    /// type it stands for: a position of it, or the same type parameter.
    pub(super) fn note_typed_index(
        &mut self,
        expr: &Expr,
        base: &Expr,
        indices: &[Expr],
        got: &[Ty],
        want: &[Ty],
    ) {
        fn space(got: &Ty, want: &Ty) -> bool {
            match (got, want) {
                (Ty::Pos(a), Ty::Pos(b)) | (Ty::Param(a), Ty::Param(b)) => a == b,
                (Ty::Tuple(a), Ty::Tuple(b)) => {
                    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| space(a, b))
                }
                _ => false,
            }
        }
        if indices.is_empty() || got.len() != indices.len() || want.len() != indices.len() {
            return;
        }
        if !got.iter().zip(want).all(|(got, want)| space(got, want)) {
            return;
        }
        let proofs: Vec<String> = indices
            .iter()
            .zip(got)
            .map(|(e, ty)| format!("{}: {ty}", self.quote(e.span)))
            .collect();
        let spaces = if proofs.len() == 1 { "space" } else { "spaces" };
        self.typed_indexes.push(crate::check::TypedIndex {
            access: expr.span,
            seq: base.span,
            index: Span::new(indices[0].span.start, indices[indices.len() - 1].span.end),
            proof: format!(
                "{}, the index {spaces} of {}",
                proofs.join(", "),
                self.quote(base.span)
            ),
        });
    }

    pub(super) fn index(&mut self, expr: &Expr, base: &Expr, indices: &[Expr]) -> Ty {
        if let [index] = indices {
            if let Some(at) = self.index_one_way(expr, base, index) {
                return at;
            }
        }
        let base_ty = self.infer(base);
        if let Some(n) = nested_arity(&base_ty, indices.len()) {
            let inner = Expr::indexed(base, &indices[..n], base.span.to(indices[n - 1].span));
            let cell = self.index_typed(&inner, base, base_ty, &indices[..n], false);
            return self.index_typed(expr, &inner, cell, &indices[n..], true);
        }
        self.index_typed(expr, base, base_ty, indices, false)
    }

    /// `index` once `base` has its type; `split` when `base` is the cell a
    /// table of tables' indices name (R01.8.5).
    fn index_typed(
        &mut self,
        expr: &Expr,
        base: &Expr,
        base_ty: Ty,
        indices: &[Expr],
        split: bool,
    ) -> Ty {
        if !split && self.double_index(base, indices) {
            return Ty::Error;
        }
        self.keyed_view_index(expr.span, base, indices);
        if self.walked_not_indexed(base) {
            for index in indices {
                self.infer(index);
            }
            return Ty::Error;
        }
        let prev_piece = std::mem::replace(&mut self.expected_piece, true);
        let index_tys: Vec<Ty> = indices.iter().map(|i| self.infer(i)).collect();
        self.expected_piece = prev_piece;
        if let (Ty::Param(param), [_]) = (&base_ty, indices) {
            if let Some(Ty::Fn { ret, .. }) = self.bound_method(param, "at") {
                return *ret;
            }
        }
        if let (ExprKind::Name(base_name), Some(idx_expr)) = (&base.kind, indices.first()) {
            let idx_owner = match &idx_expr.kind {
                ExprKind::Call { callee, .. } => match &callee.kind {
                    ExprKind::Field {
                        base: owner,
                        name: f,
                    } if f.text == "Idx" => match &owner.kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    ExprKind::Field { base: inner, .. } => match &inner.kind {
                        ExprKind::Field {
                            base: owner,
                            name: f,
                        } if f.text == "Idx" => match &owner.kind {
                            ExprKind::Name(n) => Some(n.as_str()),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            };
            if let Some(other) = idx_owner {
                if other != base_name {
                    let message = cpl_diag::msgs::check::INDEXED_IDX_NOT_IDX
                        .at(&[&base_name, &base_name, &other]);
                    self.report_said(idx_expr.span, message);
                }
            }
        }

        if self.cut_range_not_piece(&base_ty, indices, &index_tys) {
            return Ty::Error;
        }
        match &base_ty {
            // `[Sym<V>]T`: symmetric storage over unordered pairs of `V`,
            // indexed as `adj[u, v]` or `adj[p]` with `p: (V, V)`.
            Ty::Array { index, elem }
                if matches!(index.as_slice(), [Ty::Named { name, args }] if matches!(name.as_str(), "Sym" | "Pair") && args.len() == 1)
                    && !(indices.len() == 1 && self.fits(&index_tys[0], &index[0])) =>
            {
                let Ty::Named { args, .. } = &index[0] else {
                    unreachable!()
                };
                let dom_ty = &args[0];
                if indices.len() == 1
                    && (index_tys[0].is_vague()
                        || matches!(&index_tys[0], Ty::Tuple(items) if items.len() == 2))
                {
                    let want = Ty::Tuple(vec![dom_ty.clone(), dom_ty.clone()]);
                    self.expect_fits(&index_tys[0], &want, &indices[0], |c, got, want| {
                        format!(
                            "`{}` is indexed by {}, but `{}` is {}",
                            c.quote(base.span),
                            want,
                            c.quote(indices[0].span),
                            got.with_article()
                        )
                    });
                    return (**elem).clone();
                }
                if indices.len() != 2 {
                    if let Some(diag) = self.report(
                        expr.span,
                        msgs::check::WRONG_ARG_COUNT_TAKES_2_INDICES_BUT,
                        &[
                            &self.quote(base.span),
                            &dom_ty,
                            &dom_ty,
                            &indices.len(),
                            &if indices.len() == 1 { "was" } else { "were" },
                        ],
                    ) {
                        diag.help =
                            Some("index a symmetric table with two positions: `adj[u, v]`".into());
                    }
                    return Ty::Error;
                }
                for (got, index_expr) in index_tys.iter().zip(indices) {
                    if !self.fits(got, dom_ty) && !self.literal_position(index_expr, dom_ty) {
                        self.report(
                            index_expr.span,
                            msgs::check::TYPE_MISMATCH_IS_INDEXED_BY_BUT,
                            &[
                                &self.quote(base.span),
                                &dom_ty,
                                &self.quote(index_expr.span),
                                &got.with_article(),
                            ],
                        );
                    }
                }
                (**elem).clone()
            }
            // `[R, C]T` indexed by a single position `p: P` where `domain P = (R, C)`.
            Ty::Array { index, elem }
                if index.len() > 1
                    && indices.len() == 1
                    && matches!(&index_tys[0], Ty::Pos(dom) if self.product_factors(dom).is_some_and(|f| f.len() == index.len() && f.iter().zip(index).all(|((_, ft), want)| self.fits(ft, want)))) =>
            {
                (**elem).clone()
            }
            // `[State]T` where `State = (V, K)` (a domain or a type): allow multi-index
            // `dp[v, k]` in addition to `dp[s]` and `dp[(v, k)]`.
            Ty::Array { index, elem }
                if index.len() == 1
                    && indices.len() > 1
                    && self
                        .axis_factors(&index[0])
                        .is_some_and(|f| f.len() == indices.len()) =>
            {
                let wants = self.axis_factors(&index[0]).unwrap();
                self.note_typed_index(expr, base, indices, &index_tys, &wants);
                for ((got, want), index_expr) in index_tys.iter().zip(&wants).zip(indices) {
                    self.expect_fits(got, want, index_expr, |c, got, want| {
                        format!(
                            "`{}` is indexed by {}, but `{}` is {}",
                            c.quote(base.span),
                            want,
                            c.quote(index_expr.span),
                            got.with_article()
                        )
                    });
                }
                (**elem).clone()
            }
            // `[V.ranges]T`: a non-empty range of `V` is one cell, not a slice.
            Ty::Array { index, elem }
                if indices.len() == 1
                    && matches!(index.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                self.expect_fits(&index_tys[0], &index[0], &indices[0], |c, got, want| {
                    format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        c.quote(base.span),
                        want,
                        c.quote(indices[0].span),
                        got.with_article()
                    )
                });
                (**elem).clone()
            }
            // `g[p]` with `p: (Row, Col)`: `a[i, j]` is sugar for `a[(i, j)]`.
            Ty::Array { index, elem }
                if index.len() > 1
                    && indices.len() == 1
                    && (index_tys[0].is_vague()
                        || matches!(&index_tys[0], Ty::Tuple(items) if items.len() == index.len())) =>
            {
                if let (ExprKind::Tuple(parts), Ty::Tuple(got)) = (&indices[0].kind, &index_tys[0])
                {
                    for ((got, want), part) in got.iter().zip(index.iter()).zip(parts) {
                        self.expect_fits(got, want, part, |c, got, want| {
                            format!(
                                "`{}` is indexed by {}, but `{}` is {}",
                                c.quote(base.span),
                                want,
                                c.quote(part.span),
                                got.with_article()
                            )
                        });
                    }
                    return (**elem).clone();
                }
                let want = Ty::Tuple(index.clone());
                self.note_typed_index(expr, base, indices, &index_tys, std::slice::from_ref(&want));
                self.expect_fits(&index_tys[0], &want, &indices[0], |c, got, want| {
                    format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        c.quote(base.span),
                        want,
                        c.quote(indices[0].span),
                        got.with_article()
                    )
                });
                (**elem).clone()
            }
            // `grid[r]`: leading indices give the O(1) view of what is left,
            // here row `r`, indexed by the columns.
            Ty::Array { index, elem }
                if indices.len() < index.len()
                    && index_tys.iter().zip(index.iter()).zip(indices).all(
                        |((got, want), e)| self.fits(got, want) || self.literal_position(e, want),
                    ) =>
            {
                Ty::Array {
                    index: index[indices.len()..].to_vec(),
                    elem: elem.clone(),
                }
            }
            Ty::Array { index, elem } => {
                if index.len() != indices.len() && self.indexes_by_factors(&index_tys, index) {
                    return (**elem).clone();
                }
                if index.len() != indices.len() {
                    let wanted: Vec<String> = index.iter().map(Ty::to_string).collect();
                    if let Some(diag) = self.report(
                        expr.span,
                        msgs::check::NOT_INDEXABLE_TAKES_BUT_GIVEN,
                        &[
                            &self.quote(base.span),
                            &plural(index.len(), "index", "indices"),
                            &wanted.join(", "),
                            &if indices.len() == 1 {
                                "1 was".to_string()
                            } else {
                                format!("{} were", indices.len())
                            },
                        ],
                    ) {
                        diag.help = Some(
                            "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`"
                                .into(),
                        );
                    }
                    return Ty::Error;
                }
                let mut slice = false;
                // The indices a part keeps: `g[1, ..]` is a row, indexed by
                // the columns alone.
                let mut kept = Vec::new();
                for ((want, got), index_expr) in index.iter().zip(&index_tys).zip(indices) {
                    if let (
                        ExprKind::Range {
                            hi: Some(hi),
                            inclusive: true,
                            ..
                        },
                        Ty::Named { name, args },
                        Ty::Pos(dom),
                    ) = (&index_expr.kind, got, want)
                    {
                        if name == "Range" && args.len() == 1 {
                            if let Ty::Pos(cut) = &args[0] {
                                if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                                    if let Some(diag) = self.report(
                                        index_expr.span,
                                        msgs::check::WRONG_DOMAIN_ENDS_AT_A_POSITION,
                                        &[&self.quote(hi.span)],
                                    ) {
                                        diag.help = Some(
                                            "use half-open `..` for the positions between two cuts"
                                                .to_string(),
                                        );
                                    }
                                    slice = true;
                                    kept.push(want.clone());
                                    continue;
                                }
                            }
                        }
                    }
                    if let Ty::Named { name, args } = got {
                        if name == "Range" {
                            if matches!(args.as_slice(), [Ty::Pos(d)] if d.ends_with(".cuts"))
                                && !matches!(want, Ty::Pos(w) if w.ends_with(".cuts"))
                            {
                                self.report(
                                    index_expr.span,
                                    msgs::check::TYPE_MISMATCH_IS_A_CUT_RANGE,
                                    &[&self.quote(index_expr.span), &got, &self.quote(base.span)],
                                );
                            }
                            slice = true;
                            kept.push(want.clone());
                            continue;
                        }
                    }
                    if self.fits(got, want) {
                        continue;
                    }
                    if self.literal_position(index_expr, want)
                        && self.view_kind(base) != Some("slice")
                    {
                        if let (ExprKind::Int(v), Ty::Pos(d)) = (&index_expr.kind, want) {
                            let is_cuts = d.ends_with(".cuts");
                            let base_dom = d.strip_suffix(".cuts").unwrap_or(d);
                            let out_of_bounds = match self.type_def(base_dom).as_deref() {
                                Some(Def::Domain(info)) if is_cuts => {
                                    *v < 0 || info.max_size.is_some_and(|m| *v > m)
                                }
                                Some(Def::Domain(info)) => {
                                    info.min_pos.is_some_and(|lo| *v < lo)
                                        || info.max_pos.is_some_and(|hi| *v > hi)
                                }
                                _ => false,
                            };
                            if out_of_bounds {
                                self.report(
                                    index_expr.span,
                                    msgs::check::RANGE_STORE_CONST_LITERAL_IS_OUTSIDE_DOMAIN,
                                    &[&v, &d],
                                );
                            }
                        }

                        continue;
                    }
                    let message = format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        self.quote(base.span),
                        want,
                        self.quote(index_expr.span),
                        got.with_article()
                    );
                    let base_text = self.quote(base.span);
                    let index_text = self.quote(index_expr.span);
                    let boundary = match (got, want) {
                        (Ty::Pos(got), Ty::Pos(want)) => {
                            got.strip_suffix(".cuts") == Some(want)
                                || want.strip_suffix(".cuts") == Some(got)
                        }
                        _ => false,
                    };
                    // A step that was not bound, clamped or asserted.
                    if let Some(may) = self.unhandled_step(got, want, index_expr) {
                        let message = cpl_diag::msgs::check::MSG.at(&[
                            &self.quote(index_expr.span),
                            &got,
                            &may,
                        ]);
                        let help = self.step_help(index_expr);
                        let origin =
                            self.origin(index_expr, &format!("declared as {}", got.with_article()));
                        if let Some(diag) = self.report_said(index_expr.span, message) {
                            diag.help = Some(help);
                            diag.labels.extend(origin);
                        }
                    } else {
                        let origins = [
                            self.origin(base, &format!("declared over {want}")),
                            self.origin(index_expr, &format!("declared as {}", got.with_article())),
                        ];
                        let Some(diag) = self.error(codes::WRONG_DOMAIN, index_expr.span, message)
                        else {
                            continue;
                        };
                        diag.labels.extend(origins.into_iter().flatten());
                        if let (true, false, Ty::Pos(domain)) = (got.is_int(), boundary, want) {
                            diag.fixes.push(convert_to(domain, index_expr.span));
                        }
                        diag.help = Some(if boundary {
                            "a position and the boundaries around it are different indices; use `.l` / `.r` for the boundaries on either side of a position".to_string()
                        } else if got.is_int() {
                            format!(
                                "convert it with `{want}({index_text})`, or write `{base_text}.raw[{index_text}]` to index by number"
                            )
                        } else {
                            format!("index with {}", want.with_article())
                        });
                    }
                }
                if !slice {
                    self.note_typed_index(expr, base, indices, &index_tys, index);
                }
                if slice {
                    // A slice is its array seen through a range: lent,
                    // and indexed by the same positions.
                    self.note_view(expr, "slice");
                    if kept.len() == index.len() {
                        base_ty.clone()
                    } else {
                        Ty::Array {
                            index: kept,
                            elem: elem.clone(),
                        }
                    }
                } else {
                    (**elem).clone()
                }
            }
            // `s[0..5]`: O(1) read-only slice view into a string.
            Ty::Str if matches!(index_tys.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                self.note_readonly_view(expr, "slice");
                Ty::Str
            }
            // `xs[0..5]`: O(1) slice view into a list (mutable when the root is a `var`).
            ty if matches!(ty, Ty::Named { name, .. } if name == "List")
                && matches!(index_tys.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                self.note_view(expr, "slice");
                ty.clone()
            }
            Ty::Named { name, args } if name == "List" && args.len() == 1 => {
                self.list_index_by_position(base, indices, &index_tys);
                args[0].clone()
            }
            Ty::Named { name, .. } if matches!(name.as_str(), "Filter" | "Rev") => {
                self.report(
                    expr.span,
                    msgs::check::NOT_INDEXABLE_HAS_TYPE_WHICH_IS,
                    &[&self.quote(base.span), &base_ty],
                );
                Ty::Error
            }
            Ty::Named { name, .. } if matches!(name.as_str(), "Gen" | "Iter") => {
                self.report(
                    expr.span,
                    msgs::check::MISSING_IMPL_HAS_TYPE_WHICH_DOES,
                    &[&self.quote(base.span), &base_ty],
                );
                Ty::Error
            }
            Ty::Named { name, args } if name == "Range" && args.len() == 1 => args[0].clone(),
            Ty::Named { name, args } if name == "Map" && args.len() == 2 => args[1].clone(),
            Ty::Str => Ty::Char,
            ty if ty.is_numeric() || matches!(ty, Ty::Bool | Ty::Char | Ty::Fn { .. }) => {
                let help = match ty {
                    Ty::Fn { .. } => "call it with `(...)`",
                    _ => "`[...]` reads arrays, maps, ranges and strings",
                };
                if let Some(diag) = self.report(
                    base.span,
                    msgs::check::NOT_INDEXABLE_IS_AND_CAN_T,
                    &[&self.quote(base.span), &ty.widen().with_article()],
                ) {
                    diag.help = Some(help.into());
                }
                Ty::Error
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }
}

impl Checker<'_> {
    pub(crate) fn walk_only_expr(&self, e: &Expr) -> bool {
        let e = crate::check::lends::unparen(e);
        matches!(&e.kind, ExprKind::Name(n) if self.walk_only.contains(n))
    }

    /// R07.1.2: `filter` gives a `Seq`, walked in order with no `s[i]`.
    pub(super) fn walked_not_indexed(&mut self, base: &Expr) -> bool {
        if !self.walk_only_expr(base) {
            return false;
        }
        let shown = self.quote(base.span);
        let message = cpl_diag::msgs::check::FILTERED_SEQUENCE_CAN_WALKED.at(&[&shown]);
        if let Some(diag) = self.report_said(base.span, message) {
            diag.help = Some(format!("collect it first: `{shown}.collect()[i]`"));
        }
        true
    }
}

/// How many of `n` indices a table whose cells are arrays takes before the
/// cell's own: `dp[r, c, k]` on `[R, C][K]T` takes 2 (R01.8.5).
fn nested_arity(ty: &Ty, n: usize) -> Option<usize> {
    let Ty::Array { index, elem } = ty else {
        return None;
    };
    let Ty::Array { index: inner, .. } = &**elem else {
        return None;
    };
    let sym = matches!(index.as_slice(), [Ty::Named { name, .. }] if matches!(name.as_str(), "Sym" | "Pair"));
    (!sym && index.len() < n && n <= index.len() + inner.len()).then_some(index.len())
}

impl Checker<'_> {
    /// R01.8.5: `g[r][c]` is spelled `g[r, c]`.
    pub(super) fn double_index(&mut self, base: &Expr, indices: &[Expr]) -> bool {
        let ExprKind::Index {
            base: inner,
            indices: first,
        } = &crate::check::lends::unparen(base).kind
        else {
            return false;
        };
        let point = |es: &[Expr]| {
            es.iter()
                .all(|e| !matches!(crate::check::lends::unparen(e).kind, ExprKind::Range { .. }))
        };
        let ExprKind::Name(table) = &inner.kind else {
            return false;
        };
        // A row of a `List<List<T>>` is a list of its own; only a table takes `g[r, c]`.
        let ty = match self.local(table) {
            Some(local) => local.ty.clone(),
            None => self
                .global(table)
                .map_or(Ty::Unknown, |def| def.value_ty(table)),
        };
        let is_table = matches!(ty, Ty::Array { index, .. } if index.len() > 1);
        if !point(first) || !point(indices) || !is_table {
            return false;
        }
        let parts: Vec<String> = first
            .iter()
            .chain(indices)
            .map(|e| self.quote(e.span))
            .collect();
        let fixed = format!("{}[{}]", self.quote(inner.span), parts.join(", "));
        let span = base.span.to(indices.last().map_or(base.span, |e| e.span));
        if let Some(diag) = self.report(
            span,
            msgs::check::REMOVED_SYNTAX_INDEXES_TWICE_0_3,
            &[
                &self.quote(base.span),
                &indices
                    .iter()
                    .map(|e| self.quote(e.span))
                    .collect::<Vec<_>>()
                    .join(", "),
            ],
        ) {
            diag.help = Some(format!("write `{fixed}`"));
        }
        true
    }
}

impl Checker<'_> {
    /// R07.4.5: `for k in 0..w.len: w[k]` counts from 0 over a slice's keys.
    pub(crate) fn counter_over_slice(&mut self, iter: &Expr, span: cpl_diag::Span) {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            ..
        } = &crate::check::lends::unparen(iter).kind
        else {
            return;
        };
        if !matches!(lo.kind, ExprKind::Int(0)) {
            return;
        }
        let ExprKind::Field { base, name } = &hi.kind else {
            return;
        };
        if name.text != "len" || self.view_kind(base) != Some("slice") {
            return;
        }
        let w = self.quote(base.span);
        self.sink.push(
            cpl_diag::msgs::check::SLICE_COUNTS_FROM_ZERO.warning(span, &[&w, &w]).with_help(format!(
                "walk it (`for x in {w}`, `for (k, x) in {w}.items`), or `{w}.rebase` to count from 0"
            )),
        );
    }
}
