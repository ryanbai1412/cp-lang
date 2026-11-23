//! Library domains in IR: a struct with `impl Index` is a domain whose
//! positions are its values, indexed through `ord` and walked through `nth`.

use super::*;

impl<'a> Lowerer<'a> {
    /// `S.nth(k)` from user code: `k` must lie in `0..S.count`. Walks call the
    /// impl's `nth` directly, unchecked.
    pub(super) fn checked_nth(&mut self, dom: DomainId, call: Expr, span: Span) -> Expr {
        let Expr {
            kind: ExprKind::Call(fid, mut args),
            ty,
            ..
        } = call
        else {
            return call;
        };
        if let Some(Arg::Value(k)) = args.first_mut() {
            let value = std::mem::replace(k, self.int_lit(0, span));
            *k = self.ord_in_range(dom, value);
        }
        self.mk(ExprKind::Call(fid, args), ty, span)
    }

    /// `k`, trapping unless it is an ordinal of the domain, `0..T.count`:
    /// the same check `V.at(k)` makes.
    fn ord_in_range(&mut self, dom: DomainId, k: Expr) -> Expr {
        let pos = self.cast(Cast::FromOrd(dom), k, Ty::Pos(dom));
        self.cast(Cast::Ord, pos, Ty::Int)
    }
}

impl<'a> Lowerer<'a> {
    /// `T.first`, `T.rev`, `T.find(p)`, …: an `impl Index` type is the `Seq`
    /// of its walk (R04.2.20), read off `[x for x in T]`. `first` and `last`
    /// call `nth` once instead.
    pub(super) fn library_domain_seq(
        &mut self,
        base: &ast::Expr,
        sid: StructId,
        name: &ast::Name,
        args: Option<&[ast::Arg]>,
        span: Span,
    ) -> Expr {
        let ends = matches!(name.text.as_str(), "first" | "last") && args.is_none();
        if let (true, Some(dom), Some(&nth)) = (
            ends,
            self.structs[sid].index_domain,
            self.structs[sid].methods.get("nth"),
        ) {
            let opt_ty = Ty::opt(Ty::Struct(sid));
            let len = self.mk(ExprKind::DomainLen(dom), Ty::Int, span);
            let k = match name.text.as_str() {
                "first" => self.int_lit(0, span),
                _ => self.int_add(len.clone(), -1),
            };
            let zero = self.int_lit(0, span);
            let empty = self.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let item = self.mk(
                ExprKind::Call(nth, vec![Arg::Value(k)]),
                Ty::Struct(sid),
                span,
            );
            let some = self.mk(ExprKind::Some(Box::new(item)), opt_ty.clone(), span);
            let none = self.mk(ExprKind::None, opt_ty.clone(), span);
            return self.mk(
                ExprKind::If(Box::new(empty), Box::new(none), Box::new(some)),
                opt_ty,
                span,
            );
        }
        let field = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(ast::Expr::walk_list(base)),
                name: name.clone(),
            },
            span,
        };
        let Some(args) = args else {
            return self.expr(&field, None);
        };
        let call = ast::ExprKind::Call {
            callee: Box::new(field),
            args: args.to_vec(),
        };
        self.expr(&ast::Expr { kind: call, span }, None)
    }
}

impl<'a> Lowerer<'a> {
    /// The `impl Index` struct the type expression `e` names: `T`, a generic's
    /// argument, or an instance `Sub<V>`.
    pub(super) fn index_struct_of(&mut self, e: &ast::Expr) -> Option<StructId> {
        match &e.kind {
            ast::ExprKind::Name(name) if self.lookup(name).is_none() => self.index_struct(name),
            ast::ExprKind::Generic { base, args } => {
                let ast::ExprKind::Name(name) = &base.kind else {
                    return None;
                };
                let Some(Sym::GenStruct(gid)) = self.get_sym(name) else {
                    return None;
                };
                let gargs = self.resolve_gen_args_for_struct(gid, args);
                let sid = self.instantiate_struct(gid, gargs);
                self.structs[sid].index_domain.is_some().then_some(sid)
            }
            _ => None,
        }
    }

    /// The `impl Index` struct `name` names, directly or as a generic's argument.
    pub(super) fn index_struct(&self, name: &str) -> Option<StructId> {
        let sid = match self.gen_env.get(name) {
            Some(GenArg::Ty(Ty::Struct(sid))) => *sid,
            Some(_) => return None,
            None => match self.get_sym(name) {
                Some(Sym::Struct(sid)) => sid,
                _ => return None,
            },
        };
        self.structs[sid].index_domain.is_some().then_some(sid)
    }
}
