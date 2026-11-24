//! Expressions.

use super::stmt::place_to_expr;
use super::*;

mod expected_variant;
mod unwrap_cells;

/// True for a bare `..`, which as an index is all of one axis.
pub(super) fn is_whole(index: &ast::Expr) -> bool {
    matches!(index.kind, ast::ExprKind::Range { step: None, .. })
}

/// True for `read()` and `ask(...)` written with no `::<T>`.
fn is_bare_read(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Paren(inner) => is_bare_read(inner),
        ast::ExprKind::Call { callee, .. } => {
            matches!(&callee.kind, ast::ExprKind::Name(name) if name == "read" || name == "ask")
        }
        _ => false,
    }
}

impl<'a> Lowerer<'a> {
    /// Lowers `e` where a `want` is needed.
    pub(super) fn expect(&mut self, e: &ast::Expr, want: &Ty) -> Expr {
        let value = self.expr(e, Some(want));
        self.coerce(value, want, e.span)
    }

    /// Lowers `e`. `hint` is the type the context would like, which only
    /// the forms that can't tell on their own look at: `none`, `[]`,
    /// `read()`, and number literals.
    pub(super) fn expr(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        // Library code that this expression asks for reports here.
        let outer = self.asked_at.enter(e.span.shift(self.offset), self.cur_mod);
        let lowered = match self.cx.rhs_extent.take_if(|at| *at == e.span) {
            Some(_) => self.try_extent(e, e.span),
            None => self.expr_of(e, hint),
        };
        self.asked_at.leave(outer);
        lowered
    }

    fn expr_of(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        if let Some(alias) = self.subsets_alias(e) {
            return self.expr_of(&alias, hint);
        }
        // Where an optional is wanted, a plain value is typed as the inside.
        let inner_hint = match hint {
            Some(Ty::Opt(inner)) => Some(&**inner),
            other => other,
        };
        let span = e.span;
        match &e.kind {
            ast::ExprKind::By { .. } => {
                let (iter, elem) = self.iter(e);
                self.walked_list(iter, elem, span)
            }
            ast::ExprKind::Int(value) => match i64::try_from(*value) {
                Ok(value) if inner_hint == Some(&Ty::Float) => {
                    self.mk(ExprKind::Float(value as f64), Ty::Float, span)
                }
                Ok(value)
                    if matches!(
                        inner_hint,
                        Some(Ty::Pos(d))
                            if !self.product_domains.contains_key(d)
                                && !self.sum_enums.contains_key(d)
                    ) =>
                {
                    let Some(Ty::Pos(d)) = inner_hint else {
                        unreachable!()
                    };
                    let lit = self.mk(ExprKind::Int(value), Ty::Int, span);
                    self.cast(Cast::FromLabel(*d), lit, Ty::Pos(*d))
                }
                Ok(value) => self.mk(ExprKind::Int(value), Ty::Int, span),
                // `2**63` and up fits a `U64` where one is wanted: the
                // same 64 bits.
                Err(_) if inner_hint == Some(&Ty::U64) && u64::try_from(*value).is_ok() => {
                    let bits = self.mk(ExprKind::Int(*value as u64 as i64), Ty::Int, span);
                    self.cast(Cast::IntToU64, bits, Ty::U64)
                }
                Err(_) => {
                    self.error(
                        codes::BAD_NUMBER,
                        span,
                        "this number does not fit in an `Int` (64 bits)",
                    );
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Float(text) => {
                let clean: String = text.chars().filter(|c| *c != '_').collect();
                match clean.parse::<f64>() {
                    Ok(value) => self.mk(ExprKind::Float(value), Ty::Float, span),
                    Err(_) => {
                        self.error(codes::BAD_NUMBER, span, "this is not a number");
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Str(pieces) => self.string(pieces, span),
            ast::ExprKind::Char(c) => match self.ascii(&c.to_string(), span) {
                Some(bytes) => self.mk(ExprKind::Char(bytes[0]), Ty::Char, span),
                None => self.err_expr(span),
            },
            ast::ExprKind::Bool(value) => self.mk(ExprKind::Bool(*value), Ty::Bool, span),
            ast::ExprKind::None => match hint {
                Some(ty @ Ty::Opt(_)) => self.mk(ExprKind::None, ty.clone(), span),
                Some(Ty::Error) => self.err_expr(span),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell which optional this `none` is",
                    );
                    self.help("say the type where it is stored: `let best: ?Int = None`");
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Name(name) => {
                if let Some(p) = self.cx.mut_aliases.get(name) {
                    return place_to_expr(p);
                }
                if let Some(init) = self.recipe(name) {
                    return self.expr(&init, None);
                }
                self.name(name, inner_hint, span)
            }
            ast::ExprKind::SelfValue => {
                if let Some(slot) = self.lookup("self") {
                    return self.local(slot, span);
                }
                if let Some((ref recv_place, _)) = self.cx.inline_self {
                    return place_to_expr(recv_place);
                }
                if let Some((slot, _)) = self.cx.receiver {
                    return self.local(slot, span);
                }
                if let Some((slot, _)) = self.capture_self_from_outer() {
                    return self.local(slot, span);
                }
                if self.cx.is_closure {
                    if let Some(slot) = self.capture_from_outer("self") {
                        return self.local(slot, span);
                    }
                }
                self.error(codes::UNRESOLVED_NAME, span, "`self` outside a method");
                self.err_expr(span)
            }
            ast::ExprKind::Cpp(_) => self.err_expr(span),
            ast::ExprKind::Paren(inner) if self.is_filled(inner) => {
                let held = self.expr(inner, None);
                match held.ty.clone() {
                    Ty::Opt(ty) => self.mk(ExprKind::Unwrap(Box::new(held), false), *ty, span),
                    _ => held,
                }
            }
            ast::ExprKind::Paren(inner) => self.expr(inner, hint),
            ast::ExprKind::Unary { op, expr } => self.unary(*op, expr, inner_hint, span),
            ast::ExprKind::Binary { op, lhs, rhs, .. } => {
                // `(l + 1)!` and `l + 1 ?? r` ask for the optional.
                let written = std::mem::take(&mut self.cx.as_written);
                let value = self.binary(*op, lhs, rhs, hint, span);
                match value.ty.clone() {
                    // A step a test or a loop keeps inside its domain is a
                    // position. It is still checked, so a proof that is
                    // wrong stops the program and can't read past an array.
                    Ty::Opt(inner) if !written && self.step_stays(e, &inner) => {
                        self.mk(ExprKind::Unwrap(Box::new(value), true), *inner, span)
                    }
                    _ => value,
                }
            }
            ast::ExprKind::Compare { first, rest } => self.compare(first, rest, span),
            ast::ExprKind::Range {
                lo,
                hi,
                inclusive,
                step: None,
            } => match (lo, hi, hint) {
                // `..` where a `Range<D>` is wanted is all of `D`.
                (None, None, Some(Ty::Range(of))) if matches!(**of, Ty::Pos(_)) => {
                    let Ty::Pos(d) = **of else { unreachable!() };
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                _ => self.range_val(lo.as_deref(), hi.as_deref(), *inclusive, hint, span),
            },
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a stepped range kept as a value");
                self.err_expr(span)
            }
            ast::ExprKind::Call { callee, args } => self.call(e, callee, args, hint),
            ast::ExprKind::Index { base, indices } => {
                if indices.len() == 1 && !matches!(indices[0].kind, ast::ExprKind::Range { .. }) {
                    if let Some(mapped) = self.lower_map_view_index(base, indices, span) {
                        return mapped;
                    }
                }
                let raw_base = self.expr(base, None);
                let base_expr = match (&raw_base.kind, &raw_base.ty) {
                    (ExprKind::Builtin(Builtin::First, args), Ty::Opt(inner)) => Expr {
                        kind: ExprKind::Builtin(Builtin::FirstVal, args.clone()),
                        ty: (**inner).clone(),
                        span: raw_base.span,
                    },
                    (ExprKind::Builtin(Builtin::Last, args), Ty::Opt(inner)) => Expr {
                        kind: ExprKind::Builtin(Builtin::LastVal, args.clone()),
                        ty: (**inner).clone(),
                        span: raw_base.span,
                    },
                    _ => raw_base,
                };
                if let Ty::Struct(sid) = base_expr.ty {
                    if let Some(key_ty) = self.counter_key.get(&sid).cloned() {
                        if indices.len() != 1 {
                            self.error(
                                codes::WRONG_ARG_COUNT,
                                span,
                                format!("a counter takes 1 index, found {}", indices.len()),
                            );
                            return self.err_expr(span);
                        }
                        let map_ty = self.structs[sid].def.fields[0].1.clone();
                        let map_expr =
                            self.mk(ExprKind::Field(Box::new(base_expr), 0), map_ty, span);
                        let key = self.expect(&indices[0], &key_ty);
                        let found = self.mk(
                            ExprKind::Builtin(Builtin::MapGet, vec![map_expr, key]),
                            Ty::opt(Ty::Int),
                            span,
                        );
                        let zero = self.int_lit(0, span);
                        return self.mk(
                            ExprKind::Binary(BinOp::Coalesce, Box::new(found), Box::new(zero)),
                            Ty::Int,
                            span,
                        );
                    }
                    let ast_args: Vec<ast::Arg> = indices
                        .iter()
                        .map(|idx| ast::Arg {
                            name: None,
                            mutable: false,
                            value: idx.clone(),
                        })
                        .collect();
                    if let Some(&fn_id) = self.structs[sid].methods.get("get") {
                        return self.method_call(fn_id, Some((base, base_expr)), &ast_args, span);
                    }
                    if let Some(&gfn_id) = self.structs[sid].gen_methods.get("get") {
                        return self.call_gen_fn(
                            gfn_id,
                            Vec::new(),
                            Some(Arg::Value(base_expr)),
                            &ast_args,
                            span,
                        );
                    }
                    if let Some(call) = self.struct_at(sid, base, indices) {
                        return self.expr(&call, hint);
                    }
                }
                let piece_cell = matches!(&base_expr.ty, Ty::Array { dims, .. } if matches!(dims.as_slice(), [Dim::Sym(_)]));
                let sliced = !piece_cell
                    && indices.len() == 1
                    && matches!(indices[0].kind, ast::ExprKind::Range { .. });
                if sliced
                    && self.mut_slice_inits.contains(&self.span(span))
                    && self.is_mutable_place(base)
                    && Self::sliceable_mut_elem(&base_expr.ty).is_some()
                {
                    return self.mut_slice_expr(base, &indices[0], span);
                }
                if sliced {
                    return self.slice_expr(base_expr, &indices[0], span);
                }
                // `g[r]` on `[R, C]T` is the row `g[r, ..]`.
                let padded;
                let indices = match &base_expr.ty {
                    Ty::Array { dims, .. }
                        if indices.len() + 1 == dims.len()
                            && !indices.iter().any(is_whole)
                            && !self.indexes_whole_grid(&indices[0]) =>
                    {
                        padded = indices
                            .iter()
                            .cloned()
                            .chain([ast::Expr {
                                kind: ast::ExprKind::Range {
                                    lo: None,
                                    hi: None,
                                    inclusive: false,
                                    step: None,
                                },
                                span,
                            }])
                            .collect::<Vec<_>>();
                        &padded[..]
                    }
                    _ => indices,
                };
                // `g[r, ..]`, `g[.., c]`, `g[..2, ..2]`: the cells a row,
                // column or sub-rectangle names, as a list.
                if let Ty::Array { dims, elem } = &base_expr.ty {
                    let whole: Vec<usize> = (0..indices.len())
                        .filter(|&k| is_whole(&indices[k]))
                        .collect();
                    if !whole.is_empty() && !piece_cell && dims.len() == indices.len() {
                        let (dims, elem) = (dims.clone(), (**elem).clone());
                        let first = self.axis_key(&whole, &dims).map(|_| self.temp(Ty::Int));
                        let cells = self.collected(elem, span, |lw, out, push| {
                            let base = lw.hold(base_expr, out);
                            let ats: Vec<Slot> = whole
                                .iter()
                                .map(|&axis| {
                                    lw.temp(match dims[axis] {
                                        Dim::Pos(d) => Ty::Pos(d),
                                        Dim::Bound(d) => Ty::Bound(d),
                                        Dim::Enum(e) => Ty::Enum(e),
                                        Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                                    })
                                })
                                .collect();
                            lw.whole_axes = ats.clone();
                            let (cell, ty) = lw.index_args(&base.ty, indices, span);
                            lw.whole_axes.clear();
                            let cell: Vec<Expr> = cell
                                .into_iter()
                                .enumerate()
                                .map(|(k, index)| {
                                    if whole.contains(&k) {
                                        index
                                    } else {
                                        lw.hold(index, out)
                                    }
                                })
                                .collect();
                            let parts: Vec<(Expr, Expr)> = whole
                                .iter()
                                .map(|&axis| {
                                    let len = lw.mk(
                                        ExprKind::Builtin(
                                            Builtin::AxisLen,
                                            vec![base.clone(), lw.int_lit(axis as i64, span)],
                                        ),
                                        Ty::Int,
                                        span,
                                    );
                                    // `g[r, 1..3]`: the part of the row the range names.
                                    lw.axis_part(&indices[axis], dims[axis], len, out)
                                })
                                .collect();
                            if let (Some(at), Some((lo, _))) = (first, parts.first()) {
                                out.push(lw.set_local(at, lo.clone(), span));
                            }
                            let value = lw.mk(ExprKind::Index(Box::new(base), cell), ty, span);
                            let mut body = Block {
                                stmts: vec![push(lw, value)],
                            };
                            for (at, (lo, hi)) in ats.into_iter().zip(parts).rev() {
                                let lid = lw.fresh_loop();
                                let iter = Iter::Range {
                                    lo,
                                    hi,
                                    step: None,
                                    rev: false,
                                    closed: false,
                                };
                                body = Block {
                                    stmts: vec![Stmt::For(lid, at, iter, body)],
                                };
                            }
                            out.extend(body.stmts);
                        });
                        return self.keyed_axis(cells, &whole, &dims, first, span);
                    }
                }
                // Runtime `Range<T>` value passed as an index (e.g. `vals[r]` inside `Compressed.get<Range<D>>`).
                if indices.len() == 1
                    && matches!(
                        base_expr.ty,
                        Ty::Array { .. }
                            | Ty::List(_)
                            | Ty::ListView(_, _)
                            | Ty::Keyed(_, _)
                            | Ty::MutView(..)
                            | Ty::Str
                            | Ty::StrView
                    )
                {
                    if let Some(Ty::Range(_)) =
                        self.probe_ty_silent(&indices[0]).filter(|_| !piece_cell)
                    {
                        let range_val = self.expr(&indices[0], None);
                        if self.mut_slice_inits.contains(&self.span(span))
                            && self.is_mutable_place(base)
                            && Self::sliceable_mut_elem(&base_expr.ty).is_some()
                        {
                            return self.mut_slice_by_range_val(base, range_val, span);
                        }
                        return self.slice_by_range_val(base_expr, range_val, span);
                    }
                }
                if indices.len() == 1 && Self::keyed_index(&base_expr.ty) {
                    let view = self.hold_view(base_expr, span);
                    let (indices, ty) = self.index_args(&view.ty, indices, span);
                    let out = self.key_index(view, indices.into_iter().next().unwrap(), ty, span);
                    let plain = self.plain_ty(&out.ty);
                    return self.try_coerce(out, &plain).unwrap_or_else(|e| e);
                }
                let (indices, ty) = self.index_args(&base_expr.ty, indices, span);
                let indices = self.vouched(&base_expr.ty, indices);
                let out = self.mk(ExprKind::Index(Box::new(base_expr), indices), ty, span);
                let plain = self.plain_ty(&out.ty);
                self.try_coerce(out, &plain).unwrap_or_else(|e| e)
            }
            ast::ExprKind::Field { base, name } => self.field(e, base, name, inner_hint),
            ast::ExprKind::Generic { base, args } => self.generic_expr(base, args, span),
            ast::ExprKind::Tuple(items) if items.is_empty() => {
                self.mk(ExprKind::Unit, Ty::Unit, span)
            }
            ast::ExprKind::Tuple(items) => {
                if let Some(v) = self.inline_product_value(e, inner_hint) {
                    return v;
                }
                let hints = inner_hint
                    .and_then(Ty::fields)
                    .filter(|f| f.len() == items.len());
                let sig_doms = self.cx.new_domain_slots.clone();
                let mut remaps: Vec<(DomainId, DomainId)> = Vec::new();
                let mut dom_idx = 0usize;
                let mut values: Vec<Expr> = Vec::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    let v = match &hints {
                        Some(hints) => {
                            let mut h = hints[i].clone();
                            for &(from, to) in &remaps {
                                h = super::stmt::remap_domain_ty(&h, from, to);
                            }
                            self.component(item, Some(&h))
                        }
                        None => self.component(item, None),
                    };
                    if let ExprKind::DomainVal(local_dom) = v.kind {
                        if let Some(&sig_dom) = sig_doms.get(dom_idx) {
                            if sig_dom != local_dom {
                                remaps.push((sig_dom, local_dom));
                            }
                        }
                        dom_idx += 1;
                    }
                    values.push(v);
                }
                let ty = match (inner_hint, &hints) {
                    (Some(ty @ (Ty::Tuple(_) | Ty::Record(_))), Some(_)) => ty.clone(),
                    _ => Ty::Tuple(values.iter().map(|v| v.ty.clone()).collect()),
                };
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::Record(fields) => {
                let hints = match inner_hint {
                    Some(Ty::Record(want))
                        if want.len() == fields.len()
                            && want.iter().zip(fields).all(|((w, _), (f, _))| *w == f.text) =>
                    {
                        Some(want.clone())
                    }
                    _ => None,
                };
                let values: Vec<Expr> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (_, value))| match &hints {
                        Some(hints) => self.component(value, Some(&hints[i].1)),
                        None => self.component(value, None),
                    })
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&values)
                        .map(|((name, _), value)| (name.text.clone(), value.ty.clone()))
                        .collect(),
                );
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::List(items) => self.list(items, inner_hint, span),
            ast::ExprKind::Map(entries) => self.map_lit(entries, inner_hint, span),
            ast::ExprKind::Comprehension { body, clauses } => {
                self.comprehension(body, clauses, inner_hint, span)
            }
            ast::ExprKind::Table {
                domains,
                params,
                body,
            } => self.table(domains, params, body, inner_hint, span),
            ast::ExprKind::TableFill { domain, value } => {
                let fill = self.expr(value, None);
                let dim = match &domain.kind {
                    ast::ExprKind::Field { base, name } if name.text == "keys" => {
                        let b = self.expr(base, None);
                        let lo = match &b.kind {
                            ExprKind::Builtin(Builtin::ListSlice | Builtin::StrSlice, args) => args
                                .get(1)
                                .and_then(|e| match &e.kind {
                                    ExprKind::Int(v) => Some(*v),
                                    _ => None,
                                })
                                .unwrap_or(0),
                            ExprKind::Local(slot) => {
                                self.cx.view_lo.get(slot).copied().unwrap_or(0)
                            }
                            ExprKind::Global(gid) => {
                                self.global_view_lo.get(gid).copied().unwrap_or(0)
                            }
                            _ => 0,
                        };
                        let len = self.len_of(b, span);
                        if lo == 0 {
                            DimSpec::Int(Box::new(len))
                        } else {
                            DimSpec::IntFrom(lo, Box::new(len))
                        }
                    }
                    ast::ExprKind::Range {
                        lo, hi, inclusive, ..
                    } => {
                        let lo_e = lo
                            .as_ref()
                            .map(|l| self.expr(l, None))
                            .unwrap_or_else(|| self.int_lit(0, span));
                        let hi_e = hi
                            .as_ref()
                            .map(|h| self.expr(h, None))
                            .unwrap_or_else(|| self.int_lit(0, span));
                        let lo_const = match &lo_e.kind {
                            ExprKind::Int(v) => *v,
                            _ => 0,
                        };
                        let diff = self.mk(
                            ExprKind::Binary(BinOp::Sub, Box::new(hi_e), Box::new(lo_e)),
                            Ty::Int,
                            span,
                        );
                        let len = if *inclusive {
                            self.int_add(diff, 1)
                        } else {
                            diff
                        };
                        if lo_const == 0 {
                            DimSpec::Int(Box::new(len))
                        } else {
                            DimSpec::IntFrom(lo_const, Box::new(len))
                        }
                    }
                    _ => {
                        let d = self.expr(domain, None);
                        DimSpec::Int(Box::new(d))
                    }
                };
                let ty = Ty::Array {
                    dims: vec![dim.dim()],
                    elem: Box::new(fill.ty.clone()),
                };
                self.mk(ExprKind::Fill(vec![dim], Some(Box::new(fill))), ty, span)
            }
            ast::ExprKind::Lambda {
                params,
                param_tys,
                param_mut,
                ret_ty: _,
                body,
            } => self.lower_lambda(params, param_tys, param_mut, body, inner_hint, span),
            ast::ExprKind::Keep { target, max, value } => self.keep(target, *max, value, span),
            ast::ExprKind::Block(block) => self.do_expr(hint, span, |lw, out| {
                out.extend(lw.block(block, Tail::Give).stmts);
            }),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                if self.cx.exits == Some(e.span) {
                    return self.leaving_value(e, hint, span);
                }
                if let Some(value) = self.ternary(branches, otherwise.as_ref(), hint) {
                    return value;
                }
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::Match { .. } => {
                if self.cx.exits == Some(e.span) {
                    return self.leaving_value(e, hint, span);
                }
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::HasBound { param, trait_name } => {
                let has = self.has_bound(&param.text, &trait_name.text);
                self.mk(ExprKind::Bool(has), Ty::Bool, span)
            }
            ast::ExprKind::Without {
                prefix,
                base,
                removed,
            } => {
                let mut kept_tys = Vec::new();
                let mut kept_vals = Vec::new();
                for (pname, pexpr) in prefix {
                    let pv = self.expr(pexpr, None);
                    kept_tys.push((pname.text.clone(), pv.ty.clone()));
                    kept_vals.push(pv);
                }
                let b = self.expr(base, None);
                match b.ty.clone() {
                    Ty::Record(fs) => {
                        let slot = self.temp(b.ty.clone());
                        let mut stmts = vec![Stmt::Let(slot, Some(b))];
                        for (idx, (fname, fty)) in fs.into_iter().enumerate() {
                            if removed.iter().any(|r| r.text == fname) {
                                continue;
                            }
                            let base_ref = self.local(slot, span);
                            kept_vals.push(self.mk(
                                ExprKind::Field(Box::new(base_ref), idx),
                                fty.clone(),
                                span,
                            ));
                            kept_tys.push((fname, fty));
                        }
                        let out_ty = Ty::Record(kept_tys);
                        let res = self.mk(ExprKind::Tuple(kept_vals), out_ty.clone(), span);
                        stmts.push(Stmt::Give(res));
                        self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
                    }
                    _ if kept_tys.is_empty() => b,
                    _ => {
                        let out_ty = Ty::Record(kept_tys);
                        self.mk(ExprKind::Tuple(kept_vals), out_ty, span)
                    }
                }
            }
            ast::ExprKind::With { base, updates } => self.do_expr(hint, span, |lw, out| {
                let base_val = lw.expr(base, hint);
                let ty = base_val.ty.clone();
                if let Some(frame) = lw.cx.dos.last_mut() {
                    if frame.ty.is_none() {
                        frame.ty = Some(ty.clone());
                    }
                }
                let fields: Vec<(String, Ty)> = match &ty {
                    Ty::Struct(sid) => lw.structs[*sid].def.fields.clone(),
                    Ty::Record(fields) => fields.clone(),
                    Ty::Error => {
                        out.push(Stmt::Give(lw.err_expr(span)));
                        return;
                    }
                    other => {
                        let msg = format!(
                            "`with` updates fields of a struct or record, not `{}`",
                            lw.show(other)
                        );
                        lw.error(codes::TYPE_MISMATCH, span, msg);
                        out.push(Stmt::Give(lw.err_expr(span)));
                        return;
                    }
                };
                for (name, _) in updates {
                    if !fields.iter().any(|(f, _)| f == &name.text) {
                        lw.error(
                            codes::UNKNOWN_FIELD,
                            name.span,
                            format!("`{}` has no field `{}`", lw.show(&ty), name.text),
                        );
                    }
                }
                let needs_tmp = fields
                    .iter()
                    .any(|(fname, _)| !updates.iter().any(|(n, _)| &n.text == fname));
                let tmp = if needs_tmp {
                    let tmp = lw.temp(ty.clone());
                    out.push(Stmt::Let(tmp, Some(base_val)));
                    Some(tmp)
                } else {
                    if !matches!(base_val.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        out.push(Stmt::Expr(base_val));
                    }
                    None
                };
                let mut values = Vec::with_capacity(fields.len());
                for (idx, (fname, fty)) in fields.iter().enumerate() {
                    if let Some((_, val_expr)) =
                        updates.iter().rev().find(|(n, _)| &n.text == fname)
                    {
                        values.push(lw.expect(val_expr, fty));
                    } else {
                        let base_ref = lw.local(tmp.expect("needs_tmp"), span);
                        values.push(lw.mk(
                            ExprKind::Field(Box::new(base_ref), idx),
                            fty.clone(),
                            span,
                        ));
                    }
                }
                let built = lw.mk(ExprKind::Tuple(values), ty, span);
                out.push(Stmt::Give(built));
            }),
            ast::ExprKind::Mut(inner) => self.expr(inner, inner_hint),
            ast::ExprKind::Try(inner) => self.try_expr(inner, span),
            ast::ExprKind::Unwrap(inner) => {
                self.cx.as_written = narrow::asks_optional(inner);
                let value = self.expr_maybe_opt_reduce(inner);
                self.cx.as_written = false;
                match value.ty.clone() {
                    Ty::Opt(inside) => {
                        self.mk(ExprKind::Unwrap(Box::new(value), true), *inside, span)
                    }
                    Ty::Error => value,
                    ref ty if Self::unwraps_cells(ty) => self.unwrap_cells(value, span),
                    // Inside a generic function (e.g. `bsearch_first<I>` instantiated with `I = Int`),
                    // `(base + mid)!` is a no-op when `base + mid` is already non-optional `Int`.
                    _ if !self.gen_env.is_empty() => value,
                    other => {
                        let message = format!(
                            "`!` takes the value out of an optional, but this is a `{}`",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Missing | ast::ExprKind::Error => self.err_expr(span),
        }
    }

    /// A block with a value. The first value given decides the type when
    /// the context doesn't.
    /// `best min= d` as a `Bool`: the update of the statement, then whether
    /// the place holds something else than before. It runs where it is
    /// written, so not at all on the unused side of an `and`.
    fn keep(&mut self, target: &ast::Expr, max: bool, value: &ast::Expr, span: Span) -> Expr {
        // `none` on the right is skipped: nothing changes.
        if matches!(value.kind, ast::ExprKind::None) {
            return self.mk(ExprKind::Bool(false), Ty::Bool, span);
        }
        let op = if max {
            ast::AssignOp::Max
        } else {
            ast::AssignOp::Min
        };
        if self.grid_row(target).is_some() {
            return self.do_expr(Some(&Ty::Bool), span, |lw, out| {
                let changed = match lw.grid_keep(target, Some(max), value, true, out) {
                    Some(changed) => lw.local(changed, span),
                    None => lw.err_expr(span),
                };
                out.push(Stmt::Give(changed));
            });
        }
        self.do_expr(Some(&Ty::Bool), span, |lw, out| {
            // What the place needs first (the check of `o! min= d`) runs
            // here, and not ahead of the whole statement.
            let outer = std::mem::take(&mut lw.cx.pending);
            let place = lw.place(target);
            out.extend(std::mem::replace(&mut lw.cx.pending, outer));
            let place = lw.settle_indexes(place, out);
            let rhs = lw.expr(value, Some(&place.ty));
            if place.ty == Ty::Error {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            }
            if lw.is_row(&place.ty) {
                let changed = lw.row_keep(place, max, rhs, true, target.span, out);
                let changed = match changed {
                    Some(changed) => lw.local(changed, span),
                    None => lw.err_expr(span),
                };
                out.push(Stmt::Give(changed));
                return;
            }
            let now = place_to_expr(&place);
            let old = lw.temp(place.ty.clone());
            // A key the map does not have yet takes the value.
            let had = match &place.kind {
                PlaceKind::Index(base, keys)
                    if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) =>
                {
                    let has = lw.mk(
                        ExprKind::Builtin(
                            Builtin::Contains,
                            vec![place_to_expr(base), keys[0].clone()],
                        ),
                        Ty::Bool,
                        span,
                    );
                    let slot = lw.temp(Ty::Bool);
                    out.push(Stmt::Let(slot, Some(has)));
                    Some(lw.local(slot, span))
                }
                _ => None,
            };
            match &had {
                Some(had) => {
                    let held = Place {
                        kind: PlaceKind::Local(old),
                        ty: place.ty.clone(),
                        span,
                    };
                    out.push(Stmt::Let(old, None));
                    out.push(Stmt::If(
                        had.clone(),
                        Block {
                            stmts: vec![Stmt::Assign(held, AssignOp::Set, now.clone())],
                        },
                        Block::default(),
                    ));
                }
                None => out.push(Stmt::Let(old, Some(now.clone()))),
            }
            lw.emit_assign_place(place, op, rhs, target.span, out);
            let before = lw.local(old, span);
            let mut changed = lw.mk(
                ExprKind::Cmp(CmpOp::Ne, Box::new(now), Box::new(before)),
                Ty::Bool,
                span,
            );
            if let Some(had) = had {
                let fresh = lw.mk(ExprKind::Unary(UnOp::Not, Box::new(had)), Ty::Bool, span);
                changed = lw.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(fresh), Box::new(changed)),
                    Ty::Bool,
                    span,
                );
            }
            out.push(Stmt::Give(changed));
        })
    }

    pub(super) fn do_expr(
        &mut self,
        hint: Option<&Ty>,
        span: Span,
        body: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) -> Expr {
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops: self.cx.loops.len(),
            open: false,
        });
        let mut out = Vec::new();
        body(self, &mut out);
        let frame = self.cx.dos.pop().expect("pushed above");
        self.mk(
            ExprKind::Do(Block { stmts: out }),
            frame.ty.unwrap_or(Ty::Error),
            span,
        )
    }

    /// A `match` or `if` that is the whole value of a statement and has
    /// an arm that leaves (`Skip => continue`): its arms run as statements
    /// ahead of the statement, each storing its value, so the arm that
    /// leaves is an ordinary `continue`.
    fn leaving_value(&mut self, e: &ast::Expr, hint: Option<&Ty>, span: Span) -> Expr {
        let loops = self.cx.dos.last().map_or(0, |frame| frame.loops);
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops,
            open: true,
        });
        let mut stmts = Vec::new();
        self.tail(e, Tail::Give, &mut stmts);
        let frame = self.cx.dos.pop().expect("pushed above");
        let ty = frame.ty.unwrap_or(Ty::Error);
        let kept = self.temp(ty.clone());
        let place = Place {
            kind: PlaceKind::Local(kept),
            ty,
            span: self.span(span),
        };
        store_gives(&mut stmts, &place);
        self.cx.pending.push(Stmt::Let(kept, None));
        self.cx.pending.extend(stmts);
        self.local(kept, span)
    }

