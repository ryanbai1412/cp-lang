//! A method on a value, chosen by the value's type: `xs.push(x)`,
//! `m.get(k)`, `s.split(c)`, after `method` has ruled out the type-level
//! and struct receivers.

use super::*;
use cpl_diag::msgs;

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
        recv_mark: usize,
    ) -> Expr {
        let span = whole.span;
        let method = name.text.as_str();
        let mutate = |lw: &mut Self, op: Mutation, args: Vec<Expr>, result: Ty| {
            let place = lw.mut_receiver(base, method);
            lw.mk_mutate(op, place, args, result, span)
        };
        // `(lo..hi).rev.find(p)` walks down from `hi` and stops at the first
        // hit, as `rfind` does, instead of listing the whole range first.
        if let (
            ast::ExprKind::Field {
                base: inner,
                name: rev,
            },
            "find" | "rfind",
            [test],
        ) = (&base.kind, method, args)
        {
            if rev.text == "rev"
                && test.name.is_none()
                && self.probe_is(inner, |t| matches!(t, Ty::Range(_)))
            {
                let range = self.expr(inner, None);
                return self.find_first(range, elem, &test.value, method == "find", span);
            }
        }
        if self.past_builtin_row(name.span) {
            if let Some(res) = self.try_ufcs_call(method, base, args, span, Some(recv_mark), hint) {
                return res;
            }
        }
        match (&ty, method) {
            // `s.set(v, on)` is `s.with(v)` when `on` holds and `s.without(v)`
            // when it does not.
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
                    Some(k) => {
                        self.mk_builtin(Builtin::BoundStepL(d), [value, k], Ty::Bound(d), span)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "r") => {
                let d = *d;
                match self.one_arg("r", args, &Ty::Int, span) {
                    Some(k) => {
                        self.mk_builtin(Builtin::BoundStepR(d), [value, k], Ty::Bound(d), span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "get") if let Some(elem) = ty.seq_elem() => {
                let opt = Ty::opt(elem);
                match self.one_arg("get", args, &Ty::Int, span) {
                    Some(at) => self.mk_builtin(Builtin::SeqGet, [value, at], opt, span),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if let Some(elem) = ty.seq_elem().filter(|_| Self::keyed_view(ty)) => {
                match self.one_arg("at", args, &Ty::Int, span) {
                    Some(at) => {
                        let view = self.hold_view(value, span);
                        self.key_index(view, at, elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "cut_after" | "cut_before") if Self::has_cuts(ty) => {
                self.cut_beside(value, ty.clone(), method, args, span)
            }
            (ty, "nth")
                if let Some(elem) = ty.seq_elem().filter(|_| {
                    matches!(
                        ty,
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::Str | Ty::StrView
                    )
                }) =>
            {
                match self.one_arg("nth", args, &Ty::Int, span) {
                    Some(at) => self.mk_index(value, vec![at], elem, span),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if let Some(elem) = ty.seq_elem() => {
                match self.one_arg("at", args, &ty.key_ty(), span) {
                    Some(at) => {
                        let indices = self.vouched(ty, vec![at]);
                        self.mk_index(value, indices, elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "set_at") if let Some(elem) = ty.seq_elem().filter(|_| ty.is_indexed_mut()) => {
                let ty_owned = ty.clone();
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let at = self.expr(exprs[0], Some(&Ty::Int));
                        let at = self.coerce(at, &Ty::Int, exprs[0].span);
                        let val = self.expr_as(exprs[1], &elem);
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
                            out.push_give(lw.mk(ExprKind::Unit, Ty::Unit, span));
                        })
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "swap_idx") if ty.is_indexed_mut() => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let a = self.expr_as(exprs[0], &Ty::Int);
                        let b = self.expr_as(exprs[1], &Ty::Int);
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => {
                    let place = self.mut_receiver(base, method);
                    let arg = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(arg, &b, span),
                        None => arg,
                    };
                    self.mk_mutate(Mutation::Push, place, vec![arg], Ty::Unit, span)
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
                    out.push_let(held, arg);
                    let id = lw.temp(Ty::Int);
                    let list = super::stmt::place_to_expr(&place);
                    let len = lw.mk_builtin(Builtin::Len, [list], Ty::Int, span);
                    out.push_let(id, len);
                    let arg = lw.local(held, span);
                    let push = lw.mk_mutate(Mutation::Push, place, vec![arg], Ty::Unit, span);
                    out.push(Stmt::Expr(push));
                    let id = lw.local(id, span);
                    out.push_give(id);
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
                None => self.foreign_method_args(whole, base, name, args, &ty),
            },
            // One block move of the tail, `vector::insert` / `erase`, after the
            // index is held to the list as the std contract states.
            (Ty::List(_), "insert" | "remove") => {
                let inserting = method == "insert";
                match self.positional_any(method, args, if inserting { 2 } else { 1 }, span) {
                    Some(exprs) => {
                        let at = self.expr(exprs[0], Some(&Ty::Int));
                        let at = self.coerce(at, &Ty::Int, exprs[0].span);
                        let val = inserting.then(|| {
                            let v = self.expr(exprs[1], Some(&elem));
                            self.coerce(v, &elem, exprs[1].span)
                        });
                        let result = if inserting { Ty::Unit } else { elem.clone() };
                        self.do_expr(Some(&result), span, |lw, out| {
                            let place = lw.place(base);
                            let place = lw.settle_indexes(place, out);
                            let i = lw.temp(Ty::Int);
                            out.push_let(i, at);
                            let mut margs = vec![lw.local(i, span)];
                            if let Some(val) = val {
                                let val = match lw
                                    .place_range(&place)
                                    .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                                {
                                    Some(b) => lw.check_range(val, &b, span),
                                    None => val,
                                };
                                let held = lw.temp(val.ty.clone());
                                out.push_let(held, val);
                                margs.push(lw.local(held, span));
                            }
                            let list = super::stmt::place_to_expr(&place);
                            let len = lw.mk_builtin(Builtin::Len, [list], Ty::Int, span);
                            let zero = lw.mk(ExprKind::Int(0), Ty::Int, span);
                            let low = lw.mk_cmp(CmpOp::Le, zero, lw.local(i, span), Ty::Bool, span);
                            let top = if inserting { CmpOp::Le } else { CmpOp::Lt };
                            let high = lw.mk_cmp(top, lw.local(i, span), len, Ty::Bool, span);
                            let cond = lw.mk_binary(BinOp::And, low, high, Ty::Bool, span);
                            let text = if inserting {
                                "0 <= i <= self.len"
                            } else {
                                "0 <= i < self.len"
                            };
                            out.push(Stmt::Assert {
                                cond,
                                message: None,
                                text: text.to_string(),
                                line: lw.line(span),
                                note: None,
                                span: lw.span(span),
                                kept: true,
                            });
                            let op = if inserting {
                                Mutation::Insert
                            } else {
                                Mutation::Remove
                            };
                            let m = lw.mk_mutate(op, place, margs, result.clone(), span);
                            if inserting {
                                out.push(Stmt::Expr(m));
                                out.push_give(lw.mk(ExprKind::Unit, Ty::Unit, span));
                            } else {
                                out.push_give(m);
                            }
                        })
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::MutView(..) | Ty::Array { .. } | Ty::Str, "swap") => {
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
                        let a = self.expr_as(exprs[0], &idx_ty);
                        let b = self.expr_as(exprs[1], &idx_ty);
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
                    let n = self.expr_as(exprs[0], &Ty::Int);
                    let v = self.expr_as(exprs[1], &elem);
                    let place = self.mut_receiver(base, method);
                    let v = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(v, &b, exprs[1].span),
                        None => v,
                    };
                    self.mk_mutate(Mutation::Resize, place, vec![n, v], Ty::Unit, span)
                }
                None => self.foreign_method_args(whole, base, name, args, &ty),
            },
            (Ty::List(_) | Ty::Str, "truncate") => match self.one_arg(method, args, &Ty::Int, span)
            {
                Some(n) => mutate(self, Mutation::Truncate, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Map(..) | Ty::HashMap(..), "reserve") => {
                match self.one_arg(method, args, &Ty::Int, span) {
                    Some(n) => mutate(self, Mutation::Reserve, vec![n], Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_), "extend") => match self.positional_any(method, args, 1, span) {
                Some(exprs) => {
                    let other = self.expr_as(exprs[0], &ty);
                    mutate(self, Mutation::Extend, vec![other], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(..) | Ty::Array { .. } | Ty::Str, "fill") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(v) => {
                        let place = self.mut_receiver(base, method);
                        let v = match self
                            .place_range(&place)
                            .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                        {
                            Some(b) => self.check_range(v, &b, span),
                            None => v,
                        };
                        self.mk_mutate(Mutation::Fill, place, vec![v], Ty::Unit, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Str | Ty::StrView, "to_str") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk_builtin(Builtin::ToStr, [value], Ty::Str, span),
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _), "to_list") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk_builtin(Builtin::ToList, [value], out_ty, span),
                    None => self.err_expr(span),
                }
            }
            (Ty::StrView, "copy" | "clone") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk_builtin(Builtin::ToStr, [value], Ty::Str, span),
                None => self.err_expr(span),
            },
            (Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _), "copy" | "clone") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk_builtin(Builtin::ToList, [value], out_ty, span),
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
                Some(_) => self.mk_builtin(Builtin::Copy, [value], ty.clone(), span),
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
                        out.push_let(key, arg);
                        let map = super::stmt::place_to_expr(&place);
                        let key_expr = lw.local(key, span);
                        let found = lw.mk_builtin(
                            Builtin::MapGet,
                            [map, key_expr.clone()],
                            found_ty.clone(),
                            span,
                        );
                        let held = lw.temp(found_ty.clone());
                        out.push_let(held, found);
                        let remove =
                            lw.mk_mutate(Mutation::Remove, place, vec![key_expr], Ty::Bool, span);
                        // Whether it was there is already known.
                        out.push(Stmt::If(remove, Block::default(), Block::default()));
                        let held = lw.local(held, span);
                        out.push_give(held);
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
                        let k_val = self.expr_as(k_ast, &elem);
                        let b_val = self.expr_as(b_ast, &Ty::Bool);
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let place = lw.settle_indexes(place, out);
                            let k_slot = lw.temp(k_val.ty.clone());
                            out.push_let(k_slot, k_val);
                            let k_ref = lw.local(k_slot, span);
                            let ins = lw.mk_mutate(
                                Mutation::Insert,
                                place.clone(),
                                vec![k_ref.clone()],
                                Ty::Bool,
                                span,
                            );
                            let rem =
                                lw.mk_mutate(Mutation::Remove, place, vec![k_ref], Ty::Bool, span);
                            out.push(Stmt::If(
                                b_val,
                                Block {
                                    stmts: vec![Stmt::Expr(ins)],
                                },
                                Block {
                                    stmts: vec![Stmt::Expr(rem)],
                                },
                            ));
                            out.push_give(lw.mk(ExprKind::Unit, Ty::Unit, span));
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
                    match a.name.as_ref().map(|n| (n.text.as_str(), n.span)) {
                        Some(("after", _)) if after_ast.is_none() => after_ast = Some(&a.value),
                        Some(("before", _)) if before_ast.is_none() => before_ast = Some(&a.value),
                        Some(("while", _)) if while_ast.is_none() => while_ast = Some(&a.value),
                        Some((other, at)) => {
                            self.report(
                                at,
                                msgs::ir::UNKNOWN_ARG_NAME_DRAIN_HAS_NO_PARAMETER,
                                &[&other],
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
                let after_val = after_ast.map(|a| self.expr_as(a, &opt_elem));
                let before_val = before_ast.map(|b| self.expr_as(b, &opt_elem));
                let pred_ty = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
                let pred_val = self.expr_as(while_ast, &pred_ty);
                let out_ty = Ty::list(elem.clone());
                self.do_expr(Some(&out_ty), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let pred_slot = lw.temp(pred_ty.clone());
                    out.push_let(pred_slot, pred_val);
                    let out_slot = lw.temp(out_ty.clone());
                    out.push_let(
                        out_slot,
                        lw.mk(ExprKind::Tuple(Vec::new()), out_ty.clone(), span),
                    );
                    let after_slot = after_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push_let(s, v);
                        s
                    });
                    let before_slot = before_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push_let(s, v);
                        s
                    });
                    if let (Some(a_s), Some(b_s)) = (after_slot, before_slot) {
                        let both = lw.mk_binary(
                            BinOp::And,
                            lw.mk_is_some(lw.local(a_s, span), Ty::Bool, span),
                            lw.mk_is_some(lw.local(b_s, span), Ty::Bool, span),
                            Ty::Bool,
                            span,
                        );
                        let cond = lw.mk_unary(UnOp::Not, both, Ty::Bool, span);
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
                            note: None,
                            span,
                            kept: false,
                        });
                    }
                    let build_loop = |lw: &mut Lowerer<'_>, next_opt: Expr| -> Stmt {
                        let lid = lw.next_loop;
                        lw.next_loop += 1;
                        let opt_slot = lw.temp(opt_elem.clone());
                        let k_slot = lw.temp(elem.clone());
                        let is_some = lw.mk_is_some(lw.local(opt_slot, span), Ty::Bool, span);
                        let not_some = lw.mk_unary(UnOp::Not, is_some, Ty::Bool, span);
                        let unwrapped =
                            lw.mk_unwrap(lw.local(opt_slot, span), false, elem.clone(), span);
                        let ok_call = lw.mk_call_expr(
                            lw.local(pred_slot, span),
                            vec![Arg::Value(lw.local(k_slot, span))],
                            Ty::Bool,
                            span,
                        );
                        let not_ok = lw.mk_unary(UnOp::Not, ok_call, Ty::Bool, span);
                        let rem = lw.mk_mutate(
                            Mutation::Remove,
                            place.clone(),
                            vec![lw.local(k_slot, span)],
                            Ty::Bool,
                            span,
                        );
                        let out_place = Place {
                            kind: PlaceKind::Local(out_slot),
                            ty: out_ty.clone(),
                            span,
                        };
                        let push = lw.mk_mutate(
                            Mutation::Push,
                            out_place,
                            vec![lw.local(k_slot, span)],
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
                        lw.mk_builtin(Builtin::SetMin, [set_read.clone()], opt_elem.clone(), span),
                    );
                    let after_branch = match after_slot {
                        Some(a_s) => {
                            let a_val =
                                lw.mk_unwrap(lw.local(a_s, span), false, elem.clone(), span);
                            let after_loop = build_loop(
                                lw,
                                lw.mk_builtin(
                                    Builtin::FirstGt,
                                    [set_read.clone(), a_val],
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk_is_some(lw.local(a_s, span), Ty::Bool, span),
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
                            let b_val =
                                lw.mk_unwrap(lw.local(b_s, span), false, elem.clone(), span);
                            let before_loop = build_loop(
                                lw,
                                lw.mk_builtin(
                                    Builtin::LastLt,
                                    [set_read, b_val],
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk_is_some(lw.local(b_s, span), Ty::Bool, span),
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
                    out.push_give(lw.local(out_slot, span));
                })
            }
            (
                Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Str,
                "clear",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(..) | Ty::Array { .. } | Ty::Str, "sort") => {
                self.sort(base, &elem, args, span)
            }
            (ty, "sorted") if ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_)) => {
                if args.is_empty() {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, args, span)
                }
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
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
                Some(arg) => self.mk_builtin(Builtin::Contains, [value, arg], Ty::Bool, span),
                None => self.err_expr(span),
            },
            // R04b.1.6: `z.rekey(s.keys)` hands a table computed on offsets back
            // in the caller's keys, O(1).
            (Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..), "rekey")
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
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let shown = self.show(other);
                        self.report(
                            exprs[0].span,
                            msgs::ir::TYPE_MISMATCH_REKEY_TAKES_A_RANGE,
                            &[&shown],
                        );
                        return self.err_expr(span);
                    }
                };
                // A view must not outlive its cells, so a temporary gets a slot.
                let value = if matches!(value.kind, ExprKind::Local(_)) {
                    value
                } else {
                    let slot = self.new_slot("cells", value.ty.clone());
                    self.cx.pending.push_let(slot, value);
                    self.local(slot, span)
                };
                let res = if matches!(ty, Ty::List(_) | Ty::Array { .. }) {
                    Ty::Keyed(Box::new(elem.clone()), dim)
                } else {
                    Ty::ListView(Box::new(elem.clone()), dim)
                };
                self.do_expr(Some(&res.clone()), span, |lw, out| {
                    let keys = lw.hold(keys, out);
                    let lo = lw.mk_builtin(Builtin::RangeLo, [keys.clone()], Ty::Int, span);
                    let hi = lw.mk_builtin(Builtin::RangeHi, [keys], Ty::Int, span);
                    let view = lw.mk_builtin(Builtin::Rekey, [value, lo, hi], res, span);
                    out.push_give(view);
                })
            }
            // How many cells equal `x`: `[1, 2, 1].count(1)` is 2,
            // `s[0..k].count('R')` scans the view directly without copying.
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(..)
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
                            | Ty::MutView(..)
                            | Ty::Array { .. }
                    ) || self.probe_ty_silent(&args[0].value) == Some(Ty::Char)) =>
            {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                        let (seq, arg) = (lw.hold(value, out), lw.hold(arg, out));
                        let (acc, each) = (lw.temp(Ty::Int), lw.temp(elem.clone()));
                        out.push_let(acc, lw.int_lit(0, span));
                        let same = lw.mk_cmp(CmpOp::Eq, lw.local(each, span), arg, Ty::Bool, span);
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
                        out.push_give(lw.local(acc, span));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Map(..) | Ty::HashMap(..), m)
                if super::super::foreign::foreign_insert(m) && matches!(args.len(), 1 | 2) =>
            {
                self.map_insert(whole, base, name, args, &ty)
            }
            (Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_), "count") if args.len() == 1 => {
                self.keyed_count(whole, base, &args[0], &ty)
            }
            (Ty::Map(_, val) | Ty::HashMap(_, val), "get") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => {
                        self.mk_builtin(Builtin::MapGet, [value, arg], Ty::Opt(val.clone()), span)
                    }
                    None => self.foreign_method_args(whole, base, name, args, &ty),
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
                    Some(bound) => self.mk_builtin(op, [value, bound], Ty::opt(found), span),
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
                    self.mk_builtin(op, [value], Ty::opt(elem), span)
                }
                None => self.err_expr(span),
            },
            (Ty::DomainVal | Ty::Pos(_), "nth" | "at") => {
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
            // `xs.find(x => x > 3)` is the first element the test holds
            // for; `xs.find(3)` is where the value 3 is. Handled uniformly across
            // every `Indexed` sequence (`List`, `ListView`, 1D `[D]T`, `Str`, `StrView`, `Range`).
            (ty, "find" | "rfind")
                if let Some(elem) = ty
                    .seq_elem()
                    .filter(|_| args.len() == 1 && self.is_test(&args[0].value)) =>
            {
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
                self.cx.pending.truncate(recv_mark);
                let rev = ast::Expr {
                    kind: ast::ExprKind::Bool(method == "rfind"),
                    span,
                };
                let call = [base.clone(), args[0].value.clone(), rev].map(|value| ast::Arg {
                    name: None,
                    mutable: false,
                    value,
                });
                self.std_call("strings", "_find_str", &[], &call, span)
            }
            (ty, "first_ge" | "first_gt" | "last_le" | "last_lt" | "find" | "rfind")
                if let Some(elem) = ty.seq_elem() =>
            {
                let dom = match &ty {
                    Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                self.seq_search_method(method, value, elem, dom, args, span)
            }
            // `r.nth(k)` on a range of positions or cuts: its `k`-th, checked against `r`.
            (Ty::Range(inner), "nth")
                if matches!(**inner, Ty::Pos(_) | Ty::Bound(_)) && args.len() == 1 =>
            {
                let elem = (**inner).clone();
                match self.one_arg("nth", args, &Ty::Int, span) {
                    Some(at) => self.mk_index(value, vec![at], elem, span),
                    None => self.err_expr(span),
                }
            }
            // `r.nth(k)` on an I128 range: its `k`-th value, `r.start + k`.
            (Ty::Range(inner), "nth") if **inner == Ty::I128 && args.len() == 1 => {
                let Some(k) = self.one_arg("nth", args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let k = self.cast(Cast::IntToI128, k, Ty::I128);
                let lo = self.mk_builtin(Builtin::RangeLo, [value], Ty::I128, span);
                self.mk_binary(BinOp::Add, lo, k, Ty::I128, span)
            }
            // `r.nth(k)` on an Int range: its `k`-th value, `r.start + k` (R04b.5.9).
            (Ty::Range(inner), "nth") if **inner == Ty::Int && args.len() == 1 => {
                let Some(k) = self.one_arg("nth", args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let lo = self.mk_builtin(Builtin::RangeLo, [value], Ty::Int, span);
                self.mk_binary(BinOp::Add, lo, k, Ty::Int, span)
            }
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
                // A named range of positions cut by a constant: the new ends are
                // plain arithmetic on its ends, with no copy, as `lead[r.drop(1)]` wants.
                let fixed = crate::lower::range::const_int_expr(&k).filter(|k| *k >= 0);
                if let (Some(k), ExprKind::Local(_), Ty::Pos(_)) = (fixed, &value.kind, &**inner) {
                    let end =
                        |lw: &mut Self, which| lw.mk_builtin(which, [value.clone()], Ty::Int, span);
                    let (lo, hi) = (end(self, Builtin::RangeLo), end(self, Builtin::RangeHi));
                    let len = self.mk_binary(BinOp::Sub, hi.clone(), lo.clone(), Ty::Int, span);
                    let zero = self.int_lit(0, span);
                    let len = self.mk_builtin(Builtin::Max, [len, zero], Ty::Int, span);
                    let k = self.int_lit(k, span);
                    let shift = self.mk_builtin(Builtin::Min, [len, k], Ty::Int, span);
                    let (from, op) = if from_hi {
                        (hi.clone(), BinOp::Sub)
                    } else {
                        (lo.clone(), BinOp::Add)
                    };
                    let point = self.mk_binary(op, from, shift, Ty::Int, span);
                    let (lo, hi) = if cut_lo { (point, hi) } else { (lo, point) };
                    let (lo, hi) = (Box::new(lo), Box::new(hi));
                    return self.mk(ExprKind::Range { lo, hi }, range_ty, span);
                }
                self.do_expr(Some(&range_ty), span, |lw, out| {
                    let range = lw.new_slot("range", range_ty.clone());
                    out.push_let(range, value);
                    let lo_s = lw.new_slot("lo", Ty::Int);
                    let hi_s = lw.new_slot("hi", Ty::Int);
                    let r1 = lw.local(range, span);
                    let r2 = lw.local(range, span);
                    out.push_let(lo_s, lw.mk_builtin(Builtin::RangeLo, [r1], Ty::Int, span));
                    out.push_let(hi_s, lw.mk_builtin(Builtin::RangeHi, [r2], Ty::Int, span));
                    let k_s = lw.new_slot("k", Ty::Int);
                    let zero = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let k_clamped = lw.mk_builtin(Builtin::Max, [k, zero], Ty::Int, span);
                    out.push_let(k_s, k_clamped);
                    let avail = lw.range_room(lo_s, hi_s, span);
                    let shift =
                        lw.mk_builtin(Builtin::Min, [avail, lw.local(k_s, span)], Ty::Int, span);
                    let (end, op) = if from_hi {
                        (hi_s, BinOp::Sub)
                    } else {
                        (lo_s, BinOp::Add)
                    };
                    let point = lw.mk_binary(op, lw.local(end, span), shift, Ty::Int, span);
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
                    out.push_give(res);
                })
            }
            // `(lo..hi).step(k)`: lo, lo + k, ... below hi, as a list.
            (Ty::Range(inner), "step") if **inner == Ty::Int && args.len() == 1 => {
                let by = self.expr(&args[0].value, Some(&Ty::Int));
                let by = self.coerce(by, &Ty::Int, span);
                let list_ty = Ty::list(Ty::Int);
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push_let(range, value);
                    let acc = lw.new_slot("steps", list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push_let(acc, empty);
                    let i = lw.new_slot("i", Ty::Int);
                    let bound = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        lw.mk_builtin(which, [range], Ty::Int, span)
                    };
                    let (lo, hi) = (bound(lw, Builtin::RangeLo), bound(lw, Builtin::RangeHi));
                    let item = lw.local(i, span);
                    let push = lw.mk_mutate(
                        Mutation::Push,
                        Place {
                            kind: PlaceKind::Local(acc),
                            ty: list_ty.clone(),
                            span: lw.span(span),
                        },
                        vec![item],
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
                    out.push_give(lw.local(acc, span));
                })
            }
            // `xs.last(k)`: the element `k` back from the last one, so
            // `xs.last(1)` is the one before the last; `none` when there
            // are not that many.
            (ty, "last") if let Some(elem) = ty.seq_elem().filter(|_| args.len() == 1) => {
                // An array of positions is read at the position that far in.
                let of = match &ty {
                    Ty::Array { dims, .. } => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let back = self.expr_as(&args[0].value, &Ty::Int);
                self.do_expr(Some(&Ty::opt(elem.clone())), span, |lw, out| {
                    // A list that is named is read where it is; one that is
                    // computed is kept for the two reads.
                    let list = if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        value
                    } else {
                        let kept = lw.temp(value.ty.clone());
                        out.push_let(kept, value);
                        lw.local(kept, span)
                    };
                    let k = lw.temp(Ty::Int);
                    out.push_let(k, back);
                    let len = lw.mk_builtin(Builtin::Len, [list.clone()], Ty::Int, span);
                    let last = lw.int_add(len, -1);
                    let at = lw.temp(Ty::Int);
                    let from_end = lw.mk_binary(BinOp::Sub, last, lw.local(k, span), Ty::Int, span);
                    out.push_let(at, from_end);
                    let cmp = |lw: &mut Self, slot| {
                        let zero = lw.int_lit(0, span);
                        lw.mk_cmp(CmpOp::Ge, lw.local(slot, span), zero, Ty::Bool, span)
                    };
                    let (ahead, inside) = (cmp(lw, k), cmp(lw, at));
                    let there = lw.mk_binary(BinOp::And, ahead, inside, Ty::Bool, span);
                    let cell = match of {
                        Some(d) => lw.mk(
                            ExprKind::Cast(Cast::FromOrd(d), Box::new(lw.local(at, span))),
                            Ty::Pos(d),
                            span,
                        ),
                        None => lw.local(at, span),
                    };
                    let item = lw.mk_index(list, vec![cell], elem.clone(), span);
                    let opt = Ty::opt(elem);
                    let some = lw.mk_some(item, opt.clone(), span);
                    let none = lw.mk(ExprKind::None, opt.clone(), span);
                    out.push_give(lw.mk_if(there, some, none, opt, span));
                })
            }
            (Ty::Str | Ty::StrView, "rev") if args.is_empty() => {
                let value = self.hoist_view_base(value, span);
                self.mk_builtin(Builtin::Reversed, [value], Ty::StrView, span)
            }
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _), "rev")
                if args.is_empty() =>
            {
                let elem = (**e).clone();
                let value = self.hoist_view_base(value, span);
                self.mk_builtin(
                    Builtin::Reversed,
                    [value],
                    Ty::ListView(Box::new(elem), Dim::Int),
                    span,
                )
            }
            // A range answers its own `count()` from its ends, O(1) (R07.9.6).
            (Ty::Range(_), "count") if args.is_empty() => self.len_of(value, span),
            (Ty::Range(of), "rev") if args.is_empty() => {
                let of = (**of).clone();
                self.reversed_range(value, of, span)
            }
            // `r.halves()`: split `r` into `(left, right)` with `left.len == (r.len + 1) / 2`
            // and `right.len == r.len / 2` (both non-empty whenever `r.len >= 2`).
            (Ty::Range(_), "halves") if args.is_empty() => {
                let r_ty = value.ty.clone();
                let tup_ty = Ty::Tuple(vec![r_ty.clone(), r_ty.clone()]);
                self.do_expr(Some(&tup_ty.clone()), span, |lw, out| {
                    let r_slot = lw.temp(r_ty.clone());
                    out.push_let(r_slot, value);
                    let r_ref = lw.local(r_slot, span);
                    let lo_slot = lw.temp(Ty::Int);
                    let hi_slot = lw.temp(Ty::Int);
                    let lo_expr = lw.mk_builtin(Builtin::RangeLo, [r_ref.clone()], Ty::Int, span);
                    let raw_hi = lw.mk_builtin(Builtin::RangeHi, [r_ref], Ty::Int, span);
                    out.push_let(lo_slot, lo_expr);
                    let hi_expr = lw.mk_builtin(
                        Builtin::Max,
                        [raw_hi, lw.local(lo_slot, span)],
                        Ty::Int,
                        span,
                    );
                    out.push_let(hi_slot, hi_expr);
                    let diff = lw.mk_binary(
                        BinOp::Sub,
                        lw.local(hi_slot, span),
                        lw.local(lo_slot, span),
                        Ty::Int,
                        span,
                    );
                    let left_len = lw.mk_binary(
                        BinOp::Div,
                        lw.int_add(diff, 1),
                        lw.int_lit(2, span),
                        Ty::Int,
                        span,
                    );
                    let mid_slot = lw.temp(Ty::Int);
                    let mid_expr =
                        lw.mk_binary(BinOp::Add, lw.local(lo_slot, span), left_len, Ty::Int, span);
                    out.push_let(mid_slot, mid_expr);
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
                    out.push_give(pair);
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
                self.mk_builtin(
                    Builtin::Reversed,
                    [value],
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
                self.report(
                    name.span,
                    msgs::ir::UNSUPPORTED_A_REVERSED_IS_SOMETHING,
                    &[&self.show(&ty)],
                );
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
                        out.push_let(m_slot, fill);
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
                        out.push_give(lw.local(m_slot, span));
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
                let at = self.mk_binary(op, ord, k, Ty::Int, span);
                self.mk_builtin(Builtin::PosTryOrd(d), [at], Ty::opt(Ty::Pos(d)), span)
            }
            // `Reversible.rev`: a value that is not a sequence and has no
            // `rev` of its own is its own reverse.
            (_, "rev") if args.is_empty() => value,
            _ => {
                if let Some(fid) = self.value_impl_method(&ty, method) {
                    let mut all_args = vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: base.clone(),
                    }];
                    all_args.extend(args.iter().cloned());
                    self.cx.pending.truncate(recv_mark);
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
                    self.cx.pending.truncate(recv_mark);
                    return self.method(whole, &all, name, args, hint);
                }
                if self.number_max_call(&ty, base, method, args, span) {
                    return self.err_expr(span);
                }
                // UFCS free function with `self` first parameter (e.g. `p.step(d)`, `p.ray(d)`).
                if let Some(res) =
                    self.try_ufcs_call(method, base, args, span, Some(recv_mark), hint)
                {
                    return res;
                }
                if let Some(message) = typo::builtin_misuse(&ty, method) {
                    self.report_said(name.span, message);
                    return self.err_expr(span);
                }
                self.report(
                    name.span,
                    msgs::ir::UNKNOWN_FIELD_HAS_NO_METHOD,
                    &[&self.a_ty(&ty), &method],
                );
                self.member_typo(&ty, method, name.span);
                self.foreign_call_fix(&ty, base, method, span);
                self.slice_call_fix(&ty, base, method, args, span);
                self.free_call_fix(&ty, base, method, args, span);
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
