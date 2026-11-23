//! `Dsu<0..n>`: a range written where a struct's type argument is used as a
//! domain is an anonymous domain over that range (decision 255), as an
//! inline `(V, V)` is an anonymous product domain (decision 159).

use super::*;

impl<'a> Lowerer<'a> {
    /// The anonymous domain for the range `ty` passed as `decl`'s type argument
    /// `at`, when that parameter is a domain. One domain per range text in a
    /// module, so `var d: Dsu<0..n> = Dsu<0..n>()` names one type; its count is
    /// set where it is written, as `domain V = 0..n` would be.
    pub(super) fn anon_range_domain(
        &mut self,
        decl: &ast::StructDecl,
        at: usize,
        ty: &ast::Type,
    ) -> Option<DomainId> {
        let ast::TypeKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
        } = &ty.kind
        else {
            return None;
        };
        let param = decl.generics.get(at)?;
        if !struct_uses_param_as_domain(decl, &param.text) {
            return None;
        }
        let key = (self.cur_mod, self.source(ty.span));
        let d = match self.anon_range_domains.get(&key) {
            Some(&d) => d,
            None => {
                let d = self.domains.len();
                self.domains.push(format!("Range{d}"));
                self.anon_range_domains.insert(key, d);
                d
            }
        };
        let def = ast::Expr {
            kind: ast::ExprKind::Range {
                lo: Some(lo.clone()),
                hi: Some(hi.clone()),
                inclusive: *inclusive,
                step: None,
            },
            span: ty.span,
        };
        let mut defs = Vec::new();
        self.init_domain_id(d, &def, &mut defs);
        self.cx.pending.extend(defs);
        Some(d)
    }
}

pub(super) fn struct_uses_param_as_domain(decl: &ast::StructDecl, name: &str) -> bool {
    fn ty_uses_domain(ty: &ast::Type, name: &str) -> bool {
        match &ty.kind {
            ast::TypeKind::Array { index, elem } => {
                index.iter().any(|i| ty_mentions(i, name)) || ty_uses_domain(elem, name)
            }
            ast::TypeKind::Named { args, .. } => args.iter().any(|a| ty_uses_domain(a, name)),
            ast::TypeKind::OwnDomain { elem, .. }
            | ast::TypeKind::Optional(elem)
            | ast::TypeKind::Yields(elem) => ty_uses_domain(elem, name),
            ast::TypeKind::Tuple(items) => items.iter().any(|i| ty_uses_domain(i, name)),
            ast::TypeKind::Record(fields) => fields.iter().any(|(_, f)| ty_uses_domain(f, name)),
            ast::TypeKind::Fn { params, ret } => {
                params.iter().any(|p| ty_uses_domain(p, name)) || ty_uses_domain(ret, name)
            }
            _ => false,
        }
    }
    fn expr_uses_domain(e: &ast::Expr, name: &str) -> bool {
        match &e.kind {
            ast::ExprKind::Field { base, name: m }
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == name)
                    && matches!(
                        m.text.as_str(),
                        "count"
                            | "len"
                            | "all"
                            | "first"
                            | "last"
                            | "start"
                            | "end"
                            | "cuts"
                            | "mask"
                            | "size"
                            | "K"
                            | "nth"
                            | "at"
                    ) =>
            {
                true
            }
            ast::ExprKind::Field { base, .. } => expr_uses_domain(base, name),
            ast::ExprKind::Call { callee, args } => {
                expr_uses_domain(callee, name)
                    || args.iter().any(|a| expr_uses_domain(&a.value, name))
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                expr_uses_domain(lhs, name) || expr_uses_domain(rhs, name)
            }
            ast::ExprKind::Unary { expr, .. }
            | ast::ExprKind::Paren(expr)
            | ast::ExprKind::Unwrap(expr) => expr_uses_domain(expr, name),
            ast::ExprKind::Block(b) => b.stmts.iter().any(|s| stmt_uses_domain(s, name)),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches
                    .iter()
                    .any(|(_, b)| b.stmts.iter().any(|s| stmt_uses_domain(s, name)))
                    || otherwise
                        .as_ref()
                        .is_some_and(|b| b.stmts.iter().any(|s| stmt_uses_domain(s, name)))
            }
            ast::ExprKind::Lambda { body, .. } => expr_uses_domain(body, name),
            ast::ExprKind::List(items) | ast::ExprKind::Tuple(items) => {
                items.iter().any(|i| expr_uses_domain(i, name))
            }
            ast::ExprKind::Index { base, indices } => {
                expr_uses_domain(base, name) || indices.iter().any(|i| expr_uses_domain(i, name))
            }
            _ => false,
        }
    }
    fn stmt_uses_domain(s: &ast::Stmt, name: &str) -> bool {
        match &s.kind {
            ast::StmtKind::Fn(f) => {
                f.params
                    .iter()
                    .any(|p| p.ty.as_ref().is_some_and(|t| ty_uses_domain(t, name)))
                    || f.ret.as_ref().is_some_and(|t| ty_uses_domain(t, name))
                    || match &f.body {
                        ast::FnBody::Expr(e) => expr_uses_domain(e, name),
                        ast::FnBody::Block(b) => b.stmts.iter().any(|s| stmt_uses_domain(s, name)),
                        _ => false,
                    }
            }
            ast::StmtKind::Let { ty, init, .. } => {
                ty.as_ref().is_some_and(|t| ty_uses_domain(t, name))
                    || init.as_ref().is_some_and(|e| expr_uses_domain(e, name))
            }
            ast::StmtKind::Expr(e) => expr_uses_domain(e, name),
            ast::StmtKind::Return(Some(e)) | ast::StmtKind::Yield { value: e, .. } => {
                expr_uses_domain(e, name)
            }
            ast::StmtKind::For { iter, body, .. } => {
                matches!(&iter.kind, ast::ExprKind::Name(n) if n == name)
                    || expr_uses_domain(iter, name)
                    || body.stmts.iter().any(|b| stmt_uses_domain(b, name))
            }
            ast::StmtKind::While { body, .. } => {
                body.stmts.iter().any(|b| stmt_uses_domain(b, name))
            }
            ast::StmtKind::Assign { target, value, .. } => {
                expr_uses_domain(target, name) || expr_uses_domain(value, name)
            }
            _ => false,
        }
    }
    decl.fields.iter().any(|f| ty_uses_domain(&f.ty, name))
        || decl.methods.iter().any(|m| stmt_uses_domain(m, name))
}