    /// An `if` whose branches are single expressions is a conditional
    /// expression: `if a < b: b - a else: a - b`.
    fn ternary(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        fn single(block: &ast::Block) -> Option<&ast::Expr> {
            match block.stmts.as_slice() {
                [ast::Stmt {
                    kind: ast::StmtKind::Expr(e),
                    ..
                }] if !matches!(
                    e.kind,
                    ast::ExprKind::If { .. }
                        | ast::ExprKind::Match { .. }
                        | ast::ExprKind::Block(_)
                ) =>
                {
                    Some(e)
                }
                _ => None,
            }
        }
        let else_block = otherwise?;
        let otherwise = single(else_block)?;
        let mut arms = Vec::new();
        for (cond, block) in branches {
            let ast::Cond::Expr(cond) = cond else {
                return None;
            };
            arms.push((cond, single(block)?));
        }
        let mut lowered = Vec::new();
        let mut ty = hint.cloned();
        let first_needs_hint = ty.is_none()
            && arms.first().is_some_and(|(_, v)| {
                matches!(v.kind, ast::ExprKind::None)
                    || matches!(&v.kind, ast::ExprKind::List(items) if items.is_empty())
            });
        let mut pre_otherwise = None;
        let mark = self.cx.present.len();
        if first_needs_hint {
            for (at, (cond, _)) in branches.iter().enumerate() {
                let mutators = self.mutators.clone();
                let past =
                    narrow::past(cond, &branches[at + 1..], Some(else_block), &mut |f, x| {
                        mutators.writes(f, x)
                    });
                self.cx.present.extend(past.into_iter().map(String::from));
            }
            let other_val = self.expr(otherwise, None);
            self.cx.present.truncate(mark);
            if other_val.ty != Ty::Error {
                if matches!(arms[0].1.kind, ast::ExprKind::None) {
                    ty = Some(match &other_val.ty {
                        Ty::Opt(_) => other_val.ty.clone(),
                        t => Ty::opt(t.clone()),
                    });
                } else {
                    ty = Some(other_val.ty.clone());
                }
            }
            pre_otherwise = Some(other_val);
        }
        // In a branch its test holds; past it, the test has failed.
        for (at, (cond, value)) in arms.into_iter().enumerate() {
            let cond = self.expect(cond, &Ty::Bool);
            let (test, body) = &branches[at];
            let past = self.cx.present.len();
            let mutators = self.mutators.clone();
            let held = narrow::in_branch(test, body, &mut |f, x| mutators.writes(f, x));
            self.cx.present.extend(held.into_iter().map(String::from));
            let value = self.expr(value, ty.as_ref());
            self.cx.present.truncate(past);
            let mutators = self.mutators.clone();
            let failed = narrow::past(test, &branches[at + 1..], Some(else_block), &mut |f, x| {
                mutators.writes(f, x)
            });
            self.cx.present.extend(failed.into_iter().map(String::from));
            if ty.is_none() {
                ty = Some(value.ty.clone());
            }
            let at = value.span;
            lowered.push((cond, value, at));
        }
        let mut result = match pre_otherwise {
            Some(v) => v,
            None => self.expr(otherwise, ty.as_ref()),
        };
        self.cx.present.truncate(mark);
        // The branches agree on one type: the hint, or the widest of them.
        let mut ty = ty.unwrap_or(Ty::Error);
        for candidate in lowered.iter().map(|(_, v, _)| &v.ty).chain([&result.ty]) {
            if ty == Ty::Int && self.int_joins(candidate) {
                ty = candidate.clone();
            }
            if let (Ty::Range(c_inner), Ty::Range(t_inner)) = (candidate, &ty) {
                if matches!(
                    (c_inner.as_ref(), t_inner.as_ref()),
                    (Ty::Pos(d1), Ty::Bound(d2)) if d1 == d2
                ) {
                    ty = candidate.clone();
                }
            }
            if let Ty::Opt(inner) = candidate {
                if **inner == ty {
                    ty = candidate.clone();
                }
            }
        }
        result = self.coerce(result, &ty, otherwise.span);
        for (cond, value, span) in lowered.into_iter().rev() {
            let value = match self.try_coerce(value, &ty) {
                Ok(value) => value,
                Err(value) => {
                    let message = format!(
                        "the branches of this `if` have different types: `{}` and `{}`",
                        self.show(&ty),
                        self.show(&value.ty)
                    );
                    self.diags
                        .push(Diagnostic::error(codes::TYPE_MISMATCH, span, message));
                    Expr {
                        kind: ExprKind::Error,
                        ty: Ty::Error,
                        span,
                    }
                }
            };
            let span = cond.span;
            result = Expr {
                kind: ExprKind::If(Box::new(cond), Box::new(value), Box::new(result)),
                ty: ty.clone(),
                span,
            };
        }
        Some(result)
    }

    /// Whether the step `e`, a `?I`, is known to stay inside `I`: the
    /// same question `cpl check` asks ([`cpl_syntax::order`]).
    fn step_stays(&mut self, e: &ast::Expr, domain: &Ty) -> bool {
        if !matches!(domain, Ty::Pos(_)) {
            return false;
        }
        let steps = self.steps.clone();
        let Some(steps) = steps.get(self.cur_mod) else {
            return false;
        };
        steps.bounds(e, self.offset).into_iter().any(|name| {
            // `i > D.first`: an end of the domain itself.
            if let Some(of) = cpl_syntax::order::end_domain(name) {
                return matches!(domain, Ty::Pos(d) if self.resolve_domain_name(of) == Some(*d));
            }
            let (diags, written) = (self.diags.len(), self.cx.as_written);
            let ty = self.name_as_written(name, None, e.span).ty;
            self.diags.truncate(diags);
            self.cx.as_written = written;
            ty == *domain
        })
    }

    /// A name as a value. One that a test has shown to hold a value is the
    /// value it holds (see [`narrow`]).
    pub(super) fn name(&mut self, name: &str, hint: Option<&Ty>, span: Span) -> Expr {
        if name == "io" && self.lookup("io").is_none() && self.get_sym("io").is_none() {
            let gid = self.ensure_io_global();
            let ty = self.globals[gid].ty.clone();
            return self.mk(ExprKind::Global(gid), ty, span);
        }
        // `x!`, `x ?? 0`, `match x`: the optional itself is asked for.
        let written = std::mem::take(&mut self.cx.as_written);
        let value = self.name_as_written(name, hint, span);
        let shown = |cx: &FnCx| cx.present.iter().any(|n| n == name);
        let present = !written
            && (shown(&self.cx) || (self.cx.is_closure && self.outer_cxs.iter().any(shown)));
        match value.ty.clone() {
            Ty::Opt(inner) if present => {
                self.mk(ExprKind::Unwrap(Box::new(value), false), *inner, span)
            }
            _ => value,
        }
    }

