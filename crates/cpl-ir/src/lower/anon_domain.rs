//! `Dsu<0..n>`: a range written where a struct's type argument is used as a
//! domain is an anonymous domain over that range (decision 255), as an
//! inline `(V, V)` is an anonymous product domain (decision 159).

use super::*;

impl<'a> Lowerer<'a> {
    /// Unify argument `arg`'s type with its parameter's. A bare generic that `decl` uses as a
    /// domain and that stands where the argument's type is a range (`List<R>` given a
    /// `List<0..2>`) is bound to that range, so it is the anonymous domain the range is
    /// (R04.4.30). Any other generic keeps the range's `Int`, so `scan<D, T>` over `[I]-9..=9`
    /// still names `Monoid<T>`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn unify_arg(
        &mut self,
        decl: &ast::FnDecl,
        pty: &ast::Type,
        concrete: &Ty,
        arg: &ast::Arg,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        self.intern_grid_products(pty, concrete);
        self.unify_ast_ty(pty, concrete, tpl_mod, gen_names, env);
        if !matches!(arg.value.kind, ast::ExprKind::Name(_)) {
            return;
        }
        let lowered = self.probe_expr(&arg.value, None);
        if let Some(info) = self.expr_range(&lowered) {
            let domains: Vec<String> = gen_names
                .iter()
                .filter(|g| fn_uses_domain(decl, g))
                .cloned()
                .collect();
            bind_range_generics(pty, &info, &domains, env);
        }
    }

    /// Whether generic `name` is bound to a range type with constant ends.
    pub(super) fn range_generic(&self, name: &str) -> bool {
        matches!(
            self.gen_env.get(name),
            Some(GenArg::Range(Some(_), Some(_)))
        )
    }

    /// `[R]T` for a generic `R`: when `R` is a range type, its `Int`s are the positions
    /// (R04.4.30); anything else is not a domain.
    pub(super) fn generic_dim(&mut self, name: &str, arg: &GenArg, span: Span) -> Option<DimSpec> {
        if let GenArg::Range(Some(lo), Some(hi)) = *arg {
            let len = Box::new(self.int_lit((hi - lo + 1).max(0), span));
            return Some(match lo {
                0 => DimSpec::Int(len),
                _ => DimSpec::IntFrom(lo, len),
            });
        }
        // `S` inferred as `(R, Subset<T>)`: a tuple of factors is their product domain.
        if let GenArg::Ty(Ty::Tuple(items)) = arg {
            let factors = items
                .iter()
                .map(|t| self.ty_to_product_factor(t).map(|f| (None, f)))
                .collect::<Option<Vec<_>>>()
                .filter(|f| f.len() > 1);
            if let Some(factors) = factors {
                let d = self.intern_anon_product_domain(factors);
                let mut defs = Vec::new();
                self.def_anon_product_domain(d, span, &mut defs);
                self.cx.pending.extend(defs);
                return Some(DimSpec::Pos(d));
            }
        }
        self.not_a_domain(name, arg, span)
    }

    /// Whether generic `at` of `decl` is a domain: the struct or one of its
    /// `impl` blocks (`impl Index for Pair<V>: let count = V.count ...`)
    /// uses it as one.
    pub(super) fn uses_param_as_domain(&self, decl: &ast::StructDecl, at: usize) -> bool {
        if decl
            .generics
            .get(at)
            .is_some_and(|g| struct_uses_param_as_domain(decl, &g.text))
        {
            return true;
        }
        self.impls.iter().any(|imp| {
            let ast::TypeKind::Named { name, args } = &imp.target.kind else {
                return false;
            };
            let Some(ast::TypeKind::Named { name: param, .. }) = args.get(at).map(|a| &a.kind)
            else {
                return false;
            };
            name.text == decl.name.text
                && imp
                    .body
                    .stmts
                    .iter()
                    .any(|st| stmt_uses_domain(st, &param.text))
        })
    }

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
        if !self.uses_param_as_domain(decl, at) {
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
                        | "nth"
                        | "at"
                ) =>
        {
            true
        }
        ast::ExprKind::Field { base, .. } => expr_uses_domain(base, name),
        ast::ExprKind::Call { callee, args } => {
            expr_uses_domain(callee, name) || args.iter().any(|a| expr_uses_domain(&a.value, name))
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
        ast::StmtKind::Fn(f) => fn_uses_domain(f, name),
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
        ast::StmtKind::While { body, .. } => body.stmts.iter().any(|b| stmt_uses_domain(b, name)),
        ast::StmtKind::Assign { target, value, .. } => {
            expr_uses_domain(target, name) || expr_uses_domain(value, name)
        }
        _ => false,
    }
}
/// Whether fn `f` uses `name` as a domain in its signature or body.
pub(super) fn fn_uses_domain(f: &ast::FnDecl, name: &str) -> bool {
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

pub(super) fn struct_uses_param_as_domain(decl: &ast::StructDecl, name: &str) -> bool {
    decl.fields.iter().any(|f| ty_uses_domain(&f.ty, name))
        || decl.methods.iter().any(|m| stmt_uses_domain(m, name))
}

impl<'a> Lowerer<'a> {
    /// Binds the generics of an array parameter's index `[I, J]` from the
    /// dimensions it was given.
    /// A list given where a `[D]T` is asked for; a view keys `D` by its source's dimension.
    pub(super) fn unify_view_dim(
        &self,
        index: &[ast::Type],
        concrete: &Ty,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        if let Ty::ListView(_, dim) | Ty::Keyed(_, dim) = concrete {
            self.unify_array_dims(index, std::slice::from_ref(dim), tpl_mod, gen_names, env);
        }
    }

    /// A view of a domain-keyed array (`a[V(2)..=V(4)]`, `a.rev`) given for `[D]T` keeps
    /// its source's domain: `D` is that domain, not a fresh one for the view's positions.
    pub(super) fn bind_view_domain(
        &mut self,
        arg: &ast::Expr,
        name: &str,
        inferred: &mut GenEnv,
    ) -> bool {
        let Some(Ty::ListView(_, Dim::Pos(d)) | Ty::Keyed(_, Dim::Pos(d))) =
            self.probe_ty_silent(arg)
        else {
            return false;
        };
        inferred.insert(name.to_string(), GenArg::Domain(d));
        true
    }

    /// One end of `domain d = lo..hi`, as the `Int` it counts from; a `Char` end makes `d` a
    /// char domain.
    pub(super) fn domain_end(&mut self, id: DomainId, end: &ast::Expr) -> Expr {
        let e = self.expr(end, None);
        if e.ty != Ty::Char {
            return self.coerce(e, &Ty::Int, end.span);
        }
        self.char_domains.insert(id);
        self.cast(Cast::CharToInt, e, Ty::Int)
    }

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

    /// `d.next`/`d.prev` on a fieldless enum's variant: the adjacent
    /// variant, or `None` past either end, as for any position.
    pub(super) fn enum_step(&mut self, id: EnumId, value: Expr, next: bool, span: Span) -> Expr {
        let opt_ty = Ty::opt(Ty::Enum(id));
        let n = self.enums[id].variants.len() as i64;
        let ord = self.cast(Cast::EnumOrd(id), value, Ty::Int);
        self.do_expr(Some(&opt_ty.clone()), span, |lw, out| {
            let s = lw.temp(Ty::Int);
            let moved = lw.int_add(ord, if next { 1 } else { -1 });
            out.push(Stmt::Let(s, Some(moved)));
            let sv = lw.local(s, span);
            let edge = lw.int_lit(if next { n } else { -1 }, span);
            let off = lw.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(sv.clone()), Box::new(edge)),
                Ty::Bool,
                span,
            );
            let variant = lw.cast(Cast::OrdToEnum(id), sv, Ty::Enum(id));
            let some = lw.mk(ExprKind::Some(Box::new(variant)), opt_ty.clone(), span);
            let none = lw.mk(ExprKind::None, opt_ty.clone(), span);
            let res = lw.mk(
                ExprKind::If(Box::new(off), Box::new(none), Box::new(some)),
                opt_ty,
                span,
            );
            out.push(Stmt::Give(res));
        })
    }
}