impl<'a> Lowerer<'a> {
    /// Binds the generics of an array parameter's index `[I, J]` from the
    /// dimensions it was given.
    pub(super) fn unify_array_dims(
        &self,
        index: &[ast::Type],
        dims: &[Dim],
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        // `[V, V]T` given `[A, B, A, B]T`, an `[P, P]T` over `type P = (A, B)`.
        let per = dims.len() / index.len().max(1);
        if per > 1 && dims.len() == per * index.len() {
            let products: Option<Vec<_>> =
                dims.chunks(per).map(|c| self.product_of_grid(c)).collect();
            if let Some(products) = products {
                for (ix, p) in index.iter().zip(products) {
                    self.unify_ast_ty(ix, &Ty::Pos(p), tpl_mod, gen_names, env);
                }
                return;
            }
        }
        for (ix, d) in index.iter().zip(dims) {
            match d {
                Dim::Pos(did) | Dim::Sym(did) => {
                    self.unify_ast_ty(ix, &Ty::Pos(*did), tpl_mod, gen_names, env);
                }
                Dim::Bound(did) => {
                    self.unify_ast_ty(ix, &Ty::Bound(*did), tpl_mod, gen_names, env);
                }
                Dim::Enum(e) => {
                    self.unify_ast_ty(ix, &Ty::Enum(*e), tpl_mod, gen_names, env);
                }
                Dim::Int | Dim::IntFrom(_) => {
                    self.unify_ast_ty(ix, &Ty::Int, tpl_mod, gen_names, env);
                }
            }
        }
    }

    /// A fieldless enum with variants, which is a domain (R04.2.5).
    pub(super) fn unit_enum(&self, id: EnumId) -> bool {
        let variants = &self.enums[id].variants;
        !variants.is_empty() && variants.iter().all(|(_, fields)| fields.is_empty())
    }

    /// `D.count`, `D.first`, `D.filter(p)`, …: a domain is the `Seq` of its
    /// positions (R04.2.20), so a fieldless enum's members that aren't
    /// variants read off `[D.A, D.B, …]`.
    pub(super) fn enum_seq(
        &mut self,
        base: &ast::Expr,
        id: EnumId,
        name: &ast::Name,
        args: Option<&[ast::Arg]>,
        span: Span,
    ) -> Expr {
        let variants: Vec<String> = self.enums[id]
            .variants
            .iter()
            .map(|(v, _)| v.clone())
            .collect();
        if name.text == "count" && args.is_none() {
            return self.int_lit(variants.len() as i64, span);
        }
        let field = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(ast::Expr::variant_list(base, &variants)),
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