    fn name_as_written(&mut self, name: &str, hint: Option<&Ty>, span: Span) -> Expr {
        if let Some(view) = self.map_view_named(name) {
            return self.lower_map_view_materialize(&view, span);
        }
        if let Some(slot) = self.lookup(name) {
            return self.local(slot, span);
        }
        if let Some((ref recv_place, owner)) = self.cx.inline_self.clone() {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = place_to_expr(recv_place);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        if let Some((slot, owner)) = self.cx.receiver {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = self.local(slot, span);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        if let Some(sid) = self.self_struct {
            if let Some(sc) = self.structs[sid].consts.get(name).cloned() {
                return self.eval_struct_const(sid, sc, span);
            }
        }
        if let Some(slot) = self.capture_from_outer(name) {
            return self.local(slot, span);
        }
        if let Some((slot, owner)) = self.capture_self_from_outer() {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = self.local(slot, span);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        // Generic value parameter (e.g. `P` in `Mod<P>`) or generic domain parameter (`C` in `(C, comp)`).
        if let Some(ga) = self.gen_env.get(name).cloned() {
            match ga {
                GenArg::Int(v) => return self.mk(ExprKind::Int(v), Ty::Int, span),
                GenArg::RuntimeInt(gid) => return self.mk(ExprKind::Global(gid), Ty::Int, span),
                GenArg::Domain(d) => return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span),
                GenArg::Ty(_) | GenArg::Op(_) | GenArg::Range(_, _) => {}
            }
        }
        if let Some(&Ty::Pos(d)) = self.cx.local_type_aliases.get(name) {
            return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span);
        }
        match self.get_sym(name) {
            Some(Sym::Global(id)) => return self.global_read(id, span),
            Some(Sym::ModConst(mcid)) => return self.ensure_mod_const(mcid, span),
            Some(Sym::Variant(id, tag)) => {
                let (id, tag) = self.expected_variant(name, hint).unwrap_or((id, tag));
                if let Some(&d) = self.enum_to_sum_domain.get(&id) {
                    if self.sum_enums[&d].variants[tag].1.is_empty() {
                        return self.encode_sum_variant(d, tag, Vec::new(), span);
                    }
                    let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    return self.err_expr(span);
                }
                if self.enums[id].variants[tag].1.is_empty() {
                    return self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span);
                }
                let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
            }
            Some(Sym::Fn(id)) => return self.fn_as_closure(id, hint, span),
            Some(Sym::GenFn(gfid)) => return self.gen_fn_as_closure(gfid, name, hint, span),
            Some(Sym::Domain(d)) => return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span),
            Some(Sym::Struct(id)) if self.structs[id].def.fields.is_empty() => {
                return self.mk(ExprKind::Tuple(Vec::new()), Ty::Struct(id), span);
            }
            Some(Sym::Struct(id)) => {
                // Struct constructor passed as a function value (e.g., `sort(mut pts, by: ByAngle)`).
                let fields: Vec<Ty> = self.structs[id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                let mut locals = Vec::new();
                let mut params = Vec::new();
                let mut args_expr = Vec::new();
                for (i, fty) in fields.iter().enumerate() {
                    locals.push(Local {
                        name: format!("_p{i}"),
                        ty: fty.clone(),
                    });
                    params.push(i);
                    args_expr.push(Expr {
                        kind: ExprKind::Local(i),
                        ty: fty.clone(),
                        span: self.span(span),
                    });
                }
                let body = Block {
                    stmts: vec![Stmt::Return(Some(Expr {
                        kind: ExprKind::Tuple(args_expr),
                        ty: Ty::Struct(id),
                        span: self.span(span),
                    }))],
                };
                let cid = self.closures.len();
                self.closures.push(ClosureDef {
                    params,
                    muts: Vec::new(),
                    captures: Vec::new(),
                    locals,
                    ret: Ty::Struct(id),
                    body,
                });
                let fn_ty = Ty::Fn(fields, Box::new(Ty::Struct(id)));
                return self.mk(ExprKind::Closure(cid), fn_ty, span);
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                if let Ty::Pos(d) = resolved {
                    return self.mk(ExprKind::DomainVal(d), Ty::DomainVal, span);
                }
                // R04.2.10: `type Cell = (Row, Col)` as a value is the tuple of its
                // factor domains, walked like `for p in (Row, Col)`.
                if let Ty::Tuple(items) = &resolved {
                    let doms: Option<Vec<Expr>> = items
                        .iter()
                        .map(|t| match t {
                            Ty::Pos(d) => {
                                Some(self.mk(ExprKind::DomainVal(*d), Ty::DomainVal, span))
                            }
                            _ => None,
                        })
                        .collect();
                    if let Some(doms) = doms.filter(|d| d.len() > 1) {
                        let ty = Ty::Tuple(vec![Ty::DomainVal; doms.len()]);
                        return self.mk(ExprKind::Tuple(doms), ty, span);
                    }
                }
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
                if m == 0 && matches!(ty_ast.kind, ast::TypeKind::Range { .. }) {
                    self.range_alias_as_domain(name, ty_ast.span.shift(offset));
                }
            }
            Some(Sym::GenStruct(_) | Sym::Enum(_) | Sym::GenEnum(_) | Sym::GenAlias(_)) => {
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
            }
            _ if name == "INF" => return self.mk(ExprKind::Int(INF), Ty::Int, span),
            _ if matches!(name, "min" | "max") => {
                let (ps, r) = match hint {
                    Some(Ty::Fn(ps, r)) if ps.len() == 2 => (ps.clone(), r.clone()),
                    _ => (vec![Ty::Int, Ty::Int], Box::new(Ty::Int)),
                };
                let a_ty = ps[0].clone();
                let op = if name == "min" {
                    Builtin::Min
                } else {
                    Builtin::Max
                };
                let mut locals = vec![
                    Local {
                        name: "a".into(),
                        ty: a_ty.clone(),
                    },
                    Local {
                        name: "b".into(),
                        ty: a_ty.clone(),
                    },
                ];
                let body = Block {
                    stmts: vec![Stmt::Return(Some(Expr {
                        kind: ExprKind::Builtin(
                            op,
                            vec![
                                Expr {
                                    kind: ExprKind::Local(0),
                                    ty: a_ty.clone(),
                                    span: self.span(span),
                                },
                                Expr {
                                    kind: ExprKind::Local(1),
                                    ty: a_ty.clone(),
                                    span: self.span(span),
                                },
                            ],
                        ),
                        ty: (*r).clone(),
                        span: self.span(span),
                    }))],
                };
                let cid = self.closures.len();
                self.closures.push(ClosureDef {
                    params: vec![0, 1],
                    muts: Vec::new(),
                    captures: Vec::new(),
                    locals: std::mem::take(&mut locals),
                    ret: (*r).clone(),
                    body,
                });
                return self.mk(ExprKind::Closure(cid), Ty::Fn(ps, r), span);
            }
            _ if name == "todo" => {
                let at = self.span(span);
                let stmts = vec![Stmt::Trap("`todo` reached".into(), at)];
                return self.mk(ExprKind::Do(Block { stmts }), Ty::Error, span);
            }
            _ => {
                if self.tail_names.contains(name) {
                    let gen = matches!(self.aside, Some(Aside::Gen(_)));
                    let message = if gen {
                        format!(
                            "`{name}` is read from the input, which a `@gen` fn is there to make"
                        )
                    } else {
                        format!("`{name}` is the solution's own work: a brute or a checker shares only the input")
                    };
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help(if gen {
                        "a `@gen` fn shares only `const`s and fns with the file; pick the value with `rand(lo, hi)` and print it"
                    } else {
                        "it sees every top-level line down to the last `read()`; compute the rest inside the fn"
                    });
                } else if self.judge {
                    let message = format!("an `@interactor` can't see the solution's `{name}`");
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help("the judge is a program of its own: it reads the hidden test with `read()` and shares only `const`s with the solution");
                } else if !self.reported_an_error() {
                    let message = format!("no value named `{name}` here");
                    self.error(codes::UNRESOLVED_NAME, span, message);
                    self.help("a top-level `let` is visible only after the line that sets it");
                }
            }
        }
        self.err_expr(span)
    }

