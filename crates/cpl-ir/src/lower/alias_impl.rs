//! Methods an `impl` gives a generic alias: `impl Random for Perm<V>` over
//! `type Perm<V> = [V]V` is the impl of `[V]V`, its methods made for the
//! alias's arguments the first time a call `Perm<V>.gen()` reaches them.

use super::*;

impl Lowerer<'_> {
    /// `Alias<T..>.m(args)` with `m` from an `impl` over the alias.
    pub(super) fn alias_impl_call(
        &mut self,
        base: &ast::Expr,
        method: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let ast::ExprKind::Generic {
            base: head,
            args: tys,
        } = &base.kind
        else {
            return None;
        };
        let ast::ExprKind::Name(alias) = &head.kind else {
            return None;
        };
        let fn_id = self.alias_impl_method(alias, tys, &method.text, base.span)?;
        Some(self.method_call(fn_id, None, args, span))
    }

    fn alias_impl_method(
        &mut self,
        alias: &str,
        tys: &[ast::Type],
        method: &str,
        span: Span,
    ) -> Option<FnId> {
        let Some(Sym::GenAlias(gaid)) = self.get_sym(alias) else {
            return None;
        };
        let named = ast::Type {
            kind: ast::TypeKind::Named {
                name: ast::Name {
                    text: alias.into(),
                    span,
                },
                args: tys.to_vec(),
            },
            span,
        };
        let ty = self.resolve_ty(&named);
        if let Some(found) = self
            .prim_methods
            .iter()
            .find(|(t, m, _)| *t == ty && m == method)
        {
            return Some(found.2);
        }
        let gargs = self.resolve_gen_args(tys);
        for idx in 0..self.impls.len() {
            let (m, offset, target, body) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].target,
                self.impls[idx].body,
            );
            let ast::TypeKind::Named { name, args: params } = &target.kind else {
                continue;
            };
            let same =
                matches!(self.mod_syms[m].get(&name.text), Some(Sym::GenAlias(g)) if *g == gaid);
            if !same || params.len() != gargs.len() {
                continue;
            }
            let params: Option<Vec<String>> = params
                .iter()
                .map(|p| match &p.kind {
                    ast::TypeKind::Named { name, args } if args.is_empty() => {
                        Some(name.text.clone())
                    }
                    _ => None,
                })
                .collect();
            let Some(params) = params else {
                continue;
            };
            let Some(f) = body.stmts.iter().find_map(|item| match &item.kind {
                ast::StmtKind::Fn(f) if f.name.text == method && f.generics.is_empty() => Some(f),
                _ => None,
            }) else {
                continue;
            };
            let trait_name = self.impls[idx].trait_name.clone();
            let env: GenEnv = params.into_iter().zip(gargs).collect();
            let fn_id =
                self.prim_method(f, (m, offset), (m, offset), env, &ty, (trait_name, "alias"));
            return Some(fn_id);
        }
        None
    }
}
