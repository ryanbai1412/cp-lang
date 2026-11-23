//! A lambda's annotated parameters take part in a generic call's inference:
//! `undoing(s, (mut d, e: (V, V)) => ..)` binds `T` in `add: (mut S, T) -> ()`
//! to `(V, V)` before any default applies.

use super::*;

impl<'a> Lowerer<'a> {
    /// Unify each annotated parameter of the lambda `arg` with its slot in the fn type `pat`.
    pub(super) fn unify_lambda_params(
        &mut self,
        pat: &ast::Type,
        arg: &ast::Expr,
        tpl_mod: ModId,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        let (ast::TypeKind::Fn { params, .. }, ast::ExprKind::Lambda { param_tys, .. }) =
            (&pat.kind, &arg.kind)
        else {
            return;
        };
        if params.len() != param_tys.len() {
            return;
        }
        for (slot, ann) in params.iter().zip(param_tys) {
            let Some(ann) = ann else { continue };
            let slot = match &slot.kind {
                ast::TypeKind::Mut(inner) => inner,
                _ => slot,
            };
            let saved = self.diags.len();
            let ty = self.resolve_ty(ann);
            self.diags.truncate(saved);
            self.unify_ast_ty(slot, &ty, tpl_mod, gen_names, env);
        }
    }
    /// The fn type a one-parameter lambda passed as `g: Next<V>` (any trait whose first argument
    /// is the type of what the lambda takes) has, once `V` is known: `scc::<N>(u => adj[u])`.
    pub(super) fn trait_lambda_hint(
        &mut self,
        pty: &ast::Type,
        arg: &ast::Expr,
        inferred: &HashMap<String, GenArg>,
    ) -> Option<Ty> {
        let ast::ExprKind::Lambda { params, body, .. } = &arg.kind else {
            return None;
        };
        let ast::TypeKind::Named { args: targs, .. } = &pty.kind else {
            return None;
        };
        let Some(ast::Type {
            kind: ast::TypeKind::Named { name, args },
            ..
        }) = targs.first()
        else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        let pty = match inferred.get(&name.text)? {
            GenArg::Domain(d) => Ty::Pos(*d),
            GenArg::Ty(t) => t.clone(),
            _ => return None,
        };
        let param = self.one_param(params, Some(&pty))?;
        let ret = self.probe_lambda_ret(
            std::slice::from_ref(&param),
            std::slice::from_ref(&pty),
            body,
        )?;
        Some(Ty::Fn(vec![pty], Box::new(ret)))
    }

    /// The one parameter a lambda takes where a `want` is wanted. Several
    /// parameters unpack a tuple or product `want` with as many parts, as one
    /// tuple pattern: `dijkstra((v, k) => …)` over `(V, K)` (decision 159).
    pub(super) fn one_param(
        &self,
        params: &[ast::Pattern],
        want: Option<&Ty>,
    ) -> Option<ast::Pattern> {
        let parts = match want {
            Some(Ty::Tuple(items)) => items.len(),
            Some(Ty::Pos(d)) => self.product_domains.get(d).map_or(1, Vec::len),
            _ => 1,
        };
        match params {
            [param] => Some(param.clone()),
            [first, ..] if params.len() == parts => Some(ast::Pattern {
                kind: ast::PatternKind::Tuple(params.to_vec()),
                span: first.span,
            }),
            _ => None,
        }
    }
}
