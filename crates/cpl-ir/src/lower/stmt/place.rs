//! Places to store into: `x`, `a[i]`, `s.f`, `t.0`, frozen so their
//! indexes run once.

use super::*;

impl<'a> Lowerer<'a> {
    /// True for the `o` of `(o ??= init)`, which holds a value from there on.
    pub(in crate::lower) fn is_filled(&self, e: &ast::Expr) -> bool {
        self.filled
            .contains(&(self.cur_mod, e.span.start, e.span.end))
    }

    pub(in crate::lower) fn try_place(&mut self, e: &ast::Expr) -> Option<Place> {
        let diag_len = self.diags.len();
        let pending = self.cx.pending.len();
        let p = self.place(e);
        if self.diags.len() > diag_len || p.ty == Ty::Error {
            self.diags.truncate(diag_len);
            self.cx.pending.truncate(pending);
            None
        } else {
            Some(p)
        }
    }

    pub(in crate::lower) fn find_projection_on(
        &mut self,
        base: &ast::Expr,
        method: &str,
    ) -> Option<ProjectionInfo<'a>> {
        if let Some(Ty::Struct(sid)) = self.probe_ty_silent(base) {
            self.structs[sid].projections.get(method).cloned()
        } else {
            None
        }
    }

    /// Evaluate index subexpressions in `place` once into temp locals so a named
    /// mutable alias (`let st = mut a[k % 2]`) stays bound to the same element even
    /// if `k` changes later.
    pub(in crate::lower) fn freeze_place(&mut self, place: Place, out: &mut Vec<Stmt>) -> Place {
        let Place { kind, ty, span } = place;
        let kind = match kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => kind,
            PlaceKind::Field(base, fi) => {
                PlaceKind::Field(Box::new(self.freeze_place(*base, out)), fi)
            }
            PlaceKind::Some(base) => PlaceKind::Some(Box::new(self.freeze_place(*base, out))),
            PlaceKind::Index(base, idxs) => {
                let base = self.freeze_place(*base, out);
                let frozen_idxs = idxs
                    .into_iter()
                    .map(|idx| {
                        let wrap = match &idx.kind {
                            ExprKind::Cast(cast @ (Cast::Raw | Cast::Trust), _) => Some(*cast),
                            _ => None,
                        };
                        let slot = self.temp(idx.ty.clone());
                        let ispan = idx.span;
                        let ity = idx.ty.clone();
                        out.push(Stmt::Let(slot, Some(idx)));
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: ity.clone(),
                            span: ispan,
                        };
                        match wrap {
                            Some(cast) => Expr {
                                kind: ExprKind::Cast(cast, Box::new(held)),
                                ty: ity,
                                span: ispan,
                            },
                            None => held,
                        }
                    })
                    .collect();
                PlaceKind::Index(Box::new(base), frozen_idxs)
            }
        };
        Place { kind, ty, span }
    }

    /// `e` as something to store into.
    pub(in crate::lower) fn place(&mut self, e: &ast::Expr) -> Place {
        let span = self.span(e.span);
        let error = Place {
            kind: PlaceKind::Local(0),
            ty: Ty::Error,
            span,
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => {
                let place = self.place(inner);
                match &place.ty {
                    Ty::Opt(held) if self.is_filled(inner) => Place {
                        ty: (**held).clone(),
                        kind: PlaceKind::Some(Box::new(place)),
                        span,
                    },
                    _ => place,
                }
            }
            ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => {
                if let ast::ExprKind::Name(n) = &e.kind {
                    if let Some(aliased) = self.cx.mut_aliases.get(n).cloned() {
                        return aliased;
                    }
                }
                // Inside an inlined `lends` projection, `self` or bare fields of `self` refer to `inline_self`.
                if let Some((ref recv_place, sid)) = self.cx.inline_self.clone() {
                    match &e.kind {
                        ast::ExprKind::SelfValue => return recv_place.clone(),
                        ast::ExprKind::Name(n) if self.lookup(n).is_none() => {
                            if let Some((fi, fty)) = self.field_of(&Ty::Struct(sid), n) {
                                return Place {
                                    kind: PlaceKind::Field(Box::new(recv_place.clone()), fi),
                                    ty: fty,
                                    span,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                let value = self.expr(e, None);
                match value.kind {
                    ExprKind::Local(slot) => Place {
                        kind: PlaceKind::Local(slot),
                        ty: value.ty,
                        span,
                    },
                    ExprKind::Global(id) => Place {
                        kind: PlaceKind::Global(id),
                        ty: value.ty,
                        span,
                    },
                    // A bare field name inside a method.
                    ExprKind::Field(base, index) => match base.kind {
                        ExprKind::Local(slot) => Place {
                            kind: PlaceKind::Field(
                                Box::new(Place {
                                    kind: PlaceKind::Local(slot),
                                    ty: base.ty,
                                    span,
                                }),
                                index,
                            ),
                            ty: value.ty,
                            span,
                        },
                        _ => error,
                    },
                    ExprKind::Error => error,
                    _ => {
                        self.error(
                            codes::ASSIGN_TO_LET,
                            e.span,
                            "this name can't be assigned to",
                        );
                        error
                    }
                }
            }
            // `state!.k += 4`: the value inside, and the same failure as
            // reading it when there is none.
            // `xs.last!` and `xs.first!` are the cells `xs[xs.len - 1]` and
            // `xs[0]`: places, as `xs[i]` is.
            ast::ExprKind::Unwrap(inner)
                if matches!(
                    &inner.kind,
                    ast::ExprKind::Field { name, .. } if matches!(name.text.as_str(), "first" | "last")
                ) =>
            {
                let ast::ExprKind::Field { base, name } = &inner.kind else {
                    unreachable!("matched above")
                };
                let seq = self.place(base);
                let elem = match &seq.ty {
                    Ty::List(elem) => (**elem).clone(),
                    Ty::Array { dims, elem } if matches!(dims.as_slice(), [Dim::Int]) => {
                        (**elem).clone()
                    }
                    Ty::Error => return error,
                    other => {
                        let message = format!(
                            "`.{}!` is a cell of a list or a `[n]T` array, and this is a `{}`",
                            name.text,
                            self.show(other)
                        );
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        return error;
                    }
                };
                let at = if name.text == "first" {
                    self.int_lit(0, span)
                } else {
                    let len = ExprKind::Builtin(Builtin::Len, vec![place_to_expr(&seq)]);
                    let len = self.mk(len, Ty::Int, span);
                    self.int_add(len, -1)
                };
                Place {
                    kind: PlaceKind::Index(Box::new(seq), vec![at]),
                    ty: elem,
                    span,
                }
            }
            ast::ExprKind::Unwrap(inner) => {
                let place = self.place(inner);
                match place.ty.clone() {
                    Ty::Opt(held) => {
                        let mut lead = Vec::new();
                        let place = self.settle_indexes(place, &mut lead);
                        let holds = place_to_expr(&place);
                        let some = self.mk(ExprKind::IsSome(Box::new(holds)), Ty::Bool, span);
                        let none =
                            self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
                        lead.push(Stmt::If(
                            none,
                            Block {
                                stmts: vec![Stmt::Trap(NO_VALUE.to_string(), span)],
                            },
                            Block::default(),
                        ));
                        self.cx.pending.extend(lead);
                        Place {
                            ty: *held,
                            kind: PlaceKind::Some(Box::new(place)),
                            span,
                        }
                    }
                    Ty::Error => error,
                    other => {
                        let message = format!(
                            "`!` takes the value out of an optional, but this is a `{}`",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, e.span, message);
                        error
                    }
                }
            }
            ast::ExprKind::Index { base, indices } => {
                let mut base_place = self.place(base);
                if let [index] = indices.as_slice() {
                    if let Some(place) = self.slice_key_place(&base_place, index, e.span) {
                        return place;
                    }
                    if Self::sliceable_mut_elem(&base_place.ty).is_some() {
                        match self.mut_slice_place(base_place, index, e.span) {
                            Ok(view) => return view,
                            Err(place) => base_place = place,
                        }
                    }
                }
                let (indices, ty) = self.place_index_args(&base_place, indices, e.span);
                let indices = self.vouched(&base_place.ty, indices);
                Place {
                    kind: PlaceKind::Index(Box::new(base_place), indices),
                    ty,
                    span,
                }
            }
            ast::ExprKind::Field { base, name } => {
                let written = base;
                let base = self.place(base);
                match self.field_of(&base.ty, &name.text) {
                    Some((index, ty)) => Place {
                        kind: PlaceKind::Field(Box::new(base), index),
                        ty,
                        span,
                    },
                    None => {
                        if base.ty != Ty::Error {
                            let message = format!(
                                "`{}` has no field `{}` to assign to",
                                self.show(&base.ty),
                                name.text
                            );
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                            self.last_place_fix(&base.ty, written.span, name);
                        }
                        error
                    }
                }
            }
            _ => {
                if let Some(index) = self.builtin_at(e) {
                    return self.place(&index);
                }
                self.error(
                    codes::ASSIGN_TO_LET,
                    e.span,
                    "only a variable, a field, or an element can be assigned to",
                );
                error
            }
        }
    }

    /// A stored field of a struct, record or tuple (`t.0`).
    pub(in crate::lower) fn field_of(&self, ty: &Ty, name: &str) -> Option<(usize, Ty)> {
        match ty {
            Ty::Struct(id) => {
                let fields = &self.structs[*id].def.fields;
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Record(fields) => {
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Tuple(items) => {
                let index: usize = name.parse().ok()?;
                Some((index, items.get(index)?.clone()))
            }
            _ => None,
        }
    }
}

pub(in crate::lower) fn root_place(place: &Place) -> PlaceKind {
    match &place.kind {
        PlaceKind::Local(s) => PlaceKind::Local(*s),
        PlaceKind::Global(g) => PlaceKind::Global(*g),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            root_place(base)
        }
    }
}

/// Whether `e` only names where a value lives, a variable or a field of
/// one: reading it again costs nothing, and holding it would copy it.
pub(in crate::lower) fn names_a_place(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) => names_a_place(base),
        _ => false,
    }
}