    fn generic_expr(&mut self, base: &ast::Expr, args: &[ast::Type], span: Span) -> Expr {
        if let ast::ExprKind::Name(name) = &base.kind {
            if let Some(Sym::GenConst(gcid)) = self.get_sym(name) {
                let gargs = self.resolve_gen_args(args);
                return self.eval_gen_const(gcid, gargs);
            }
        }
        if let ast::ExprKind::Field { base: mbase, name } = &base.kind {
            if let Some(Sym::Mod(target_mod)) = self.sym(mbase) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                self.cur_mod = saved_mod;
                if let Some(Sym::GenConst(gcid)) = sym {
                    let gargs = self.resolve_gen_args(args);
                    return self.eval_gen_const(gcid, gargs);
                }
            }
        }
        self.unsupported(span, "explicit type arguments here");
        self.err_expr(span)
    }

    /// `j..i` with integer literal ends where a `Range<V>` is wanted: the
    /// piece between the cuts they label (R04.4.26, R04.8.8).
    fn literal_piece(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        expected: Option<&Ty>,
        span: Span,
    ) -> Option<Expr> {
        let Some(Ty::Range(of)) = expected else {
            return None;
        };
        let Ty::Pos(d) = **of else {
            return None;
        };
        let literal =
            |e: Option<&ast::Expr>| e.is_none_or(|e| matches!(e.kind, ast::ExprKind::Int(_)));
        if (lo.is_none() && hi.is_none()) || !literal(lo) || !literal(hi) {
            return None;
        }
        let ord = |lw: &mut Self, e: &ast::Expr| {
            let label = lw.expr(e, None);
            let first = lw.domain_first_label(d, span);
            lw.mk(
                ExprKind::Binary(BinOp::Sub, Box::new(label), Box::new(first)),
                Ty::Int,
                span,
            )
        };
        let lo = match lo {
            Some(e) => ord(self, e),
            None => self.int_lit(0, span),
        };
        let hi = match hi {
            Some(e) if inclusive => {
                let at = ord(self, e);
                self.int_add(at, 1)
            }
            Some(e) => ord(self, e),
            None => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
        };
        Some(self.mk(
            ExprKind::Range {
                lo: Box::new(lo),
                hi: Box::new(hi),
            },
            Ty::Range(Box::new(Ty::Pos(d))),
            span,
        ))
    }

    fn range_val(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        expected: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if let Some(piece) = self.literal_piece(lo, hi, inclusive, expected, span) {
            return piece;
        }
        let lo_e = lo.map(|e| self.expr(e, None));
        let hi_e = hi.map(|e| self.expr(e, None));
        match (lo_e, hi_e) {
            (Some(l), Some(r)) => match (&l.ty, &r.ty) {
                (Ty::Int, Ty::Int) => {
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
                (Ty::I128, Ty::I128) | (Ty::I128, Ty::Int) | (Ty::Int, Ty::I128) => {
                    let l = self.coerce(l, &Ty::I128, span);
                    let r = self.coerce(r, &Ty::I128, span);
                    let hi = if inclusive {
                        let one = self.coerce(self.int_lit(1, span), &Ty::I128, span);
                        self.mk(
                            ExprKind::Binary(BinOp::Add, Box::new(r), Box::new(one)),
                            Ty::I128,
                            span,
                        )
                    } else {
                        r
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::I128)),
                        span,
                    )
                }
                (Ty::Pos(d), Ty::Pos(d2)) if d == d2 => {
                    let d = *d;
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                (Ty::Bound(d), Ty::Bound(d2)) if d == d2 => {
                    let d = *d;
                    let convert_to_pos = !inclusive
                        && matches!(
                            expected,
                            Some(Ty::Range(inner)) if matches!(inner.as_ref(), Ty::Pos(ed) if *ed == d)
                        );
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if !convert_to_pos && inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    let elem = if convert_to_pos {
                        Ty::Pos(d)
                    } else {
                        Ty::Bound(d)
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(elem)),
                        span,
                    )
                }
                // `lo..hi` with `F64` ends is an interval: it is searched, never walked.
                (Ty::Float, Ty::Float) if !inclusive => self.mk(
                    ExprKind::Range {
                        lo: Box::new(l),
                        hi: Box::new(r),
                    },
                    Ty::Range(Box::new(Ty::Float)),
                    span,
                ),
                (Ty::Float, Ty::Float) => {
                    let message = "an interval of `F64`s is written `lo..hi`, not `..=`";
                    self.error(codes::TYPE_MISMATCH, span, message.to_string());
                    self.help("a real interval has no last element, so `..` and `..=` are the same set to a search");
                    self.err_expr(span)
                }
                (Ty::Error, _) | (_, Ty::Error) => self.err_expr(span),
                (Ty::U64, _) | (_, Ty::U64) => {
                    self.unsupported(span, "a range of `U64` values");
                    self.help("loop over an `Int` or `I128` range and narrow inside: `U64(x)`");
                    self.err_expr(span)
                }
                _ => {
                    self.bad_range_ends((lo, hi), (&l.ty, &r.ty), inclusive, span);
                    self.err_expr(span)
                }
            },
            (Some(l), None) => match l.ty {
                Ty::Pos(d) | Ty::Bound(d) => {
                    let want_bound =
                        matches!(l.ty, Ty::Bound(_)) && Self::cut_range_wanted(expected, d);
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let hi_ord = if want_bound {
                        self.int_add(len, 1)
                    } else {
                        len
                    };
                    let elem = if want_bound { Ty::Bound(d) } else { Ty::Pos(d) };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(elem)),
                        span,
                    )
                }
                // `(0..).find(..)`: a range with no end stops when the
                // search does. It runs to `Int.MAX`.
                Ty::Int => {
                    let hi = self.int_lit(i64::MAX, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, Some(r)) => match r.ty {
                Ty::Int => {
                    let lo = self.int_lit(0, span);
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::Range(Box::new(Ty::Int)),
                        span,
                    )
                }
                Ty::Pos(d) => {
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(Ty::Pos(d))),
                        span,
                    )
                }
                Ty::Bound(d) => {
                    let want_bound = Self::cut_range_wanted(expected, d);
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if want_bound && inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    let elem = if want_bound { Ty::Bound(d) } else { Ty::Pos(d) };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::Range(Box::new(elem)),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, None) => {
                self.unsupported(span, "a range with neither end");
                self.err_expr(span)
            }
        }
    }

    /// `(d? + 1) ?? -1`: a `?` on the left of `??` makes the whole left
    /// side `None`, and the default is taken. (R02.4.7)
    fn try_extent(&mut self, lhs: &ast::Expr, span: Span) -> Expr {
        let lid = self.next_loop;
        self.next_loop += 1;
        let outer = self.cx.try_extent.replace(lid);
        let pending = std::mem::take(&mut self.cx.pending);
        let value = self.expr_maybe_opt_reduce(lhs);
        let mut body = std::mem::replace(&mut self.cx.pending, pending);
        self.cx.try_extent = outer;
        let (ty, value) = match value.ty {
            Ty::Error => return value,
            Ty::Opt(_) => (value.ty.clone(), value),
            ref t => {
                let ty = Ty::opt(t.clone());
                (
                    ty.clone(),
                    self.mk(ExprKind::Some(Box::new(value)), ty, span),
                )
            }
        };
        self.do_expr(Some(&ty), span, |lw, out| {
            let slot = lw.temp(ty.clone());
            out.push(Stmt::Let(
                slot,
                Some(lw.mk(ExprKind::None, ty.clone(), span)),
            ));
            let place = Place {
                kind: PlaceKind::Local(slot),
                ty: ty.clone(),
                span: lw.span(span),
            };
            body.push(Stmt::Assign(place, AssignOp::Set, value));
            body.push(Stmt::Break(Some(lid)));
            let yes = lw.mk(ExprKind::Bool(true), Ty::Bool, span);
            out.push(Stmt::While(lid, yes, Block { stmts: body }));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }

    fn try_expr(&mut self, inner: &ast::Expr, span: Span) -> Expr {
        self.cx.as_written = narrow::asks_optional(inner);
        let opt = self.expr(inner, None);
        self.cx.as_written = false;
        let Ty::Opt(inner_ty) = opt.ty.clone() else {
            if opt.ty != Ty::Error {
                self.error(
                    codes::TYPE_MISMATCH,
                    span,
                    format!("`?` needs an optional, found `{}`", self.show(&opt.ty)),
                );
            }
            return self.err_expr(span);
        };
        if let Some(lid) = self.cx.try_extent {
            // Hoisted ahead of the left side of `??`, inside its loop: a
            // `break` from a value block would not reach the loop in C++.
            let slot = self.temp(opt.ty.clone());
            self.cx.pending.push(Stmt::Let(slot, Some(opt)));
            let held = self.local(slot, span);
            let is_some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
            let is_none = self.mk(
                ExprKind::Unary(UnOp::Not, Box::new(is_some)),
                Ty::Bool,
                span,
            );
            let leave = Block {
                stmts: vec![Stmt::Break(Some(lid))],
            };
            self.cx
                .pending
                .push(Stmt::If(is_none, leave, Block::default()));
            return self.mk(ExprKind::Unwrap(Box::new(held), false), *inner_ty, span);
        }
        let ret_ty = self
            .cx
            .ret
            .clone()
            .unwrap_or_else(|| Ty::opt((*inner_ty).clone()));
        self.do_expr(Some(&inner_ty), span, |lw, out| {
            let slot = lw.temp(opt.ty.clone());
            out.push(Stmt::Let(slot, Some(opt)));
            let held = lw.local(slot, span);
            let is_some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
            let is_none = lw.mk(
                ExprKind::Unary(UnOp::Not, Box::new(is_some)),
                Ty::Bool,
                span,
            );
            let ret_none = lw.mk(ExprKind::None, ret_ty.clone(), span);
            out.push(Stmt::If(
                is_none,
                Block {
                    stmts: vec![Stmt::Return(Some(ret_none))],
                },
                Block::default(),
            ));
            let val = lw.mk(
                ExprKind::Unwrap(Box::new(held), false),
                (*inner_ty).clone(),
                span,
            );
            out.push(Stmt::Give(val));
        })
    }

    fn map_lit(
        &mut self,
        entries: &[(ast::Expr, ast::Expr)],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let (kh, vh, hashed) = match hint {
            Some(Ty::Map(k, v)) => (Some((**k).clone()), Some((**v).clone()), false),
            Some(Ty::HashMap(k, v)) => (Some((**k).clone()), Some((**v).clone()), true),
            _ => (None, None, false),
        };
        let mut pairs = Vec::with_capacity(entries.len());
        let mut kt = kh;
        let mut vt = vh;
        for (k, v) in entries {
            let ke = match &kt {
                Some(t) => self.expect(k, t),
                None => {
                    let e = self.expr(k, None);
                    kt = Some(e.ty.clone());
                    e
                }
            };
            let ve = match &vt {
                Some(t) => self.expect(v, t),
                None => {
                    let e = self.expr(v, None);
                    vt = Some(e.ty.clone());
                    e
                }
            };
            pairs.push((ke, ve));
        }
        let k_ty = kt.unwrap_or(Ty::Int);
        let v_ty = vt.unwrap_or(Ty::Int);
        let map_ty = if hashed {
            Ty::HashMap(Box::new(k_ty), Box::new(v_ty.clone()))
        } else {
            Ty::Map(Box::new(k_ty), Box::new(v_ty.clone()))
        };
        self.do_expr(Some(&map_ty), span, |lw, out| {
            let slot = lw.temp(map_ty.clone());
            out.push(Stmt::Let(
                slot,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span)),
            ));
            for (k, v) in pairs {
                let place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(slot),
                            ty: map_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![k],
                    ),
                    ty: v_ty.clone(),
                    span: lw.span(span),
                };
                out.push(Stmt::Assign(place, AssignOp::Set, v));
            }
            let res = lw.local(slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// The bytes of literal text. `Char` is a byte and `Str` is bytes, so
    /// text outside ASCII has no value.
    fn ascii(&mut self, text: &str, span: Span) -> Option<Vec<u8>> {
        match text.chars().find(|c| !c.is_ascii()) {
            None => Some(text.as_bytes().to_vec()),
            Some(c) => {
                let message =
                    format!("`{c}` is not ASCII: a `Char` is one byte and a `Str` is a byte array");
                self.error(codes::NON_ASCII, span, message);
                self.help("only ASCII can be written in a literal; other bytes come from input");
                None
            }
        }
    }

    fn string(&mut self, pieces: &[ast::StrPiece], span: Span) -> Expr {
        let mut out: Vec<FmtPiece> = Vec::new();
        for piece in pieces {
            match piece {
                ast::StrPiece::Text(text) => {
                    let Some(bytes) = self.ascii(text, span) else {
                        return self.err_expr(span);
                    };
                    // `{{` and `}}` are how a brace is written.
                    let mut plain = Vec::with_capacity(bytes.len());
                    let mut i = 0;
                    while i < bytes.len() {
                        plain.push(bytes[i]);
                        let doubled =
                            matches!(bytes[i], b'{' | b'}') && bytes.get(i + 1) == Some(&bytes[i]);
                        i += if doubled { 2 } else { 1 };
                    }
                    match out.last_mut() {
                        Some(FmtPiece::Text(prev)) => prev.extend(plain),
                        _ => out.push(FmtPiece::Text(plain)),
                    }
                }
                ast::StrPiece::Expr(e) => {
                    let value = self.expr(e, None);
                    if let Some(arg) = self.fmt_arg(value, crate::FLOAT_DIGITS, e.span) {
                        out.push(FmtPiece::Value(arg));
                    }
                }
                ast::StrPiece::Spec(e, spec) => {
                    let value = self.expr(e, None);
                    let fixed = match spec.kind {
                        ast::FmtKind::Fixed(places) => Some(places),
                        _ => None,
                    };
                    let written = spec.to_string();
                    let help = "use `:.6f` for an F64, `:04` / `:x` / `:b` for an Int";
                    let (value, kind) = match (&value.ty, fixed, spec.kind) {
                        (Ty::Error, _, _) => continue,
                        (Ty::Float, Some(places), _) => (value, NumKind::Fixed(places)),
                        (Ty::Int, Some(places), _) => {
                            let at = value.span;
                            let cast = ExprKind::Cast(Cast::IntToFloat, Box::new(value));
                            (self.mk(cast, Ty::Float, at), NumKind::Fixed(places))
                        }
                        (Ty::Int, None, ast::FmtKind::Hex) => (value, NumKind::Hex),
                        (Ty::Int, None, ast::FmtKind::Bin) => (value, NumKind::Bin),
                        (Ty::Int, None, _) => (value, NumKind::Dec),
                        (Ty::Float, None, _) => {
                            let message =
                                format!("`{written}` does not say how many decimals an F64 gets");
                            self.error(codes::TYPE_MISMATCH, e.span, message);
                            self.help(help);
                            continue;
                        }
                        (ty, _, _) => {
                            let message = format!(
                                "`{written}` is not a number format this value takes: it is a `{}`",
                                self.show(ty)
                            );
                            self.error(codes::TYPE_MISMATCH, e.span, message);
                            self.help(help);
                            continue;
                        }
                    };
                    let fmt = NumFmt {
                        zero: spec.zero,
                        width: spec.width,
                        kind,
                    };
                    out.push(FmtPiece::Number(value, fmt));
                }
            }
        }
        match out.as_slice() {
            [] => self.mk(ExprKind::Str(Vec::new()), Ty::Str, span),
            [FmtPiece::Text(text)] => self.mk(ExprKind::Str(text.clone()), Ty::Str, span),
            _ => self.mk(ExprKind::Format(out), Ty::Str, span),
        }
    }

    /// A `Str` or a `Char` as a `Str`, for `+`.
    pub(super) fn as_str(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Str | Ty::Error => value,
            Ty::Char => {
                let at = value.span;
                Expr {
                    kind: ExprKind::Format(vec![FmtPiece::Value(FmtArg {
                        value,
                        digits: crate::FLOAT_DIGITS,
                    })]),
                    ty: Ty::Str,
                    span: at,
                }
            }
            _ => {
                self.mismatch(span, &Ty::Str, &value.ty);
                self.err_expr(span)
            }
        }
    }

    fn unary(&mut self, op: UnOp, operand: &ast::Expr, hint: Option<&Ty>, span: Span) -> Expr {
        match op {
            UnOp::Pos => {
                let value = self.expr(operand, hint);
                match &value.ty {
                    Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Error => value,
                    _ => {
                        let message =
                            format!("`+` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Neg => {
                let value = self.expr(operand, hint);
                match (&value.kind, &value.ty) {
                    (ExprKind::Int(v), _) => self.mk(ExprKind::Int(-*v), Ty::Int, span),
                    (ExprKind::Float(v), _) => self.mk(ExprKind::Float(-*v), Ty::Float, span),
                    (_, Ty::Int | Ty::I128 | Ty::U64 | Ty::Float) => {
                        let ty = value.ty.clone();
                        self.mk(ExprKind::Unary(op, Box::new(value)), ty, span)
                    }
                    (_, Ty::Struct(sid)) => {
                        let sid = *sid;
                        if let Some(fn_id) = self.struct_op_fn(sid, "neg") {
                            let ret = self.ret_of(fn_id, span);
                            return self.mk(
                                ExprKind::Call(fn_id, vec![Arg::Value(value)]),
                                ret,
                                span,
                            );
                        }
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                    (_, Ty::Error) => value,
                    // `-x` on a `?Int` or `?F64` keeps `none`.
                    (_, Ty::Opt(inner)) if matches!(**inner, Ty::Int | Ty::Float) => {
                        let zero = if **inner == Ty::Int {
                            self.mk(ExprKind::Int(0), Ty::Int, span)
                        } else {
                            self.mk(ExprKind::Float(0.0), Ty::Float, span)
                        };
                        self.tropical(BinOp::Sub, zero, value, span)
                    }
                    _ => {
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Not => {
                let value = self.expect(operand, &Ty::Bool);
                self.mk(ExprKind::Unary(op, Box::new(value)), Ty::Bool, span)
            }
            UnOp::BitNot => {
                let value = self.expr(operand, hint.filter(|ty| matches!(ty, Ty::I128 | Ty::U64)));
                if let Ty::Opt(inner) = &value.ty {
                    if **inner == Ty::Int {
                        let neg_one = self.mk(ExprKind::Int(-1), Ty::Int, span);
                        return self.tropical(BinOp::BitXor, value, neg_one, span);
                    }
                }
                let ty = match value.ty {
                    Ty::I128 | Ty::U64 | Ty::Error => value.ty.clone(),
                    _ => Ty::Int,
                };
                let value = self.coerce(value, &ty, operand.span);
                self.mk(ExprKind::Unary(op, Box::new(value)), ty, span)
            }
        }
    }

    pub(super) fn cast(&self, cast: Cast, value: Expr, ty: Ty) -> Expr {
        let span = value.span;
        Expr {
            kind: self.vouch_pos(ExprKind::Cast(cast, Box::new(value)), &ty, span),
            ty,
            span,
        }
    }

    fn binary(
        &mut self,
        op: BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        use BinOp::*;
        match op {
            And | Or => {
                let l = self.expect(lhs, &Ty::Bool);
                let past = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_rhs(op, lhs, rhs, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let r = self.expect(rhs, &Ty::Bool);
                self.cx.present.truncate(past);
                return self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Bool,
                    span,
                );
            }
            Coalesce => {
                self.cx.as_written = narrow::asks_optional(lhs);
                let l = if has_try(lhs) {
                    self.try_extent(lhs, span)
                } else {
                    self.expr_maybe_opt_reduce(lhs)
                };
                self.cx.as_written = false;
                let Ty::Opt(inner) = l.ty.clone() else {
                    if l.ty != Ty::Error {
                        let message = format!(
                            "`??` gives a default for an optional, but the left side is a `{}`",
                            self.show(&l.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.expr(rhs, None);
                    }
                    return self.err_expr(span);
                };
                if let ast::ExprKind::Block(block) = &rhs.kind {
                    if let [way_out @ ast::Stmt {
                        kind:
                            ast::StmtKind::Return(_)
                            | ast::StmtKind::Break(_)
                            | ast::StmtKind::Continue(_),
                        ..
                    }] = block.stmts.as_slice()
                    {
                        if self.cx.way_out.take() != Some(span) {
                            self.error(
                                codes::UNSUPPORTED,
                                rhs.span,
                                "`?? return`, `?? break` and `?? continue` have to be the whole right side of a `let`, an assignment or a statement",
                            );
                            self.help("give the value a name first: `let u = o ?? return`");
                            return self.err_expr(span);
                        }
                        let slot = self.temp(l.ty.clone());
                        let held = self.local(slot, span);
                        let some =
                            self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                        let missing =
                            self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
                        let mut leave = Vec::new();
                        // The way out is lowered as the statement it is,
                        // outside any value block.
                        let dos = std::mem::take(&mut self.cx.dos);
                        self.stmt(way_out, &mut leave);
                        self.cx.dos = dos;
                        self.cx.pending.push(Stmt::Let(slot, Some(l)));
                        self.cx.pending.push(Stmt::If(
                            missing,
                            Block { stmts: leave },
                            Block::default(),
                        ));
                        return self.mk(ExprKind::Unwrap(Box::new(held), false), *inner, span);
                    }
                }
                // The default is read as what the optional holds, so that a
                // plain value stays one when the left side is an optional of
                // optionals (`xs.last ?? 7` on a `List<?Int>`). Only a bare
                // `none` needs the optional's own type to be one.
                let hint = match rhs.kind {
                    ast::ExprKind::None => &l.ty,
                    _ => &*inner,
                };
                let r = self.expr(rhs, Some(hint));
                // `a ?? b ?? 0`: an optional default keeps the result optional.
                let ty = if r.ty == l.ty { l.ty.clone() } else { *inner };
                let r = self.coerce(r, &ty, rhs.span);
                return self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span);
            }
            _ => {}
        }
        let unwrap_first_last = |e: Expr| -> Expr {
            match (&e.kind, &e.ty) {
                (ExprKind::Builtin(Builtin::First, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::FirstVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                (ExprKind::Builtin(Builtin::Last, args), Ty::Opt(inner)) => Expr {
                    kind: ExprKind::Builtin(Builtin::LastVal, args.clone()),
                    ty: (**inner).clone(),
                    span: e.span,
                },
                _ => e,
            }
        };
        // Where an `I128` or a `U64` is wanted, the arithmetic is done in
        // one: `let p: I128 = a * b` and `let top: U64 = 1 << 63` do not
        // overflow an `Int` on the way.
        let wide = hint.filter(|ty| matches!(ty, Ty::I128 | Ty::U64));
        // Where a list is wanted, a list written out on the left of `+`
        // holds what the list wanted holds: `let xs: List<Z> = [0] + rest`.
        let joined = hint.filter(|ty| {
            op == Add
                && matches!(ty, Ty::List(_))
                && matches!(
                    lhs.kind,
                    ast::ExprKind::List(_) | ast::ExprKind::Binary { .. } | ast::ExprKind::Paren(_)
                )
        });
        let lhs_bitset_hint = if matches!(op, Sub | BitAnd | BitOr | BitXor)
            && matches!(&lhs.kind, ast::ExprKind::Field { name, .. } if name.text == "all")
        {
            hint.filter(|ty| matches!(ty, Ty::Struct(_)))
                .cloned()
                .or_else(|| {
                    self.probe_ty_silent(rhs)
                        .filter(|ty| matches!(ty, Ty::Struct(_)))
                })
        } else {
            None
        };
        let joined_owned = if op == Add
            && joined.is_none()
            && matches!(&lhs.kind, ast::ExprKind::List(items) if items.is_empty())
        {
            self.probe_ty_silent(rhs)
                .and_then(|ty| seq_concat_elem(&ty).cloned())
                .map(Ty::list)
        } else {
            None
        };
        let mut l = unwrap_first_last(
            self.expr(
                lhs,
                wide.or(joined)
                    .or(joined_owned.as_ref())
                    .or(lhs_bitset_hint.as_ref()),
            ),
        );
        if let (Some(wide), Ty::Int) = (wide, &l.ty) {
            l = self.coerce(l, wide, lhs.span);
        }
        // A place, a parameter or a return type says what `read()` reads;
        // the other side of an operator does not.
        // A shift is by an `Int`, whatever is shifted.
        let r_concat_hint = (op == Add && !is_bare_read(rhs))
            .then(|| seq_concat_elem(&l.ty).cloned().map(Ty::list))
            .flatten();
        let r_hint = match op {
            Shl | Shr | Pow => Some(&Ty::Int),
            _ if r_concat_hint.is_some() => r_concat_hint.as_ref(),
            _ if matches!(l.ty, Ty::Pos(_) | Ty::Bound(_)) => None,
            _ => (!is_bare_read(rhs)).then_some(&l.ty),
        };
        let r = unwrap_first_last(self.expr(rhs, r_hint));
        // Dispatch operator traits (`impl Add/Sub/Mul/Div for Struct`).
        if let Ty::Struct(sid) = l.ty {
            // `z ** e` is the struct's `pow(e: Int)`: `Mod<P>` has one.
            if let (Pow, Ty::Int, Some(fn_id)) =
                (op, &r.ty, self.structs[sid].methods.get("pow").copied())
            {
                self.queue_fn(fn_id);
                let ret = self.ret_of(fn_id, span);
                return self.mk(
                    ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                    ret,
                    span,
                );
            }
            let mname = match op {
                Add => Some("add"),
                Sub => Some("sub"),
                Mul => Some("mul"),
                Div => Some("div"),
                Rem => Some("rem"),
                BitOr => Some("bitor"),
                BitAnd => Some("bitand"),
                BitXor => Some("bitxor"),
                _ => None,
            };
            if let Some((fn_id, want_rhs)) =
                mname.and_then(|m| self.struct_op_fn_for(sid, m, &r.ty))
            {
                let r = self.coerce(r, &want_rhs, rhs.span);
                let ret = self.ret_of(fn_id, span);
                return self.mk(
                    ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                    ret,
                    span,
                );
            }
        }
        // `V.all - side` (or `&`, `|`, `^`) with `side: Bitset<V>`: lift `V.all` to `Bitset<V>`.
        if let (Ty::Range(of), Ty::Struct(sid)) = (&l.ty, &r.ty) {
            if let Ty::Pos(dom) = of.as_ref() {
                let sid = *sid;
                if self.is_bitset_of(sid, *dom) {
                    let mname = match op {
                        Sub => Some("sub"),
                        BitOr => Some("bitor"),
                        BitAnd => Some("bitand"),
                        BitXor => Some("bitxor"),
                        _ => None,
                    };
                    if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                        let l = self.coerce(l, &Ty::Struct(sid), lhs.span);
                        let ret = self.ret_of(fn_id, span);
                        return self.mk(
                            ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                            ret,
                            span,
                        );
                    }
                }
            }
        }
        // `1 - z` with `z: Mod<P>`: the number becomes a `Mod<P>` first,
        // as it does on the right in `z - 1`.
        if let (Ty::Int, Ty::Struct(sid)) = (&l.ty, &r.ty) {
            let sid = *sid;
            let mname = match op {
                Add => Some("add"),
                Sub => Some("sub"),
                Mul => Some("mul"),
                Div => Some("div"),
                _ => None,
            };
            if self.is_number(sid) {
                if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                    let l = self.coerce(l, &Ty::Struct(sid), lhs.span);
                    let ret = self.ret_of(fn_id, span);
                    return self.mk(
                        ExprKind::Call(fn_id, vec![Arg::Value(l), Arg::Value(r)]),
                        ret,
                        span,
                    );
                }
            }
        }
        let arith = matches!(op, Add | Sub | Mul | Div | Pow);
        let (l, r, ty) = match (&l.ty, &r.ty) {
            (Ty::Error, _) | (_, Ty::Error) => return self.err_expr(span),
            (a, b)
                if (matches!(op, Add | Sub | Mul | Div)
                    && tropical_side(a).is_some()
                    && tropical_side(b).is_some()
                    || matches!(op, Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr)
                        && tropical_side(a) == Some(Ty::Int)
                        && tropical_side(b) == Some(Ty::Int))
                    && (matches!(a, Ty::Opt(_)) || matches!(b, Ty::Opt(_))) =>
            {
                return self.tropical(op, l, r, span);
            }
            (Ty::Int, Ty::Int) => (l, r, Ty::Int),
            // A shift keeps the type of what is shifted.
            (Ty::I128 | Ty::U64, Ty::Int) if matches!(op, Shl | Shr) => {
                let ty = l.ty.clone();
                (l, r, ty)
            }
            // An `Int` beside an `I128` or a `U64` becomes one.
            (wide @ (Ty::I128 | Ty::U64), Ty::Int) | (Ty::Int, wide @ (Ty::I128 | Ty::U64))
                if !matches!(op, Shl | Shr) =>
            {
                let wide = wide.clone();
                let l = self.coerce(l, &wide, lhs.span);
                let r = self.coerce(r, &wide, rhs.span);
                (l, r, wide)
            }
            (Ty::I128, Ty::I128) | (Ty::U64, Ty::U64) if !matches!(op, Shl | Shr) => {
                let ty = l.ty.clone();
                (l, r, ty)
            }
            (Ty::Float, Ty::Float) if arith => (l, r, Ty::Float),
            (Ty::Float, Ty::Int) if arith => {
                let r = self.coerce(r, &Ty::Float, rhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Int, Ty::Float) if arith => {
                let l = self.coerce(l, &Ty::Float, lhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Bool, Ty::Bool) if matches!(op, BitAnd | BitOr | BitXor) => (l, r, Ty::Bool),
            (Ty::Str | Ty::StrView | Ty::Char, Ty::Str | Ty::StrView)
            | (Ty::Str | Ty::StrView, Ty::Char)
                if op == Add =>
            {
                let has_view = l.ty == Ty::StrView || r.ty == Ty::StrView;
                if has_view {
                    let l = if l.ty == Ty::Char {
                        self.as_str(l, lhs.span)
                    } else {
                        l
                    };
                    let r = if r.ty == Ty::Char {
                        self.as_str(r, rhs.span)
                    } else {
                        r
                    };
                    return self.mk(
                        ExprKind::Builtin(Builtin::StrConcat, vec![l, r]),
                        Ty::Str,
                        span,
                    );
                }
                let l = self.as_str(l, lhs.span);
                let r = self.as_str(r, rhs.span);
                (l, r, Ty::Str)
            }
            (a, b)
                if op == Add
                    && seq_concat_elem(a).is_some()
                    && seq_concat_elem(a) == seq_concat_elem(b) =>
            {
                let elem = seq_concat_elem(a).unwrap().clone();
                let list_ty = Ty::list(elem);
                if !matches!(l.ty, Ty::List(_)) || !matches!(r.ty, Ty::List(_)) {
                    return self.mk(
                        ExprKind::Builtin(Builtin::ListConcat, vec![l, r]),
                        list_ty,
                        span,
                    );
                }
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    let cloned = lw.mk(
                        ExprKind::Builtin(Builtin::Copy, vec![l]),
                        list_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Let(slot, Some(cloned)));
                    let ext = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Extend,
                            Box::new(Place {
                                kind: PlaceKind::Local(slot),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![r],
                        ),
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(ext));
                    let res = lw.local(slot, span);
                    out.push(Stmt::Give(res));
                });
            }
            (Ty::Char, Ty::Char) if op == Sub => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let r = self.cast(Cast::CharToInt, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Char, Ty::Int) if matches!(op, Add | Sub) => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let sum = self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Int,
                    span,
                );
                return self.cast(Cast::StepToChar, sum, Ty::Char);
            }
            (Ty::Pos(a), Ty::Pos(b)) | (Ty::Bound(a), Ty::Bound(b)) if a == b && op == Sub => {
                let l = self.cast(Cast::Ord, l, Ty::Int);
                let r = self.cast(Cast::Ord, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Pos(d), Ty::Int) if matches!(op, Add | Sub) => {
                let d = *d;
                (self.cast(Cast::Label(d), l, Ty::Int), r, Ty::Int)
            }
            (Ty::List(elem), Ty::Int) if op == Mul => {
                let elem = (**elem).clone();
                return self.list_repeat(l, r, elem, span);
            }
            _ => {
                let message = format!(
                    "`{}` can't be applied to `{}` and `{}`",
                    op.text(),
                    self.show(&l.ty),
                    self.show(&r.ty)
                );
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        };
        if let Some(wrap) = self.wrapping(op, &ty).filter(|_| l.ty == ty && r.ty == ty) {
            return self.mk(ExprKind::Builtin(wrap, vec![l, r]), ty, span);
        }
        self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span)
    }

    /// `a + b` and `a - b` where a side is a `?Int` or `?F64`: `none` on
    /// either side makes the whole `none`.
    fn tropical(&mut self, op: BinOp, l: Expr, r: Expr, span: Span) -> Expr {
        let float =
            tropical_side(&l.ty) == Some(Ty::Float) || tropical_side(&r.ty) == Some(Ty::Float);
        let num = if float { Ty::Float } else { Ty::Int };
        let ty = Ty::opt(num.clone());
        self.do_expr(Some(&ty), span, |lw, out| {
            let mut present: Option<Expr> = None;
            let mut sides = Vec::with_capacity(2);
            for side in [l, r] {
                let side_ty = side.ty.clone();
                let slot = lw.temp(side_ty.clone());
                out.push(Stmt::Let(slot, Some(side)));
                let held = lw.local(slot, span);
                let value = if let Ty::Opt(inner) = &side_ty {
                    let some = lw.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                    present = Some(match present.take() {
                        Some(first) => lw.mk(
                            ExprKind::Binary(BinOp::And, Box::new(first), Box::new(some)),
                            Ty::Bool,
                            span,
                        ),
                        None => some,
                    });
                    lw.mk(
                        ExprKind::Unwrap(Box::new(held), false),
                        (**inner).clone(),
                        span,
                    )
                } else {
                    held
                };
                sides.push(lw.coerce(value, &num, span));
            }
            let r = sides.pop().expect("two sides");
            let l = sides.pop().expect("two sides");
            let sum = lw.mk(
                ExprKind::Binary(op, Box::new(l), Box::new(r)),
                num.clone(),
                span,
            );
            let some = lw.coerce(sum, &ty, span);
            let none = lw.mk(ExprKind::None, ty.clone(), span);
            let cond = present.expect("one side is optional");
            let value = lw.mk(
                ExprKind::If(Box::new(cond), Box::new(some), Box::new(none)),
                ty.clone(),
                span,
            );
            out.push(Stmt::Give(value));
        })
    }

    /// Whether `<` means something for the type.
    pub(super) fn ordered(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Bool
            | Ty::Char
            | Ty::Str
            | Ty::StrView
            | Ty::Pos(_)
            | Ty::Bound(_) => true,
            Ty::Tuple(items) => items.iter().all(|t| self.ordered(t)),
            Ty::Record(fields) => fields.iter().all(|(_, t)| self.ordered(t)),
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
                self.ordered(elem)
            }
            Ty::Struct(id) => {
                let fields: Vec<Ty> = self.structs[*id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                self.structs[*id].def.ord.is_some() || fields.iter().all(|t| self.ordered(t))
            }
            Ty::Enum(id) => {
                let payloads: Vec<Ty> = self.enums[*id]
                    .variants
                    .iter()
                    .flat_map(|(_, fs)| fs.clone())
                    .collect();
                payloads.iter().all(|t| self.ordered(t))
            }
            Ty::Error => true,
            _ => false,
        }
    }

    /// A comparison. One that tests for `none` reads its names as the
    /// optionals they were declared as.
    fn compare(&mut self, first: &ast::Expr, rest: &[(CmpOp, ast::Expr)], span: Span) -> Expr {
        if !narrow::tests_none(first, rest) {
            return self.compare_sides(first, rest, span);
        }
        let present = std::mem::take(&mut self.cx.present);
        let value = self.compare_sides(first, rest, span);
        self.cx.present = present;
        value
    }

    fn compare_sides(
        &mut self,
        first: &ast::Expr,
        rest: &[(CmpOp, ast::Expr)],
        span: Span,
    ) -> Expr {
        if let [(op @ (CmpOp::In | CmpOp::NotIn), container)] = rest {
            let found = self.contains(first, container, span);
            return if *op == CmpOp::In {
                found
            } else {
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(found)), Ty::Bool, span)
            };
        }
        // `none == x` needs the other side's type first.
        let mut operands: Vec<(Expr, Span)> = Vec::new();
        let asts: Vec<&ast::Expr> = std::iter::once(first)
            .chain(rest.iter().map(|(_, e)| e))
            .collect();
        for (i, e) in asts.iter().enumerate() {
            let value = if matches!(e.kind, ast::ExprKind::None) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) => {
                        let hint = Ty::opt(strip_opt(&prev.ty));
                        self.expr(e, Some(&hint))
                    }
                    (None, Some(next)) => {
                        let next = self.expr(next, None);
                        let hint = Ty::opt(strip_opt(&next.ty));
                        self.expr(e, Some(&hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(&e.kind, ast::ExprKind::List(_)) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) if matches!(prev.ty, Ty::List(_) | Ty::Array { .. }) => {
                        self.expr(e, Some(&prev.ty))
                    }
                    (Some((prev, _)), _) if matches!(&prev.ty, Ty::Opt(inner) if matches!(**inner, Ty::List(_) | Ty::Array { .. })) =>
                    {
                        let Ty::Opt(inner) = &prev.ty else {
                            unreachable!()
                        };
                        self.expr(e, Some(inner))
                    }
                    (None, Some(next)) if matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()) =>
                    {
                        let next = self.expr(next, None);
                        let hint = match &next.ty {
                            Ty::Opt(inner) if matches!(**inner, Ty::List(_) | Ty::Array { .. }) => {
                                inner.as_ref()
                            }
                            other => other,
                        };
                        self.expr(e, Some(hint))
                    }
                    _ => self.expr(e, None),
                }
            } else if matches!(
                &e.kind,
                ast::ExprKind::If { .. }
                    | ast::ExprKind::Paren(_)
                    | ast::ExprKind::Block(_)
                    | ast::ExprKind::Tuple(_)
                    | ast::ExprKind::Record(_)
                    | ast::ExprKind::Range { .. }
            ) || operands
                .last()
                .is_some_and(|(prev, _)| matches!(prev.ty, Ty::Enum(_)))
            {
                // `x == E.V(..)` of a generic enum: the variant takes `x`'s type arguments.
                self.expr(e, operands.last().map(|(prev, _)| &prev.ty))
            } else if operands
                .last()
                .is_some_and(|(prev, _)| matches!(prev.ty, Ty::Opt(_)))
            {
                if matches!(
                    &e.kind,
                    ast::ExprKind::Call { callee, .. }
                        if matches!(&callee.kind, ast::ExprKind::Name(n) if n == "some" || n == "Some")
                ) {
                    self.expr(e, operands.last().map(|(prev, _)| &prev.ty))
                } else {
                    self.expr_maybe_opt_reduce(e)
                }
            } else if matches!(e.kind, ast::ExprKind::Int(_))
                && operands.last().is_some_and(|(prev, _)| prev.ty == Ty::U64)
            {
                // `x == 0xFFFF_FFFF_FFFF_FFFF` beside a `U64` is one.
                self.expr(e, Some(&Ty::U64))
            } else {
                self.expr(e, None)
            };
            operands.push((value, e.span));
        }
        if rest.len() == 1 {
            let (r, r_span) = operands.pop().expect("two operands");
            let (l, l_span) = operands.pop().expect("two operands");
            return self.cmp(rest[0].0, l, l_span, r, r_span, span);
        }
        // `lo <= x < hi`: each operand is evaluated once, left to right.
        let mut stmts = Vec::new();
        let held: Vec<(Expr, Span)> = operands
            .into_iter()
            .map(|(value, at)| {
                let atom = matches!(
                    value.kind,
                    ExprKind::Local(_)
                        | ExprKind::Global(_)
                        | ExprKind::Int(_)
                        | ExprKind::Char(_)
                        | ExprKind::Float(_)
                );
                if atom {
                    (value, at)
                } else {
                    (self.hold(value, &mut stmts), at)
                }
            })
            .collect();
        let mut all: Option<Expr> = None;
        for (i, (op, _)) in rest.iter().enumerate() {
            let (l, l_span) = held[i].clone();
            let (r, r_span) = held[i + 1].clone();
            let one = self.cmp(*op, l, l_span, r, r_span, span);
            all = Some(match all {
                None => one,
                Some(prev) => self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(prev), Box::new(one)),
                    Ty::Bool,
                    span,
                ),
            });
        }
        let all = all.expect("a comparison has an operator");
        if stmts.is_empty() {
            return all;
        }
        stmts.push(Stmt::Give(all));
        self.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
    }

    fn cmp(&mut self, op: CmpOp, l: Expr, l_span: Span, r: Expr, r_span: Span, span: Span) -> Expr {
        if l.ty == Ty::Error || r.ty == Ty::Error {
            return self.err_expr(span);
        }
        match op {
            CmpOp::In | CmpOp::NotIn => {
                self.unsupported(span, "`in` inside a chain of comparisons");
                return self.err_expr(span);
            }
            CmpOp::Order => {
                self.unsupported(span, "`<=>`");
                return self.err_expr(span);
            }
            _ => {}
        }
        // Bring both sides to one type.
        let (l, r) = if l.ty == r.ty
            || matches!(
                (&l.ty, &r.ty),
                (Ty::Str, Ty::StrView) | (Ty::StrView, Ty::Str)
            )
            || matches!(
                (&l.ty, &r.ty),
                (Ty::List(a), Ty::ListView(b, _) | Ty::Keyed(b, _) | Ty::MutView(b, _))
                    | (Ty::ListView(a, _) | Ty::Keyed(a, _) | Ty::MutView(a, _), Ty::List(b))
                    | (Ty::ListView(a, _) | Ty::Keyed(a, _), Ty::MutView(b, _))
                    | (Ty::MutView(a, _), Ty::ListView(b, _) | Ty::Keyed(b, _))
                    if a == b
            ) {
            (l, r)
        } else if matches!((&l.ty, &r.ty), (Ty::Int, Ty::U64) | (Ty::U64, Ty::Int)) {
            // An `Int` can be negative and a `U64` past `2**63`, so the
            // two are compared as the numbers they are, in an `I128`. A
            // literal that is not negative is a `U64` already.
            let as_u64 = |e: &Expr| matches!(e.kind, ExprKind::Int(v) if v >= 0) || e.ty == Ty::U64;
            let (cast_int, cast_u64, ty) = if as_u64(&l) && as_u64(&r) {
                (Some(Cast::IntToU64), None, Ty::U64)
            } else {
                (Some(Cast::IntToI128), Some(Cast::U64ToI128), Ty::I128)
            };
            let bring = |lw: &Self, e: Expr| match if e.ty == Ty::Int { cast_int } else { cast_u64 }
            {
                Some(cast) => lw.cast(cast, e, ty.clone()),
                None => e,
            };
            (bring(self, l), bring(self, r))
        } else {
            let want = match (&l.ty, &r.ty) {
                (a, b) if a.list_meet(b).is_some() => a.list_meet(b).expect("checked"),
                (Ty::Float, Ty::Int) | (Ty::Int, Ty::Float) => Ty::Float,
                (Ty::Int, wide @ (Ty::I128 | Ty::U64)) => wide.clone(),
                (Ty::Opt(_), _) => l.ty.clone(),
                (_, Ty::Opt(_)) => r.ty.clone(),
                _ => l.ty.clone(),
            };
            let l = match self.try_coerce(l, &want) {
                Ok(l) => l,
                Err(l) => {
                    self.compare_mismatch(span, &l, &r);
                    return self.err_expr(span);
                }
            };
            let r = match self.try_coerce(r, &want) {
                Ok(r) => r,
                Err(r) => {
                    self.compare_mismatch(span, &l, &r);
                    return self.err_expr(span);
                }
            };
            (l, r)
        };
        let _ = (l_span, r_span);
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            if matches!(l.ty, Ty::Opt(_)) {
                self.error(
                    codes::OPTIONAL_COMPARE,
                    span,
                    "an optional has no order: `none` is neither small nor large",
                );
                self.help("give it a value first: `(x ?? INF) < y`");
                return self.err_expr(span);
            }
            if !self.ordered(&l.ty) {
                let message = format!("a `{}` has no order to compare by", self.show(&l.ty));
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        }
        self.mk(ExprKind::Cmp(op, Box::new(l), Box::new(r)), Ty::Bool, span)
    }

    /// The indices of `base[...]` and the type of the element.
    /// Inside `unchecked:`, the `Int` that indexes a list is marked as
    /// vouched for.
    pub(super) fn vouched(&mut self, base: &Ty, indices: Vec<Expr>) -> Vec<Expr> {
        if self.unchecked == 0
            || !matches!(
                base,
                Ty::List(_)
                    | Ty::ListView(_, _)
                    | Ty::Keyed(_, _)
                    | Ty::MutView(..)
                    | Ty::Array { .. }
                    | Ty::Str
                    | Ty::StrView
            )
        {
            return indices;
        }
        indices
            .into_iter()
            .map(|index| match index.ty {
                Ty::Int => {
                    let span = index.span;
                    Expr {
                        kind: ExprKind::Cast(Cast::Raw, Box::new(index)),
                        ty: Ty::Int,
                        span,
                    }
                }
                _ => index,
            })
            .collect()
    }

    /// Whether a lone index names a whole cell of a grid: a tuple or a
    /// product position. Lowered once to see its type; nothing is kept.
    fn indexes_whole_grid(&mut self, index: &ast::Expr) -> bool {
        match self.probe_expr(index, None).ty {
            Ty::Tuple(_) | Ty::Record(_) => true,
            Ty::Pos(p) => self.product_grid(p).is_some(),
            _ => false,
        }
    }

    pub(super) fn index_args(
        &mut self,
        base: &Ty,
        indices: &[ast::Expr],
        span: Span,
    ) -> (Vec<Expr>, Ty) {
        let one = |lw: &mut Self, want: &Ty| -> Vec<Expr> {
            if indices.len() != 1 {
                let message = format!("this takes 1 index, but {} were given", indices.len());
                lw.error(codes::WRONG_ARG_COUNT, span, message);
                return vec![lw.err_expr(span)];
            }
            vec![lw.expect(&indices[0], want)]
        };
        match base {
            Ty::Array { dims, elem } => {
                if let [Dim::Pos(d)] = dims.as_slice() {
                    if indices.len() > 1 {
                        if let Some(factors) = self.product_domains.get(d).cloned() {
                            if factors.len() == indices.len() {
                                let parts: Vec<Expr> = factors
                                    .iter()
                                    .zip(indices)
                                    .map(|((_, f), idx)| {
                                        let want = self.product_factor_ty(f);
                                        self.expect(idx, &want)
                                    })
                                    .collect();
                                let pos = self.encode_product(*d, parts, span);
                                return (vec![pos], (**elem).clone());
                            }
                        }
                    }
                }
                if let [Dim::Sym(d)] = dims.as_slice() {
                    let (u, v) = if indices.len() == 2 {
                        (
                            self.expect(&indices[0], &Ty::Pos(*d)),
                            self.expect(&indices[1], &Ty::Pos(*d)),
                        )
                    } else if indices.len() == 1 {
                        let pair = self.expr(&indices[0], None);
                        if matches!(&pair.ty, Ty::Range(inner) if matches!(**inner, Ty::Int) || **inner == Ty::Pos(*d))
                        {
                            let flat = self.piece_idx(pair, *d, span);
                            return (vec![flat], (**elem).clone());
                        }
                        let (ft0, ft1) = match &pair.ty {
                            Ty::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                            Ty::Record(fs) if fs.len() == 2 => (fs[0].1.clone(), fs[1].1.clone()),
                            _ => {
                                self.error(
                                    codes::WRONG_ARG_COUNT,
                                    span,
                                    "a symmetric table takes 2 indices `adj[u, v]` or a pair `(u, v)`",
                                );
                                return (vec![self.err_expr(span)], (**elem).clone());
                            }
                        };
                        let f0 = self.mk(
                            ExprKind::Field(Box::new(pair.clone()), 0),
                            ft0,
                            indices[0].span,
                        );
                        let f1 = self.mk(ExprKind::Field(Box::new(pair), 1), ft1, indices[0].span);
                        (
                            self.coerce(f0, &Ty::Pos(*d), indices[0].span),
                            self.coerce(f1, &Ty::Pos(*d), indices[0].span),
                        )
                    } else {
                        self.error(
                            codes::WRONG_ARG_COUNT,
                            span,
                            format!("this takes 2 indices, but {} were given", indices.len()),
                        );
                        return (vec![self.err_expr(span)], (**elem).clone());
                    };
                    let a = self.cast(Cast::Ord, u, Ty::Int);
                    let b = self.cast(Cast::Ord, v, Ty::Int);
                    let sym = self.mk(
                        ExprKind::Builtin(Builtin::SymIdx, vec![a, b]),
                        Ty::Int,
                        span,
                    );
                    let flat = self.cast(Cast::Raw, sym, Ty::Int);
                    return (vec![flat], (**elem).clone());
                }
                // `g[p]` with `p: (Row, Col)`, or `d[p, q]` on `[P, P]T`.
                if let Some(parts) = self.spread_indices(dims, indices) {
                    return (parts, (**elem).clone());
                }
                if let ([Dim::Pos(d)], 2..) = (dims.as_slice(), indices.len()) {
                    if let Some(pos) = self.library_key_call(*d, indices, span) {
                        return (vec![pos], (**elem).clone());
                    }
                }
                if dims.len() != indices.len() {
                    let message = format!(
                        "a `{}` takes {} indices, but {} were given",
                        self.show(base),
                        dims.len(),
                        indices.len()
                    );
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    if dims.len() > 1 && indices.len() == 1 {
                        self.help("a grid is indexed `g[r, c]`, never `g[r][c]`");
                    }
                    return (vec![self.err_expr(span)], (**elem).clone());
                }
                let mut out = Vec::new();
                let mut axes = self.whole_axes.clone().into_iter();
                for (dim, index) in dims.iter().zip(indices) {
                    let want = match dim {
                        Dim::Pos(d) => Ty::Pos(*d),
                        Dim::Bound(d) => Ty::Bound(*d),
                        Dim::Enum(e) => Ty::Enum(*e),
                        Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                    };
                    if let Some(slot) = is_whole(index).then(|| axes.next()).flatten() {
                        out.push(self.mk(ExprKind::Local(slot), want, index.span));
                        continue;
                    }
                    let hint = if want == Ty::Int { None } else { Some(&want) };
                    let mut value = self.expr(index, hint);
                    // `b[i]` of a `[1..=9]T` is the cell `i - 1`.
                    if let (Dim::IntFrom(lo), Ty::Int) = (dim, &value.ty) {
                        value = self.int_add(value, -*lo);
                    }
                    if let Dim::Enum(e) = dim {
                        let checked = self.coerce(value, &Ty::Enum(*e), index.span);
                        let ord = self.cast(Cast::EnumOrd(*e), checked, Ty::Int);
                        out.push(self.cast(Cast::Raw, ord, Ty::Int));
                        continue;
                    }
                    if let Dim::Pos(d) = dim {
                        value = match self.library_key_pos(*d, value, index.span) {
                            Ok(pos) => {
                                out.push(pos);
                                continue;
                            }
                            Err(value) => value,
                        };
                    }
                    // A domain that stands for an array's plain range is
                    // indexed by the plain `Int`, checked.
                    if let (Dim::Pos(d), Ty::Int) = (dim, &value.ty) {
                        if self.range_domains.contains_key(d) {
                            out.push(self.cast(Cast::FromLabel(*d), value, Ty::Pos(*d)));
                            continue;
                        }
                    }
                    // A `Bool` picks a cell of a `[Bool]T`: `false` the
                    // first, `true` the second. The checker has made
                    // sure the array is one, so the cell needs no check.
                    if (*dim, &value.ty) == (Dim::Int, &Ty::Bool) {
                        let (one, zero) =
                            (self.int_lit(1, index.span), self.int_lit(0, index.span));
                        let cell = self.mk(
                            ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
                            Ty::Int,
                            index.span,
                        );
                        out.push(self.cast(Cast::Raw, cell, Ty::Int));
                        continue;
                    }
                    // A fieldless enum variant picks a cell of `[E]T` by its tag.
                    if let (Dim::Int, Ty::Enum(eid)) = (dim, &value.ty) {
                        let eid = *eid;
                        let count = self.enums[eid].variants.len();
                        if count > 0 && self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty())
                        {
                            let tag_expr = self.do_expr(Some(&Ty::Int), index.span, |lw, stmts| {
                                let tmp = lw.temp(Ty::Enum(eid));
                                stmts.push(Stmt::Let(tmp, Some(value)));
                                let mut acc = lw.int_lit((count - 1) as i64, index.span);
                                for tag in (0..count - 1).rev() {
                                    let is_t = lw.mk(
                                        ExprKind::IsVariant(
                                            Box::new(lw.local(tmp, index.span)),
                                            tag,
                                        ),
                                        Ty::Bool,
                                        index.span,
                                    );
                                    let idx_lit = lw.int_lit(tag as i64, index.span);
                                    acc = lw.mk(
                                        ExprKind::If(
                                            Box::new(is_t),
                                            Box::new(idx_lit),
                                            Box::new(acc),
                                        ),
                                        Ty::Int,
                                        index.span,
                                    );
                                }
                                stmts.push(Stmt::Give(acc));
                            });
                            out.push(self.cast(Cast::Raw, tag_expr, Ty::Int));
                            continue;
                        }
                    }
                    if value.ty != want && value.ty != Ty::Error {
                        if want == Ty::Int {
                            if let Ty::Pos(d) = value.ty {
                                if let Some(&lo) = self.range_domains.get(&d) {
                                    let as_int = self.mk(
                                        ExprKind::Cast(Cast::Trust, Box::new(value)),
                                        Ty::Int,
                                        index.span,
                                    );
                                    let shifted = if lo == 0 {
                                        as_int
                                    } else {
                                        self.int_add(as_int, lo)
                                    };
                                    out.push(shifted);
                                    continue;
                                }
                            }
                        }
                        // R04.8.8: an integer literal is the position it labels.
                        if let (ExprKind::Int(_), Ty::Pos(d)) = (&value.kind, &want) {
                            let d = *d;
                            out.push(self.cast(Cast::FromLabel(d), value, want.clone()));
                            continue;
                        }
                        if matches!(want, Ty::Bound(_))
                            || matches!(want, Ty::Pos(d) if self.product_domains.contains_key(&d))
                        {
                            match self.try_coerce(value, &want) {
                                Ok(coerced) => {
                                    out.push(coerced);
                                    continue;
                                }
                                Err(orig) => value = orig,
                            }
                        }
                        let message = format!(
                            "this is indexed by `{}`, but the index is a `{}`",
                            self.show(&want),
                            self.show(&value.ty)
                        );
                        self.error(codes::WRONG_DOMAIN, index.span, message);
                    }
                    out.push(value);
                }
                (out, (**elem).clone())
            }
            // `a[V(2)..=V(4)][V(3)]`: a slice of an array keeps its
            // positions, held as their ordinals.
            Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _)
                if indices.len() == 1 =>
            {
                let value = self.expr(&indices[0], None);
                let key = match value.ty {
                    Ty::Pos(_) | Ty::Bound(_) => self.cast(Cast::Ord, value, Ty::Int),
                    _ => self.coerce(value, &Ty::Int, indices[0].span),
                };
                (vec![key], (**elem).clone())
            }
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
                (one(self, &Ty::Int), (**elem).clone())
            }
            Ty::Str | Ty::StrView => (one(self, &Ty::Int), Ty::Char),
            Ty::Range(inner) => (one(self, &Ty::Int), (**inner).clone()),
            Ty::Map(key, value) | Ty::HashMap(key, value) => (one(self, key), (**value).clone()),
            Ty::Error => {
                for index in indices {
                    self.expr(index, None);
                }
                (vec![self.err_expr(span)], Ty::Error)
            }
            other => {
                let message = format!("a `{}` can't be indexed", self.show(other));
                self.error(codes::NOT_INDEXABLE, span, message);
                self.tuple_index_fix(other, &indices[0], span);
                (vec![self.err_expr(span)], Ty::Error)
            }
        }
    }

    /// True when `e` refers to a mutable place whose backing storage can be
    /// mutably sliced in O(1) without copying.
    pub(super) fn is_mutable_place(&self, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unwrap(inner) => {
                self.is_mutable_place(inner)
            }
            ast::ExprKind::Field { base, .. } | ast::ExprKind::Index { base, .. } => {
                self.is_mutable_place(base)
            }
            ast::ExprKind::SelfValue => {
                self.cx
                    .receiver
                    .is_some_and(|(slot, _)| self.cx.mut_slots.contains(&slot))
                    || self.cx.inline_self.is_some()
            }
            ast::ExprKind::Name(n) => {
                if self.cx.mut_aliases.contains_key(n) {
                    return true;
                }
                if let Some(slot) = self.lookup(n) {
                    return self.cx.mut_slots.contains(&slot)
                        || matches!(self.cx.locals[slot].ty, Ty::MutView(..));
                }
                if let Some(Sym::Global(gid)) = self.get_sym(n) {
                    return self.mut_globals.contains(&gid)
                        || matches!(self.globals[gid].ty, Ty::MutView(..));
                }
                if let Some((_, sid)) = self.cx.receiver {
                    if self.field_of(&Ty::Struct(sid), n).is_some() {
                        return self
                            .cx
                            .receiver
                            .is_some_and(|(slot, _)| self.cx.mut_slots.contains(&slot));
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(super) fn sliceable_mut_elem(ty: &Ty) -> Option<Ty> {
        match ty {
            Ty::List(e) | Ty::MutView(e, _) => Some((**e).clone()),
            Ty::Array { dims, elem } if dims.len() == 1 && !matches!(dims[0], Dim::Sym(_)) => {
                Some((**elem).clone())
            }
            _ => None,
        }
    }

    /// Computes `(lo, hi, elem_ty)` (0-based half-open offsets) for slicing `base_place`
    /// by a syntactic range `lo..hi` / `lo..=hi` / `..hi` / `lo..`.
    pub(super) fn mut_slice_bounds(
        &mut self,
        base_place: &Place,
        range: &ast::Expr,
        span: Span,
    ) -> Option<(Expr, Expr, Ty)> {
        let elem_ty = Self::sliceable_mut_elem(&base_place.ty)?;
        let ast::ExprKind::Range {
            lo,
            hi,
            inclusive,
            step,
        } = &range.kind
        else {
            return None;
        };
        if step.is_some() {
            self.unsupported(span, "stepped slice");
            return None;
        }
        for bound in [lo, hi].into_iter().flatten() {
            let negative = matches!(
                &bound.kind,
                ast::ExprKind::Unary { op: UnOp::Neg, expr } if matches!(expr.kind, ast::ExprKind::Int(n) if n > 0)
            );
            if negative {
                self.error(
                    codes::BAD_OPERANDS,
                    bound.span,
                    "a slice bound counts from the start, so it can't be negative",
                );
                self.help("count from the end with the length: `a[..a.len - 1]`");
                return None;
            }
        }
        let first = match &base_place.ty {
            Ty::Array { dims, .. } => match dims[..] {
                [Dim::IntFrom(first)] => first,
                _ => 0,
            },
            _ => 0,
        };
        let len_expr = match &base_place.ty {
            Ty::Array { dims, .. } => match dims[0] {
                Dim::Pos(d) => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                Dim::Bound(d) => {
                    let n = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    self.int_add(n, 1)
                }
                _ => self.mk(
                    ExprKind::Builtin(Builtin::Len, vec![place_to_expr(base_place)]),
                    Ty::Int,
                    span,
                ),
            },
            _ => self.mk(
                ExprKind::Builtin(Builtin::Len, vec![place_to_expr(base_place)]),
                Ty::Int,
                span,
            ),
        };
        let key_lo = self.mut_view_key_lo(base_place, span);
        let to_idx = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
            let v = lw.expr(e, None);
            match v.ty {
                Ty::Int => lw.int_add(v, i64::from(is_end && *inclusive) - first),
                Ty::Pos(_) | Ty::Bound(_) => {
                    let ord = lw.cast(Cast::Ord, v, Ty::Int);
                    let ord = lw.at_view_key(ord, &key_lo, e.span);
                    if is_end && *inclusive {
                        lw.int_add(ord, 1)
                    } else {
                        ord
                    }
                }
                _ => lw.coerce(v, &Ty::Int, e.span),
            }
        };
        let lo_e = match lo {
            Some(l) => to_idx(self, l, false),
            None => self.int_lit(0, span),
        };
        let hi_e = match hi {
            Some(h) => to_idx(self, h, true),
            None => len_expr,
        };
        Some((lo_e, hi_e, elem_ty))
    }

    /// Computes `(lo, hi, elem_ty)` for slicing `base_place` by a runtime `Range<T>` value.
    pub(super) fn mut_slice_range_val_bounds(
        &mut self,
        base_place: &Place,
        range_val: Expr,
        span: Span,
    ) -> Option<(Expr, Expr, Ty)> {
        let elem_ty = Self::sliceable_mut_elem(&base_place.ty)?;
        let first = match &base_place.ty {
            Ty::Array { dims, .. } => match dims[..] {
                [Dim::IntFrom(first)] => first,
                _ => 0,
            },
            _ => 0,
        };
        let range_ty = range_val.ty.clone();
        let range_inner = match &range_ty {
            Ty::Range(inner) => (**inner).clone(),
            _ => Ty::Int,
        };
        let r_slot = self.temp(range_ty);
        self.cx.pending.push(Stmt::Let(r_slot, Some(range_val)));
        let r_ref = self.local(r_slot, span);
        let lo_bound = self.mk(
            ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
            range_inner.clone(),
            span,
        );
        let hi_bound = self.mk(
            ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
            range_inner.clone(),
            span,
        );
        let key_lo = self.mut_view_key_lo(base_place, span);
        let lo = self.slice_bound_at(lo_bound, first, &key_lo, span);
        let hi = self.slice_bound_at(hi_bound, first, &key_lo, span);
        Some((lo, hi, elem_ty))
    }

    fn mut_slice_expr(&mut self, base: &ast::Expr, range: &ast::Expr, span: Span) -> Expr {
        let base_place = self.place(base);
        let Some((lo, hi, elem_ty)) = self.mut_slice_bounds(&base_place, range, span) else {
            return self.err_expr(span);
        };
        let ty = Self::mut_view_ty(&base_place.ty, elem_ty);
        self.mk(
            ExprKind::MutSlice(Box::new(base_place), Box::new(lo), Box::new(hi)),
            ty,
            span,
        )
    }

    fn mut_slice_by_range_val(&mut self, base: &ast::Expr, range_val: Expr, span: Span) -> Expr {
        let base_place = self.place(base);
        let Some((lo, hi, elem_ty)) = self.mut_slice_range_val_bounds(&base_place, range_val, span)
        else {
            return self.err_expr(span);
        };
        let ty = Self::mut_view_ty(&base_place.ty, elem_ty);
        self.mk(
            ExprKind::MutSlice(Box::new(base_place), Box::new(lo), Box::new(hi)),
            ty,
            span,
        )
    }

    /// Ensures the base of a borrowed view outlives the view expression in C++.
    /// Places (locals, globals, fields) and existing views are passed directly
    /// with zero copies; temporary owned values are hoisted into `self.cx.pending`.
    /// Whether indexing `ty` names source keys rather than positions: in
    /// v0.3 a slice keeps its source's keys (R04b.1).
    pub(super) fn keyed_view(ty: &Ty) -> bool {
        matches!(
            ty,
            Ty::StrView | Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(_, Dim::Pos(_))
        )
    }

    /// `view` in a form that can be read more than once.
    pub(super) fn hold_view(&mut self, view: Expr, span: Span) -> Expr {
        if matches!(view.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
            return view;
        }
        let slot = self.temp(view.ty.clone());
        self.cx.pending.push(Stmt::Let(slot, Some(view)));
        self.local(slot, span)
    }

    /// `view[key]`, reading the cell that holds the source key `key`.
    pub(super) fn key_index(&mut self, view: Expr, key: Expr, elem: Ty, span: Span) -> Expr {
        let pos = self.mk(
            ExprKind::Builtin(Builtin::KeyPos, vec![view.clone(), key]),
            Ty::Int,
            span,
        );
        self.mk(ExprKind::Index(Box::new(view), vec![pos]), elem, span)
    }

    /// The smallest key of a view, `0` for a whole list or string.
    pub(super) fn key_lo(&mut self, seq: &Expr, span: Span) -> Expr {
        if Self::keyed_view(&seq.ty) {
            self.mk(
                ExprKind::Builtin(Builtin::KeyLo, vec![seq.clone()]),
                Ty::Int,
                span,
            )
        } else {
            self.int_lit(0, span)
        }
    }

    /// Whether a `for` over an open-ended range ending in `end` walks cuts:
    /// a range's elements are the kind of its ends in v0.3 (R04.4.10).
    fn walks_cuts(end: &Ty) -> bool {
        matches!(end, Ty::Bound(_))
    }

    /// Whether an open-ended range with a cut end of `d` walks cuts: always,
    /// unless a piece (`Range<V>`) is expected (R04.4.10).
    fn cut_range_wanted(expected: Option<&Ty>, d: DomainId) -> bool {
        let wants = |want: fn(&Ty, DomainId) -> bool| matches!(expected, Some(Ty::Range(inner)) if want(inner, d));
        !wants(|t, d| matches!(t, Ty::Pos(ed) if *ed == d))
    }

    /// `base[lo..hi]` in key terms on a view and positions otherwise.
    fn slice_builtin(&mut self, base: Expr, lo: Expr, hi: Expr, span: Span) -> Expr {
        let keyed = Self::keyed_view(&base.ty);
        let (op, ty) = match &base.ty {
            Ty::Str | Ty::StrView => (
                if keyed {
                    Builtin::KeySlice
                } else {
                    Builtin::StrSlice
                },
                Ty::StrView,
            ),
            Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => (
                if keyed {
                    Builtin::KeySlice
                } else {
                    Builtin::ListSlice
                },
                Ty::ListView(e.clone(), base.ty.seq_key().unwrap_or(Dim::Int)),
            ),
            // R04b.1.2: a slice of an array keeps its positions as keys.
            Ty::Array { elem, dims } => (Builtin::KeySlice, Ty::ListView(elem.clone(), dims[0])),
            _ => unreachable!(),
        };
        self.mk(ExprKind::Builtin(op, vec![base, lo, hi]), ty, span)
    }

    pub(super) fn hoist_view_base(&mut self, base: Expr, span: Span) -> Expr {
        if matches!(
            base.ty,
            Ty::StrView | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..)
        ) || matches!(
            base.kind,
            ExprKind::Local(_) | ExprKind::Global(_) | ExprKind::Field(..)
        ) {
            return base;
        }
        let slot = self.temp(base.ty.clone());
        self.cx.pending.push(Stmt::Let(slot, Some(base)));
        self.local(slot, span)
    }

    /// Lowers a slice by a runtime `Range<T>` value (`base[r]`) into a view or `List<T>`.
    fn slice_by_range_val(&mut self, base: Expr, range_val: Expr, span: Span) -> Expr {
        let keyed_array = matches!(&base.ty, Ty::Array { dims, .. } if dims.len() == 1);
        if keyed_array
            || matches!(
                base.ty,
                Ty::Str
                    | Ty::StrView
                    | Ty::List(_)
                    | Ty::ListView(_, _)
                    | Ty::Keyed(_, _)
                    | Ty::MutView(..)
            )
        {
            let base = self.hoist_view_base(base, span);
            let range_ty = range_val.ty.clone();
            let range_inner = match &range_ty {
                Ty::Range(inner) => (**inner).clone(),
                _ => Ty::Int,
            };
            let r_slot = self.temp(range_ty);
            self.cx.pending.push(Stmt::Let(r_slot, Some(range_val)));
            let r_ref = self.local(r_slot, span);
            let lo_bound = self.mk(
                ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
                range_inner.clone(),
                span,
            );
            let hi_bound = self.mk(
                ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
                range_inner.clone(),
                span,
            );
            let lo = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                self.cast(Cast::Ord, lo_bound, Ty::Int)
            } else {
                lo_bound
            };
            let hi = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                self.cast(Cast::Ord, hi_bound, Ty::Int)
            } else {
                hi_bound
            };
            return self.slice_builtin(base, lo, hi, span);
        }
        let is_str = base.ty == Ty::Str;
        let (elem_ty, dim_dom) = match &base.ty {
            Ty::List(e) => ((**e).clone(), None),
            Ty::Str => (Ty::Char, None),
            Ty::Array { dims, elem } if dims.len() == 1 => {
                let d = match dims[0] {
                    Dim::Pos(id) => Some(id),
                    _ => None,
                };
                ((**elem).clone(), d)
            }
            _ => return self.err_expr(span),
        };
        let result_ty = if is_str {
            Ty::Str
        } else {
            Ty::list(elem_ty.clone())
        };
        let base_ty = base.ty.clone();
        let range_ty = range_val.ty.clone();
        let range_inner = match &range_ty {
            Ty::Range(inner) => (**inner).clone(),
            _ => Ty::Int,
        };
        self.do_expr(Some(&result_ty), span, |lw, out| {
            let b_slot = lw.temp(base_ty.clone());
            out.push(Stmt::Let(b_slot, Some(base)));
            let r_slot = lw.temp(range_ty.clone());
            out.push(Stmt::Let(r_slot, Some(range_val)));
            let acc_slot = lw.temp(result_ty.clone());
            let init = if is_str {
                lw.mk(ExprKind::Str(Vec::new()), Ty::Str, span)
            } else {
                lw.mk(ExprKind::Tuple(Vec::new()), result_ty.clone(), span)
            };
            out.push(Stmt::Let(acc_slot, Some(init)));
            let r_ref = lw.local(r_slot, span);
            let lo_bound = lw.mk(
                ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
                range_inner.clone(),
                span,
            );
            let hi_bound = lw.mk(
                ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
                range_inner.clone(),
                span,
            );
            let lo_ord = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                lw.cast(Cast::Ord, lo_bound, Ty::Int)
            } else {
                lo_bound
            };
            let hi_ord = if matches!(range_inner, Ty::Pos(_) | Ty::Bound(_)) {
                lw.cast(Cast::Ord, hi_bound, Ty::Int)
            } else {
                hi_bound
            };
            let i_slot = lw.temp(Ty::Int);
            let i_ref = lw.local(i_slot, span);
            let idx_expr = match dim_dom {
                Some(d) => lw.mk(
                    ExprKind::Cast(Cast::Trust, Box::new(i_ref)),
                    Ty::Pos(d),
                    span,
                ),
                None => i_ref,
            };
            let elem = lw.mk(
                ExprKind::Index(Box::new(lw.local(b_slot, span)), vec![idx_expr]),
                elem_ty.clone(),
                span,
            );
            // A `Str` grows by `push` like a list: `acc = acc + c` would copy it each step.
            let push_stmt = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(acc_slot),
                        ty: result_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![elem],
                ),
                Ty::Unit,
                span,
            ));
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::For(
                lid,
                i_slot,
                Iter::Range {
                    lo: lo_ord,
                    hi: hi_ord,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            ));
            let res = lw.local(acc_slot, span);
            out.push(Stmt::Give(res));
        })
    }

    /// Lowers a slice expression `base[lo..hi]` / `base[lo..=hi]` / `base[..hi]` / `base[lo..]`
    /// on a `List<T>`, `Str`, or `[D]T` into a `List<T>` (or `Str` for `Str`).
    /// Where `g[r, lo..hi]` starts and stops along its axis of `len`
    /// cells, counted from 0. `..` is all of it. A part is
    /// `0 <= lo <= hi <= len`, as for a slice; anything else stops the
    /// program.
    pub(super) fn axis_part(
        &mut self,
        range: &ast::Expr,
        dim: Dim,
        len: Expr,
        out: &mut Vec<Stmt>,
    ) -> (Expr, Expr) {
        let span = range.span;
        let ast::ExprKind::Range {
            lo, hi, inclusive, ..
        } = &range.kind
        else {
            unreachable!("an axis is taken whole or in part by a range")
        };
        if lo.is_none() && hi.is_none() {
            return (self.int_lit(0, span), len);
        }
        let first = match dim {
            Dim::IntFrom(first) => first,
            _ => 0,
        };
        let cell = |lw: &mut Self, e: &ast::Expr, end: bool| {
            let hint = match dim {
                // `g[.., 2..4]` over `C = 0..4`: an exclusive end is a cut.
                Dim::Pos(d) if end && !*inclusive => Some(Ty::Bound(d)),
                Dim::Pos(d) => Some(Ty::Pos(d)),
                Dim::Bound(d) => Some(Ty::Bound(d)),
                Dim::Enum(_) | Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => None,
            };
            let v = lw.expr(e, hint.as_ref());
            let at = match v.ty {
                Ty::Pos(_) | Ty::Bound(_) => lw.cast(Cast::Ord, v, Ty::Int),
                _ => {
                    let v = lw.coerce(v, &Ty::Int, e.span);
                    lw.int_add(v, -first)
                }
            };
            if end && *inclusive {
                lw.int_add(at, 1)
            } else {
                at
            }
        };
        let lo_e = match lo {
            Some(lo) => cell(self, lo, false),
            None => self.int_lit(0, span),
        };
        let hi_e = match hi {
            Some(hi) => cell(self, hi, true),
            None => len.clone(),
        };
        let (lo_slot, hi_slot) = (self.temp(Ty::Int), self.temp(Ty::Int));
        out.push(Stmt::Let(lo_slot, Some(lo_e)));
        out.push(Stmt::Let(hi_slot, Some(hi_e)));
        let (lo_e, hi_e) = (self.local(lo_slot, span), self.local(hi_slot, span));
        let cmp = |lw: &mut Self, op: CmpOp, l: &Expr, r: &Expr| {
            lw.mk(
                ExprKind::Cmp(op, Box::new(l.clone()), Box::new(r.clone())),
                Ty::Bool,
                span,
            )
        };
        let zero = self.int_lit(0, span);
        let below = cmp(self, CmpOp::Lt, &lo_e, &zero);
        let reversed = cmp(self, CmpOp::Lt, &hi_e, &lo_e);
        let beyond = cmp(self, CmpOp::Gt, &hi_e, &len);
        let bad = self.mk(
            ExprKind::Binary(BinOp::Or, Box::new(below), Box::new(reversed)),
            Ty::Bool,
            span,
        );
        let bad = self.mk(
            ExprKind::Binary(BinOp::Or, Box::new(bad), Box::new(beyond)),
            Ty::Bool,
            span,
        );
        let message = match first {
            0 => "a part `lo..hi` of a row or column needs 0 <= lo <= hi <= its length".to_string(),
            _ => format!("a part `lo..hi` of a row or column indexed from {first} needs {first} <= lo <= hi <= {first} + its length"),
        };
        let trap = Stmt::Trap(message, self.span(span));
        out.push(Stmt::If(bad, Block { stmts: vec![trap] }, Block::default()));
        (lo_e, hi_e)
    }

    fn slice_expr(&mut self, base: Expr, range: &ast::Expr, span: Span) -> Expr {
        let ast::ExprKind::Range {
            lo,
            hi,
            inclusive,
            step,
        } = &range.kind
        else {
            return self.err_expr(span);
        };
        if step.is_some() {
            self.unsupported(span, "stepped slice");
            return self.err_expr(span);
        }
        // `a[..-1]` is known to be wrong before the program runs.
        for bound in [lo, hi].into_iter().flatten() {
            let negative = matches!(
                &bound.kind,
                ast::ExprKind::Unary { op: UnOp::Neg, expr } if matches!(expr.kind, ast::ExprKind::Int(n) if n > 0)
            );
            if negative {
                self.error(
                    codes::BAD_OPERANDS,
                    bound.span,
                    "a slice bound counts from the start, so it can't be negative",
                );
                self.help("count from the end with the length: `a[..a.len - 1]`");
                return self.err_expr(span);
            }
        }
        if matches!(
            base.ty,
            Ty::Str
                | Ty::StrView
                | Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
        ) {
            let base = self.hoist_view_base(base, span);
            let to_idx = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
                let v = lw.expr(e, None);
                match v.ty {
                    Ty::Int => lw.int_add(v, i64::from(is_end && *inclusive)),
                    Ty::Pos(_) => {
                        let ord = lw.cast(Cast::Ord, v, Ty::Int);
                        if is_end && *inclusive {
                            lw.int_add(ord, 1)
                        } else {
                            ord
                        }
                    }
                    Ty::Bound(_) => lw.cast(Cast::Ord, v, Ty::Int),
                    _ => lw.coerce(v, &Ty::Int, e.span),
                }
            };
            let base = if Self::keyed_view(&base.ty) {
                self.hold_view(base, span)
            } else {
                base
            };
            let lo_e = match lo {
                Some(l) => to_idx(self, l, false),
                None => self.key_lo(&base, span),
            };
            let hi_e = match hi {
                Some(h) => to_idx(self, h, true),
                None => {
                    let len = self.mk(
                        ExprKind::Builtin(Builtin::Len, vec![base.clone()]),
                        Ty::Int,
                        span,
                    );
                    let first = self.key_lo(&base, span);
                    if Self::keyed_view(&base.ty) {
                        self.int_sum(first, len)
                    } else {
                        len
                    }
                }
            };
            return self.slice_builtin(base, lo_e, hi_e, span);
        }
        // R04b.1.2: `a[V(2)..=V(4)]` keeps `a`'s positions as its keys.
        // A `[0..n]T` is a view too, as `Slice<S>` says: its keys are its positions.
        if let Ty::Array { dims, .. } = &base.ty {
            if let [dim @ (Dim::Pos(_) | Dim::Int)] = &dims[..] {
                let d = match *dim {
                    Dim::Pos(d) => Some(d),
                    _ => None,
                };
                let base = self.hoist_view_base(base, span);
                let to_ord = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
                    let v = lw.expr(e, None);
                    let ord = match (v.ty.clone(), d) {
                        (Ty::Int, None) => v,
                        (Ty::Int, Some(d)) => {
                            let first = lw.domain_first_label(d, e.span);
                            lw.mk(
                                ExprKind::Binary(BinOp::Sub, Box::new(v), Box::new(first)),
                                Ty::Int,
                                e.span,
                            )
                        }
                        (Ty::Pos(_), _) => lw.cast(Cast::Ord, v, Ty::Int),
                        (Ty::Bound(_), _) => return lw.cast(Cast::Ord, v, Ty::Int),
                        _ => return lw.coerce(v, &Ty::Int, e.span),
                    };
                    lw.int_add(ord, i64::from(is_end && *inclusive))
                };
                let lo_e = match lo {
                    Some(l) => to_ord(self, l, false),
                    None => self.int_lit(0, span),
                };
                let hi_e = match (hi, d) {
                    (Some(h), _) => to_ord(self, h, true),
                    (None, Some(d)) => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                    (None, None) => {
                        let at = vec![base.clone()];
                        self.mk(ExprKind::Builtin(Builtin::Len, at), Ty::Int, span)
                    }
                };
                return self.slice_builtin(base, lo_e, hi_e, span);
            }
        }
        let is_str = base.ty == Ty::Str;
        // `a[2..=4]` of a `[1..=4]T` is said in the array's own indices.
        let first = match &base.ty {
            Ty::Array { dims, .. } => match dims[..] {
                [Dim::IntFrom(first)] => first,
                _ => 0,
            },
            _ => 0,
        };
        let (elem_ty, dim_dom) = match &base.ty {
            Ty::List(e) => ((**e).clone(), None),
            Ty::Str => (Ty::Char, None),
            Ty::Array { dims, elem } if dims.len() == 1 => {
                let d = match dims[0] {
                    Dim::Pos(d) => Some(d),
                    _ => None,
                };
                ((**elem).clone(), d)
            }
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::BAD_OPERANDS,
                    span,
                    format!("can't slice a `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        let out_ty = if is_str {
            Ty::Str
        } else {
            Ty::list(elem_ty.clone())
        };
        self.do_expr(Some(&out_ty), span, |lw, out| {
            let base_slot = lw.temp(base.ty.clone());
            out.push(Stmt::Let(base_slot, Some(base)));
            let base_ref = lw.local(base_slot, span);
            let len_expr = match dim_dom {
                Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                None => lw.mk(ExprKind::Builtin(Builtin::Len, vec![base_ref.clone()]), Ty::Int, span),
            };
            let to_idx = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
                let v = lw.expr(e, None);
                match v.ty {
                    Ty::Int => lw.int_add(v, i64::from(is_end && *inclusive) - first),
                    Ty::Pos(_) => {
                        let ord = lw.cast(Cast::Ord, v, Ty::Int);
                        if is_end && *inclusive {
                            lw.int_add(ord, 1)
                        } else {
                            ord
                        }
                    }
                    Ty::Bound(_) => lw.cast(Cast::Ord, v, Ty::Int),
                    _ => lw.coerce(v, &Ty::Int, e.span),
                }
            };
            let lo_e = match lo {
                Some(l) => to_idx(lw, l, false),
                None => lw.int_lit(0, span),
            };
            let hi_e = match hi {
                Some(h) => to_idx(lw, h, true),
                None => len_expr.clone(),
            };
            // A slice is `0 <= lo <= hi <= len`. Anything else stops the
            // program: `a[..-1]` is not "all but the last", and going on
            // with an empty list would hide the mistake.
            let (lo_slot, hi_slot) = (lw.temp(Ty::Int), lw.temp(Ty::Int));
            out.push(Stmt::Let(lo_slot, Some(lo_e)));
            out.push(Stmt::Let(hi_slot, Some(hi_e)));
            let (lo_e, hi_e) = (lw.local(lo_slot, span), lw.local(hi_slot, span));
            let cmp = |lw: &mut Self, op: CmpOp, l: &Expr, r: &Expr| {
                lw.mk(ExprKind::Cmp(op, Box::new(l.clone()), Box::new(r.clone())), Ty::Bool, span)
            };
            let or = |lw: &mut Self, l: Expr, r: Expr| {
                lw.mk(ExprKind::Binary(BinOp::Or, Box::new(l), Box::new(r)), Ty::Bool, span)
            };
            let zero = lw.int_lit(0, span);
            let below = cmp(lw, CmpOp::Lt, &lo_e, &zero);
            let reversed = cmp(lw, CmpOp::Lt, &hi_e, &lo_e);
            let beyond = cmp(lw, CmpOp::Gt, &hi_e, &len_expr);
            let bad = or(lw, below, reversed);
            let bad = or(lw, bad, beyond);
            let message = match first {
                0 => "a slice `a[lo..hi]` needs 0 <= lo <= hi <= a.len".to_string(),
                _ => format!("a slice `a[lo..hi]` of an array indexed from {first} needs {first} <= lo <= hi <= {first} + a.len"),
            };
            out.push(Stmt::If(
                bad,
                Block {
                    stmts: vec![Stmt::Trap(message, lw.span(span))],
                },
                Block::default(),
            ));
            let acc_slot = lw.temp(out_ty.clone());
            let empty_init = if is_str {
                lw.mk(ExprKind::Str(Vec::new()), Ty::Str, span)
            } else {
                lw.mk(ExprKind::Tuple(Vec::new()), out_ty.clone(), span)
            };
            out.push(Stmt::Let(acc_slot, Some(empty_init)));
            let idx_slot = lw.temp(Ty::Int);
            let idx_e = lw.local(idx_slot, span);
            let index_arg = match dim_dom {
                Some(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(idx_e)), Ty::Pos(d), span),
                None => idx_e,
            };
            let item = lw.mk(
                ExprKind::Index(Box::new(base_ref), vec![index_arg]),
                elem_ty.clone(),
                span,
            );
            // A `Str` grows by `push` like a list: `acc = acc + c` would copy it each step.
            let push_stmt = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(acc_slot),
                        ty: out_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![item],
                ),
                Ty::Unit,
                span,
            ));
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::For(
                lid,
                idx_slot,
                Iter::Range {
                    lo: lo_e,
                    hi: hi_e,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            ));
            let res = lw.local(acc_slot, span);
            out.push(Stmt::Give(res));
        })
    }

    pub(crate) fn list(&mut self, items: &[ast::Expr], hint: Option<&Ty>, span: Span) -> Expr {
        // `BeatsSegTree([5, 1, 4])`: a literal given for a `[D]T` whose `D`
        // stands for a list's positions is that list, and sets `D`'s size.
        if let Some(want @ Ty::Array { dims, elem }) = hint {
            if matches!(dims[..], [Dim::Pos(d)] if self.list_domains.contains(&d)) {
                let list_ty = Ty::List(Box::new(self.plain_ty(elem)));
                let list = self.list(items, Some(&list_ty), span);
                return self.coerce(list, want, span);
            }
        }
        match hint {
            Some(ty @ (Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem))) => {
                let values = items.iter().map(|item| self.expect(item, elem)).collect();
                self.mk(ExprKind::Tuple(values), ty.clone(), span)
            }
            // `let primes: [Small]Int = [2, 3, 5]`: one value per position,
            // which is checked when the domain's size is known: at run time
            // (or statically for a plain enum domain `[Side]T`).
            Some(ty @ Ty::Array { dims, elem })
                if matches!(
                    dims[..],
                    [Dim::Pos(_) | Dim::Bound(_) | Dim::Enum(_) | Dim::Int]
                ) =>
            {
                let (dim, len) = match dims[0] {
                    Dim::Pos(d) => (
                        DimSpec::Pos(d),
                        self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                    ),
                    Dim::Bound(d) => {
                        let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let one = self.mk(ExprKind::Int(1), Ty::Int, span);
                        let gaps = ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one));
                        (DimSpec::Bound(d), self.mk(gaps, Ty::Int, span))
                    }
                    Dim::Enum(e) => {
                        let n = self.enums[e].variants.len() as i64;
                        (DimSpec::Enum(e), self.mk(ExprKind::Int(n), Ty::Int, span))
                    }
                    Dim::Int => {
                        let n = self.mk(ExprKind::Int(items.len() as i64), Ty::Int, span);
                        (DimSpec::Int(Box::new(n.clone())), n)
                    }
                    Dim::Sym(_) | Dim::IntFrom(_) => unreachable!("matched above"),
                };
                // `domain V = 0..3` writes its count, so `[V]Int = [0, 2]` is
                // as wrong as `[0..3]Int = [0, 2]` (R04.8.10).
                let cells = match dims[0] {
                    Dim::Pos(d) => self
                        .domain_lens
                        .get(&d)
                        .filter(|(l, h)| l == h)
                        .map(|l| l.0),
                    Dim::Bound(d) => self
                        .domain_lens
                        .get(&d)
                        .filter(|(l, h)| l == h)
                        .map(|l| l.0 + 1),
                    _ => None,
                };
                if let Some(cells) = cells.filter(|&c| c != items.len() as i64) {
                    let message = format!(
                        "this array has {}, but the list gives {}",
                        super::stmt::plural(cells, "cell"),
                        super::stmt::plural(items.len() as i64, "value"),
                    );
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return self.err_expr(span);
                }
                let index_ty = match dims[0] {
                    Dim::Bound(d) => Ty::Bound(d),
                    Dim::Pos(d) => Ty::Pos(d),
                    Dim::Enum(e) => Ty::Enum(e),
                    Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                };
                let values: Vec<Expr> = items.iter().map(|item| self.expect(item, elem)).collect();
                let ty = ty.clone();
                let elem = (**elem).clone();
                self.do_expr(Some(&ty), span, |lw, out| {
                    let count = lw.mk(ExprKind::Int(values.len() as i64), Ty::Int, span);
                    let wrong = lw.mk(
                        ExprKind::Cmp(CmpOp::Ne, Box::new(len), Box::new(count)),
                        Ty::Bool,
                        span,
                    );
                    let trap = Stmt::Trap(
                        "an array literal needs one value per position".to_string(),
                        lw.span(span),
                    );
                    out.push(Stmt::If(
                        wrong,
                        Block { stmts: vec![trap] },
                        Block::default(),
                    ));
                    let slot = lw.temp(ty.clone());
                    let blank = lw.mk(ExprKind::Fill(vec![dim], None), ty.clone(), span);
                    out.push(Stmt::Let(slot, Some(blank)));
                    for (i, value) in values.into_iter().enumerate() {
                        let at = lw.mk(ExprKind::Int(i as i64), index_ty.clone(), span);
                        let whole = Place {
                            kind: PlaceKind::Local(slot),
                            ty: ty.clone(),
                            span,
                        };
                        let cell = Place {
                            kind: PlaceKind::Index(Box::new(whole), vec![at]),
                            ty: elem.clone(),
                            span,
                        };
                        out.push(Stmt::Assign(cell, AssignOp::Set, value));
                    }
                    out.push(Stmt::Give(lw.local(slot, span)));
                })
            }
            Some(ty @ (Ty::Map(..) | Ty::HashMap(..))) if items.is_empty() => {
                self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)
            }
            _ => {
                let Some((first, rest)) = items.split_first() else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell what this empty list holds",
                    );
                    self.help("say the type where it is stored: `var xs: List<Int> = []`");
                    return self.err_expr(span);
                };
                let has_none = items
                    .iter()
                    .any(|it| matches!(it.kind, ast::ExprKind::None));
                if has_none {
                    let non_none = items
                        .iter()
                        .find(|it| !matches!(it.kind, ast::ExprKind::None));
                    if let Some(sample) = non_none {
                        let sample_e = self.expr(sample, None);
                        let elem = match &sample_e.ty {
                            Ty::Opt(_) => sample_e.ty.clone(),
                            other => Ty::opt(other.clone()),
                        };
                        let values = items.iter().map(|item| self.expect(item, &elem)).collect();
                        return self.mk(ExprKind::Tuple(values), Ty::list(elem), span);
                    }
                }
                let first = self.expr(first, None);
                let elem = first.ty.clone();
                let mut values = vec![first];
                for item in rest {
                    values.push(self.expect(item, &elem));
                }
                self.mk(ExprKind::Tuple(values), Ty::list(elem), span)
            }
        }
    }

    /// What a `for` walks over, and the type of each value.
    /// What `e` is where a sequence is wanted and `e` is something a
    /// `for` walks without being one: a range, a domain, `Bool`. It is
    /// the values the walk gives, `x for x in e`, so `sorted(0..n)`,
    /// `sum(lo..hi)` and `max(D)` need no rule of their own.
    pub(super) fn walked(&mut self, e: &ast::Expr) -> Option<ast::Expr> {
        let mut inner = e;
        while let ast::ExprKind::Paren(next) = &inner.kind {
            inner = next;
        }
        let walks = if self.is_sorted_range_expr(inner) {
            true
        } else {
            match &inner.kind {
                ast::ExprKind::Range { .. } => true,
                ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                    name == "Bool"
                        || matches!(self.gen_env.get(name), Some(GenArg::Domain(_)))
                        || matches!(self.get_sym(name), Some(Sym::Domain(_)))
                        || self.index_struct(name).is_some()
                }
                ast::ExprKind::Generic { .. } => self.index_struct_of(inner).is_some(),
                ast::ExprKind::Field { base, name } => {
                    let (field_base, field_name, rev) = if name.text == "rev" {
                        match &base.kind {
                            ast::ExprKind::Field { base: b, name: n } => {
                                (b.as_ref(), n.text.as_str(), true)
                            }
                            _ => (base.as_ref(), "", false),
                        }
                    } else {
                        (base.as_ref(), name.text.as_str(), false)
                    };
                    if matches!(field_name, "keys" | "values") {
                        match self.probe_ty_silent(field_base) {
                            Some(Ty::Map(..)) => true,
                            Some(Ty::HashMap(..)) if !rev => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        };
        let item = "item";
        walks.then(|| ast::Expr {
            span: e.span,
            kind: ast::ExprKind::Comprehension {
                body: Box::new(ast::Expr {
                    kind: ast::ExprKind::Name(item.to_string()),
                    span: e.span,
                }),
                clauses: vec![ast::Clause {
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Name(item.to_string()),
                        span: e.span,
                    },
                    iter: e.clone(),
                    conds: Vec::new(),
                }],
            },
        })
    }

    pub(super) fn iter(&mut self, e: &ast::Expr) -> (Iter, Ty) {
        let span = e.span;
        if let Some(alias) = self.subsets_alias(e) {
            return self.iter(&alias);
        }
        if let Some(res) = self.sorted_view_iter(e, span) {
            return res;
        }
        // Direct zero-overhead range iteration for `V.after(pos)` and `V.before(pos)`.
        if let ast::ExprKind::Call { callee, args } = &e.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if let Some(Sym::Domain(id)) = self.sym(base) {
                    if matches!(name.text.as_str(), "after" | "before") && args.len() == 1 {
                        let pos = self.expect(&args[0].value, &Ty::Pos(id));
                        let (lo, hi) = if name.text == "after" {
                            (
                                self.int_add(self.cast(Cast::Ord, pos, Ty::Int), 1),
                                self.mk(ExprKind::DomainLen(id), Ty::Int, span),
                            )
                        } else {
                            (self.int_lit(0, span), self.cast(Cast::Ord, pos, Ty::Int))
                        };
                        return (
                            Iter::Range {
                                lo,
                                hi,
                                step: None,
                                rev: false,
                                closed: false,
                            },
                            Ty::Pos(id),
                        );
                    }
                }
            }
        }
        // `for b in Bool` walks `false`, then `true`.
        if matches!(&e.kind, ast::ExprKind::Name(name) if name == "Bool" && self.lookup(name).is_none())
        {
            let both = [false, true].map(|b| ast::Expr {
                kind: ast::ExprKind::Bool(b),
                span,
            });
            return self.iter(&ast::Expr {
                kind: ast::ExprKind::List(both.to_vec()),
                span,
            });
        }
        // `for (k, v) in m.items` goes over the map itself.
        if let ast::ExprKind::Field { base, name } = &e.kind {
            if name.text == "items"
                && matches!(
                    self.probe_ty_silent(base),
                    Some(Ty::Map(..) | Ty::HashMap(..))
                )
            {
                return self.iter(base);
            }
        }
        let error = |lw: &mut Self| {
            let zero = lw.int_lit(0, span);
            (
                Iter::Range {
                    lo: zero.clone(),
                    hi: zero,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Ty::Error,
            )
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.iter(inner),
            ast::ExprKind::Name(n) if self.alias_factors(n).is_some() => {
                let factors = self.alias_factors(n).unwrap();
                self.product_walk(factors, span)
            }
            ast::ExprKind::Tuple(items) if !items.is_empty() => {
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = items
                    .iter()
                    .map(|it| self.expr_to_product_factor(it).map(|f| (None, f)))
                    .collect();
                if let Some(factors) = factors {
                    return self.product_walk(factors, span);
                }
                let value = self.expr(e, None);
                let message = format!("a `for` can't walk over a `{}`", self.show(&value.ty));
                self.error(codes::TYPE_MISMATCH, span, message);
                error(self)
            }
            ast::ExprKind::Record(fields) if !fields.is_empty() => {
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = fields
                    .iter()
                    .map(|(n, it)| {
                        self.expr_to_product_factor(it)
                            .map(|f| (Some(n.text.clone()), f))
                    })
                    .collect();
                if let Some(factors) = factors {
                    return self.product_walk(factors, span);
                }
                let value = self.expr(e, None);
                let message = format!("a `for` can't walk over a `{}`", self.show(&value.ty));
                self.error(codes::TYPE_MISMATCH, span, message);
                error(self)
            }
            // `for p in ?Col:` walks `none` and then every `Col`.
            ast::ExprKind::Generic { base, args }
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == "Opt")
                    && args.len() == 1 =>
            {
                let domain = match self.resolve_ty(&args[0]) {
                    Ty::Pos(domain) => domain,
                    Ty::Error => return error(self),
                    other => {
                        let message = format!(
                            "a `for` can walk the values of a domain's `?D`, and `{}` is not a domain",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        return error(self);
                    }
                };
                let elem = Ty::opt(Ty::Pos(domain));
                let none = self.mk(ExprKind::None, elem.clone(), span);
                let some_ty = elem.clone();
                let all = self.listed(domain, elem.clone(), Some(none), span, |lw, ord| {
                    let pos = lw.cast(Cast::Trust, ord, Ty::Pos(domain));
                    (None, lw.mk(ExprKind::Some(Box::new(pos)), some_ty, span))
                });
                (Iter::Seq(all, false), elem)
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let lo = self.expr(lo, None);
                let ty = lo.ty.clone();
                if !matches!(ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_) | Ty::Error) {
                    let message = format!(
                        "a range runs over numbers or positions, not `{}`",
                        self.show(&ty)
                    );
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return error(self);
                }
                let mut hi = self.expect(hi, &ty);
                let mut lo = lo;
                if ty != Ty::Int {
                    lo = self.cast(Cast::Ord, lo, Ty::Int);
                    hi = self.cast(Cast::Ord, hi, Ty::Int);
                }
                // Positions have a next one; the last `Int` does not.
                let closed = *inclusive && ty == Ty::Int;
                if *inclusive && !closed {
                    hi = self.int_add(hi, 1);
                }
                let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                (
                    Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: false,
                        closed,
                    },
                    ty,
                )
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: None,
                step,
                ..
            } => {
                let lo = self.expr(lo, None);
                match lo.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let cuts = Self::walks_cuts(&lo.ty);
                        let lo_ord = self.cast(Cast::Ord, lo, Ty::Int);
                        let mut hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        if cuts {
                            hi = self.int_add(hi, 1);
                        }
                        let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                        (
                            Iter::Range {
                                lo: lo_ord,
                                hi,
                                step,
                                rev: false,
                                closed: false,
                            },
                            if cuts { Ty::Bound(d) } else { Ty::Pos(d) },
                        )
                    }
                    // `for i in 3..:` counts up until the body leaves.
                    Ty::Int => {
                        let hi = self.int_lit(INF, span);
                        let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                        (
                            Iter::Range {
                                lo,
                                hi,
                                step,
                                rev: false,
                                closed: false,
                            },
                            Ty::Int,
                        )
                    }
                    Ty::Error => error(self),
                    _ => {
                        let message = format!(
                            "a `for` over `lo..` counts from an `Int` or a domain value, found `{}`",
                            self.show(&lo.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range {
                lo: None,
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let hi = self.expr(hi, None);
                match hi.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let cuts = Self::walks_cuts(&hi.ty);
                        let lo_ord = self.int_lit(0, span);
                        let mut hi_ord = self.cast(Cast::Ord, hi, Ty::Int);
                        if *inclusive {
                            hi_ord = self.int_add(hi_ord, 1);
                        }
                        let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                        (
                            Iter::Range {
                                lo: lo_ord,
                                hi: hi_ord,
                                step,
                                rev: false,
                                closed: false,
                            },
                            if cuts { Ty::Bound(d) } else { Ty::Pos(d) },
                        )
                    }
                    _ => {
                        self.unsupported(span, "a range without a start in a `for`");
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a range without both ends in a `for`");
                error(self)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none() && self.resolve_domain_name(name).is_some() =>
            {
                let id = self.resolve_domain_name(name).unwrap();
                let lo = self.int_lit(0, span);
                let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Pos(id),
                )
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && matches!(self.gen_env.get(name), Some(GenArg::Ty(Ty::Bound(_)))) =>
            {
                let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(name).cloned() else {
                    unreachable!()
                };
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Bound(id),
                )
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && matches!(self.get_sym(name), Some(Sym::Enum(eid)) if self.enums[eid].variants.iter().all(|(_, f)| f.is_empty())) =>
            {
                let Some(Sym::Enum(eid)) = self.get_sym(name) else {
                    unreachable!()
                };
                let lo = self.int_lit(0, span);
                let hi = self.int_lit(self.enums[eid].variants.len() as i64, span);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Enum(eid),
                )
            }
            ast::ExprKind::Name(_) | ast::ExprKind::Generic { .. }
                if self.index_struct_of(e).is_some() =>
            {
                let Some(sid) = self.index_struct_of(e) else {
                    unreachable!()
                };
                let dom = self.structs[sid].index_domain.unwrap();
                if let Some(&nth) = self.structs[sid].methods.get("nth") {
                    return (self.walk(dom, nth, span), Ty::Struct(sid));
                }
                error(self)
            }
            // `(lo..hi).rev by step` counts down from `hi - 1`: the values
            // of `lo'..hi by step` backward, where `lo'` is the last one.
            ast::ExprKind::By { walk, step } => {
                let (iter, ty) = self.iter(walk);
                let step = self.expect(step, &Ty::Int);
                let Iter::Range {
                    lo,
                    hi,
                    step: None,
                    rev,
                    closed,
                } = iter
                else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "`by` steps a range of numbers written out, as in `(lo..hi).rev by 2`"
                            .to_string(),
                    );
                    return (iter, ty);
                };
                // `for v in V by 2` steps along the positions of `V`.
                if !matches!(ty, Ty::Int | Ty::Pos(_)) {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        format!("`by` steps a range of Int, not of `{}`", self.show(&ty)),
                    );
                    return (
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev,
                            closed,
                        },
                        ty,
                    );
                }
                if !rev {
                    let iter = Iter::Range {
                        lo,
                        hi,
                        step: Some(step),
                        rev,
                        closed,
                    };
                    return (iter, ty);
                }
                let down = |lw: &mut Self, lo: Expr, hi: Expr, step: Expr| {
                    let bin = |lw: &mut Self, op, l: Expr, r: Expr| {
                        lw.mk(
                            ExprKind::Binary(op, Box::new(l), Box::new(r)),
                            Ty::Int,
                            span,
                        )
                    };
                    let one = lw.int_lit(1, span);
                    let top = if closed {
                        hi.clone()
                    } else {
                        bin(lw, BinOp::Sub, hi.clone(), one)
                    };
                    let room = bin(lw, BinOp::Sub, top.clone(), lo);
                    let steps = bin(lw, BinOp::Div, room, step.clone());
                    let back = bin(lw, BinOp::Mul, steps, step.clone());
                    let lo = bin(lw, BinOp::Sub, top, back);
                    Iter::Range {
                        lo,
                        hi,
                        step: Some(step),
                        rev: true,
                        closed,
                    }
                };
                if [&lo, &hi, &step].into_iter().all(super::stmt::settled) {
                    return (down(self, lo, hi, step), ty);
                }
                // Ends that do work are worked out once, ahead of the walk.
                let list = self.collected(Ty::Int, span, |lw, out, push| {
                    let (lo, hi, step) = (lw.hold(lo, out), lw.hold(hi, out), lw.hold(step, out));
                    let iter = down(lw, lo, hi, step);
                    let item = lw.temp(Ty::Int);
                    let lid = lw.fresh_loop();
                    let at = lw.local(item, span);
                    let body = Block {
                        stmts: vec![push(lw, at)],
                    };
                    out.push(Stmt::For(lid, item, iter, body));
                });
                (Iter::Seq(list, false), ty)
            }
            // `xs.rev()` walks the same way as `xs.rev`.
            ast::ExprKind::Call { callee, args }
                if args.is_empty()
                    && matches!(&callee.kind, ast::ExprKind::Field { name, .. } if name.text == "rev") =>
            {
                self.iter(callee)
            }
            ast::ExprKind::Field { base, name } if name.text == "rev" => {
                let (iter, ty) = self.iter(base);
                let iter = match iter {
                    Iter::Range {
                        lo,
                        hi,
                        step,
                        rev,
                        closed,
                    } => Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: !rev,
                        closed,
                    },
                    Iter::RangeVal(value, rev) => Iter::RangeVal(value, !rev),
                    Iter::Seq(value, rev) => Iter::Seq(value, !rev),
                    Iter::Walk {
                        k,
                        n,
                        len,
                        nth,
                        recv,
                        rev,
                    } => Iter::Walk {
                        k,
                        n,
                        len,
                        nth,
                        recv,
                        rev: !rev,
                    },
                    Iter::TreeRange {
                        container,
                        lo,
                        hi,
                        rev,
                    } => Iter::TreeRange {
                        container,
                        lo,
                        hi,
                        rev: !rev,
                    },
                };
                (iter, ty)
            }
            ast::ExprKind::Field { base, name }
                if matches!(name.text.as_str(), "bounds" | "cuts") =>
            {
                let domain = match &base.kind {
                    ast::ExprKind::Name(n) if self.lookup(n).is_none() => {
                        self.resolve_domain_name(n)
                    }
                    _ => None,
                };
                let Some(id) = domain else {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        "`.cuts` belongs to a domain: `V.cuts`",
                    );
                    return error(self);
                };
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Bound(id),
                )
            }
            _ => {
                if let ast::ExprKind::Field { base, name } = &e.kind {
                    if let ast::ExprKind::Generic {
                        base: gbase,
                        args: gargs,
                    } = &base.kind
                    {
                        if let Some(Sym::GenStruct(gid)) = self.sym(gbase) {
                            let resolved = self.resolve_gen_args_for_struct(gid, gargs);
                            let sid = self.instantiate_struct(gid, resolved);
                            if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                                let lo = self.int_lit(0, span);
                                let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                                return (
                                    Iter::Range {
                                        lo,
                                        hi,
                                        step: None,
                                        rev: false,
                                        closed: false,
                                    },
                                    Ty::Pos(dom_id),
                                );
                            }
                        }
                    }
                }
                let value = self.expr(e, None);
                if let ExprKind::DomainVal(dom_id) = value.kind {
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                    return (
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        Ty::Pos(dom_id),
                    );
                }
                if let Ty::Range(inner) = value.ty.clone() {
                    return (Iter::RangeVal(value, false), *inner);
                }
                // A struct with a zero-arg `each(self)` (such as `Bitset<D>`), or the
                // `iter(self)` of its `impl Seq`, walks what that call gives.
                if let Ty::Struct(sid) = value.ty {
                    let walk = ["each", "iter"].into_iter().find_map(|m| {
                        let f = *self.structs[sid].methods.get(m)?;
                        self.sigs[f].params.is_empty().then_some(f)
                    });
                    if let Some(each_fn) = walk {
                        self.queue_fn(each_fn);
                        let ret = self.ret_of(each_fn, span);
                        if let Ty::List(elem) = ret {
                            let call = self.mk(
                                ExprKind::Call(each_fn, vec![Arg::Value(value)]),
                                Ty::List(elem.clone()),
                                span,
                            );
                            return (Iter::Seq(call, false), *elem);
                        }
                    }
                    // A struct with `len` and `nth(self, j)` (an `Indexed` view such
                    // as `Stepped<S>`) walks `nth(j)` for `j` in `0..len`.
                    let len_fn = self.structs[sid].methods.get("len").copied();
                    let nth_fn = self.structs[sid].methods.get("nth").copied().or_else(|| {
                        // `nth<T>(self, j) -> T` over a sequence parameter: `T` is its element.
                        let gfid = *self.structs[sid].gen_methods.get("nth")?;
                        if self.gen_fns[gfid].decl.generics.len() != 1 {
                            return None;
                        }
                        let elem = self.structs[sid].env.values().find_map(|a| match a {
                            GenArg::Ty(t) => t.seq_elem(),
                            _ => None,
                        })?;
                        Some(self.instantiate_fn(
                            gfid,
                            vec![GenArg::Ty(elem)],
                            &[],
                            &[],
                            Vec::new(),
                        ))
                    });
                    if let (Some(len_fn), Some(nth_fn)) = (len_fn, nth_fn) {
                        if self.sigs[len_fn].params.is_empty()
                            && self.sigs[nth_fn].params.len() == 1
                        {
                            self.queue_fn(len_fn);
                            self.queue_fn(nth_fn);
                            let elem = self.ret_of(nth_fn, span);
                            let recv = self.temp(value.ty.clone());
                            let at = self.mk(ExprKind::Local(recv), value.ty.clone(), span);
                            self.cx.pending.push(Stmt::Let(recv, Some(value)));
                            let len = self.mk(
                                ExprKind::Call(len_fn, vec![Arg::Value(at.clone())]),
                                Ty::Int,
                                span,
                            );
                            let iter = Iter::Walk {
                                k: self.temp(Ty::Int),
                                n: self.temp(Ty::Int),
                                len,
                                nth: nth_fn,
                                recv: Some(at),
                                rev: false,
                            };
                            return (iter, elem);
                        }
                    }
                }
                let elem = match &value.ty {
                    Ty::Array { elem, .. }
                    | Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem, _)
                    | Ty::Set(elem)
                    | Ty::HashSet(elem) => (**elem).clone(),
                    Ty::Str | Ty::StrView => Ty::Char,
                    Ty::Map(key, val) | Ty::HashMap(key, val) => {
                        Ty::Tuple(vec![(**key).clone(), (**val).clone()])
                    }
                    Ty::Error => return error(self),
                    Ty::Int => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            span,
                            "a `for` can't walk over an `Int`",
                        );
                        self.help("count with a range: `for i in 0..n`");
                        return error(self);
                    }
                    other => {
                        let message = format!("a `for` can't walk over a `{}`", self.show(other));
                        self.error(codes::TYPE_MISMATCH, span, message);
                        return error(self);
                    }
                };
                (Iter::Seq(value, false), elem)
            }
        }
    }
}

