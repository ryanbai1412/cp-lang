//! Places inside a mutable slice: `a[l..r] = ..`, `a[l..][k] += 1`.

use super::*;

impl<'a> Lowerer<'a> {
    /// `a[range]` as a mutable view over `base_place`, held in a temp whose
    /// first position is a local, or `base_place` back when `range` is not
    /// a range.
    pub(super) fn mut_slice_place(
        &mut self,
        base_place: Place,
        range: &ast::Expr,
        span: Span,
    ) -> Result<Place, Place> {
        let bounds = if matches!(range.kind, ast::ExprKind::Range { .. }) {
            self.mut_slice_bounds(&base_place, range, span)
        } else if matches!(self.probe_ty_silent(range), Some(Ty::Range(_))) {
            let range_val = self.expr(range, None);
            self.mut_slice_range_val_bounds(&base_place, range_val, span)
        } else {
            return Err(base_place);
        };
        let Some((lo, hi, elem_ty)) = bounds else {
            return Err(base_place);
        };
        let lo_slot = self.temp(Ty::Int);
        self.cx.pending.push(Stmt::Let(lo_slot, Some(lo)));
        let lo = self.local(lo_slot, span);
        let view_ty = Self::mut_view_ty(&base_place.ty, elem_ty);
        let view = self.mk(
            ExprKind::MutSlice(Box::new(base_place), Box::new(lo), Box::new(hi)),
            view_ty.clone(),
            span,
        );
        let tmp = self.temp(view_ty.clone());
        self.cx.pending.push(Stmt::Let(tmp, Some(view)));
        Ok(Place {
            kind: PlaceKind::Local(tmp),
            ty: view_ty,
            span: self.span(span),
        })
    }

    /// `a[l..][k]` is the cell of `a` at key `k`, since a slice keeps its
    /// source's keys (R04b.1): the view's position is `k`'s position in `a`
    /// less the slice's first. `None` unless `view` was just made by
    /// `mut_slice_place` over a list or array.
    pub(super) fn slice_key_place(
        &mut self,
        view: &Place,
        key: &ast::Expr,
        span: Span,
    ) -> Option<Place> {
        let PlaceKind::Local(slot) = view.kind else {
            return None;
        };
        let Some(Stmt::Let(made, Some(slice))) = self.cx.pending.last() else {
            return None;
        };
        let ExprKind::MutSlice(source, lo, _) = &slice.kind else {
            return None;
        };
        if *made != slot || !matches!(lo.kind, ExprKind::Local(_)) {
            return None;
        }
        let first = match &source.ty {
            Ty::List(_) => 0,
            Ty::Array { dims, .. } => match dims[..] {
                [Dim::IntFrom(first)] => first,
                [Dim::Int | Dim::Pos(_) | Dim::Bound(_)] => 0,
                _ => return None,
            },
            _ => return None,
        };
        let lo = (**lo).clone();
        let key = self.expr(key, None);
        let at = match key.ty {
            Ty::Int => self.int_add(key, -first),
            Ty::Pos(_) | Ty::Bound(_) => self.cast(Cast::Ord, key, Ty::Int),
            _ => self.coerce(key, &Ty::Int, span),
        };
        let pos = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(at), Box::new(lo)),
            Ty::Int,
            span,
        );
        let elem = Self::sliceable_mut_elem(&view.ty)?;
        Some(Place {
            kind: PlaceKind::Index(Box::new(view.clone()), vec![pos]),
            ty: elem,
            span: self.span(span),
        })
    }

    /// A mutable slice of `source` keeps its keys: a slice of a `[V]T` is
    /// keyed by `V` (R07.5.11).
    pub(in crate::lower) fn mut_view_ty(source: &Ty, elem: Ty) -> Ty {
        let key = match source {
            Ty::Array { dims, .. } => match dims.as_slice() {
                [Dim::Pos(d)] => Dim::Pos(*d),
                _ => Dim::Int,
            },
            Ty::MutView(_, key) => *key,
            _ => Dim::Int,
        };
        Ty::MutView(Box::new(elem), key)
    }

    /// The first key of `view` when it is a keyed mutable view.
    pub(in crate::lower) fn mut_view_key_lo(&mut self, view: &Place, span: Span) -> Option<Expr> {
        let keyed = matches!(view.ty, Ty::MutView(..)) && Self::keyed_view(&view.ty);
        keyed.then(|| self.key_lo(&place_to_expr(view), span))
    }

    /// The position of ordinal `ord` in a view whose first key is `key_lo`.
    pub(in crate::lower) fn at_view_key(
        &mut self,
        ord: Expr,
        key_lo: &Option<Expr>,
        span: Span,
    ) -> Expr {
        match key_lo {
            Some(lo) => self.mk(
                ExprKind::Binary(BinOp::Sub, Box::new(ord), Box::new(lo.clone())),
                Ty::Int,
                span,
            ),
            None => ord,
        }
    }

    /// Where a slice bound read from a range value falls in the sliced place.
    pub(in crate::lower) fn slice_bound_at(
        &mut self,
        bound: Expr,
        first: i64,
        key_lo: &Option<Expr>,
        span: Span,
    ) -> Expr {
        if !matches!(bound.ty, Ty::Pos(_) | Ty::Bound(_)) {
            return self.int_add(bound, -first);
        }
        let ord = self.cast(Cast::Ord, bound, Ty::Int);
        self.at_view_key(ord, key_lo, span)
    }

    /// The index arguments of a place `base[indices]`: a keyed mutable view
    /// is written at its source's keys.
    pub(in crate::lower) fn place_index_args(
        &mut self,
        base: &Place,
        indices: &[ast::Expr],
        span: Span,
    ) -> (Vec<Expr>, Ty) {
        let (mut indices, ty) = self.index_args(&base.ty, indices, span);
        if let (Some(_), [_]) = (self.mut_view_key_lo(base, span), indices.as_slice()) {
            let key = indices.remove(0);
            let view = place_to_expr(base);
            indices.push(self.mk(
                ExprKind::Builtin(Builtin::KeyPos, vec![view, key]),
                Ty::Int,
                span,
            ));
        }
        (indices, ty)
    }
}
