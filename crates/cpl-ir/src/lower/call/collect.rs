//! First/last where, adjacency, spans, counts and list-form calls.

use super::*;
use cpl_diag::msgs;

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
        let pred = self.expr_as(pred_ast, &fn_ty);
        let opt_ty = Ty::opt(elem_ty.clone());
        self.do_expr(Some(&opt_ty), span, |lw, out| {
            let pred_val = lw.hold(pred, out);
            let found = lw.temp(opt_ty.clone());
            let init = lw.mk(ExprKind::None, opt_ty.clone(), span);
            out.push_let(found, init);
            let item = lw.temp(elem_ty.clone());
            let lid = lw.fresh_loop();
            let item_expr = lw.local(item, span);
            let cond = lw.mk_call_expr(
                pred_val,
                vec![Arg::Value(item_expr.clone())],
                Ty::Bool,
                span,
            );
            let some_val = lw.mk_some(item_expr, opt_ty.clone(), span);
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
            lw.push_for(out, lid, item, iter, body);
            out.push_give(lw.local(found, span));
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
            out.push_let(acc, lw.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span));
            let list_ty = ty.clone();
            let push = move |lw: &mut Self, value: Expr| {
                let place = Place {
                    kind: PlaceKind::Local(acc),
                    ty: list_ty.clone(),
                    span: lw.span(span),
                };
                Stmt::Expr(lw.mk_mutate(Mutation::Push, place, vec![value], Ty::Unit, span))
            };
            build(lw, out, &push);
            out.push_give(lw.local(acc, span));
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
            lw.push_for(out, lid, item, iter, body);
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
            self.report(
                span,
                msgs::ir::BAD_OPERANDS_A_HAS_NO_ORDER,
                &[&self.show(&elem)],
            );
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
            out.push_let(
                m_slot,
                lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span),
            );
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
            lw.push_for(
                out,
                lid,
                item,
                iter,
                Block {
                    stmts: vec![Stmt::Assign(place, AssignOp::Binary(BinOp::Add), one)],
                },
            );
            let res = lw.mk(
                ExprKind::Tuple(vec![lw.local(m_slot, span)]),
                struct_ty.clone(),
                span,
            );
            out.push_give(res);
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
                // A list of records is `Indexed` by its positions: the fn itself takes it.
                Ty::List(elem) if matches!(**elem, Ty::Tuple(_)) => elem.fields().map(|f| f.len()),
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
            out.push_let(src, xs);
            let acc = lw.temp(list_ty.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push_let(acc, empty);
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
            out.push_give(lw.local(acc, span));
        })
    }
}