fn strip_opt(ty: &Ty) -> Ty {
    match ty {
        Ty::Opt(inner) => (**inner).clone(),
        other => other.clone(),
    }
}

/// The number type of one side of a `none`-absorbing `+` or `-`: the type
/// itself, or what its optional holds.
fn tropical_side(ty: &Ty) -> Option<Ty> {
    let inner = match ty {
        Ty::Opt(inner) => &**inner,
        other => other,
    };
    matches!(inner, Ty::Int | Ty::Float).then(|| inner.clone())
}

/// Turns each value the arms give into a store to `place`.
/// A value block that can leave its fn or loop from inside: `x?`.
fn leaves(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Do(block) if block_leaves(block))
}

fn block_leaves(block: &Block) -> bool {
    let mut found = false;
    crate::walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::Return(_) | Stmt::Break(..) | Stmt::Continue(..));
    });
    found
}

impl Lowerer<'_> {
    /// `a + x?` cannot leave the fn from the middle of an expression in
    /// the C++, so the statements after `at` that hold such a value get
    /// it worked out ahead of them, operand by operand in the order
    /// written, and then use it.
    pub(super) fn lift_leaving(&mut self, out: &mut Vec<Stmt>, at: usize) {
        let mut done = Vec::with_capacity(out.len() - at);
        for mut stmt in out.drain(at..) {
            match &mut stmt {
                // The cell stored to is found first, then the value.
                Stmt::Assign(place, _, e) => {
                    let mut at = &mut *place;
                    let mut indexes = Vec::new();
                    loop {
                        at = match &mut at.kind {
                            PlaceKind::Index(base, index) => {
                                indexes.push(index);
                                base
                            }
                            PlaceKind::Field(base, _) | PlaceKind::Some(base) => base,
                            PlaceKind::Local(_) | PlaceKind::Global(_) => break,
                        };
                    }
                    for index in indexes.into_iter().rev().flatten() {
                        self.lift_value(index, &mut done);
                    }
                    self.lift_value(e, &mut done);
                }
                // A local that is such a block is what the block stores to.
                Stmt::Let(slot, Some(e)) => {
                    let unit = Expr {
                        kind: ExprKind::Unit,
                        ty: Ty::Unit,
                        span: e.span,
                    };
                    let lifted = self.lifted(std::mem::replace(e, unit));
                    match lifted {
                        Expr {
                            kind: ExprKind::Do(mut block),
                            ty,
                            span,
                        } if block_leaves(&block) => {
                            let place = Place {
                                kind: PlaceKind::Local(*slot),
                                ty,
                                span,
                            };
                            store_gives(&mut block.stmts, &place);
                            done.push(Stmt::Let(*slot, None));
                            done.extend(block.stmts);
                            continue;
                        }
                        lifted => *e = lifted,
                    }
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) => self.lift_value(e, &mut done),
                // A block run for what it does is its statements.
                Stmt::Expr(e) => {
                    self.lift_value(e, &mut done);
                    if matches!(e.kind, ExprKind::Unit) {
                        continue;
                    }
                }
                // A test and the ends of a range are read once, before
                // the statement.
                Stmt::If(cond, _, otherwise) => {
                    self.lift_value(cond, &mut done);
                    // An `elif` is an `if` alone in the `else`.
                    if matches!(otherwise.stmts.as_slice(), [Stmt::If(..)]) {
                        self.lift_leaving(&mut otherwise.stmts, 0);
                    }
                }
                Stmt::Assert { cond, .. } => self.lift_value(cond, &mut done),
                Stmt::For(_, _, Iter::Range { lo, hi, .. }, _) => {
                    self.lift_value(lo, &mut done);
                    self.lift_value(hi, &mut done);
                }
                Stmt::For(_, _, Iter::RangeVal(e, _) | Iter::Seq(e, _), _) => {
                    self.lift_value(e, &mut done);
                }
                _ => {}
            }
            done.push(stmt);
        }
        out.extend(done);
    }

    /// `value` with what leaves the fn worked out in `done` first.
    fn lift_value(&mut self, value: &mut Expr, done: &mut Vec<Stmt>) {
        let unit = Expr {
            kind: ExprKind::Unit,
            ty: Ty::Unit,
            span: value.span,
        };
        let lifted = self.lifted(std::mem::replace(value, unit));
        *value = if leaves(&lifted) {
            self.ahead(lifted, done)
        } else {
            lifted
        };
    }

    /// The block's statements go to `out`; what it gives is held and read.
    fn ahead(&mut self, e: Expr, out: &mut Vec<Stmt>) -> Expr {
        let Expr {
            kind: ExprKind::Do(mut block),
            ty,
            span,
        } = e
        else {
            return e;
        };
        if ty == Ty::Unit {
            run_gives(&mut block.stmts);
            out.extend(block.stmts);
            return Expr {
                kind: ExprKind::Unit,
                ty,
                span,
            };
        }
        let slot = self.temp(ty.clone());
        let place = Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span,
        };
        store_gives(&mut block.stmts, &place);
        out.push(Stmt::Let(slot, None));
        out.extend(block.stmts);
        Expr {
            kind: ExprKind::Local(slot),
            ty,
            span,
        }
    }

    /// `e` with every leaving block among its operands brought to the
    /// front: the result is then one such block, or `e` as it was.
    fn lifted(&mut self, e: Expr) -> Expr {
        let Expr { kind, ty, span } = e;
        let mut parts: Vec<Expr> = Vec::new();
        // The operands that are always worked out, in order; `shape` puts
        // them back.
        let shape: Box<dyn FnOnce(Vec<Expr>) -> ExprKind> = match kind {
            ExprKind::Binary(op, l, r)
                if !matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce) =>
            {
                parts.extend([*l, *r]);
                Box::new(move |p| {
                    let [l, r]: [Expr; 2] = p.try_into().expect("two operands");
                    ExprKind::Binary(op, Box::new(l), Box::new(r))
                })
            }
            ExprKind::Cmp(op, l, r) => {
                parts.extend([*l, *r]);
                Box::new(move |p| {
                    let [l, r]: [Expr; 2] = p.try_into().expect("two operands");
                    ExprKind::Cmp(op, Box::new(l), Box::new(r))
                })
            }
            ExprKind::Unary(op, x) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Unary(op, Box::new(p.remove(0))))
            }
            ExprKind::Cast(cast, x) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Cast(cast, Box::new(p.remove(0))))
            }
            ExprKind::Some(x) => {
                parts.push(*x);
                Box::new(|mut p| ExprKind::Some(Box::new(p.remove(0))))
            }
            ExprKind::Unwrap(x, checked) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Unwrap(Box::new(p.remove(0)), checked))
            }
            ExprKind::RangeCheck {
                value,
                lo,
                hi,
                text,
            } => {
                let has_lo = lo.is_some();
                let has_hi = hi.is_some();
                parts.push(*value);
                if let Some(l) = lo {
                    parts.push(*l);
                }
                if let Some(h) = hi {
                    parts.push(*h);
                }
                Box::new(move |mut p| {
                    let value = Box::new(p.remove(0));
                    let lo = has_lo.then(|| Box::new(p.remove(0)));
                    let hi = has_hi.then(|| Box::new(p.remove(0)));
                    ExprKind::RangeCheck {
                        value,
                        lo,
                        hi,
                        text,
                    }
                })
            }
            ExprKind::Field(x, at) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Field(Box::new(p.remove(0)), at))
            }
            ExprKind::Tuple(items) => {
                parts = items;
                Box::new(ExprKind::Tuple)
            }
            ExprKind::Index(base, at) => {
                parts.push(*base);
                parts.extend(at);
                Box::new(|mut p| {
                    let base = p.remove(0);
                    ExprKind::Index(Box::new(base), p)
                })
            }
            ExprKind::Call(id, args) => {
                let mut places = Vec::new();
                for arg in args {
                    match arg {
                        Arg::Value(e) => {
                            parts.push(e);
                            places.push(None);
                        }
                        Arg::Mut(place) => places.push(Some(place)),
                    }
                }
                Box::new(move |p| {
                    let mut values = p.into_iter();
                    let args = places
                        .into_iter()
                        .map(|place| match place {
                            Some(place) => Arg::Mut(place),
                            None => Arg::Value(values.next().expect("one value per slot")),
                        })
                        .collect();
                    ExprKind::Call(id, args)
                })
            }
            // A block's own statements may hold values that leave.
            ExprKind::Do(mut block) => {
                self.lift_leaving(&mut block.stmts, 0);
                return Expr {
                    kind: ExprKind::Do(block),
                    ty,
                    span,
                };
            }
            ExprKind::Mutate(how, place, args) => {
                parts = args;
                Box::new(move |p| ExprKind::Mutate(how, place, p))
            }
            ExprKind::Print(args, end) => {
                let digits: Vec<_> = args.iter().map(|arg| arg.digits).collect();
                parts = args.into_iter().map(|arg| arg.value).collect();
                Box::new(move |p| {
                    let args = p
                        .into_iter()
                        .zip(digits)
                        .map(|(value, digits)| FmtArg { value, digits })
                        .collect();
                    ExprKind::Print(args, end)
                })
            }
            ExprKind::Builtin(builtin, args) => {
                parts = args;
                Box::new(move |p| ExprKind::Builtin(builtin, p))
            }
            kind => return Expr { kind, ty, span },
        };
        let mut parts: Vec<Expr> = parts.into_iter().map(|part| self.lifted(part)).collect();
        let Some(last) = parts.iter().rposition(leaves) else {
            return Expr {
                kind: shape(parts),
                ty,
                span,
            };
        };
        let mut stmts = Vec::new();
        for part in &mut parts[..=last] {
            if steady(part) {
                continue;
            }
            let taken = std::mem::replace(
                part,
                Expr {
                    kind: ExprKind::Unit,
                    ty: Ty::Unit,
                    span,
                },
            );
            *part = match leaves(&taken) {
                true => self.ahead(taken, &mut stmts),
                false => {
                    let slot = self.temp(taken.ty.clone());
                    let held = Expr {
                        kind: ExprKind::Local(slot),
                        ty: taken.ty.clone(),
                        span: taken.span,
                    };
                    stmts.push(Stmt::Let(slot, Some(taken)));
                    held
                }
            };
        }
        stmts.push(Stmt::Give(Expr {
            kind: shape(parts),
            ty: ty.clone(),
            span,
        }));
        Expr {
            kind: ExprKind::Do(Block { stmts }),
            ty,
            span,
        }
    }
}

