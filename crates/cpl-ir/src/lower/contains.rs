use super::*;

impl Lowerer<'_> {
    /// `x in c`: a range, a domain, or anything that holds values.
    pub(super) fn contains(&mut self, item: &ast::Expr, container: &ast::Expr, span: Span) -> Expr {
        let mut inner = container;
        while let ast::ExprKind::Paren(e) = &inner.kind {
            inner = e;
        }
        let between = |lw: &mut Self, value: Expr, lo: Expr, hi: Expr, closed: bool| {
            // `lo <= x and x < hi`, with `x` evaluated once.
            let mut stmts = Vec::new();
            let value = lw.hold(value, &mut stmts);
            let ge = lw.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(lo), Box::new(value.clone())),
                Ty::Bool,
                span,
            );
            let lt = lw.mk(
                ExprKind::Cmp(
                    if closed { CmpOp::Le } else { CmpOp::Lt },
                    Box::new(value),
                    Box::new(hi),
                ),
                Ty::Bool,
                span,
            );
            let both = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(lt)),
                Ty::Bool,
                span,
            );
            if stmts.is_empty() {
                return both;
            }
            stmts.push(Stmt::Give(both));
            lw.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
        };
        match &inner.kind {
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } if self.probe_ty_silent(lo) != Some(Ty::Float) => {
                let value = self.expect(item, &Ty::Int);
                let lo = self.expect(lo, &Ty::Int);
                let hi = self.expect(hi, &Ty::Int);
                between(self, value, lo, hi, *inclusive)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && (matches!(self.gen_env.get(name), Some(GenArg::Domain(_)))
                        || matches!(self.get_sym(name), Some(Sym::Domain(_)))) =>
            {
                let id = match self.gen_env.get(name) {
                    Some(GenArg::Domain(id)) => *id,
                    _ => match self.get_sym(name) {
                        Some(Sym::Domain(id)) => id,
                        _ => unreachable!(),
                    },
                };
                let value = self.expect(item, &Ty::Int);
                let lo = self.mk(ExprKind::DomainLo(id), Ty::Int, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(len)),
                    Ty::Int,
                    span,
                );
                between(self, value, lo, hi, false)
            }
            _ => {
                let holder = self.expr(container, None);
                if let Ty::Struct(sid) = holder.ty {
                    let s = &self.structs[sid];
                    let plain = s.methods.get("has").or_else(|| s.methods.get("contains"));
                    let generic = s
                        .gen_methods
                        .get("has")
                        .or_else(|| s.gen_methods.get("contains"));
                    let (plain, generic) = (plain.copied(), generic.copied());
                    let arg = [ast::Arg {
                        name: None,
                        mutable: false,
                        value: item.clone(),
                    }];
                    if let Some(fn_id) = plain {
                        return self.method_call(fn_id, Some((container, holder)), &arg, span);
                    }
                    if let Some(gfid) = generic {
                        let receiver = Some(Arg::Value(holder));
                        return self.call_gen_fn(gfid, Vec::new(), receiver, &arg, span);
                    }
                }
                if matches!(&holder.ty, Ty::Range(e) if **e == Ty::Float) {
                    let value = self.expect(item, &Ty::Float);
                    let mut stmts = Vec::new();
                    let holder = self.hold(holder, &mut stmts);
                    let lo = self.mk(
                        ExprKind::Builtin(Builtin::RangeLo, vec![holder.clone()]),
                        Ty::Float,
                        span,
                    );
                    let hi = self.mk(
                        ExprKind::Builtin(Builtin::RangeHi, vec![holder]),
                        Ty::Float,
                        span,
                    );
                    let found = between(self, value, lo, hi, false);
                    if stmts.is_empty() {
                        return found;
                    }
                    stmts.push(Stmt::Give(found));
                    return self.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span);
                }
                let elem = match &holder.ty {
                    Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem, _)
                    | Ty::Set(elem)
                    | Ty::HashSet(elem)
                    | Ty::Range(elem) => (**elem).clone(),
                    Ty::Array { elem, .. } => (**elem).clone(),
                    Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
                    // `"an" in s`: a `Str` is looked for as a part of the text.
                    Ty::Str | Ty::StrView
                        if matches!(self.probe_ty_silent(item), Some(Ty::Str | Ty::StrView)) =>
                    {
                        Ty::Str
                    }
                    Ty::Str | Ty::StrView => Ty::Char,
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!("`in` can't look inside a `{}`", self.show(other));
                        self.error(codes::BAD_OPERANDS, span, message);
                        return self.err_expr(span);
                    }
                };
                let value = self.expect(item, &elem);
                self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![holder, value]),
                    Ty::Bool,
                    span,
                )
            }
        }
    }
}
