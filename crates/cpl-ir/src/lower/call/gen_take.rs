//! `g(a).take(k)` or `x.g(a).take(k)` on a generator: a loop `g` feeds that stops after `k`
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
        let ast::ExprKind::Call { callee, .. } = &base.kind else {
            return None;
        };
        if k.name.is_some() {
            return None;
        }
        let generator = match self.sym(callee) {
            Some(Sym::Fn(id)) => self.sigs[id].is_generator,
            Some(Sym::GenFn(id)) => matches!(
                self.gen_fns[id].decl.ret.as_ref().map(|t| &t.kind),
                Some(ast::TypeKind::Yields(_))
            ),
            // `x.g(a)`: which `g` only lowering the receiver tells.
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
                        out.push(Stmt::Let(
                            held,
                            Some(std::mem::replace(e, lw.local(held, e_span))),
                        ));
                    }
                }
            }
            let elem = match &seq.ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let list_ty = Ty::list(elem.clone());
            ty = list_ty.clone();
            let count = lw.expr(&k.value, Some(&Ty::Int));
            let count = lw.coerce(count, &Ty::Int, k.value.span);
            let k_slot = lw.temp(Ty::Int);
            out.push(Stmt::Let(k_slot, Some(count)));
            let acc = lw.temp(list_ty.clone());
            let taken = lw.temp(Ty::Int);
            lw.cx.mut_slots.insert(acc);
            lw.cx.mut_slots.insert(taken);
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
            out.push(Stmt::Let(acc, Some(empty)));
            let zero = lw.int_lit(0, span);
            out.push(Stmt::Let(taken, Some(zero)));
            let id = lw.next_loop;
            lw.next_loop += 1;
            let x = lw.temp(elem);
            let place = |slot, ty| Place {
                kind: PlaceKind::Local(slot),
                ty,
                span: lw.span(span),
            };
            let (acc_place, taken_place) = (place(acc, list_ty.clone()), place(taken, Ty::Int));
            let push = lw.mk(
                ExprKind::Mutate(Mutation::Push, Box::new(acc_place), vec![lw.local(x, span)]),
                Ty::Unit,
                span,
            );
            let one = lw.int_lit(1, span);
            let enough = lw.mk(
                ExprKind::Cmp(
                    CmpOp::Ge,
                    Box::new(lw.local(taken, span)),
                    Box::new(lw.local(k_slot, span)),
                ),
                Ty::Bool,
                span,
            );
            let body = vec![
                Stmt::Expr(push),
                Stmt::Assign(taken_place, AssignOp::Binary(BinOp::Add), one),
                Stmt::If(
                    enough,
                    Block {
                        stmts: vec![Stmt::Break(Some(id))],
                    },
                    Block::default(),
                ),
            ];
            let positive = lw.mk(
                ExprKind::Cmp(
                    CmpOp::Gt,
                    Box::new(lw.local(k_slot, span)),
                    Box::new(lw.int_lit(0, span)),
                ),
                Ty::Bool,
                span,
            );
            let walk = Stmt::For(id, x, Iter::Seq(seq, false), Block { stmts: body });
            out.push(Stmt::If(
                positive,
                Block { stmts: vec![walk] },
                Block::default(),
            ));
            out.push(Stmt::Give(lw.local(acc, span)));
        });
        taken.ty = ty;
        Some(taken)
    }
}