/// A name or a field of one: reading it runs nothing, so it is read
/// where it stands and not copied ahead.
fn steady(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Field(of, _) => steady(of),
        _ => crate::walk::is_simple(e),
    }
}

fn store_gives(stmts: &mut [Stmt], place: &Place) {
    for stmt in stmts {
        match stmt {
            Stmt::Give(value) => {
                let value = std::mem::replace(
                    value,
                    Expr {
                        kind: ExprKind::Unit,
                        ty: Ty::Unit,
                        span: place.span,
                    },
                );
                *stmt = Stmt::Assign(place.clone(), AssignOp::Set, value);
            }
            Stmt::If(_, then, otherwise) => {
                store_gives(&mut then.stmts, place);
                store_gives(&mut otherwise.stmts, place);
            }
            _ => {}
        }
    }
}

/// A block with nothing to give has its last value run where it gave it.
fn run_gives(stmts: &mut [Stmt]) {
    for stmt in stmts {
        match stmt {
            Stmt::Give(value) => {
                let unit = Expr {
                    kind: ExprKind::Unit,
                    ty: Ty::Unit,
                    span: value.span,
                };
                *stmt = Stmt::Expr(std::mem::replace(value, unit));
            }
            Stmt::If(_, then, otherwise) => {
                run_gives(&mut then.stmts);
                run_gives(&mut otherwise.stmts);
            }
            _ => {}
        }
    }
}

fn seq_concat_elem(ty: &Ty) -> Option<&Ty> {
    match ty {
        Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => Some(e),
        Ty::Array { dims, elem } if dims.len() == 1 => Some(elem),
        _ => None,
    }
}

/// Whether a `?` is written somewhere in `e`.
fn has_try(e: &ast::Expr) -> bool {
    struct Find(bool);
    impl cpl_syntax::visit::Visitor for Find {
        fn expr(&mut self, e: &ast::Expr) {
            self.0 |= matches!(e.kind, ast::ExprKind::Try(_));
        }
    }
    let mut find = Find(false);
    cpl_syntax::visit::walk_expr(e, &mut find);
    find.0
}
