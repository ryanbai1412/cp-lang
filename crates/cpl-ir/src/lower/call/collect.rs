//! First/last where, adjacency, spans, counts and list-form calls.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn lower_first_or_last_where(
        &mut self,
        base: &ast::Expr,
        pred_ast: &ast::Expr,
        stop_at_first: bool,
        span: Span,
    ) -> Expr {
        let (iter, elem_ty) = self.iter(base);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        let fn_ty = Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool));
        let pred = self.expect(pred_ast, &fn_ty);
        let opt_ty = Ty::opt(elem_ty.clone());
        self.do_expr(Some(&opt_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let found = lw.temp(opt_ty.clone());
            let init = lw.mk(ExprKind::None, opt_ty.clone(), span);
            out.push(Stmt::Let(found, Some(init)));
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk(
                ExprKind::CallExpr(Box::new(pred_val), vec![Arg::Value(item_expr.clone())]),
                Ty::Bool,
                span,
            );
            let some_val = lw.mk(ExprKind::Some(Box::new(item_expr)), opt_ty.clone(), span);
            let set_found = Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(found),
                    ty: opt_ty.clone(),
                    span: lw.span(span),
                },
                AssignOp::Set,
                some_val,
            );
            let mut then_stmts = vec![set_found];
            if stop_at_first {
                then_stmts.push(Stmt::Break(Some(lid)));
            }
            let body = Block {
                stmts: vec![Stmt::If(
                    cond,
                    Block { stmts: then_stmts },
                    Block::default(),
                )],
            };
            out.push(Stmt::For(lid, item, iter, body));
            out.push(Stmt::Give(lw.local(found, span)));
        })
    }

    /// `adjacency(edges)` -> `[V]List<V>` and `adjacency(edges, into: Bitset)` -> `[V]Bitset<V>`.
    pub(in crate::lower) fn lower_adjacency(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        if args.is_empty() || args.len() > 2 {
            self.error(
                codes::WRONG_ARG_COUNT,
                span,
                "`adjacency` takes `(edges)` or `(edges, into: Bitset)`",
            );
            return self.err_expr(span);
        }
        let mut into_bitset = false;
        if let Some(opt) = args.get(1) {
            if opt.name.as_ref().map(|n| n.text.as_str()) == Some("into")
                && matches!(&opt.value.kind, ast::ExprKind::Name(n) if n == "Bitset")
            {
                into_bitset = true;
            } else {
                self.error(
                    codes::TYPE_MISMATCH,
                    opt.value.span,
                    "`adjacency` expects `into: Bitset`",
                );
                return self.err_expr(span);
            }
        }
        // Fuse `edges.filter(e => cond)` to avoid materializing a temporary edge list.
        let (edges_ast, filter_lambda) = match &args[0].value.kind {
            ast::ExprKind::Call {
                callee,
                args: fargs,
            } => match (&callee.kind, fargs.as_slice()) {
                (
                    ast::ExprKind::Field { base, name },
                    [ast::Arg {
                        name: None,
                        value:
                            ast::Expr {
                                kind: ast::ExprKind::Lambda { params, body, .. },
                                ..
                            },
                        ..
                    }],
                ) if name.text == "filter" && params.len() == 1 => {
                    (base.as_ref(), Some((&params[0], body.as_ref())))
                }
                _ => (&args[0].value, None),
            },
            _ => (&args[0].value, None),
        };
        let (iter, elem_ty) = self.iter(edges_ast);
        let (dom, u_idx, v_idx) = match &elem_ty {
            Ty::Record(fields) => {
                let u = fields.iter().position(|(n, _)| n == "u");
                let v = fields.iter().position(|(n, _)| n == "v");
                match (u, v) {
                    (Some(ui), Some(vi)) => match (&fields[ui].1, &fields[vi].1) {
                        (Ty::Pos(du), Ty::Pos(dv)) if du == dv => (*du, ui, vi),
                        _ => {
                            self.error(
                                codes::TYPE_MISMATCH,
                                args[0].value.span,
                                "`adjacency` edges must have `u` and `v` in the same vertex domain",
                            );
                            return self.err_expr(span);
                        }
                    },
                    _ => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            args[0].value.span,
                            "`adjacency` edges must be `(u: V, v: V, ...)` or `(V, V)`",
                        );
                        return self.err_expr(span);
                    }
                }
            }
            Ty::Tuple(parts) if parts.len() >= 2 => match (&parts[0], &parts[1]) {
                (Ty::Pos(du), Ty::Pos(dv)) if du == dv => (*du, 0, 1),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        args[0].value.span,
                        "`adjacency` edges must have endpoints in the same vertex domain",
                    );
                    return self.err_expr(span);
                }
            },
            Ty::Error => return self.err_expr(span),
            _ => {
                self.error(
                    codes::TYPE_MISMATCH,
                    args[0].value.span,
                    "`adjacency` expects a sequence of `(u: V, v: V, ...)` or `(V, V)` edges",
                );
                return self.err_expr(span);
            }
        };
        let (cell_ty, init_expr, add_fn_opt) = if into_bitset {
            let Some(sid) = self.instantiate_bitset(dom, span) else {
                return self.err_expr(span);
            };
            let Some(&new_fn) = self.structs[sid].methods.get("new") else {
                return self.err_expr(span);
            };
            let Some(&add_fn) = self.structs[sid].methods.get("add") else {
                return self.err_expr(span);
            };
            self.queue_fn(new_fn);
            self.queue_fn(add_fn);
            let init = self.mk(ExprKind::Call(new_fn, Vec::new()), Ty::Struct(sid), span);
            (Ty::Struct(sid), init, Some(add_fn))
        } else {
            let list_ty = Ty::list(Ty::Pos(dom));
            let init = self.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            (list_ty, init, None)
        };
        let arr_ty = Ty::Array {
            dims: vec![Dim::Pos(dom)],
            elem: Box::new(cell_ty.clone()),
        };
        self.do_expr(Some(&arr_ty), span, |lw, out| {
            let acc = lw.temp(arr_ty.clone());
            let fill = lw.mk(
                ExprKind::Fill(vec![DimSpec::Pos(dom)], Some(Box::new(init_expr))),
                arr_ty.clone(),
                span,
            );
            out.push(Stmt::Let(acc, Some(fill)));
            let mut loop_pre = Vec::new();
            let (edge, filter_cond) = if let Some((pat, cond_ast)) = filter_lambda {
                lw.cx.scopes.push(HashMap::new());
                let slot = lw.loop_var(pat, elem_ty.clone(), &mut loop_pre);
                let cond = lw.expect(cond_ast, &Ty::Bool);
                lw.cx.scopes.pop();
                (slot, Some(cond))
            } else {
                (lw.temp(elem_ty.clone()), None)
            };
            let edge_expr = lw.local(edge, span);
            let u_expr = lw.mk(
                ExprKind::Field(Box::new(edge_expr.clone()), u_idx),
                Ty::Pos(dom),
                span,
            );
            let v_expr = lw.mk(
                ExprKind::Field(Box::new(edge_expr), v_idx),
                Ty::Pos(dom),
                span,
            );
            let acc_place = Place {
                kind: PlaceKind::Local(acc),
                ty: arr_ty.clone(),
                span: lw.span(span),
            };
            let u_cell = Place {
                kind: PlaceKind::Index(Box::new(acc_place.clone()), vec![u_expr.clone()]),
                ty: cell_ty.clone(),
                span: lw.span(span),
            };
            let v_cell = Place {
                kind: PlaceKind::Index(Box::new(acc_place), vec![v_expr.clone()]),
                ty: cell_ty.clone(),
                span: lw.span(span),
            };
            let mut body_stmts = Vec::new();
            if let Some(add_fn) = add_fn_opt {
                let c1 = lw.mk(
                    ExprKind::Call(add_fn, vec![Arg::Mut(u_cell), Arg::Value(v_expr)]),
                    Ty::Unit,
                    span,
                );
                let c2 = lw.mk(
                    ExprKind::Call(add_fn, vec![Arg::Mut(v_cell), Arg::Value(u_expr)]),
                    Ty::Unit,
                    span,
                );
                body_stmts.push(Stmt::Expr(c1));
                body_stmts.push(Stmt::Expr(c2));
            } else {
                let p1 = lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(u_cell), vec![v_expr]),
                    Ty::Unit,
                    span,
                );
                let p2 = lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(v_cell), vec![u_expr]),
                    Ty::Unit,
                    span,
                );
                body_stmts.push(Stmt::Expr(p1));
                body_stmts.push(Stmt::Expr(p2));
            }
            let loop_body = if let Some(cond) = filter_cond {
                loop_pre.push(Stmt::If(
                    cond,
                    Block { stmts: body_stmts },
                    Block::default(),
                ));
                Block { stmts: loop_pre }
            } else {
                Block { stmts: body_stmts }
            };
            let lid = lw.fresh_loop();
            out.push(Stmt::For(lid, edge, iter, loop_body));
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }

    /// Materializes `V.pairs` as `List<(V, V)>` for all `0 <= i < j < V.count`.
    pub(in crate::lower) fn domain_pairs_list(&mut self, id: DomainId, span: Span) -> Expr {
        let pair_ty = Ty::Tuple(vec![Ty::Pos(id), Ty::Pos(id)]);
        self.collected(pair_ty.clone(), span, |lw, out, push| {
            let len = lw.mk(ExprKind::DomainLen(id), Ty::Int, span);
            let i_slot = lw.temp(Ty::Pos(id));
            let j_slot = lw.temp(Ty::Pos(id));
            let i_var = lw.mk(ExprKind::Local(i_slot), Ty::Pos(id), span);
            let j_var = lw.mk(ExprKind::Local(j_slot), Ty::Pos(id), span);
            let i_ord = lw.cast(Cast::Ord, i_var.clone(), Ty::Int);
            let j_lo = lw.int_add(i_ord, 1);
            let inner_id = lw.fresh_loop();
            let outer_id = lw.fresh_loop();
            let pair = lw.mk(ExprKind::Tuple(vec![i_var, j_var]), pair_ty, span);
            let push_stmt = push(lw, pair);
            let inner = Stmt::For(
                inner_id,
                j_slot,
                Iter::Range {
                    lo: j_lo,
                    hi: len.clone(),
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block {
                    stmts: vec![push_stmt],
                },
            );
            out.push(Stmt::For(
                outer_id,
                i_slot,
                Iter::Range {
                    lo: lw.int_lit(0, span),
                    hi: len,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Block { stmts: vec![inner] },
            ));
        })
    }

    /// A `[]` list local, the loop that pushes `value` for each `item` of
    /// `iter`, and the list.
    pub(in crate::lower) fn collected(
        &mut self,
        elem: Ty,
        span: Span,
        build: impl FnOnce(&mut Self, &mut Vec<Stmt>, &dyn Fn(&mut Self, Expr) -> Stmt),
    ) -> Expr {
        let ty = Ty::list(elem);
        self.do_expr(Some(&ty), span, |lw, out| {
            let acc = lw.temp(ty.clone());
            out.push(Stmt::Let(
                acc,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
            ));
            let list_ty = ty.clone();
            let push = move |lw: &mut Self, value: Expr| {
                let place = Place {
                    kind: PlaceKind::Local(acc),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                };
                Stmt::Expr(lw.mk(
                    ExprKind::Mutate(Mutation::Push, Box::new(place), vec![value]),
                    Ty::Unit,
                    span,
                ))
            };
            build(lw, out, &push);
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }

    pub(in crate::lower) fn collect_iter_to_list(&mut self, src: &ast::Expr, span: Span) -> Expr {
        let (iter, elem) = self.iter(src);
        self.collected(elem.clone(), span, |lw, out, push| {
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let value = lw.local(item, span);
            let body = Block {
                stmts: vec![push(lw, value)],
            };
            out.push(Stmt::For(lid, item, iter, body));
        })
    }

    /// `xs.adjacent`: the pairs `(xs[i], xs[i + 1])`, as a list.
    pub(in crate::lower) fn lower_adjacent(&mut self, value: Expr, span: Span) -> Expr {
        let (elem_ty, dim) = match &value.ty {
            Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => {
                ((**e).clone(), Dim::Int)
            }
            Ty::Str | Ty::StrView => (Ty::Char, Dim::Int),
            Ty::Array { dims, elem } if dims.len() == 1 => ((**elem).clone(), dims[0]),
            _ => return self.err_expr(span),
        };
        let pair_ty = Ty::Tuple(vec![elem_ty.clone(), elem_ty.clone()]);
        let reusable = matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_));
        self.collected(pair_ty.clone(), span, |lw, out, push| {
            let seq = if reusable {
                value
            } else {
                let held = lw.temp(value.ty.clone());
                out.push(Stmt::Let(held, Some(value)));
                lw.local(held, span)
            };
            let len = lw.len_of(seq.clone(), span);
            let hi = lw.int_add(len, -1);
            let lo = lw.int_lit(0, span);
            let slot = lw.temp(Ty::Int);
            let lid = lw.fresh_loop();
            let at = |lw: &mut Self, offset: i64| {
                let i = lw.local(slot, span);
                let i = if offset == 0 {
                    i
                } else {
                    lw.int_add(i, offset)
                };
                let index = match dim {
                    Dim::Pos(d) => lw.cast(Cast::Trust, i, Ty::Pos(d)),
                    Dim::Bound(d) => lw.cast(Cast::Trust, i, Ty::Bound(d)),
                    Dim::Enum(e) => lw.cast(Cast::EnumOrd(e), i, Ty::Int),
                    Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => i,
                };
                lw.mk(
                    ExprKind::Index(Box::new(seq.clone()), vec![index]),
                    elem_ty.clone(),
                    span,
                )
            };
            let (a, b) = (at(lw, 0), at(lw, 1));
            let pair = lw.mk(ExprKind::Tuple(vec![a, b]), pair_ty, span);
            let body = Block {
                stmts: vec![push(lw, pair)],
            };
            out.push(Stmt::For(
                lid,
                slot,
                Iter::Range {
                    lo,
                    hi,
                    step: None,
                    rev: false,
                    closed: false,
                },
                body,
            ));
        })
    }

    /// `span(a, b)`: smallest `Range<D>` covering positions or bounds `a` and `b` of domain `D`.
    pub(in crate::lower) fn lower_span(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("span", args, 2, span) else {
            return self.err_expr(span);
        };
        let a = self.expr(exprs[0], None);
        let b = self.expr(exprs[1], None);
        if a.ty == Ty::Error || b.ty == Ty::Error {
            return self.err_expr(span);
        }
        let dom_of = |ty: &Ty| match ty {
            Ty::Pos(d) => Some((*d, true)),
            Ty::Bound(d) => Some((*d, false)),
            _ => None,
        };
        let (Some((da, a_pos)), Some((db, b_pos))) = (dom_of(&a.ty), dom_of(&b.ty)) else {
            let message = format!(
                "`span` takes two positions or bounds of a domain, found `{}` and `{}`",
                self.show(&a.ty),
                self.show(&b.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        if da != db {
            let message = format!(
                "`span` takes two positions or bounds of one domain, found `{}` and `{}`",
                self.show(&a.ty),
                self.show(&b.ty)
            );
            self.error(codes::WRONG_DOMAIN, span, message);
            return self.err_expr(span);
        }
        let out_ty = Ty::Range(Box::new(Ty::Pos(da)));
        let ao = self.cast(Cast::Ord, a, Ty::Int);
        let bo = self.cast(Cast::Ord, b, Ty::Int);
        self.do_expr(Some(&out_ty), span, |lw, out| {
            let (aslot, bslot) = (lw.temp(Ty::Int), lw.temp(Ty::Int));
            out.push(Stmt::Let(aslot, Some(ao)));
            out.push(Stmt::Let(bslot, Some(bo)));
            let (av, bv) = (lw.local(aslot, span), lw.local(bslot, span));
            let lo = lw.mk(
                ExprKind::Builtin(Builtin::Min, vec![av.clone(), bv.clone()]),
                Ty::Int,
                span,
            );
            let a_end = if a_pos { lw.int_add(av, 1) } else { av };
            let b_end = if b_pos { lw.int_add(bv, 1) } else { bv };
            let hi = lw.mk(
                ExprKind::Builtin(Builtin::Max, vec![a_end, b_end]),
                Ty::Int,
                span,
            );
            let range = lw.mk(
                ExprKind::Range {
                    lo: Box::new(lo),
                    hi: Box::new(hi),
                },
                out_ty.clone(),
                span,
            );
            out.push(Stmt::Give(range));
        })
    }

    /// `counts(xs)`: how many times each value occurs; `c[k]` is `0` for
    /// a value that does not.
    pub(in crate::lower) fn lower_counts(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("counts", args, 1, span) else {
            return self.err_expr(span);
        };
        let (iter, elem) = self.iter(exprs[0]);
        if elem == Ty::Error {
            return self.err_expr(span);
        }
        let hashable = matches!(
            elem,
            Ty::Int | Ty::Char | Ty::Bool | Ty::Str | Ty::Pos(_) | Ty::Bound(_)
        );
        if !hashable && !self.ordered(&elem) {
            let message = format!("a `{}` has no order to count by", self.show(&elem));
            self.error(codes::BAD_OPERANDS, span, message);
            return self.err_expr(span);
        }
        let sid = match self.counter_structs.get(&elem) {
            Some(&sid) => sid,
            None => {
                let map_ty = if hashable {
                    Ty::HashMap(Box::new(elem.clone()), Box::new(Ty::Int))
                } else {
                    Ty::Map(Box::new(elem.clone()), Box::new(Ty::Int))
                };
                let name = format!("Counter_{}", self.counter_structs.len());
                let sid = self.push_synth_struct(name, vec![("m".into(), map_ty)]);
                self.counter_structs.insert(elem.clone(), sid);
                self.counter_key.insert(sid, elem.clone());
                sid
            }
        };
        let map_ty = self.structs[sid].def.fields[0].1.clone();
        let struct_ty = Ty::Struct(sid);
        self.do_expr(Some(&struct_ty), span, |lw, out| {
            let m_slot = lw.temp(map_ty.clone());
            out.push(Stmt::Let(
                m_slot,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span)),
            ));
            let item = lw.temp(elem);
            let lid = lw.fresh_loop();
            let place = Place {
                kind: PlaceKind::Index(
                    Box::new(Place {
                        kind: PlaceKind::Local(m_slot),
                        ty: map_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![lw.local(item, span)],
                ),
                ty: Ty::Int,
                span: lw.span(span),
            };
            let one = lw.int_lit(1, span);
            out.push(Stmt::For(
                lid,
                item,
                iter,
                Block {
                    stmts: vec![Stmt::Assign(place, AssignOp::Binary(BinOp::Add), one)],
                },
            ));
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(m_slot, span)]),
                struct_ty.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }

    /// `T.f(list)` where `list` is a plain list (or array over a plain
    /// range) of pairs or triples: the `f_of_pairs` / `f_of_triples` fn
    /// `T`'s module declares, if any.
    pub(in crate::lower) fn list_form_call(
        &mut self,
        gid: GenStructId,
        name: &ast::Name,
        type_args: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let tpl_mod = self.gen_structs[gid].mod_id;
        // A static fn can take a plain list of pairs or triples when
        // its module declares `<name>_of_pairs` / `<name>_of_triples`:
        // `Graph.undirected(pairs)` needs no edge domain.
        let [only] = args else {
            return None;
        };
        {
            let probed = self.probe_expr(&only.value, None);
            // So does an array over a plain range, `[0..m](u: V, v: V)`:
            // it names no edge domain either. And one of plain
            // tuples, `[E](V, V, W)`, whose cells have no `u` and `v`.
            let arity = match &probed.ty {
                Ty::List(elem) => elem.fields().map(|f| f.len()),
                Ty::Array { dims, elem }
                    if matches!(dims.as_slice(), [Dim::Int])
                        || (dims.len() == 1 && matches!(**elem, Ty::Tuple(_))) =>
                {
                    elem.fields().map(|f| f.len())
                }
                _ => None,
            };
            let suffix = match arity {
                Some(2) => Some("pairs"),
                Some(3) => Some("triples"),
                _ => None,
            };
            let free = suffix.and_then(|s| {
                self.mod_syms[tpl_mod]
                    .get(&format!("{}_of_{s}", name.text))
                    .copied()
            });
            if let Some(Sym::GenFn(gfid)) = free {
                // `Graph<V>.undirected(pairs)` is the fn `Graph.undirected(pairs)`
                // is: the receiver's type arguments fill its like-named generics.
                let struct_generics = &self.gen_structs[gid].decl.generics;
                let explicit: Vec<GenArg> = self.gen_fns[gfid]
                    .decl
                    .generics
                    .iter()
                    .map_while(|g| {
                        let at = struct_generics.iter().position(|s| s.text == g.text)?;
                        type_args.get(at)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|t| self.resolve_gen_arg(t))
                    .collect();
                if !matches!(probed.ty, Ty::Array { .. }) {
                    return Some(self.call_gen_fn(gfid, explicit, None, args, span));
                }
                // A 1D array has the same backing storage as a list; `List(edges)`
                // lowers to `Builtin::ArrayToList`, avoiding a copy loop.
                let at = only.value.span;
                let list_call = ast::ExprKind::Call {
                    callee: Box::new(ast::Expr {
                        kind: ast::ExprKind::Name("List".into()),
                        span: at,
                    }),
                    args: vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: only.value.clone(),
                    }],
                };
                let listed = ast::Arg {
                    name: None,
                    mutable: false,
                    value: ast::Expr {
                        kind: list_call,
                        span: at,
                    },
                };
                return Some(self.call_gen_fn(gfid, explicit, None, &[listed], span));
            }
        }
        None
    }

    /// `[x] * n`: `n` copies of the list, one after another (decision 135).
    pub(in crate::lower) fn list_repeat(
        &mut self,
        xs: Expr,
        n: Expr,
        elem: Ty,
        span: Span,
    ) -> Expr {
        let list_ty = Ty::list(elem);
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let src = lw.temp(list_ty.clone());
            out.push(Stmt::Let(src, Some(xs)));
            let acc = lw.temp(list_ty.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(acc, Some(empty)));
            let place = Place {
                kind: PlaceKind::Local(acc),
                ty: list_ty.clone(),
                span: lw.span(span),
            };
            let copy = lw.local(src, span);
            let extend = ExprKind::Mutate(Mutation::Extend, Box::new(place), vec![copy]);
            let extend = lw.mk(extend, Ty::Unit, span);
            let lid = lw.next_loop;
            lw.next_loop += 1;
            let walk = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: n,
                step: None,
                rev: false,
                closed: false,
            };
            let i = lw.temp(Ty::Int);
            let body = Block {
                stmts: vec![Stmt::Expr(extend)],
            };
            out.push(Stmt::For(lid, i, walk, body));
            out.push(Stmt::Give(lw.local(acc, span)));
        })
    }
}
