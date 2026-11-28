//! A `-> yields T` call made outside a `for` is a value: the call held, walked by each `for` over
//! it, which reruns it from the start (R07.6.2).
use super::expr::block;
use super::*;

impl Cx<'_> {
    /// `gen(args)` as a value: the `@role(recipe)` struct around a closure whose body walks
    /// the call. Its arguments are evaluated now; a place stays a place, so the value is lent from
    /// what it names as long as it is held.
    pub(super) fn recipe(&mut self, mut pre: Vec<TStmt>, call: TExpr) -> TExpr {
        let span = call.span;
        let Some(gen) = self.d.roles().recipe else {
            return block(pre, call);
        };
        let (callee, recv, args, elem) = match call.kind {
            TK::Call { f, recv, args } => {
                let Some((false, elem)) = self.d.f(f.decl).region.clone() else {
                    return block(
                        pre,
                        TExpr {
                            kind: TK::Call { f, recv, args },
                            ..call
                        },
                    );
                };
                let elem = self.result_of(&f, &elem);
                (Callee::Fn(f), recv, args, elem)
            }
            TK::CallTrait {
                tr,
                method,
                recv: Some(recv),
                self_ty,
                args,
                ..
            } => {
                let m = self.d.member(&self.d.t(tr.tr).methods, &method);
                let Some((false, elem)) = m.and_then(|m| self.d.f(m).region.clone()) else {
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                };
                let mut tys = vec![*self_ty];
                tys.extend(tr.args.iter().cloned());
                let elem = Subst {
                    tys: &tys,
                    sets: &[],
                }
                .ty(&elem);
                (Callee::Trait(tr, method), Some(recv), args, elem)
            }
            kind => return block(pre, TExpr { kind, ..call }),
        };
        let mut hold = |c: &mut Self, a: TArg| match a {
            TArg::Value(v)
                if !matches!(v.kind, TK::Var(_))
                    || v.ty.copied(&cpl_next_decls::fields::fields(c.d)) =>
            {
                let ty = v.ty.clone();
                let t = c.bind("$arg", ty.clone());
                pre.push(TStmt::Let(t, v));
                TArg::Value(TExpr {
                    kind: TK::Var(t),
                    ty,
                    span,
                })
            }
            a => a,
        };
        let recv = recv.map(|r| Box::new(hold(self, *r)));
        let args = args.into_iter().map(|a| hold(self, a)).collect();
        self.b.scopes.push(HashMap::new());
        let var = self.bind("$item", elem.clone());
        self.b.scopes.pop();
        let unit = |kind| TExpr {
            kind,
            ty: Ty::Unit,
            span,
        };
        let item = TExpr {
            kind: TK::Var(var),
            ty: elem.clone(),
            span,
        };
        let walk = TStmt::Region {
            callee,
            recv,
            args,
            var,
            lend: false,
            body: unit(TK::Block(vec![TStmt::Yield(item)], None)),
        };
        let id = self.new_lambda(LambdaInfo {
            params: Vec::new(),
            ret: Ty::Unit,
            owner: self.b.decl,
            group: Vec::new(),
            yields: Some(elem.clone()),
            written: Vec::new(),
        });
        let lambda = TExpr {
            kind: TK::Lambda {
                id,
                params: Vec::new(),
                body: Box::new(unit(TK::Block(vec![walk], None))),
            },
            ty: self.lambda_ty(id),
            span,
        };
        let sets: Vec<SetTy> = (0..self.d.s(gen).n_sets).map(|_| self.new_set()).collect();
        let field = Subst {
            tys: std::slice::from_ref(&elem),
            sets: &sets,
        }
        .ty(&self.d.s(gen).fields[0].1);
        let ty = Ty::Struct(gen, vec![elem], sets);
        let field = self.fresh_sets(&field);
        let run = self.coerce(lambda, &field);
        block(
            pre,
            TExpr {
                kind: TK::Struct(vec![run]),
                ty,
                span,
            },
        )
    }

    /// Fn item `f` as a value. A `-> yields T` fn's is the fn that gives the value of a call
    /// held, `(ps) => f(ps)`, so calling it makes a generator value (R07.6.2).
    pub(super) fn fn_item_value(&mut self, f: FnRef, span: Span) -> TExpr {
        let generator = matches!(self.d.f(f.decl).region, Some((false, _)));
        let (ps, _) = self.clo_sig(&CloRef::Item(f.clone()));
        if !generator
            || self.d.roles().recipe.is_none()
            || ps.iter().any(|p| matches!(p, Ty::Mut(_)))
        {
            return TExpr {
                kind: TK::FnItem(f.clone()),
                ty: Ty::Clo(Box::new(CloRef::Item(f))),
                span,
            };
        }
        let id = self.new_lambda(LambdaInfo {
            params: ps.clone(),
            ret: Ty::Unit,
            owner: self.b.decl,
            group: Vec::new(),
            yields: None,
            written: Vec::new(),
        });
        self.b.scopes.push(HashMap::new());
        let mut ids = Vec::new();
        let mut args = Vec::new();
        for (i, t) in ps.iter().enumerate() {
            let v = self.bind(&format!("$param{i}"), t.clone());
            let (VarRef::Local(l) | VarRef::Global(l)) = v;
            ids.push(l);
            args.push(TArg::Value(TExpr {
                kind: TK::Var(v),
                ty: t.clone(),
                span,
            }));
        }
        let ty = self.ret_ty(&f);
        let call = TExpr {
            kind: TK::Call {
                f,
                recv: None,
                args,
            },
            ty,
            span,
        };
        let body = self.recipe(Vec::new(), call);
        self.b.scopes.pop();
        self.lambdas[id.0].ret = body.ty.clone();
        TExpr {
            kind: TK::Lambda {
                id,
                params: ids,
                body: Box::new(body),
            },
            ty: self.lambda_ty(id),
            span,
        }
    }

    /// The element a `for` over a generator value walks.
    pub(super) fn recipe_walk(&mut self, e: &TExpr) -> Option<Ty> {
        match self.shallow(&e.ty) {
            Ty::Struct(s, targs, _) if Some(s) == self.d.roles().recipe => targs.first().cloned(),
            _ => None,
        }
    }

    /// `for x in g` with `g` a generator value: run its closure, with the body at each `yield`.
    pub(super) fn value_walk(
        &mut self,
        elem: Ty,
        g: TExpr,
        each: impl FnOnce(&mut Self, Ty) -> (VarRef, TExpr),
        out: &mut Vec<TStmt>,
    ) {
        let Ty::Struct(s, targs, sets) = self.shallow(&g.ty) else {
            return;
        };
        let fty = Subst {
            tys: &targs,
            sets: &sets,
        }
        .ty(&self.d.s(s).fields[0].1);
        let span = g.span;
        let run = TExpr {
            kind: TK::Field(Box::new(g), 0),
            ty: fty,
            span,
        };
        let (var, body) = each(self, elem);
        out.push(TStmt::Region {
            callee: Callee::Value(Box::new(run)),
            recv: None,
            args: Vec::new(),
            var,
            lend: false,
            body,
        });
    }
}
