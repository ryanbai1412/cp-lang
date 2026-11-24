//! A lambda's type: its parameters from the annotations or the call it is
//! passed to, its result from its body.

use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    /// An unannotated lambda checks as `Unknown`, so a call it is passed to
    /// is not held to a guess; its fact is the signature its body gave it.
    pub(super) fn lambda(&mut self, expr: &Expr) -> Ty {
        let ExprKind::Lambda {
            params,
            param_tys,
            param_mut,
            ret_ty,
            body,
        } = &expr.kind
        else {
            return Ty::Error;
        };
        let mut given = match self
            .lambda_params
            .iter()
            .position(|(at, _)| *at == expr.span)
        {
            Some(i) => self.lambda_params.swap_remove(i).1,
            None => Vec::new(),
        };
        let fields = match given.as_slice() {
            [one] => one.param_fields().filter(|f| f.len() == params.len()),
            _ => None,
        };
        given = fields.filter(|_| params.len() > 1).unwrap_or(given);
        if !given.is_empty() && params.len() != given.len() && param_tys.iter().all(Option::is_none)
        {
            self.lambda_arity_mismatch(params, &given);
            given = vec![Ty::Error; params.len()];
        }
        self.push_scope();
        let mut sig_params = Vec::with_capacity(params.len());
        let any_typed = param_tys.iter().any(Option::is_some);
        for (i, param) in params.iter().enumerate() {
            let ty = param_tys
                .get(i)
                .and_then(|t| t.as_ref())
                .map(|t| self.resolve_type(t))
                .unwrap_or_else(|| given.get(i).cloned().unwrap_or(Ty::Unknown));
            let mutable = param_mut.get(i).copied().unwrap_or(false);
            self.bind(param, &ty, mutable, "param");
            sig_params.push(crate::ty::Param {
                name: String::new(),
                ty,
                mutable,
                has_default: false,
                named_only: false,
                variadic: false,
                lazy: false,
            });
        }
        // A lambda's `return` and trailing value belong to the lambda.
        let declared_ret = ret_ty.as_ref().map(|t| self.resolve_type(t));
        let lam_depth = self.scopes.len();
        self.fns.push((
            String::new(),
            declared_ret.clone().unwrap_or(Ty::Unknown),
            false,
            lam_depth,
        ));
        self.branch += 1;
        let ret = self.infer(body);
        self.branch -= 1;
        self.fns.pop();
        self.pop_scope();
        let final_ret = if let Some(dret) = declared_ret {
            if !matches!(ret, Ty::Unknown | Ty::Error) && !self.fits(&ret, &dret) {
                self.error(
                    codes::TYPE_MISMATCH,
                    body.span,
                    format!("expected `{dret}`, found `{ret}`"),
                );
            }
            dret
        } else {
            ret
        };
        let sig = Ty::Fn {
            params: sig_params,
            ret: Box::new(final_ret),
        };
        if any_typed || ret_ty.is_some() {
            sig
        } else {
            self.expr_types.push((expr.span, sig));
            Ty::Unknown
        }
    }
}
