//! Coercions: a value as the type its context wants, converting where the
//! language does it silently.

use super::*;

impl<'a> Lowerer<'a> {
    /// `e` as a `want`, converting where the language does it silently.
    pub(in crate::lower) fn coerce(&mut self, e: Expr, want: &Ty, span: Span) -> Expr {
        match self.try_coerce(e, want) {
            Ok(e) => e,
            Err(e) => {
                self.mismatch(span, want, &e.ty);
                self.err_expr(span)
            }
        }
    }

    /// Whether an `if` with an `Int` branch beside a `ty` branch is a `ty`: `F64`, or a number
    /// struct an `Int` is made into (R04.4.28).
    pub(crate) fn int_joins(&mut self, ty: &Ty) -> bool {
        *ty == Ty::Float || matches!(ty, Ty::Struct(sid) if self.is_number(*sid))
    }

    pub(crate) fn try_coerce(&mut self, e: Expr, want: &Ty) -> Result<Expr, Expr> {
        if e.ty == *want || e.ty == Ty::Error || *want == Ty::Error {
            return Ok(e);
        }
        let span = e.span;
        if let (Ty::Fn(..), Some(op)) = (want, self.op_of_type(&e.ty)) {
            return Ok(self.op_closure(op, want, span));
        }
        let e = match self.range_domain_coerce(e, want) {
            Ok(done) => return Ok(done),
            Err(e) => e,
        };
        if same_cells(want, &e.ty) || self.same_value_domain(want, &e.ty) {
            return Ok(Expr {
                ty: want.clone(),
                ..e
            });
        }
        match (want, &e.ty) {
            // §124: integer literal / arithmetic to `D.bounds`.
            (Ty::Bound(d), Ty::Int) => {
                let d = *d;
                Ok(self.cast(Cast::Trust, e, Ty::Bound(d)))
            }
            // R04.2.18: an integer literal where a position is wanted is the position it labels.
            (Ty::Pos(d), Ty::Int)
                if matches!(e.kind, ExprKind::Int(_)) && !self.product_domains.contains_key(d) =>
            {
                let d = *d;
                Ok(self.cast(Cast::FromLabel(d), e, Ty::Pos(d)))
            }
            // §124: half-open cut range `Range<D.bounds>` -> `Range<D>` (same two ints, free).
            (Ty::Range(want_elem), Ty::Range(got_elem))
                if matches!(
                    (want_elem.as_ref(), got_elem.as_ref()),
                    (Ty::Pos(d1), Ty::Bound(d2)) if d1 == d2
                ) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::List(want_elem), Ty::List(got_elem))
                if matches!(
                    (want_elem.as_ref(), got_elem.as_ref()),
                    (Ty::Range(w), Ty::Range(g))
                        if matches!((w.as_ref(), g.as_ref()), (Ty::Pos(d1), Ty::Bound(d2)) if d1 == d2)
                ) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            // §124: tuple and record element-wise coercion (`(Int, Range<D.bounds>)` -> `(Int, Range<D>)`).
            (Ty::Tuple(_), Ty::Tuple(_)) | (Ty::Record(_), Ty::Record(_))
                if want.same_shape(&e.ty) =>
            {
                let (want_ty, want_items) = (want.clone(), want.fields().expect("tuple"));
                let got_items = e.ty.fields().expect("tuple");
                if let ExprKind::Tuple(elems) = e.kind {
                    let mut out_elems = Vec::with_capacity(elems.len());
                    for (el, w) in elems.into_iter().zip(&want_items) {
                        match self.try_coerce(el, w) {
                            Ok(c) => out_elems.push(c),
                            Err(orig) => {
                                return Err(Expr {
                                    kind: ExprKind::Tuple(vec![orig]),
                                    ty: e.ty,
                                    span,
                                });
                            }
                        }
                    }
                    return Ok(self.mk(ExprKind::Tuple(out_elems), want_ty, span));
                }
                let mut ok = true;
                let coerced = self.do_expr(Some(&want_ty.clone()), span, |lw, out| {
                    let held = lw.hold(e.clone(), out);
                    let mut elems = Vec::with_capacity(want_items.len());
                    for (i, w) in want_items.iter().enumerate() {
                        let field_ty = got_items[i].clone();
                        let f = lw.mk_field(held.clone(), i, field_ty, span);
                        match lw.try_coerce(f, w) {
                            Ok(c) => elems.push(c),
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        let tup = lw.mk(ExprKind::Tuple(elems), want_ty.clone(), span);
                        out.push_give(tup);
                    }
                });
                if ok {
                    return Ok(coerced);
                }
                Err(e)
            }
            (Ty::List(want_elem), Ty::DomainVal) => {
                if let ExprKind::DomainVal(d) = e.kind {
                    if **want_elem == Ty::Pos(d) {
                        return Ok(self.positions(d, span));
                    }
                }
                Err(e)
            }
            (Ty::Range(want_elem), Ty::DomainVal) => {
                if let ExprKind::DomainVal(d) = e.kind {
                    if **want_elem == Ty::Pos(d) {
                        let lo = self.int_lit(0, span);
                        let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        return Ok(self.mk(
                            ExprKind::Range {
                                lo: Box::new(lo),
                                hi: Box::new(hi),
                            },
                            Ty::range(Ty::Pos(d)),
                            span,
                        ));
                    }
                }
                Err(e)
            }
            (Ty::ListView(we, wk), Ty::Keyed(ge, gk))
                if we == ge && wk == gk && matches!(e.kind, ExprKind::Local(_)) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Str, Ty::StrView) => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ToStr, vec![e]),
                ty: Ty::Str,
                span,
            }),
            (
                Ty::List(want_elem),
                Ty::ListView(got_elem, _) | Ty::Keyed(got_elem, _) | Ty::MutView(got_elem, _),
            ) if want_elem == got_elem => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::Pos(d), Ty::Tuple(ts))
                if self
                    .product_domains
                    .get(d)
                    .is_some_and(|fs| fs.len() == ts.len()) =>
            {
                let d = *d;
                let ts = ts.clone();
                let mut stmts = Vec::new();
                let held = self.hold(e, &mut stmts);
                let parts: Vec<Expr> = ts
                    .into_iter()
                    .enumerate()
                    .map(|(i, fty)| self.mk_field(held.clone(), i, fty, span))
                    .collect();
                let encoded = self.encode_product(d, parts, span);
                if stmts.is_empty() {
                    Ok(encoded)
                } else {
                    stmts.push_give(encoded);
                    Ok(self.mk(ExprKind::Do(Block { stmts }), Ty::Pos(d), span))
                }
            }
            (Ty::Pos(d), Ty::Record(rfs))
                if self
                    .product_domains
                    .get(d)
                    .is_some_and(|fs| fs.len() == rfs.len()) =>
            {
                let d = *d;
                let rfs = rfs.clone();
                let mut stmts = Vec::new();
                let held = self.hold(e, &mut stmts);
                let parts: Vec<Expr> = rfs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, fty))| self.mk_field(held.clone(), i, fty, span))
                    .collect();
                let encoded = self.encode_product(d, parts, span);
                if stmts.is_empty() {
                    Ok(encoded)
                } else {
                    stmts.push_give(encoded);
                    Ok(self.mk(ExprKind::Do(Block { stmts }), Ty::Pos(d), span))
                }
            }
            // R04.2.10: a position of a product domain is a tuple of factor positions.
            (Ty::Tuple(ts), Ty::Pos(d))
                if self
                    .product_domains
                    .get(d)
                    .is_some_and(|fs| fs.len() == ts.len()) =>
            {
                let d = *d;
                let mut stmts = Vec::new();
                let parts = self.decode_product(d, e, span, &mut stmts);
                let tup = self.mk(ExprKind::Tuple(parts), want.clone(), span);
                if stmts.is_empty() {
                    Ok(tup)
                } else {
                    stmts.push_give(tup);
                    Ok(self.mk(ExprKind::Do(Block { stmts }), want.clone(), span))
                }
            }
            (Ty::Record(rfs), Ty::Pos(d))
                if self.anon_product_domain_set.contains(d)
                    && self
                        .product_domains
                        .get(d)
                        .is_some_and(|fs| fs.len() == rfs.len()) =>
            {
                let d = *d;
                let mut stmts = Vec::new();
                let parts = self.decode_product(d, e, span, &mut stmts);
                let rec = self.mk(ExprKind::Tuple(parts), want.clone(), span);
                if stmts.is_empty() {
                    Ok(rec)
                } else {
                    stmts.push_give(rec);
                    Ok(self.mk(ExprKind::Do(Block { stmts }), want.clone(), span))
                }
            }
            (Ty::Pos(d), Ty::DomainVal) => Ok(Expr {
                kind: ExprKind::Cast(
                    Cast::Trust,
                    Box::new(Expr {
                        kind: ExprKind::Int(0),
                        ty: Ty::Int,
                        span,
                    }),
                ),
                ty: Ty::Pos(*d),
                span,
            }),
            // An `Int` where a number struct is wanted is made into it
            // (decision 3.10): `var ans: Z = 1` is `Z(1)`.
            (Ty::Struct(sid), Ty::Int) if self.is_number(*sid) => {
                match self.own_method(*sid, "new") {
                    Some(new_fn) => {
                        let ret = self.ret_of(new_fn, span);
                        Ok(Expr {
                            kind: ExprKind::Call(new_fn, vec![Arg::Value(e)]),
                            ty: ret,
                            span,
                        })
                    }
                    None => Err(e),
                }
            }
            (Ty::I128, Ty::Int) => Ok(self.cast(Cast::IntToI128, e, Ty::I128)),
            (Ty::U64, Ty::Int) => Ok(self.cast(Cast::IntToU64, e, Ty::U64)),
            (Ty::Float, Ty::Int) => Ok(match e.kind {
                ExprKind::Int(v) => Expr {
                    kind: ExprKind::Float(v as f64),
                    ty: Ty::Float,
                    span,
                },
                _ => Expr {
                    kind: ExprKind::Cast(Cast::IntToFloat, Box::new(e)),
                    ty: Ty::Float,
                    span,
                },
            }),
            (Ty::Opt(inner), _) if !matches!(e.kind, ExprKind::None) => {
                let inner = self.try_coerce(e, inner)?;
                Ok(Expr {
                    kind: ExprKind::Some(Box::new(inner)),
                    ty: want.clone(),
                    span,
                })
            }
            (Ty::List(elem), Ty::Str) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::StrToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::Str, Ty::List(elem)) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ListToStr, vec![e]),
                ty: Ty::Str,
                span,
            }),
            // `place(D.all)` where a `List<D>` is wanted: a range written
            // out is the list of what it goes over.
            (Ty::List(elem), Ty::Range(of))
                if elem == of && matches!(e.kind, ExprKind::Range { .. }) =>
            {
                let ExprKind::Range { lo, hi } = e.kind else {
                    unreachable!()
                };
                Ok(self.range_list(*lo, *hi, (**elem).clone(), span))
            }
            // A struct that walks (a std view such as `Mapped`) where a `List`/`Seq` is
            // wanted is the walk of its `Seq` impl (R07.1.2).
            (Ty::List(_), Ty::Struct(sid)) if self.struct_walk(*sid, span).is_some() => {
                let walked = self.walked_struct(e);
                self.try_coerce(walked, want)
            }
            (Ty::List(elem), Ty::Array { elem: aelem, .. }) if *elem == *aelem => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ArrayToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::HashMap(..), Ty::Map(..)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::HashSet(_), Ty::Set(_)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::List(w_elem), Ty::List(e_elem)) if w_elem != e_elem => {
                // Element-wise list coercion (e.g. `List<Int>` -> `List<?Int>`, `List<(Char, Int)>` -> `List<(val: Char, len: Int)>`).
                let can_elem = matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Opt(inner), other) if **inner == *other
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Record(wf), Ty::Tuple(et)) if wf.len() == et.len() && wf.iter().zip(et).all(|((_, a), b)| a == b)
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Tuple(wt), Ty::Record(ef)) if wt.len() == ef.len() && wt.iter().zip(ef).all(|(a, (_, b))| a == b)
                );
                if !can_elem {
                    return Err(e);
                }
                let dst_ty = want.clone();
                let w_elem = (**w_elem).clone();
                let e_elem = (**e_elem).clone();
                Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                    let src_ref = lw.hold(e, out);
                    let acc_slot = lw.temp(dst_ty.clone());
                    out.push_let(
                        acc_slot,
                        lw.mk(ExprKind::Tuple(Vec::new()), dst_ty.clone(), span),
                    );
                    let item_slot = lw.temp(e_elem.clone());
                    let item_expr = lw.local(item_slot, span);
                    let coerced = lw.try_coerce(item_expr, &w_elem).unwrap_or_else(|x| x);
                    let push = lw.mk_mutate(
                        Mutation::Push,
                        Place {
                            kind: PlaceKind::Local(acc_slot),
                            ty: dst_ty.clone(),
                            span,
                        },
                        vec![coerced],
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        item_slot,
                        Iter::Seq(src_ref, false),
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push_give(lw.local(acc_slot, span));
                }))
            }
            (
                Ty::Array {
                    dims: w_dims,
                    elem: w_elem,
                },
                Ty::Array {
                    dims: e_dims,
                    elem: e_elem,
                },
            ) if w_dims == e_dims && w_dims.len() == 1 => {
                // Structural record projection for 1D arrays, e.g. `[E](u: V, v: V, w: Int)` -> `[E](u: V, v: V)`.
                if let (Ty::Record(w_fields), Ty::Record(e_fields)) = (&**w_elem, &**e_elem) {
                    let indices: Option<Vec<usize>> = w_fields
                        .iter()
                        .map(|(wn, wt)| e_fields.iter().position(|(en, et)| en == wn && et == wt))
                        .collect();
                    if let Some(indices) = indices {
                        let dim = w_dims[0];
                        let dst_ty = want.clone();
                        let w_rec = (**w_elem).clone();
                        let e_rec = (**e_elem).clone();
                        return Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                            let src = lw.hold(e, out);
                            let dim_spec = match dim {
                                Dim::Pos(d) => DimSpec::Pos(d),
                                Dim::Bound(d) => DimSpec::Bound(d),
                                Dim::Sym(d) => DimSpec::Sym(d),
                                Dim::Enum(e) => DimSpec::Enum(e),
                                Dim::Int | Dim::IntFrom(_) => {
                                    let sref = src.clone();
                                    DimSpec::Int(Box::new(lw.len_of(sref, span)))
                                }
                            };
                            let arr_slot = lw.temp(dst_ty.clone());
                            out.push_let(
                                arr_slot,
                                lw.mk(ExprKind::Fill(vec![dim_spec], None), dst_ty.clone(), span),
                            );
                            let idx_ty = match dim {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Enum(e) => Ty::Enum(e),
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                            };
                            let idx_slot = lw.temp(idx_ty.clone());
                            let idx_expr = lw.local(idx_slot, span);
                            let sref = src.clone();
                            let elem_expr = lw.mk_index(
                                sref.clone(),
                                vec![idx_expr.clone()],
                                e_rec.clone(),
                                span,
                            );
                            let elem_slot = lw.temp(e_rec);
                            let el_ref = lw.local(elem_slot, span);
                            let proj_fields: Vec<Expr> = indices
                                .iter()
                                .zip(w_fields)
                                .map(|(&fi, (_, ft))| {
                                    lw.mk_field(el_ref.clone(), fi, ft.clone(), span)
                                })
                                .collect();
                            let proj_val = lw.mk(ExprKind::Tuple(proj_fields), w_rec.clone(), span);
                            let set_stmt = Stmt::Assign(
                                Place {
                                    kind: PlaceKind::Index(
                                        Box::new(Place {
                                            kind: PlaceKind::Local(arr_slot),
                                            ty: dst_ty.clone(),
                                            span,
                                        }),
                                        vec![idx_expr],
                                    ),
                                    ty: w_rec,
                                    span,
                                },
                                AssignOp::Set,
                                proj_val,
                            );
                            let iter = match dim {
                                Dim::Pos(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Bound(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.int_add(lw.mk(ExprKind::DomainLen(d), Ty::Int, span), 1),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Enum(e) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.int_lit(lw.enums[e].variants.len() as i64, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.len_of(sref, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                            };
                            let lid = lw.next_loop;
                            lw.next_loop += 1;
                            out.push(Stmt::For(
                                lid,
                                idx_slot,
                                iter,
                                Block {
                                    stmts: vec![Stmt::Let(elem_slot, Some(elem_expr)), set_stmt],
                                },
                            ));
                            out.push_give(lw.local(arr_slot, span));
                        }));
                    }
                }
                Err(e)
            }
            (Ty::Record(fields), Ty::Tuple(items))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Tuple(items), Ty::Record(fields))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Record(f1), Ty::Record(f2))
                if f1.len() == f2.len() && f1.iter().zip(f2).all(|((_, a), (_, b))| a == b) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            // Structural struct subtyping (e.g. `Group<T>` -> `Monoid<T>` -> `Semigroup<T>`).
            (Ty::Struct(want_id), Ty::Struct(have_id)) => {
                if let Some(projected) =
                    self.coerce_struct_projection(e.clone(), *have_id, *want_id)
                {
                    Ok(projected)
                } else {
                    Err(e)
                }
            }
            _ => Err(e),
        }
    }
}
