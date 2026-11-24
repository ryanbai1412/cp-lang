//! Between an array's plain range and the domain a generic fn knows it by:
//! sizing a domain from the array given for it.

use super::*;

impl<'a> Lowerer<'a> {
    /// Whether `d` stands for the plain range that indexes an array, and
    /// this code is not the generic fn that was given it: here its
    /// positions are the plain `Int`s the array is indexed with.
    pub(crate) fn is_range_domain(&self, d: DomainId) -> bool {
        self.range_domains.contains_key(&d)
            && !self.gen_env.values().any(|ga| *ga == GenArg::Domain(d))
    }

    /// The type as the caller of a generic fn writes it: a position or
    /// boundary of an array's plain range is an `Int`.
    pub(crate) fn plain_ty(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Pos(d) | Ty::Bound(d) if self.is_range_domain(*d) => Ty::Int,
            Ty::Opt(inner) => Ty::Opt(Box::new(self.plain_ty(inner))),
            Ty::Range(inner) => Ty::Range(Box::new(self.plain_ty(inner))),
            Ty::List(inner) => Ty::List(Box::new(self.plain_ty(inner))),
            Ty::Array { dims, elem } if matches!(dims.as_slice(), [Dim::Pos(d)] if self.is_range_domain(*d) && self.list_domains.contains(d)) => {
                Ty::List(Box::new(self.plain_ty(elem)))
            }
            Ty::Array { dims, elem } => Ty::Array {
                dims: dims
                    .iter()
                    .map(|dim| match dim {
                        Dim::Pos(d) if self.is_range_domain(*d) => match self.range_domains[d] {
                            0 => Dim::Int,
                            lo => Dim::IntFrom(lo),
                        },
                        other => *other,
                    })
                    .collect(),
                elem: Box::new(self.plain_ty(elem)),
            },
            other => other.clone(),
        }
    }

    /// The `List<T>` that was given when `want` is a `[D]T` whose `D`
    /// stands for a list's positions.
    pub(crate) fn list_given(&self, want: &Ty) -> Option<Ty> {
        match want {
            Ty::Array { dims, elem } if matches!(dims.as_slice(), [Dim::Pos(d)] if self.list_domains.contains(d)) => {
                Some(Ty::List(Box::new(self.plain_ty(elem))))
            }
            _ => None,
        }
    }

    /// The domain and first index when an array indexed by a plain range
    /// is given where its `[D]T` is asked for.
    pub(crate) fn range_domain_of(&self, want: &Ty, have: &Ty) -> Option<(DomainId, i64)> {
        let Ty::Array {
            dims: want_dims,
            elem: want_elem,
        } = want
        else {
            return None;
        };
        let (lo, have_elem) = match have {
            Ty::Array { dims, elem } => match dims.as_slice() {
                [Dim::Int] => (0, elem),
                [Dim::IntFrom(lo)] => (*lo, elem),
                _ => return None,
            },
            // A list lent `mut` stays a list: see `list_given`.
            _ => return None,
        };
        match want_dims.as_slice() {
            [Dim::Pos(d)] if want_elem == have_elem && self.range_domains.get(d) == Some(&lo) => {
                Some((*d, lo))
            }
            _ => None,
        }
    }

    /// What an array given for `D` does to it: the first sets its size,
    /// each one after must have that size.
    pub(crate) fn size_domain(&mut self, d: DomainId, lo: i64, len: Expr, span: Span) -> Stmt {
        let first = self.call_sized.as_mut().is_none_or(|sized| sized.insert(d));
        if first {
            return Stmt::DefDomain {
                id: d,
                lo: self.int_lit(lo, span),
                len,
            };
        }
        let sized = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        let differ = self.mk(
            ExprKind::Cmp(CmpOp::Ne, Box::new(len), Box::new(sized)),
            Ty::Bool,
            span,
        );
        let name = self.domain_said(d);
        let message = format!("the arrays given for one `{name}` must have the same length");
        let trap = Stmt::Trap(message, self.span(span));
        Stmt::If(differ, Block { stmts: vec![trap] }, Block::default())
    }

    pub(in crate::lower) fn coerce_list_elems(
        &mut self,
        seq: Expr,
        elem_ty: Ty,
        want_elem_ty: Ty,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> Expr {
        let dst_ty = Ty::list(want_elem_ty.clone());
        let dst = self.temp(dst_ty.clone());
        let empty = self.mk(ExprKind::Tuple(Vec::new()), dst_ty.clone(), span);
        out.push(Stmt::Let(dst, Some(empty)));
        let item = self.temp(elem_ty);
        let coerced = self.coerce(self.local(item, span), &want_elem_ty, span);
        let push = Stmt::Expr(self.mk(
            ExprKind::Mutate(
                Mutation::Push,
                Box::new(Place {
                    kind: PlaceKind::Local(dst),
                    ty: dst_ty,
                    span: self.span(span),
                }),
                vec![coerced],
            ),
            Ty::Unit,
            span,
        ));
        let id = self.fresh_loop();
        out.push(Stmt::For(
            id,
            item,
            Iter::Seq(seq, false),
            Block { stmts: vec![push] },
        ));
        self.local(dst, span)
    }

    /// Between an array's plain range and the domain a generic fn knows
    /// it by: `[0..n]T` to `[D]T` (which sets the size of `D`), an `Int`
    /// to a position or boundary (checked), and each of those back.
    pub(in crate::lower) fn range_domain_coerce(
        &mut self,
        e: Expr,
        want: &Ty,
    ) -> Result<Expr, Expr> {
        // `e.span` is already where the file's text is; `mk` shifts again.
        let span = match self.cur_mod {
            0 => Span::new(
                e.span.start.saturating_sub(self.offset),
                e.span.end.saturating_sub(self.offset),
            ),
            _ => e.span,
        };
        // A list is an array over `0..len`, and that array a list.
        let e = match (want, &e.ty) {
            (
                Ty::Array {
                    dims,
                    elem: want_elem,
                },
                Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _),
            ) if want_elem == elem || self.plain_ty(want_elem) == **elem => match dims.as_slice() {
                [Dim::Pos(d)] if self.list_domains.contains(d) => {
                    let d = *d;
                    let needs_elem_coerce = want_elem != elem;
                    let want_elem_ty = (**want_elem).clone();
                    let elem_ty = (**elem).clone();
                    // Passing the list sets the size of its domain.
                    return Ok(self.do_expr(Some(want), span, |lw, out| {
                        let held = lw.hold(e, out);
                        let len = lw.mk(
                            ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                            Ty::Int,
                            span,
                        );
                        let (size, held) = lw.size_for_list(d, len, held, span);
                        out.push(size);
                        let list_expr = if needs_elem_coerce {
                            lw.coerce_list_elems(held, elem_ty, want_elem_ty, span, out)
                        } else {
                            held
                        };
                        let array = ExprKind::Builtin(Builtin::ListToArray, vec![list_expr]);
                        out.push(Stmt::Give(lw.mk(array, want.clone(), span)));
                    }));
                }
                // Cells keyed by `d`'s positions (`cells.rekey(s.keys)` with `s: [d]T`) are a
                // `[d]T` once they cover all of `d`.
                [Dim::Pos(d)]
                    if want_elem == elem
                        && matches!(&e.ty, Ty::Keyed(_, Dim::Pos(k)) | Ty::ListView(_, Dim::Pos(k)) if k == d) =>
                {
                    let d = *d;
                    let list_ty = Ty::list((**elem).clone());
                    return Ok(self.do_expr(Some(want), span, |lw, out| {
                        let held = lw.hold(e, out);
                        let len = lw.mk(
                            ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                            Ty::Int,
                            span,
                        );
                        let n = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let short = lw.mk(
                            ExprKind::Cmp(CmpOp::Ne, Box::new(len), Box::new(n)),
                            Ty::Bool,
                            span,
                        );
                        let trap = Stmt::Trap(
                            "a table keyed by part of a domain is not an array over all of it"
                                .into(),
                            lw.span(span),
                        );
                        out.push(Stmt::If(
                            short,
                            Block { stmts: vec![trap] },
                            Block::default(),
                        ));
                        let list = lw.mk(
                            ExprKind::Builtin(Builtin::ToList, vec![held]),
                            list_ty,
                            span,
                        );
                        let array = ExprKind::Builtin(Builtin::ListToArray, vec![list]);
                        out.push(Stmt::Give(lw.mk(array, want.clone(), span)));
                    }));
                }
                _ => e,
            },
            (Ty::List(want_elem), Ty::Array { dims, elem })
                if (want_elem == elem || self.plain_ty(elem) == **want_elem)
                    && matches!(dims.as_slice(), [Dim::Pos(d)] if self.list_domains.contains(d)) =>
            {
                if want_elem == elem {
                    let list = ExprKind::Builtin(Builtin::ArrayToList, vec![e]);
                    return Ok(self.mk(list, want.clone(), span));
                }
                let want_elem_ty = (**want_elem).clone();
                let elem_ty = (**elem).clone();
                return Ok(self.do_expr(Some(want), span, |lw, out| {
                    let held = lw.hold(e, out);
                    let raw = lw.mk(
                        ExprKind::Builtin(Builtin::ArrayToList, vec![held]),
                        Ty::list(elem_ty.clone()),
                        span,
                    );
                    let dst = lw.coerce_list_elems(raw, elem_ty, want_elem_ty, span, out);
                    out.push(Stmt::Give(dst));
                }));
            }
            (Ty::List(want_elem), Ty::List(have_elem))
                if want_elem != have_elem && self.plain_ty(have_elem) == **want_elem =>
            {
                let want_elem_ty = (**want_elem).clone();
                let have_elem_ty = (**have_elem).clone();
                return Ok(self.do_expr(Some(want), span, |lw, out| {
                    let dst = lw.coerce_list_elems(e, have_elem_ty, want_elem_ty, span, out);
                    out.push(Stmt::Give(dst));
                }));
            }
            _ => e,
        };
        let have = e.ty.clone();
        match (want, &have) {
            (Ty::Pos(d), Ty::Int) if self.range_domains.contains_key(d) => {
                Ok(self.cast(Cast::FromLabel(*d), e, want.clone()))
            }
            (Ty::Int, Ty::Pos(d)) if self.range_domains.contains_key(d) => {
                Ok(self.cast(Cast::Label(*d), e, Ty::Int))
            }
            (Ty::Int, Ty::Bound(d)) if self.range_domains.contains_key(d) => {
                let ord = self.cast(Cast::Ord, e, Ty::Int);
                let lo = self.mk(ExprKind::DomainLo(*d), Ty::Int, span);
                Ok(self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(ord), Box::new(lo)),
                    Ty::Int,
                    span,
                ))
            }
            (Ty::Bound(d), Ty::Int) if self.range_domains.contains_key(d) => {
                let d = *d;
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let lo = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                    let ord = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(e), Box::new(lo)),
                        Ty::Int,
                        span,
                    );
                    let slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(slot, Some(ord)));
                    let below = lw.mk(
                        ExprKind::Cmp(
                            CmpOp::Lt,
                            Box::new(lw.local(slot, span)),
                            Box::new(lw.int_lit(0, span)),
                        ),
                        Ty::Bool,
                        span,
                    );
                    let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let above = lw.mk(
                        ExprKind::Cmp(CmpOp::Lt, Box::new(len), Box::new(lw.local(slot, span))),
                        Ty::Bool,
                        span,
                    );
                    let outside = lw.mk(
                        ExprKind::Binary(BinOp::Or, Box::new(below), Box::new(above)),
                        Ty::Bool,
                        span,
                    );
                    let trap = Stmt::Trap(
                        "a boundary outside the array's range".to_string(),
                        lw.span(span),
                    );
                    out.push(Stmt::If(
                        outside,
                        Block { stmts: vec![trap] },
                        Block::default(),
                    ));
                    let bound = lw.cast(Cast::Trust, lw.local(slot, span), Ty::Bound(d));
                    out.push(Stmt::Give(bound));
                }))
            }
            (Ty::Range(want_of), Ty::Range(have_of)) => {
                let (d, into) = match (&**want_of, &**have_of) {
                    (Ty::Pos(d), Ty::Int) if self.range_domains.contains_key(d) => (*d, true),
                    (Ty::Int, Ty::Pos(d)) if self.range_domains.contains_key(d) => (*d, false),
                    _ => return Err(e),
                };
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let held = lw.hold(e, out);
                    let mut end = |lw: &mut Self, which: Builtin| {
                        let at = lw.mk(ExprKind::Builtin(which, vec![held.clone()]), Ty::Int, span);
                        let first = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                        let op = if into { BinOp::Sub } else { BinOp::Add };
                        let moved = lw.mk(
                            ExprKind::Binary(op, Box::new(at), Box::new(first)),
                            Ty::Int,
                            span,
                        );
                        let slot = lw.temp(Ty::Int);
                        out.push(Stmt::Let(slot, Some(moved)));
                        lw.local(slot, span)
                    };
                    let (lo, hi) = (end(lw, Builtin::RangeLo), end(lw, Builtin::RangeHi));
                    if into {
                        // An empty range reads nothing, wherever it is.
                        let cmp = |lw: &Self, a: Expr, b: Expr| {
                            lw.mk(
                                ExprKind::Cmp(CmpOp::Lt, Box::new(a), Box::new(b)),
                                Ty::Bool,
                                span,
                            )
                        };
                        let or = |lw: &Self, op: BinOp, a: Expr, b: Expr| {
                            lw.mk(
                                ExprKind::Binary(op, Box::new(a), Box::new(b)),
                                Ty::Bool,
                                span,
                            )
                        };
                        let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let outside = or(
                            lw,
                            BinOp::Or,
                            cmp(lw, lo.clone(), lw.int_lit(0, span)),
                            cmp(lw, len, hi.clone()),
                        );
                        let wrong = or(lw, BinOp::And, cmp(lw, lo.clone(), hi.clone()), outside);
                        let trap = Stmt::Trap(
                            "a range outside the array's range".to_string(),
                            lw.span(span),
                        );
                        out.push(Stmt::If(
                            wrong,
                            Block { stmts: vec![trap] },
                            Block::default(),
                        ));
                    }
                    let range = ExprKind::Range {
                        lo: Box::new(lo),
                        hi: Box::new(hi),
                    };
                    out.push(Stmt::Give(lw.mk(range, want.clone(), span)));
                }))
            }
            (Ty::Opt(want_inner), Ty::Opt(have_inner))
                if want_inner != have_inner
                    && self.plain_ty(want_inner) == self.plain_ty(have_inner) =>
            {
                let (want_inner, have) = ((**want_inner).clone(), have.clone());
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let slot = lw.temp(have);
                    out.push(Stmt::Let(slot, Some(e)));
                    let held = lw.local(slot, span);
                    let inner_ty = match &held.ty {
                        Ty::Opt(inner) => (**inner).clone(),
                        _ => Ty::Error,
                    };
                    let inner = lw.mk(
                        ExprKind::Unwrap(Box::new(held.clone()), false),
                        inner_ty,
                        span,
                    );
                    let inner = lw.coerce(inner, &want_inner, span);
                    let some = lw.mk(ExprKind::Some(Box::new(inner)), want.clone(), span);
                    let none = lw.mk(ExprKind::None, want.clone(), span);
                    let is_some = lw.mk(ExprKind::IsSome(Box::new(held)), Ty::Bool, span);
                    out.push(Stmt::Give(lw.mk(
                        ExprKind::If(Box::new(is_some), Box::new(some), Box::new(none)),
                        want.clone(),
                        span,
                    )));
                }))
            }
            (
                Ty::Array {
                    dims: want_dims,
                    elem: want_elem,
                },
                Ty::Array {
                    dims: have_dims,
                    elem: have_elem,
                },
            ) if want_elem == have_elem
                && want_dims != have_dims
                && want_dims.len() == have_dims.len() =>
            {
                let first = |dim: &Dim| match dim {
                    Dim::Int => Some(0),
                    Dim::IntFrom(lo) => Some(*lo),
                    _ => None,
                };
                // The domains each axis sets the size of.
                let mut sized = Vec::new();
                for (axis, (w, h)) in want_dims.iter().zip(have_dims).enumerate() {
                    match (w, h) {
                        _ if w == h => {}
                        (Dim::Pos(d), h)
                            if self
                                .range_domains
                                .get(d)
                                .is_some_and(|lo| first(h) == Some(*lo)) =>
                        {
                            sized.push((axis, *d, self.range_domains[d]));
                        }
                        (w, Dim::Pos(d))
                            if self
                                .range_domains
                                .get(d)
                                .is_some_and(|lo| first(w) == Some(*lo)) => {}
                        _ => return Err(e),
                    }
                }
                if sized.is_empty() {
                    return Ok(Expr {
                        ty: want.clone(),
                        ..e
                    });
                }
                let grid = want_dims.len() > 1;
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let held = lw.hold(e, out);
                    for (axis, d, lo) in sized {
                        let len = match grid {
                            false => ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                            true => ExprKind::Builtin(
                                Builtin::AxisLen,
                                vec![held.clone(), lw.int_lit(axis as i64, span)],
                            ),
                        };
                        let len = lw.mk(len, Ty::Int, span);
                        let size = lw.size_domain(d, lo, len, span);
                        out.push(size);
                    }
                    out.push(Stmt::Give(Expr {
                        ty: want.clone(),
                        ..held
                    }));
                }))
            }
            _ => Err(e),
        }
    }
}