/// Bind each bare generic of `pty` that stands where `info` records a constant range.
fn bind_range_generics(pty: &ast::Type, info: &RangeInfo, gen_names: &[String], env: &mut GenEnv) {
    match (&pty.kind, info) {
        (ast::TypeKind::Named { name, args }, RangeInfo::Scalar(b))
            if args.is_empty() && gen_names.contains(&name.text) =>
        {
            if let (Some(lo), Some(hi), None | Some(GenArg::Ty(Ty::Int))) =
                (b.lo_const, b.hi_const, env.get(&name.text))
            {
                env.insert(name.text.clone(), GenArg::Range(Some(lo), Some(hi)));
            }
        }
        (ast::TypeKind::Named { args, .. }, RangeInfo::Elem(inner)) if args.len() == 1 => {
            bind_range_generics(&args[0], inner, gen_names, env);
        }
        (
            ast::TypeKind::Array { elem, .. } | ast::TypeKind::OwnDomain { elem, .. },
            RangeInfo::Elem(inner),
        ) => bind_range_generics(elem, inner, gen_names, env),
        (ast::TypeKind::Optional(inner_ty), RangeInfo::Opt(inner)) => {
            bind_range_generics(inner_ty, inner, gen_names, env);
        }
        _ => {}
    }
}
