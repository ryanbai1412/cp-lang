//! A method on a value, chosen by the value's type: `xs.push(x)`,
//! `m.get(k)`, `s.split(c)`, after `method` has ruled out the type-level
//! and struct receivers.

use super::*;

impl<'a> Lowerer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::lower) fn value_method(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        value: Expr,
        ty: Ty,
        elem: Ty,
    ) -> Expr {
        let span = whole.span;
        let method = name.text.as_str();
        let mutate = |lw: &mut Self, op: Mutation, args: Vec<Expr>, result: Ty| {
            let place = lw.place(base);
            lw.mk(ExprKind::Mutate(op, Box::new(place), args), result, span)
        };
        match (&ty, method) {
            (Ty::Pos(d), "with") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord), Box::new(bit)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            // `s.set(v, on)` is `s.with(v)` when `on` holds and `s.without(v)`
            // when it does not.
            (Ty::Pos(d), "set") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(exprs) = self.positional_any(method, args, 2, span) else {
                    return self.err_expr(span);
                };
                let v = self.expect(exprs[0], &Ty::Pos(base_d));
                let on = self.expect(exprs[1], &Ty::Bool);
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let with = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord.clone()), Box::new(bit.clone())),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let without = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                // One side runs, so `s` and `v` are each worked out once.
                let res = self.mk(
                    ExprKind::If(Box::new(on), Box::new(with), Box::new(without)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Pos(d), "without") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Error, _) => {
                let reported = self.diags.len();
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.diags.truncate(reported);
                self.err_expr(span)
            }
            (Ty::Pos(d), "l") => {
                let d = *d;
                match self.one_arg("l", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepL(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "r") => {
                let d = *d;
                match self.one_arg("r", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepR(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "offset" | "ahead") => {
                let d = *d;
                match self.one_arg(method, args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosOffset(d), vec![value, k]),
                        Ty::opt(Ty::Pos(d)),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "back") => {
                let d = *d;
                match self.one_arg("back", args, &Ty::Int, span) {
                    Some(k) => {
                        let neg_k = self.mk(ExprKind::Unary(UnOp::Neg, Box::new(k)), Ty::Int, span);
                        self.mk(
                            ExprKind::Builtin(Builtin::PosOffset(d), vec![value, neg_k]),
                            Ty::opt(Ty::Pos(d)),
                            span,
                        )
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "cyc") => {
                let d = *d;
                match self.one_arg("cyc", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosCyc(d), vec![value, k]),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Str, "join") => {
                let Some(exprs) = self.positional_any("join", args, 1, span) else {
                    return self.err_expr(span);
                };
                let items = self.expr(exprs[0], None);
                self.join(value, items, None, exprs[0].span)
            }
            (ty, "get") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                let opt = Ty::opt(elem);
                match self.one_arg("get", args, &Ty::Int, span) {
                    Some(at) => self.mk(
                        ExprKind::Builtin(Builtin::SeqGet, vec![value, at]),
                        opt,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if Self::keyed_view(ty) => {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("at", args, &Ty::Int, span) {
                    Some(at) => {
                        let view = self.hold_view(value, span);
                        self.key_index(view, at, elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "nth")
                if ty.is_indexed()
                    && matches!(
                        ty,
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::Str | Ty::StrView
                    ) =>
            {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("nth", args, &Ty::Int, span) {
                    Some(at) => self.mk(ExprKind::Index(Box::new(value), vec![at]), elem, span),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("at", args, &ty.key_ty(), span) {
                    Some(at) => {
                        let indices = self.vouched(ty, vec![at]);
                        self.mk(ExprKind::Index(Box::new(value), indices), elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "set_at") if ty.is_indexed_mut() => {
                let elem = ty.seq_elem().unwrap();
                let ty_owned = ty.clone();
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let at = self.expect(exprs[0], &Ty::Int);
                        let val = self.expect(exprs[1], &elem);
                        let val_span = exprs[1].span;
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let val = match lw
                                .place_range(&place)
                                .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                            {
                                Some(b) => lw.check_range(val, &b, val_span),
                                None => val,
                            };
                            let indices = lw.vouched(&ty_owned, vec![at]);
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(place), indices),
                                ty: elem,
                                span: lw.span(span),
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, val));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "swap_idx") if ty.is_indexed_mut() => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let a = self.expect(exprs[0], &Ty::Int);
                        let b = self.expect(exprs[1], &Ty::Int);
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => {
                    let place = self.place(base);
                    let arg = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(arg, &b, span),
                        None => arg,
                    };
                    self.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]),
                        Ty::Unit,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            (Ty::List(_), "push_id") => match self.one_arg(method, args, &elem, span) {
                // The index the new element lands on.
                Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let arg = match lw
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => lw.check_range(arg, &b, span),
                        None => arg,
                    };
                    let held = lw.temp(arg.ty.clone());
                    out.push(Stmt::Let(held, Some(arg)));
                    let id = lw.temp(Ty::Int);
                    let list = super::stmt::place_to_expr(&place);
                    let len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![list]), Ty::Int, span);
                    out.push(Stmt::Let(id, Some(len)));
                    let arg = lw.local(held, span);
                    let push = lw.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]),
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(push));
                    let id = lw.local(id, span);
                    out.push(Stmt::Give(id));
                }),
                None => self.err_expr(span),
            },
            (
                Ty::List(_) | Ty::Str | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..),
                "take",
            ) if args.is_empty() => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Take, Vec::new(), ty.clone()),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::Map(..), "pop_first" | "pop_last") => {
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => elem.clone(),
                };
                let op = if method == "pop_first" {
                    Mutation::PopFirst
                } else {
                    Mutation::PopLast
                };
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, op, Vec::new(), Ty::opt(found)),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "pop") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    if matches!(elem, Ty::Opt(_)) {
                        mutate(self, Mutation::PopVal, Vec::new(), elem)
                    } else {
                        mutate(self, Mutation::Pop, Vec::new(), Ty::opt(elem))
                    }
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "swap") => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let idx_ty = match &ty {
                            Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Enum(e) => Ty::Enum(e),
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                            },
                            _ => Ty::Int,
                        };
                        let a = self.expect(exprs[0], &idx_ty);
                        let b = self.expect(exprs[1], &idx_ty);
                        let a = if matches!(a.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, a, Ty::Int)
                        } else {
                            a
                        };
                        let b = if matches!(b.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, b, Ty::Int)
                        } else {
                            b
                        };
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "resize") => match self.positional_any(method, args, 2, span) {
                Some(exprs) => {
                    let n = self.expect(exprs[0], &Ty::Int);
                    let v = self.expect(exprs[1], &elem);
                    let place = self.place(base);
                    let v = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(v, &b, exprs[1].span),
                        None => v,
                    };
                    self.mk(
                        ExprKind::Mutate(Mutation::Resize, Box::new(place), vec![n, v]),
                        Ty::Unit,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Str, "truncate") => match self.one_arg(method, args, &Ty::Int, span)
            {
                Some(n) => mutate(self, Mutation::Truncate, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "reserve") => match self.one_arg(method, args, &Ty::Int, span) {
                Some(n) => mutate(self, Mutation::Reserve, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "extend") => match self.positional_any(method, args, 1, span) {
                Some(exprs) => {
                    let other = self.expect(exprs[0], &ty);
                    mutate(self, Mutation::Extend, vec![other], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "fill") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(v) => {
                        let place = self.place(base);
                        let v = match self
                            .place_range(&place)
                            .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                        {
                            Some(b) => self.check_range(v, &b, span),
                            None => v,
                        };
                        self.mk(
                            ExprKind::Mutate(Mutation::Fill, Box::new(place), vec![v]),
                            Ty::Unit,
                            span,
                        )
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Str | Ty::StrView, "to_str") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::ToStr, vec![value]),
                    Ty::Str,
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "to_list") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![value]),
                        out_ty,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::StrView, "copy" | "clone") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::ToStr, vec![value]),
                    Ty::Str,
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "copy" | "clone") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![value]),
                        out_ty,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (
                Ty::List(_)
                | Ty::Array { .. }
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Str,
                "copy" | "clone",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::Copy, vec![value]),
                    ty.clone(),
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::HashSet(_), "insert" | "add") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => mutate(self, Mutation::Insert, vec![arg], Ty::Bool),
                    None => self.err_expr(span),
                }
            }
            // `m.remove(k)` gives the value that was there, or `none`.
            (Ty::Map(_, val) | Ty::HashMap(_, val), "remove") => {
                let found_ty = Ty::Opt(val.clone());
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&found_ty), span, |lw, out| {
                        let place = lw.place(base);
                        let place = lw.settle_indexes(place, out);
                        let key = lw.temp(arg.ty.clone());
                        out.push(Stmt::Let(key, Some(arg)));
                        let map = super::stmt::place_to_expr(&place);
                        let key_expr = lw.local(key, span);
                        let found = lw.mk(
                            ExprKind::Builtin(Builtin::MapGet, vec![map, key_expr.clone()]),
                            found_ty.clone(),
                            span,
                        );
                        let held = lw.temp(found_ty.clone());
                        out.push(Stmt::Let(held, Some(found)));
                        let remove = lw.mk(
                            ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![key_expr]),
                            Ty::Bool,
                            span,
                        );
                        // Whether it was there is already known.
                        out.push(Stmt::If(remove, Block::default(), Block::default()));
                        let held = lw.local(held, span);
                        out.push(Stmt::Give(held));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "remove") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => mutate(self, Mutation::Remove, vec![arg], Ty::Bool),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "set") => {
                let mut key_ast = None;
                let mut on_ast = None;
                let mut ok = args.len() == 2;
                for a in args {
                    match a.name.as_ref().map(|n| n.text.as_str()) {
                        None if key_ast.is_none() => key_ast = Some(&a.value),
                        Some("on") | None if on_ast.is_none() => on_ast = Some(&a.value),
                        _ => ok = false,
                    }
                }
                match (ok, key_ast, on_ast) {
                    (true, Some(k_ast), Some(b_ast)) => {
                        let k_val = self.expect(k_ast, &elem);
                        let b_val = self.expect(b_ast, &Ty::Bool);
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let place = lw.settle_indexes(place, out);
                            let k_slot = lw.temp(k_val.ty.clone());
                            out.push(Stmt::Let(k_slot, Some(k_val)));
                            let k_ref = lw.local(k_slot, span);
                            let ins = lw.mk(
                                ExprKind::Mutate(
                                    Mutation::Insert,
                                    Box::new(place.clone()),
                                    vec![k_ref.clone()],
                                ),
                                Ty::Bool,
                                span,
                            );
                            let rem = lw.mk(
                                ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![k_ref]),
                                Ty::Bool,
                                span,
                            );
                            out.push(Stmt::If(
                                b_val,
                                Block {
                                    stmts: vec![Stmt::Expr(ins)],
                                },
                                Block {
                                    stmts: vec![Stmt::Expr(rem)],
                                },
                            ));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    _ => self.err_expr(span),
                }
            }
            (Ty::Set(_), "drain") => {
                let mut after_ast = None;
                let mut before_ast = None;
                let mut while_ast = None;
                let mut ok = true;
                for a in args {
                    match a.name.as_ref().map(|n| n.text.as_str()) {
                        Some("after") if after_ast.is_none() => after_ast = Some(&a.value),
                        Some("before") if before_ast.is_none() => before_ast = Some(&a.value),
                        Some("while") if while_ast.is_none() => while_ast = Some(&a.value),
                        Some(other) => {
                            self.error(
                                codes::UNKNOWN_ARG_NAME,
                                a.name.as_ref().unwrap().span,
                                format!("`drain` has no parameter named `{other}`"),
                            );
                            ok = false;
                        }
                        None => ok = false,
                    }
                }
                let Some(while_ast) = while_ast.filter(|_| ok) else {
                    return self.err_expr(span);
                };
                let opt_elem = Ty::opt(elem.clone());
                let after_val = after_ast.map(|a| self.expect(a, &opt_elem));
                let before_val = before_ast.map(|b| self.expect(b, &opt_elem));
                let pred_ty = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
                let pred_val = self.expect(while_ast, &pred_ty);
                let out_ty = Ty::list(elem.clone());
                self.do_expr(Some(&out_ty), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let pred_slot = lw.temp(pred_ty.clone());
                    out.push(Stmt::Let(pred_slot, Some(pred_val)));
                    let out_slot = lw.temp(out_ty.clone());
                    out.push(Stmt::Let(
                        out_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), out_ty.clone(), span)),
                    ));
                    let after_slot = after_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push(Stmt::Let(s, Some(v)));
                        s
                    });
                    let before_slot = before_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push(Stmt::Let(s, Some(v)));
                        s
                    });
                    if let (Some(a_s), Some(b_s)) = (after_slot, before_slot) {
                        let both = lw.mk(
                            ExprKind::Binary(
                                BinOp::And,
                                Box::new(lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(a_s, span))),
                                    Ty::Bool,
                                    span,
                                )),
                                Box::new(lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(b_s, span))),
                                    Ty::Bool,
                                    span,
                                )),
                            ),
                            Ty::Bool,
                            span,
                        );
                        let cond =
                            lw.mk(ExprKind::Unary(UnOp::Not, Box::new(both)), Ty::Bool, span);
                        out.push(Stmt::Assert {
                            cond,
                            message: Some(lw.mk(
                                ExprKind::Str(
                                    "drain takes either after: or before:, not both".into(),
                                ),
                                Ty::Str,
                                span,
                            )),
                            text: "not (after != None and before != None)".into(),
                            line: lw.line(span),
                            span,
                            kept: false,
                        });
                    }
                    let build_loop = |lw: &mut Lowerer<'_>, next_opt: Expr| -> Stmt {
                        let lid = lw.next_loop;
                        lw.next_loop += 1;
                        let opt_slot = lw.temp(opt_elem.clone());
                        let k_slot = lw.temp(elem.clone());
                        let is_some = lw.mk(
                            ExprKind::IsSome(Box::new(lw.local(opt_slot, span))),
                            Ty::Bool,
                            span,
                        );
                        let not_some = lw.mk(
                            ExprKind::Unary(UnOp::Not, Box::new(is_some)),
                            Ty::Bool,
                            span,
                        );
                        let unwrapped = lw.mk(
                            ExprKind::Unwrap(Box::new(lw.local(opt_slot, span)), false),
                            elem.clone(),
                            span,
                        );
                        let ok_call = lw.mk(
                            ExprKind::CallExpr(
                                Box::new(lw.local(pred_slot, span)),
                                vec![Arg::Value(lw.local(k_slot, span))],
                            ),
                            Ty::Bool,
                            span,
                        );
                        let not_ok = lw.mk(
                            ExprKind::Unary(UnOp::Not, Box::new(ok_call)),
                            Ty::Bool,
                            span,
                        );
                        let rem = lw.mk(
                            ExprKind::Mutate(
                                Mutation::Remove,
                                Box::new(place.clone()),
                                vec![lw.local(k_slot, span)],
                            ),
                            Ty::Bool,
                            span,
                        );
                        let out_place = Place {
                            kind: PlaceKind::Local(out_slot),
                            ty: out_ty.clone(),
                            span,
                        };
                        let push = lw.mk(
                            ExprKind::Mutate(
                                Mutation::Push,
                                Box::new(out_place),
                                vec![lw.local(k_slot, span)],
                            ),
                            Ty::Unit,
                            span,
                        );
                        let body = vec![
                            Stmt::Let(opt_slot, Some(next_opt)),
                            Stmt::If(
                                not_some,
                                Block {
                                    stmts: vec![Stmt::Break(Some(lid))],
                                },
                                Block { stmts: Vec::new() },
                            ),
                            Stmt::Let(k_slot, Some(unwrapped)),
                            Stmt::If(
                                not_ok,
                                Block {
                                    stmts: vec![Stmt::Break(Some(lid))],
                                },
                                Block { stmts: Vec::new() },
                            ),
                            Stmt::Expr(rem),
                            Stmt::Expr(push),
                        ];
                        Stmt::While(
                            lid,
                            lw.mk(ExprKind::Bool(true), Ty::Bool, span),
                            Block { stmts: body },
                        )
                    };
                    let set_read = super::stmt::place_to_expr(&place);
                    let first_loop = build_loop(
                        lw,
                        lw.mk(
                            ExprKind::Builtin(Builtin::SetMin, vec![set_read.clone()]),
                            opt_elem.clone(),
                            span,
                        ),
                    );
                    let after_branch = match after_slot {
                        Some(a_s) => {
                            let a_val = lw.mk(
                                ExprKind::Unwrap(Box::new(lw.local(a_s, span)), false),
                                elem.clone(),
                                span,
                            );
                            let after_loop = build_loop(
                                lw,
                                lw.mk(
                                    ExprKind::Builtin(
                                        Builtin::FirstGt,
                                        vec![set_read.clone(), a_val],
                                    ),
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(a_s, span))),
                                    Ty::Bool,
                                    span,
                                ),
                                Block {
                                    stmts: vec![after_loop],
                                },
                                Block {
                                    stmts: vec![first_loop],
                                },
                            )
                        }
                        None => first_loop,
                    };
                    let top_stmt = match before_slot {
                        Some(b_s) => {
                            let b_val = lw.mk(
                                ExprKind::Unwrap(Box::new(lw.local(b_s, span)), false),
                                elem.clone(),
                                span,
                            );
                            let before_loop = build_loop(
                                lw,
                                lw.mk(
                                    ExprKind::Builtin(Builtin::LastLt, vec![set_read, b_val]),
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(b_s, span))),
                                    Ty::Bool,
                                    span,
                                ),
                                Block {
                                    stmts: vec![before_loop],
                                },
                                Block {
                                    stmts: vec![after_branch],
                                },
                            )
                        }
                        None => after_branch,
                    };
                    out.push(top_stmt);
                    out.push(Stmt::Give(lw.local(out_slot, span)));
                })
            }
            (
                Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Str,
                "clear",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "reverse") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, Mutation::Reverse, Vec::new(), Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "sort") => {
                self.sort(base, &elem, args, span)
            }
            (ty, "sorted") if ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_)) => {
                if args.is_empty() {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, args, span)
                }
            }
            (Ty::Str | Ty::StrView, "reversed") => match self.positional_any(method, args, 0, span)
            {
                Some(_) => {
                    let rev = self.mk(
                        ExprKind::Builtin(Builtin::Reversed, vec![value]),
                        Ty::StrView,
                        span,
                    );
                    self.mk(ExprKind::Builtin(Builtin::ToStr, vec![rev]), Ty::Str, span)
                }
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "reversed") => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone(), Dim::Int);
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(
                            ExprKind::Builtin(Builtin::Reversed, vec![value]),
                            view_ty,
                            span,
                        );
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Array { dims, elem: e }, "reversed") if dims.len() == 1 => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone(), Dim::Int);
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(
                            ExprKind::Builtin(Builtin::Reversed, vec![value]),
                            view_ty,
                            span,
                        );
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView
                | Ty::Range(_),
                "contains",
            ) => match self.one_arg(method, args, &elem, span) {
                Some(arg) => self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![value, arg]),
                    Ty::Bool,
                    span,
                ),
                None => self.err_expr(span),
            },
            // R04b.1.6: `z.rekey(s.keys)` hands a table computed on offsets back
            // in the caller's keys, O(1).
            (Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(_), "rekey")
            | (Ty::Array { .. }, "rekey")
                if args.len() == 1 && ty.seq_key().is_some() =>
            {
                let Some(exprs) = self.positional_any(method, args, 1, span) else {
                    return self.err_expr(span);
                };
                let keys = self.expr(exprs[0], None);
                let dim = match &keys.ty {
                    Ty::Range(k) => match **k {
                        Ty::Pos(d) | Ty::Bound(d) => Dim::Pos(d),
                        _ => Dim::Int,
                    },
                    _ => return self.err_expr(span),
                };
                // A view must not outlive its cells, so a temporary gets a slot.
                let value = if matches!(value.kind, ExprKind::Local(_)) {
                    value
                } else {
                    let slot = self.new_slot("cells", value.ty.clone());
                    self.cx.pending.push(Stmt::Let(slot, Some(value)));
                    self.local(slot, span)
                };
                let res = if matches!(ty, Ty::List(_) | Ty::Array { .. }) {
                    Ty::Keyed(Box::new(elem.clone()), dim)
                } else {
                    Ty::ListView(Box::new(elem.clone()), dim)
                };
                self.do_expr(Some(&res.clone()), span, |lw, out| {
                    let keys = lw.hold(keys, out);
                    let lo = lw.mk(
                        ExprKind::Builtin(Builtin::RangeLo, vec![keys.clone()]),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.mk(
                        ExprKind::Builtin(Builtin::RangeHi, vec![keys]),
                        Ty::Int,
                        span,
                    );
                    let view = lw.mk(
                        ExprKind::Builtin(Builtin::Rekey, vec![value, lo, hi]),
                        res,
                        span,
                    );
                    out.push(Stmt::Give(view));
                })
            }
            // How many cells equal `x`: `[1, 2, 1].count(1)` is 2,
            // `s[0..k].count('R')` scans the view directly without copying.
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Array { .. },
                "count",
            )
            | (Ty::Str | Ty::StrView, "count")
                if args.len() == 1
                    && (matches!(
                        &ty,
                        Ty::List(_)
                            | Ty::ListView(_, _)
                            | Ty::Keyed(_, _)
                            | Ty::MutView(_)
                            | Ty::Array { .. }
                    ) || self.probe_ty_silent(&args[0].value) == Some(Ty::Char)) =>
            {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                        let (seq, arg) = (lw.hold(value, out), lw.hold(arg, out));
                        let (acc, each) = (lw.temp(Ty::Int), lw.temp(elem.clone()));
                        out.push(Stmt::Let(acc, Some(lw.int_lit(0, span))));
                        let same = lw.mk(
                            ExprKind::Cmp(CmpOp::Eq, Box::new(lw.local(each, span)), Box::new(arg)),
                            Ty::Bool,
                            span,
                        );
                        let place = Place {
                            kind: PlaceKind::Local(acc),
                            ty: Ty::Int,
                            span: lw.span(span),
                        };
                        let inc =
                            Stmt::Assign(place, AssignOp::Binary(BinOp::Add), lw.int_lit(1, span));
                        let body = Block {
                            stmts: vec![Stmt::If(
                                same,
                                Block { stmts: vec![inc] },
                                Block::default(),
                            )],
                        };
                        let lid = lw.fresh_loop();
                        out.push(Stmt::For(lid, each, Iter::Seq(seq, false), body));
                        out.push(Stmt::Give(lw.local(acc, span)));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Map(_, val) | Ty::HashMap(_, val), "get") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.mk(
                        ExprKind::Builtin(Builtin::MapGet, vec![value, arg]),
                        Ty::Opt(val.clone()),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(key) | Ty::Map(key, _), "first_ge" | "first_gt" | "last_le" | "last_lt") => {
                let op = match method {
                    "first_ge" => Builtin::FirstGe,
                    "first_gt" => Builtin::FirstGt,
                    "last_le" => Builtin::LastLe,
                    _ => Builtin::LastLt,
                };
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => (**key).clone(),
                };
                match self.one_arg(method, args, key, span) {
                    Some(bound) => self.mk(
                        ExprKind::Builtin(op, vec![value, bound]),
                        Ty::opt(found),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_), "min" | "max") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    let op = if method == "min" {
                        Builtin::SetMin
                    } else {
                        Builtin::SetMax
                    };
                    self.mk(ExprKind::Builtin(op, vec![value]), Ty::opt(elem), span)
                }
                None => self.err_expr(span),
            },
            (Ty::DomainVal | Ty::Pos(_), "nth" | "at" | "from_idx") => {
                let Some(idx_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => match value.ty {
                        Ty::Pos(d) => Some(d),
                        _ => self.gen_env.values().find_map(|ga| match ga {
                            GenArg::Domain(d) => Some(*d),
                            _ => None,
                        }),
                    },
                };
                match dom_id {
                    Some(d) if method == "at" => self.domain_try_label(d, idx_expr, span),
                    Some(d) => self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(idx_expr)),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::DomainVal, "windows") => {
                let Some(k_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => None,
                };
                let range_ty = match dom_id {
                    Some(d) => Ty::Range(Box::new(Ty::Pos(d))),
                    None => Ty::Range(Box::new(Ty::Int)),
                };
                let list_ty = Ty::list(range_ty.clone());
                let list_ty_hint = list_ty.clone();
                self.do_expr(Some(&list_ty_hint), span, |lw, out| {
                    let k_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k_slot, Some(k_expr)));
                    let k_ref = lw.local(k_slot, span);
                    let n_expr = match dom_id {
                        Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                        None => lw.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span),
                    };
                    let n_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(n_slot, Some(n_expr)));
                    let n_ref = lw.local(n_slot, span);
                    let acc_slot = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(
                        acc_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
                    ));
                    let lim = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(n_ref), Box::new(k_ref.clone())),
                        Ty::Int,
                        span,
                    );
                    let hi_loop = lw.int_add(lim, 1);
                    let i_slot = lw.temp(Ty::Int);
                    let i_ref = lw.local(i_slot, span);
                    let end_e = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(i_ref.clone()), Box::new(k_ref)),
                        Ty::Int,
                        span,
                    );
                    let rng = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(i_ref),
                            hi: Box::new(end_e),
                        },
                        range_ty,
                        span,
                    );
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc_slot),
                                ty: list_ty.clone(),
                                span,
                            }),
                            vec![rng],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i_slot,
                        Iter::Range {
                            lo: lw.int_lit(0, span),
                            hi: hi_loop,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                })
            }
            // `xs.find(x => x > 3)` is the first element the test holds
            // for; `xs.find(3)` is where the value 3 is. Handled uniformly across
            // every `Indexed` sequence (`List`, `ListView`, 1D `[D]T`, `Str`, `StrView`, `Range`).
            (ty, "find" | "rfind")
                if ty.is_indexed() && args.len() == 1 && self.is_test(&args[0].value) =>
            {
                let elem = ty.seq_elem().unwrap();
                self.find_first(value, elem, &args[0].value, method == "rfind", span)
            }
            (ty, "find" | "rfind")
                if matches!(ty, Ty::Str | Ty::StrView)
                    && args.len() == 1
                    && args[0].name.is_none()
                    && matches!(
                        self.probe_ty_silent(&args[0].value),
                        Some(Ty::Str | Ty::StrView)
                    ) =>
            {
                let pat = self.expr(&args[0].value, None);
                let rev = method == "rfind";
                let out_ty = Ty::opt(Ty::Int);
                let out_ty_hint = out_ty.clone();
                self.do_expr(Some(&out_ty_hint), span, |lw, out| {
                    let s_slot = lw.temp(value.ty.clone());
                    out.push(Stmt::Let(s_slot, Some(value)));
                    let s_ref = lw.local(s_slot, span);
                    let p_slot = lw.temp(pat.ty.clone());
                    out.push(Stmt::Let(p_slot, Some(pat)));
                    let p_ref = lw.local(p_slot, span);
                    let n_slot = lw.temp(Ty::Int);
                    let s_len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![s_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(n_slot, Some(s_len)));
                    let m_slot = lw.temp(Ty::Int);
                    let p_len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![p_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(m_slot, Some(p_len)));
                    let res_slot = lw.temp(out_ty.clone());
                    out.push(Stmt::Let(
                        res_slot,
                        Some(lw.mk(ExprKind::None, out_ty.clone(), span)),
                    ));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(n_slot, span)),
                            Box::new(lw.local(m_slot, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.int_add(diff, 1);
                    let i_slot = lw.temp(Ty::Int);
                    let i_ref = lw.local(i_slot, span);
                    let ok_slot = lw.temp(Ty::Bool);
                    let j_slot = lw.temp(Ty::Int);
                    let j_ref = lw.local(j_slot, span);
                    let ij = lw.mk(
                        ExprKind::Binary(
                            BinOp::Add,
                            Box::new(i_ref.clone()),
                            Box::new(j_ref.clone()),
                        ),
                        Ty::Int,
                        span,
                    );
                    let sc = lw.mk(ExprKind::Index(Box::new(s_ref), vec![ij]), Ty::Char, span);
                    let pc = lw.mk(
                        ExprKind::Index(Box::new(p_ref), vec![j_ref]),
                        Ty::Char,
                        span,
                    );
                    let ne = lw.mk(
                        ExprKind::Cmp(CmpOp::Ne, Box::new(sc), Box::new(pc)),
                        Ty::Bool,
                        span,
                    );
                    let inner_lid = lw.next_loop;
                    lw.next_loop += 1;
                    let outer_lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        outer_lid,
                        i_slot,
                        Iter::Range {
                            lo: lw.int_lit(0, span),
                            hi,
                            step: None,
                            rev,
                            closed: false,
                        },
                        Block {
                            stmts: vec![
                                Stmt::Let(
                                    ok_slot,
                                    Some(lw.mk(ExprKind::Bool(true), Ty::Bool, span)),
                                ),
                                Stmt::For(
                                    inner_lid,
                                    j_slot,
                                    Iter::Range {
                                        lo: lw.int_lit(0, span),
                                        hi: lw.local(m_slot, span),
                                        step: None,
                                        rev: false,
                                        closed: false,
                                    },
                                    Block {
                                        stmts: vec![Stmt::If(
                                            ne,
                                            Block {
                                                stmts: vec![
                                                    Stmt::Assign(
                                                        Place {
                                                            kind: PlaceKind::Local(ok_slot),
                                                            ty: Ty::Bool,
                                                            span,
                                                        },
                                                        AssignOp::Set,
                                                        lw.mk(
                                                            ExprKind::Bool(false),
                                                            Ty::Bool,
                                                            span,
                                                        ),
                                                    ),
                                                    Stmt::Break(None),
                                                ],
                                            },
                                            Block::default(),
                                        )],
                                    },
                                ),
                                Stmt::If(
                                    lw.local(ok_slot, span),
                                    Block {
                                        stmts: vec![
                                            Stmt::Assign(
                                                Place {
                                                    kind: PlaceKind::Local(res_slot),
                                                    ty: out_ty.clone(),
                                                    span,
                                                },
                                                AssignOp::Set,
                                                lw.mk(
                                                    ExprKind::Some(Box::new(i_ref)),
                                                    out_ty.clone(),
                                                    span,
                                                ),
                                            ),
                                            Stmt::Break(None),
                                        ],
                                    },
                                    Block::default(),
                                ),
                            ],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(res_slot, span)));
                })
            }
            (ty, "first_ge" | "first_gt" | "last_le" | "last_lt" | "find" | "rfind")
                if ty.is_indexed() =>
            {
                let dom = match &ty {
                    Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let elem = ty.seq_elem().unwrap();
                self.seq_search_method(method, value, elem, dom, args, span)
            }
            (Ty::Fn(_, _), "out") => self.call_fn_val(value, args, span),
            // `Neighbours` for a table of neighbour lists: `adj.out(u)` is
            // `adj[u]`, as it is `adj(u)` for a fn.
            (Ty::Array { dims, elem }, "out")
                if dims.len() == 1 && args.len() == 1 && matches!(**elem, Ty::List(_)) =>
            {
                let at = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(base.clone()),
                        indices: vec![args[0].value.clone()],
                    },
                    span,
                };
                self.expr(&at, None)
            }
            // `r.nth(k)` on a range of positions is `r[k]`, checked against `r`.
            (Ty::Range(inner), "nth") if matches!(**inner, Ty::Pos(_)) && args.len() == 1 => {
                let at = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(base.clone()),
                        indices: vec![args[0].value.clone()],
                    },
                    span,
                };
                self.expr(&at, None)
            }
            // `r.nth(k)` on an Int range: its `k`-th value, `r.start + k` (R04b.5.9).
            (Ty::Range(inner), "nth") if **inner == Ty::Int && args.len() == 1 => {
                let Some(k) = self.one_arg("nth", args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let lo = self.mk(
                    ExprKind::Builtin(Builtin::RangeLo, vec![value]),
                    Ty::Int,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo), Box::new(k)),
                    Ty::Int,
                    span,
                )
            }
            // `(lo..hi).clamp(D)`: the part of an Int range that lies in
            // `D`, as a range of D's positions; empty when none does.
            (Ty::Range(inner), "clamp") if **inner == Ty::Int && args.len() == 1 => {
                let domain = match &args[0].value.kind {
                    ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                        match (self.gen_env.get(name.as_str()), self.get_sym(name)) {
                            (Some(GenArg::Domain(id)), _) => Some(*id),
                            (_, Some(Sym::Domain(id))) => Some(id),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                let Some(d) = domain else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        args[0].value.span,
                        "`clamp` takes a domain",
                    );
                    self.help("write the domain the range is cut to: `(l..=r).clamp(V)`");
                    return self.err_expr(span);
                };
                let range_ty = Ty::Range(Box::new(Ty::Pos(d)));
                self.do_expr(Some(&range_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    // An end of the range as an ordinal of `D`, kept in 0..=len.
                    let end = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        let at = lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span);
                        let first = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                        let ord = lw.mk(
                            ExprKind::Binary(BinOp::Sub, Box::new(at), Box::new(first)),
                            Ty::Int,
                            span,
                        );
                        let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let zero = lw.int_lit(0, span);
                        let capped = lw.mk(
                            ExprKind::Builtin(Builtin::Min, vec![ord, len]),
                            Ty::Int,
                            span,
                        );
                        lw.mk(
                            ExprKind::Builtin(Builtin::Max, vec![capped, zero]),
                            Ty::Int,
                            span,
                        )
                    };
                    let lo = lw.new_slot("lo", Ty::Int);
                    let from = end(lw, Builtin::RangeLo);
                    out.push(Stmt::Let(lo, Some(from)));
                    let to = end(lw, Builtin::RangeHi);
                    // An empty range stays empty: its end is not before its start.
                    let lo_again = lw.local(lo, span);
                    let hi = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![to, lo_again]),
                        Ty::Int,
                        span,
                    );
                    let lo = lw.local(lo, span);
                    let cut = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(cut));
                })
            }
            // `r.drop(k)` / `r.take(k)` on `Range<Int>` or `Range<D>`: sub-range.
            (Ty::Range(inner), "drop" | "take" | "drop_last" | "take_last")
                if **inner != Ty::I128 =>
            {
                // The new end is `shift` in from `lo` or from `hi`, and replaces `lo` or `hi`.
                let (from_hi, cut_lo) = match method {
                    "drop" => (false, true),
                    "take" => (false, false),
                    "drop_last" => (true, false),
                    _ => (true, true),
                };
                let range_ty = value.ty.clone();
                let Some(k) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                self.do_expr(Some(&range_ty), span, |lw, out| {
                    let range = lw.new_slot("range", range_ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let lo_s = lw.new_slot("lo", Ty::Int);
                    let hi_s = lw.new_slot("hi", Ty::Int);
                    let r1 = lw.local(range, span);
                    let r2 = lw.local(range, span);
                    out.push(Stmt::Let(
                        lo_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeLo, vec![r1]), Ty::Int, span)),
                    ));
                    out.push(Stmt::Let(
                        hi_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeHi, vec![r2]), Ty::Int, span)),
                    ));
                    let k_s = lw.new_slot("k", Ty::Int);
                    let zero = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let k_clamped = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![k, zero]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(k_s, Some(k_clamped)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_s, span)),
                            Box::new(lw.local(lo_s, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let zero2 = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let avail = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![diff, zero2]),
                        Ty::Int,
                        span,
                    );
                    let shift = lw.mk(
                        ExprKind::Builtin(Builtin::Min, vec![avail, lw.local(k_s, span)]),
                        Ty::Int,
                        span,
                    );
                    let (end, op) = if from_hi {
                        (hi_s, BinOp::Sub)
                    } else {
                        (lo_s, BinOp::Add)
                    };
                    let point = lw.mk(
                        ExprKind::Binary(op, Box::new(lw.local(end, span)), Box::new(shift)),
                        Ty::Int,
                        span,
                    );
                    let (new_lo, new_hi) = if cut_lo {
                        (point, lw.local(hi_s, span))
                    } else {
                        (lw.local(lo_s, span), point)
                    };
                    let res = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(new_lo),
                            hi: Box::new(new_hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            // `(lo..hi).step(k)`: lo, lo + k, ... below hi, as a list.
            (Ty::Range(inner), "step") if **inner == Ty::Int && args.len() == 1 => {
                let by = self.expr(&args[0].value, Some(&Ty::Int));
                let by = self.coerce(by, &Ty::Int, span);
                let list_ty = Ty::list(Ty::Int);
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let acc = lw.new_slot("steps", list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push(Stmt::Let(acc, Some(empty)));
                    let i = lw.new_slot("i", Ty::Int);
                    let bound = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span)
                    };
                    let (lo, hi) = (bound(lw, Builtin::RangeLo), bound(lw, Builtin::RangeHi));
                    let item = lw.local(i, span);
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![item],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i,
                        Iter::Range {
                            lo,
                            hi,
                            step: Some(by),
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            // `xs.last(k)`: the element `k` back from the last one, so
            // `xs.last(1)` is the one before the last; `none` when there
            // are not that many.
            (ty, "last") if ty.is_indexed() && args.len() == 1 => {
                let elem = ty.seq_elem().unwrap();
                // An array of positions is read at the position that far in.
                let of = match &ty {
                    Ty::Array { dims, .. } => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let back = self.expect(&args[0].value, &Ty::Int);
                self.do_expr(Some(&Ty::opt(elem.clone())), span, |lw, out| {
                    // A list that is named is read where it is; one that is
                    // computed is kept for the two reads.
                    let list = if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        value
                    } else {
                        let kept = lw.temp(value.ty.clone());
                        out.push(Stmt::Let(kept, Some(value)));
                        lw.local(kept, span)
                    };
                    let k = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k, Some(back)));
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![list.clone()]),
                        Ty::Int,
                        span,
                    );
                    let last = lw.int_add(len, -1);
                    let at = lw.temp(Ty::Int);
                    let from_end = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(last), Box::new(lw.local(k, span))),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(at, Some(from_end)));
                    let cmp = |lw: &mut Self, slot| {
                        let zero = lw.int_lit(0, span);
                        lw.mk(
                            ExprKind::Cmp(
                                CmpOp::Ge,
                                Box::new(lw.local(slot, span)),
                                Box::new(zero),
                            ),
                            Ty::Bool,
                            span,
                        )
                    };
                    let (ahead, inside) = (cmp(lw, k), cmp(lw, at));
                    let there = lw.mk(
                        ExprKind::Binary(BinOp::And, Box::new(ahead), Box::new(inside)),
                        Ty::Bool,
                        span,
                    );
                    let cell = match of {
                        Some(d) => lw.mk(
                            ExprKind::Cast(Cast::FromOrd(d), Box::new(lw.local(at, span))),
                            Ty::Pos(d),
                            span,
                        ),
                        None => lw.local(at, span),
                    };
                    let item = lw.mk(
                        ExprKind::Index(Box::new(list), vec![cell]),
                        elem.clone(),
                        span,
                    );
                    let opt = Ty::opt(elem);
                    let some = lw.mk(ExprKind::Some(Box::new(item)), opt.clone(), span);
                    let none = lw.mk(ExprKind::None, opt.clone(), span);
                    out.push(Stmt::Give(lw.mk(
                        ExprKind::If(Box::new(there), Box::new(some), Box::new(none)),
                        opt,
                        span,
                    )));
                })
            }
            (Ty::Str | Ty::StrView, "rev") if args.is_empty() => {
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::StrView,
                    span,
                )
            }
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "rev")
                if args.is_empty() =>
            {
                let elem = (**e).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem), Dim::Int),
                    span,
                )
            }
            // A range answers its own `count()` from its ends, O(1) (R07.9.6).
            (Ty::Range(_), "count") if args.is_empty() => self.len_of(value, span),
            (Ty::Range(_), "rev") if args.is_empty() => self.reversed_range(value, span),
            // `r.halves()`: split `r` into `(left, right)` with `left.len == (r.len + 1) / 2`
            // and `right.len == r.len / 2` (both non-empty whenever `r.len >= 2`).
            (Ty::Range(_), "halves") if args.is_empty() => {
                let r_ty = value.ty.clone();
                let tup_ty = Ty::Tuple(vec![r_ty.clone(), r_ty.clone()]);
                self.do_expr(Some(&tup_ty.clone()), span, |lw, out| {
                    let r_slot = lw.temp(r_ty.clone());
                    out.push(Stmt::Let(r_slot, Some(value)));
                    let r_ref = lw.local(r_slot, span);
                    let lo_slot = lw.temp(Ty::Int);
                    let hi_slot = lw.temp(Ty::Int);
                    let lo_expr = lw.mk(
                        ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    let raw_hi = lw.mk(
                        ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(lo_slot, Some(lo_expr)));
                    let hi_expr = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![raw_hi, lw.local(lo_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(hi_slot, Some(hi_expr)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_slot, span)),
                            Box::new(lw.local(lo_slot, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let left_len = lw.mk(
                        ExprKind::Binary(
                            BinOp::Div,
                            Box::new(lw.int_add(diff, 1)),
                            Box::new(lw.int_lit(2, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let mid_slot = lw.temp(Ty::Int);
                    let mid_expr = lw.mk(
                        ExprKind::Binary(
                            BinOp::Add,
                            Box::new(lw.local(lo_slot, span)),
                            Box::new(left_len),
                        ),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(mid_slot, Some(mid_expr)));
                    let left = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(lo_slot, span)),
                            hi: Box::new(lw.local(mid_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let right = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(mid_slot, span)),
                            hi: Box::new(lw.local(hi_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let pair = lw.mk(ExprKind::Tuple(vec![left, right]), tup_ty.clone(), span);
                    out.push(Stmt::Give(pair));
                })
            }
            // `r.find(i => ..)`: the first of the range the test holds
            // for, `none` when there is none. `(0..).find(..)` has no end
            // and stops there.
            (Ty::Range(elem), "find" | "rfind") if args.len() == 1 => {
                let elem = (**elem).clone();
                self.find_first(value, elem, &args[0].value, method == "rfind", span)
            }
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 && args.is_empty() => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem), Dim::Int),
                    span,
                )
            }
            // A sequence walked backwards is something to loop over. As a
            // value it would be the sequence unchanged, which is not what
            // `.rev()` says.
            (
                Ty::Array { .. } | Ty::Set(_) | Ty::Map(..) | Ty::HashSet(_) | Ty::HashMap(..),
                "rev",
            ) if args.is_empty() => {
                let message = format!(
                    "a reversed `{}` is something to loop over, not a value: \
                     `for x in xs.rev`, or `[x for x in xs.rev]` for the list",
                    self.show(&ty)
                );
                self.error(codes::UNSUPPORTED, name.span, message);
                self.err_expr(span)
            }
            (Ty::DomainVal, "mask") if args.len() == 1 && args[0].name.is_none() => {
                if let ExprKind::DomainVal(d) = value.kind {
                    let arr_ty = Ty::Array {
                        dims: vec![Dim::Pos(d)],
                        elem: Box::new(Ty::Bool),
                    };
                    let list = self.collect_iter_to_list(&args[0].value, span);
                    let elem_ty = match &list.ty {
                        Ty::List(e) => (**e).clone(),
                        _ => Ty::Pos(d),
                    };
                    self.do_expr(Some(&arr_ty), span, |lw, out| {
                        let m_slot = lw.temp(arr_ty.clone());
                        let false_val = lw.mk(ExprKind::Bool(false), Ty::Bool, span);
                        let fill = lw.mk(
                            ExprKind::Fill(vec![DimSpec::Pos(d)], Some(Box::new(false_val))),
                            arr_ty.clone(),
                            span,
                        );
                        out.push(Stmt::Let(m_slot, Some(fill)));
                        let x_slot = lw.temp(elem_ty.clone());
                        let x_val = lw.coerce(lw.local(x_slot, span), &Ty::Pos(d), span);
                        let place = Place {
                            kind: PlaceKind::Index(
                                Box::new(Place {
                                    kind: PlaceKind::Local(m_slot),
                                    ty: arr_ty.clone(),
                                    span,
                                }),
                                vec![x_val],
                            ),
                            ty: Ty::Bool,
                            span,
                        };
                        let lid = lw.next_loop;
                        lw.next_loop += 1;
                        out.push(Stmt::For(
                            lid,
                            x_slot,
                            Iter::Seq(list, false),
                            Block {
                                stmts: vec![Stmt::Assign(
                                    place,
                                    AssignOp::Set,
                                    lw.mk(ExprKind::Bool(true), Ty::Bool, span),
                                )],
                            },
                        ));
                        out.push(Stmt::Give(lw.local(m_slot, span)));
                    })
                } else {
                    self.err_expr(span)
                }
            }
            // `v.next(k)`, `v.prev(k)`: step k positions, `None` past an end (R04.8.1).
            (Ty::Pos(d), "next" | "prev") if args.len() == 1 => {
                let d = *d;
                let op = if method == "next" {
                    BinOp::Add
                } else {
                    BinOp::Sub
                };
                let Some(k) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let at = self.mk(
                    ExprKind::Binary(op, Box::new(ord), Box::new(k)),
                    Ty::Int,
                    span,
                );
                self.mk(
                    ExprKind::Builtin(Builtin::PosTryOrd(d), vec![at]),
                    Ty::opt(Ty::Pos(d)),
                    span,
                )
            }
            // `Reversible.rev`: a value that is not a sequence and has no
            // `rev` of its own is its own reverse.
            (_, "rev") if args.is_empty() => value,
            _ => {
                let prim = self
                    .prim_methods
                    .iter()
                    .find(|(t, m, _)| *t == ty && m == method)
                    .map(|p| p.2);
                if let Some(fid) = prim.or_else(|| self.optional_impl_method(&ty, method)) {
                    let mut all_args = vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: base.clone(),
                    }];
                    all_args.extend(args.iter().cloned());
                    return self.user_call(fid, None, &all_args, span);
                }
                // A domain is the sequence of its positions: `Cell.find(p)` is
                // `Cell.all.find(p)`.
                if ty == Ty::DomainVal && method != "all" {
                    let all = ast::Expr {
                        kind: ast::ExprKind::Field {
                            base: Box::new(base.clone()),
                            name: ast::Name {
                                text: "all".to_string(),
                                span: base.span,
                            },
                        },
                        span: base.span,
                    };
                    return self.method(whole, &all, name, args, hint);
                }
                if self.number_max_call(&ty, base, method, args, span) {
                    return self.err_expr(span);
                }
                // UFCS free function with `self` first parameter (e.g. `p.step(d)`, `p.ray(d)`).
                if let Some(res) = self.try_ufcs_call(method, base, args, span) {
                    return res;
                }
                let message = format!(
                    "{} has no method `{method}` that `cpl run` knows",
                    self.a_ty(&ty)
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                self.member_typo(&ty, method, name.span);
                self.foreign_call_fix(&ty, base, method, span);
                self.slice_call_fix(&ty, base, method, args, span);
                // Without the method's signature the arguments (and the
                // lambdas' parameters) have no types to check against.
                let reported = self.diags.len();
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.diags.truncate(reported);
                self.err_expr(span)
            }
        }
    }
}
