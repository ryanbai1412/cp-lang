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
}
