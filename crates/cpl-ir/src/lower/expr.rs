//! Expressions.

use super::stmt::place_to_expr;
use super::*;
use blocks::block_leaves;
use cpl_diag::msgs;

mod blocks;
mod compare;
mod expected_variant;
mod for_iter;
mod index;
mod literal;
mod name_value;
mod operator;
mod range_val;
mod slice;
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
    pub(super) fn expr_as(&mut self, e: &ast::Expr, want: &Ty) -> Expr {
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
                    if let Some(Ty::Pos(d)) = inner_hint
                        && !self.product_domains.contains_key(d)
                        && !self.sum_enums.contains_key(d) =>
                {
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
                    self.report(span, msgs::ir::BAD_NUMBER_THIS_NUMBER_DOES_NOT, &[]);
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Float(text) => {
                let clean: String = text.chars().filter(|c| *c != '_').collect();
                match clean.parse::<f64>() {
                    Ok(value) => self.mk(ExprKind::Float(value), Ty::Float, span),
                    Err(_) => {
                        self.report(span, msgs::ir::BAD_NUMBER_THIS_IS_NOT_A, &[]);
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
                    self.report(span, msgs::ir::TYPE_MISMATCH_CAN_T_TELL_WHICH, &[]);
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
                self.report(span, msgs::ir::UNRESOLVED_NAME_SELF_OUTSIDE_A_METHOD, &[]);
                self.err_expr(span)
            }
            ast::ExprKind::Cpp(_) | ast::ExprKind::Intrinsic(_) => self.err_expr(span),
            ast::ExprKind::Paren(inner) if self.is_filled(inner) => {
                let held = self.expr(inner, None);
                match held.ty.clone() {
                    Ty::Opt(ty) => self.mk_unwrap(held, false, *ty, span),
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
                        self.mk_unwrap(value, true, *inner, span)
                    }
                    _ => value,
                }
            }
            ast::ExprKind::Compare { first, rest } => self.settled_compare(first, rest, span),
            ast::ExprKind::Range {
                lo,
                hi,
                inclusive,
                step: None,
            } => match (lo, hi, hint) {
                // `..` where a `Range<D>` is wanted is all of `D`.
                (None, None, Some(Ty::Range(of))) if let Ty::Pos(d) = **of => {
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Pos(d)),
                        span,
                    )
                }
                _ => self.range_val(lo.as_deref(), hi.as_deref(), *inclusive, hint, span),
            },
            ast::ExprKind::Range { .. } => self.stepped_range_val(e, hint),
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
                if let Some(n) = nested_arity(&base_expr.ty, indices.len()) {
                    let inner =
                        ast::Expr::indexed(base, &indices[..n], base.span.to(indices[n - 1].span));
                    return self.expr(&ast::Expr::indexed(&inner, &indices[n..], e.span), hint);
                }
                if let Ty::Struct(sid) = base_expr.ty {
                    if let Some(key_ty) = self.counter_key.get(&sid).cloned() {
                        if indices.len() != 1 {
                            self.report(
                                span,
                                msgs::ir::WRONG_ARG_COUNT_A_COUNTER_TAKES_1,
                                &[&indices.len()],
                            );
                            return self.err_expr(span);
                        }
                        let map_ty = self.structs[sid].def.fields[0].1.clone();
                        let map_expr = self.mk_field(base_expr, 0, map_ty, span);
                        let key = self.expr_as(&indices[0], &key_ty);
                        let found = self.mk_builtin(
                            Builtin::MapGet,
                            [map_expr, key],
                            Ty::opt(Ty::Int),
                            span,
                        );
                        let zero = self.int_lit(0, span);
                        return self.mk_binary(BinOp::Coalesce, found, zero, Ty::Int, span);
                    }
                    let ast_args: Vec<ast::Arg> = indices
                        .iter()
                        .map(|idx| ast::Arg {
                            name: None,
                            mutable: false,
                            value: idx.clone(),
                        })
                        .collect();
                    if let Some(fn_id) = self.own_method(sid, "get") {
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
                        if let Some(line) = self.grid_line(base, &base_expr, indices, &whole, span)
                        {
                            return line;
                        }
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
                                    let len = lw.mk_builtin(
                                        Builtin::AxisLen,
                                        [base.clone(), lw.int_lit(axis as i64, span)],
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
                            let value = lw.mk_index(base, cell, ty, span);
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
                    let Ok([key]) = <[Expr; 1]>::try_from(indices) else {
                        return self.err_expr(span);
                    };
                    let out = self.key_index(view, key, ty, span);
                    let plain = self.plain_ty(&out.ty);
                    return self.try_coerce(out, &plain).unwrap_or_else(|e| e);
                }
                let (indices, ty) = self.index_args(&base_expr.ty, indices, span);
                let indices = self.vouched(&base_expr.ty, indices);
                let out = self.mk_index(base_expr, indices, ty, span);
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
                        let diff = self.mk_binary(BinOp::Sub, hi_e, lo_e, Ty::Int, span);
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
            ast::ExprKind::HasBound { param, bound } => {
                let has = self.has_bound_test(&param.text, bound);
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
                        // A base that is a plain read is read once per kept field, with no
                        // temporary; anything else is bound once first.
                        let plain = is_plain_read(&b) && kept_vals.iter().all(is_plain_read);
                        let mut stmts = Vec::new();
                        let base = if plain {
                            b
                        } else {
                            let slot = self.temp(b.ty.clone());
                            stmts.push(Stmt::Let(slot, Some(b)));
                            self.local(slot, span)
                        };
                        for (idx, (fname, fty)) in fs.into_iter().enumerate() {
                            if removed.iter().any(|r| r.text == fname) {
                                continue;
                            }
                            kept_vals.push(self.mk_field(base.clone(), idx, fty.clone(), span));
                            kept_tys.push((fname, fty));
                        }
                        let out_ty = Ty::Record(kept_tys);
                        let res = self.mk(ExprKind::Tuple(kept_vals), out_ty.clone(), span);
                        if stmts.is_empty() {
                            res
                        } else {
                            stmts.push_give(res);
                            self.mk(ExprKind::Do(Block { stmts }), out_ty, span)
                        }
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
                        out.push_give(lw.err_expr(span));
                        return;
                    }
                    other => {
                        let msg =
                            cpl_diag::msgs::ir::WITH_UPDATES_FIELDS_STRUCT.at(&[&lw.show(other)]);
                        lw.report_said(span, msg);
                        out.push_give(lw.err_expr(span));
                        return;
                    }
                };
                for (name, _) in updates {
                    if !fields.iter().any(|(f, _)| f == &name.text) {
                        lw.report(
                            name.span,
                            cpl_diag::msgs::ir::UNKNOWN_FIELD_HAS_NO_FIELD,
                            &[&lw.show(&ty), &name.text],
                        );
                    }
                }
                let needs_tmp = fields
                    .iter()
                    .any(|(fname, _)| !updates.iter().any(|(n, _)| &n.text == fname));
                let tmp = if needs_tmp {
                    let tmp = lw.temp(ty.clone());
                    out.push_let(tmp, base_val);
                    Some(tmp)
                } else {
                    if !matches!(base_val.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        out.push(Stmt::Expr(base_val));
                    }
                    None
                };
                let mut values = Vec::with_capacity(fields.len());
                for (idx, (fname, fty)) in fields.iter().enumerate() {
                    let update = updates.iter().rev().find(|(n, _)| &n.text == fname);
                    // With no `tmp`, every field has an update.
                    match (update, tmp) {
                        (Some((_, val_expr)), _) => values.push(lw.expr_as(val_expr, fty)),
                        (None, Some(tmp)) => {
                            let base_ref = lw.local(tmp, span);
                            values.push(lw.mk_field(base_ref, idx, fty.clone(), span));
                        }
                        (None, None) => {}
                    }
                }
                let built = lw.mk(ExprKind::Tuple(values), ty, span);
                out.push_give(built);
            }),
            ast::ExprKind::Mut(inner) => self.expr(inner, inner_hint),
            ast::ExprKind::Try(inner) => self.try_expr(inner, span),
            ast::ExprKind::Unwrap(inner) => {
                self.cx.as_written = narrow::asks_optional(inner);
                let value = self.expr_maybe_opt_reduce(inner);
                self.cx.as_written = false;
                match value.ty.clone() {
                    Ty::Opt(inside) => self.mk_unwrap(value, true, *inside, span),
                    Ty::Error => value,
                    ref ty if let Some(cells) = Self::cells_of(ty) => {
                        self.unwrap_cells(value, cells, span)
                    }
                    // Inside a generic function (e.g. `bsearch_first<I>` instantiated with `I = Int`),
                    // `(base + mid)!` is a no-op when `base + mid` is already non-optional `Int`.
                    _ if !self.gen_env.is_empty() => value,
                    other => {
                        self.report(
                            span,
                            msgs::ir::TYPE_MISMATCH_TAKES_THE_VALUE_OUT,
                            &[&self.show(&other)],
                        );
                        self.drop_fix("drop `!`", Span::new(inner.span.end, span.end));
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Missing | ast::ExprKind::Error => self.err_expr(span),
        }
    }
}

fn strip_opt(ty: &Ty) -> Ty {
    match ty {
        Ty::Opt(inner) => (**inner).clone(),
        other => other.clone(),
    }
}

/// Turns each value the arms give into a store to `place`.
/// A value block that can leave its fn or loop from inside: `x?`.
fn leaves(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Do(block) if block_leaves(block))
}

/// How many of `n` indices a table whose cells are arrays takes before the
/// cell's own: `dp[r, c, k]` on `[R, C][K]T` takes 2 (R01.8.5).
pub(in crate::lower) fn nested_arity(ty: &Ty, n: usize) -> Option<usize> {
    let Ty::Array { dims, elem } = ty else {
        return None;
    };
    let Ty::Array { dims: inner, .. } = &**elem else {
        return None;
    };
    let sym = matches!(dims.as_slice(), [Dim::Sym(_)]);
    (!sym && dims.len() < n && n <= dims.len() + inner.len()).then_some(dims.len())
}

/// A read of a local or a field of one: evaluating it again gives the same value and does nothing.
fn is_plain_read(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Int(_) | ExprKind::Bool(_) => true,
        ExprKind::Field(base, _) => is_plain_read(base),
        _ => false,
    }
}
