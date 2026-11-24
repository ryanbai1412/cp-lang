//! Slices and views: `base[lo..hi]`, slices by a runtime `Range<T>`, `mut`
//! slices, keyed views and cut ranges.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// True when `e` refers to a mutable place whose backing storage can be
    /// mutably sliced in O(1) without copying.
    pub(in crate::lower) fn is_mutable_place(&self, e: &ast::Expr) -> bool {
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

    pub(in crate::lower) fn sliceable_mut_elem(ty: &Ty) -> Option<Ty> {
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
    pub(in crate::lower) fn mut_slice_bounds(
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
                self.report(bound.span, msgs::ir::BAD_OPERANDS_A_SLICE_BOUND_COUNTS, &[]);
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
                _ => self.mk_builtin(Builtin::Len, [place_to_expr(base_place)], Ty::Int, span),
            },
            _ => self.mk_builtin(Builtin::Len, [place_to_expr(base_place)], Ty::Int, span),
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
    pub(in crate::lower) fn mut_slice_range_val_bounds(
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
        self.cx.pending.push_let(r_slot, range_val);
        let r_ref = self.local(r_slot, span);
        let lo_bound =
            self.mk_builtin(Builtin::RangeLo, [r_ref.clone()], range_inner.clone(), span);
        let hi_bound = self.mk_builtin(Builtin::RangeHi, [r_ref], range_inner.clone(), span);
        let key_lo = self.mut_view_key_lo(base_place, span);
        let lo = self.slice_bound_at(lo_bound, first, &key_lo, span);
        let hi = self.slice_bound_at(hi_bound, first, &key_lo, span);
        Some((lo, hi, elem_ty))
    }

    pub(in crate::lower) fn mut_slice_expr(
        &mut self,
        base: &ast::Expr,
        range: &ast::Expr,
        span: Span,
    ) -> Expr {
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

    pub(in crate::lower) fn mut_slice_by_range_val(
        &mut self,
        base: &ast::Expr,
        range_val: Expr,
        span: Span,
    ) -> Expr {
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
    pub(in crate::lower) fn keyed_view(ty: &Ty) -> bool {
        matches!(
            ty,
            Ty::StrView | Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(_, Dim::Pos(_))
        )
    }

    /// `view` in a form that can be read more than once.
    pub(in crate::lower) fn hold_view(&mut self, view: Expr, span: Span) -> Expr {
        if matches!(view.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
            return view;
        }
        let slot = self.temp(view.ty.clone());
        self.cx.pending.push_let(slot, view);
        self.local(slot, span)
    }

    /// `view[key]`, reading the cell that holds the source key `key`.
    pub(in crate::lower) fn key_index(
        &mut self,
        view: Expr,
        key: Expr,
        elem: Ty,
        span: Span,
    ) -> Expr {
        let pos = self.mk_builtin(Builtin::KeyPos, [view.clone(), key], Ty::Int, span);
        self.mk_index(view, vec![pos], elem, span)
    }

    /// The smallest key of a view, `0` for a whole list or string.
    pub(in crate::lower) fn key_lo(&mut self, seq: &Expr, span: Span) -> Expr {
        if Self::keyed_view(&seq.ty) {
            self.mk_builtin(Builtin::KeyLo, [seq.clone()], Ty::Int, span)
        } else {
            self.int_lit(0, span)
        }
    }

    /// Whether a `for` over an open-ended range ending in `end` walks cuts:
    /// a range's elements are the kind of its ends in v0.3 (R04.4.10).
    pub(in crate::lower) fn walks_cuts(end: &Ty) -> bool {
        matches!(end, Ty::Bound(_))
    }

    /// Whether an open-ended range with a cut end of `d` walks cuts: always,
    /// unless a piece (`Range<V>`) is expected (R04.4.10).
    pub(in crate::lower) fn cut_range_wanted(expected: Option<&Ty>, d: DomainId) -> bool {
        let wants = |want: fn(&Ty, DomainId) -> bool| matches!(expected, Some(Ty::Range(inner)) if want(inner, d));
        !wants(|t, d| matches!(t, Ty::Pos(ed) if *ed == d))
    }

    pub(in crate::lower) fn hoist_view_base(&mut self, base: Expr, span: Span) -> Expr {
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
        self.cx.pending.push_let(slot, base);
        self.local(slot, span)
    }

    /// Lowers a slice by a runtime `Range<T>` value (`base[r]`) into a view or `List<T>`.
    pub(in crate::lower) fn slice_by_range_val(
        &mut self,
        base: Expr,
        range_val: Expr,
        span: Span,
    ) -> Expr {
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
            self.cx.pending.push_let(r_slot, range_val);
            let r_ref = self.local(r_slot, span);
            let lo_bound =
                self.mk_builtin(Builtin::RangeLo, [r_ref.clone()], range_inner.clone(), span);
            let hi_bound = self.mk_builtin(Builtin::RangeHi, [r_ref], range_inner.clone(), span);
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
            out.push_let(b_slot, base);
            let r_slot = lw.temp(range_ty.clone());
            out.push_let(r_slot, range_val);
            let acc_slot = lw.temp(result_ty.clone());
            let init = if is_str {
                lw.mk(ExprKind::Str(Vec::new()), Ty::Str, span)
            } else {
                lw.mk(ExprKind::Tuple(Vec::new()), result_ty.clone(), span)
            };
            out.push_let(acc_slot, init);
            let r_ref = lw.local(r_slot, span);
            let lo_bound =
                lw.mk_builtin(Builtin::RangeLo, [r_ref.clone()], range_inner.clone(), span);
            let hi_bound = lw.mk_builtin(Builtin::RangeHi, [r_ref], range_inner.clone(), span);
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
            let elem = lw.mk_index(
                lw.local(b_slot, span),
                vec![idx_expr],
                elem_ty.clone(),
                span,
            );
            // A `Str` grows by `push` like a list: `acc = acc + c` would copy it each step.
            let push_stmt = Stmt::Expr(lw.mk_mutate(
                Mutation::Push,
                Place {
                    kind: PlaceKind::Local(acc_slot),
                    ty: result_ty.clone(),
                    span: lw.span(span),
                },
                vec![elem],
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
            out.push_give(res);
        })
    }

    /// Lowers a slice expression `base[lo..hi]` / `base[lo..=hi]` / `base[..hi]` / `base[lo..]`
    /// on a `List<T>`, `Str`, or `[D]T` into a `List<T>` (or `Str` for `Str`).
    /// Where `g[r, lo..hi]` starts and stops along its axis of `len`
    /// cells, counted from 0. `..` is all of it. A part is
    /// `0 <= lo <= hi <= len`, as for a slice; anything else stops the
    /// program.
    pub(in crate::lower) fn axis_part(
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
            crate::invariant::broken_invariant("an axis is taken whole or in part by a range")
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
        out.push_let(lo_slot, lo_e);
        out.push_let(hi_slot, hi_e);
        let (lo_e, hi_e) = (self.local(lo_slot, span), self.local(hi_slot, span));
        let cmp = |lw: &mut Self, op: CmpOp, l: &Expr, r: &Expr| {
            lw.mk_cmp(op, l.clone(), r.clone(), Ty::Bool, span)
        };
        let zero = self.int_lit(0, span);
        let below = cmp(self, CmpOp::Lt, &lo_e, &zero);
        let reversed = cmp(self, CmpOp::Lt, &hi_e, &lo_e);
        let beyond = cmp(self, CmpOp::Gt, &hi_e, &len);
        let bad = self.mk_binary(BinOp::Or, below, reversed, Ty::Bool, span);
        let bad = self.mk_binary(BinOp::Or, bad, beyond, Ty::Bool, span);
        let message = match first {
            0 => "a part `lo..hi` of a row or column needs 0 <= lo <= hi <= its length".to_string(),
            _ => format!("a part `lo..hi` of a row or column indexed from {first} needs {first} <= lo <= hi <= {first} + its length"),
        };
        let trap = Stmt::Trap(message, self.span(span));
        out.push(Stmt::If(bad, Block { stmts: vec![trap] }, Block::default()));
        (lo_e, hi_e)
    }

    pub(in crate::lower) fn slice_expr(
        &mut self,
        base: Expr,
        range: &ast::Expr,
        span: Span,
    ) -> Expr {
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
                self.report(bound.span, msgs::ir::BAD_OPERANDS_A_SLICE_BOUND_COUNTS, &[]);
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
                    let len = self.mk_builtin(Builtin::Len, [base.clone()], Ty::Int, span);
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
                            lw.mk_binary(BinOp::Sub, v, first, Ty::Int, e.span)
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
        // R07.3.1: a range is keyed by its values, so its slice is the sub-range between two keys.
        if let Ty::Range(elem) = &base.ty {
            if matches!(**elem, Ty::Int | Ty::Pos(_)) {
                return self.slice_range(base, lo.as_deref(), hi.as_deref(), *inclusive, span);
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
                self.report(
                    span,
                    msgs::ir::BAD_OPERANDS_CAN_T_SLICE_A,
                    &[&self.show(other)],
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
            out.push_let(base_slot, base);
            let base_ref = lw.local(base_slot, span);
            let len_expr = match dim_dom {
                Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                None => lw.mk_builtin(Builtin::Len, [base_ref.clone()], Ty::Int, span),
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
            out.push_let(lo_slot, lo_e);
            out.push_let(hi_slot, hi_e);
            let (lo_e, hi_e) = (lw.local(lo_slot, span), lw.local(hi_slot, span));
            let cmp = |lw: &mut Self, op: CmpOp, l: &Expr, r: &Expr| {
                lw.mk_cmp(op, l.clone(), r.clone(), Ty::Bool, span)
            };
            let or = |lw: &mut Self, l: Expr, r: Expr| {
                lw.mk_binary(BinOp::Or, l, r, Ty::Bool, span)
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
            out.push_let(acc_slot, empty_init);
            let idx_slot = lw.temp(Ty::Int);
            let idx_e = lw.local(idx_slot, span);
            let index_arg = match dim_dom {
                Some(d) => lw.mk(ExprKind::Cast(Cast::FromOrd(d), Box::new(idx_e)), Ty::Pos(d), span),
                None => idx_e,
            };
            let item = lw.mk_index(base_ref, vec![index_arg], elem_ty.clone(), span);
            // A `Str` grows by `push` like a list: `acc = acc + c` would copy it each step.
            let push_stmt = Stmt::Expr(lw.mk_mutate(Mutation::Push, Place {
                        kind: PlaceKind::Local(acc_slot),
                        ty: out_ty.clone(),
                        span: lw.span(span),
                    }, vec![item], Ty::Unit, span));
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
            out.push_give(res);
        })
    }
}
