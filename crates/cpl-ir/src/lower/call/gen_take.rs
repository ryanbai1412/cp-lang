//! `g(a).take(k)`, `x.g(a).take(k)` or a getter's `x.g.take(k)` on a generator: a loop `g` feeds that stops after `k`
//! values, so only `k` are pulled and an infinite generator gives its first
//! `k` (R07.1.3).

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn take_from_generator(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let ([k], "take") = (args, name.text.as_str()) else {
            return None;
        };
        if k.name.is_some() {
            return None;
        }
        // `g(a).map(f).filter(p).take(k)`: the adapters run inside the fed loop.
        let mut base = base;
        let mut adapters = Vec::new();
        while let ast::ExprKind::Call { callee, args } = &base.kind {
            let (ast::ExprKind::Field { base: inner, name }, [f]) = (&callee.kind, args.as_slice())
            else {
                break;
            };
            if f.name.is_some() || !matches!(name.text.as_str(), "map" | "filter") {
                break;
            }
            adapters.push((name.text == "map", &f.value));
            base = inner;
        }
        adapters.reverse();
        let callee = match &base.kind {
            ast::ExprKind::Call { callee, .. } => Some(callee),
            // `x.g` on a `@get` generator.
            ast::ExprKind::Field { .. } => None,
            _ => return None,
        };
        let generator = match callee.and_then(|c| self.sym(c)) {
            Some(Sym::Fn(id)) => self.sigs[id].is_generator,
            Some(Sym::GenFn(id)) => matches!(
                self.gen_fns[id].decl.ret.as_ref().map(|t| &t.kind),
                Some(ast::TypeKind::Yields(_))
            ),
            // `x.g(a)` or `x.g`: which `g` only lowering the receiver tells.
            _ => matches!(
                self.probe_expr(base, None).kind,
                ExprKind::Call(id, _) if self.sigs.get(id).is_some_and(|s| s.is_generator)
            ),
        };
        if !generator {
            return None;
        }
        let mut ty = Ty::Error;
        let mut taken = self.do_expr(None, span, |lw, out| {
            let mut seq = lw.expr(base, None);
            if let ExprKind::Call(_, call_args) = &mut seq.kind {
                for arg in call_args {
                    if let Arg::Value(e) = arg {
                        let held = lw.temp(e.ty.clone());
                        let e_span = e.span;
                        out.push_let(held, std::mem::replace(e, lw.local(held, e_span)));
                    }
                }
            }
            let mut elem = match &seq.ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let x = lw.temp(elem.clone());
            let mut scope = HashMap::default();
            let mut step = Vec::new();
            let mut cur = x;
            for (i, (map, f)) in adapters.iter().enumerate() {
                let hint = Ty::Fn(vec![elem.clone()], Box::new(Ty::Error));
                let fv = lw.expr(f, Some(&hint));
                let ret = match &fv.ty {
                    Ty::Fn(_, r) => (**r).clone(),
                    _ => Ty::Error,
                };
                let f_name = format!("__take_f{i}");
                let f_slot = lw.new_slot(&f_name, fv.ty.clone());
                out.push_let(f_slot, fv);
                let x_name = format!("__take_x{i}");
                scope.insert(f_name.clone(), f_slot);
                scope.insert(x_name.clone(), cur);
                let at = |kind| ast::Expr { kind, span };
                let call = at(ast::ExprKind::Call {
                    callee: Box::new(at(ast::ExprKind::Name(f_name))),
                    args: vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: at(ast::ExprKind::Name(x_name)),
                    }],
                });
                if *map {
                    elem = ret.clone();
                    cur = lw.temp(ret);
                }
                step.push((*map, call, cur));
            }
            let list_ty = Ty::list(elem.clone());
            ty = list_ty.clone();
            let count = lw.expr(&k.value, Some(&Ty::Int));
            let count = lw.coerce(count, &Ty::Int, k.value.span);
            let k_slot = lw.temp(Ty::Int);
            out.push_let(k_slot, count);
            let acc = lw.temp(list_ty.clone());
            let taken = lw.temp(Ty::Int);
            lw.cx.mut_slots.insert(acc);
            lw.cx.mut_slots.insert(taken);
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push_let(acc, empty);
            let zero = lw.int_lit(0, span);
            out.push_let(taken, zero);
            let id = lw.next_loop;
            lw.next_loop += 1;
            let place = |slot, ty| Place {
                kind: PlaceKind::Local(slot),
                ty,
                span: lw.span(span),
            };
            let (acc_place, taken_place) = (place(acc, list_ty.clone()), place(taken, Ty::Int));
            let push = lw.mk_mutate(
                Mutation::Push,
                acc_place,
                vec![lw.local(cur, span)],
                Ty::Unit,
                span,
            );
            let one = lw.int_lit(1, span);
            let enough = lw.mk_cmp(
                CmpOp::Ge,
                lw.local(taken, span),
                lw.local(k_slot, span),
                Ty::Bool,
                span,
            );
            lw.cx.scopes.push(scope);
            let mut body = Vec::new();
            for (map, call, to) in step {
                let v = lw.expr(&call, None);
                body.push(if map {
                    Stmt::Let(to, Some(v))
                } else {
                    let skip = Block {
                        stmts: vec![Stmt::Continue(Some(id))],
                    };
                    Stmt::If(v, Block::default(), skip)
                });
            }
            lw.cx.scopes.pop();
            body.extend([
                Stmt::Expr(push),
                Stmt::Assign(taken_place, AssignOp::Binary(BinOp::Add), one),
                Stmt::If(
                    enough,
                    Block {
                        stmts: vec![Stmt::Break(Some(id))],
                    },
                    Block::default(),
                ),
            ]);
            let positive = lw.mk_cmp(
                CmpOp::Gt,
                lw.local(k_slot, span),
                lw.int_lit(0, span),
                Ty::Bool,
                span,
            );
            let walk = Stmt::For(id, x, Iter::Seq(seq, false), Block { stmts: body });
            out.push(Stmt::If(
                positive,
                Block { stmts: vec![walk] },
                Block::default(),
            ));
            out.push_give(lw.local(acc, span));
        });
        taken.ty = ty;
        Some(taken)
    }
}
