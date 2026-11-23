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
        let view_ty = Ty::MutView(Box::new(elem_ty));
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
}
