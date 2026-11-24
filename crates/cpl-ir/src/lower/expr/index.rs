//! Indexing: `base[...]` on arrays, grids and lists, with vouched indices.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// The indices of `base[...]` and the type of the element.
    /// Inside `unchecked:`, the `Int` that indexes a list is marked as
    /// vouched for.
    pub(in crate::lower) fn vouched(&mut self, base: &Ty, indices: Vec<Expr>) -> Vec<Expr> {
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
    pub(in crate::lower) fn indexes_whole_grid(&mut self, index: &ast::Expr) -> bool {
        match self.probe_expr(index, None).ty {
            Ty::Tuple(_) | Ty::Record(_) => true,
            Ty::Pos(p) => self.product_grid(p).is_some(),
            _ => false,
        }
    }

    pub(in crate::lower) fn index_args(
        &mut self,
        base: &Ty,
        indices: &[ast::Expr],
        span: Span,
    ) -> (Vec<Expr>, Ty) {
        let one = |lw: &mut Self, want: &Ty| -> Vec<Expr> {
            if indices.len() != 1 {
                let message = cpl_diag::msgs::ir::TAKES_1_INDEX_BUT.at(&[&indices.len()]);
                lw.report_said(span, message);
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
                                self.report(
                                    span,
                                    msgs::ir::WRONG_ARG_COUNT_A_SYMMETRIC_TABLE_TAKES,
                                    &[],
                                );
                                return (vec![self.err_expr(span)], (**elem).clone());
                            }
                        };
                        let f0 = self.mk_field(pair.clone(), 0, ft0, indices[0].span);
                        let f1 = self.mk_field(pair, 1, ft1, indices[0].span);
                        (
                            self.coerce(f0, &Ty::Pos(*d), indices[0].span),
                            self.coerce(f1, &Ty::Pos(*d), indices[0].span),
                        )
                    } else {
                        self.report(
                            span,
                            msgs::ir::WRONG_ARG_COUNT_THIS_TAKES_2_INDICES,
                            &[&indices.len()],
                        );
                        return (vec![self.err_expr(span)], (**elem).clone());
                    };
                    let a = self.cast(Cast::Ord, u, Ty::Int);
                    let b = self.cast(Cast::Ord, v, Ty::Int);
                    let sym = self.mk_builtin(Builtin::SymIdx, [a, b], Ty::Int, span);
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
                    self.report(
                        span,
                        msgs::ir::WRONG_ARG_COUNT_A_TAKES_INDICES_BUT,
                        &[&self.show(base), &dims.len(), &indices.len()],
                    );
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
                        let cell = self.mk_if(value, one, zero, Ty::Int, index.span);
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
                                stmts.push_let(tmp, value);
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
                                    acc = lw.mk_if(is_t, idx_lit, acc, Ty::Int, index.span);
                                }
                                stmts.push_give(acc);
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
                        self.report(
                            index.span,
                            msgs::ir::WRONG_DOMAIN_THIS_IS_INDEXED_BY,
                            &[&self.show(&want), &self.show(&value.ty)],
                        );
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
                self.report(
                    span,
                    msgs::ir::NOT_INDEXABLE_A_CAN_T_BE,
                    &[&self.show(other)],
                );
                self.tuple_index_fix(other, &indices[0], span);
                (vec![self.err_expr(span)], Ty::Error)
            }
        }
    }
}
