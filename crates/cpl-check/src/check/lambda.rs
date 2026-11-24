//! A lambda's type: its parameters from the annotations or the call it is
//! passed to, its result from its body.

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl Checker<'_> {
    /// R08.2.11: a lambda with unwritten parameter types (`+` is one) given
    /// to a type parameter nothing else fixes takes `Int`s, as a literal
    /// does; one whose body needs other types is an error, never unknown.
    pub(super) fn lambda_defaults(
        &mut self,
        arg: &Arg,
        expected: &Ty,
        callee: &str,
        takes: &[crate::ty::Param],
    ) -> bool {
        let (
            ExprKind::Lambda {
                params, param_tys, ..
            },
            Ty::Param(open),
        ) = (&arg.value.kind, expected)
        else {
            return false;
        };
        if params.is_empty() || param_tys.iter().any(Option::is_some) {
            return false;
        }
        // A bound or another parameter may fix `open`; only an open one defaults.
        let names_open = |ty: &Ty| {
            let mut found = false;
            ty.map(&mut |t| {
                found |= matches!(t, Ty::Param(n) if n == open);
                None
            });
            found
        };
        let bounded = match self.global(callee).as_deref() {
            Some(Def::Fn {
                bounds,
                trait_bounds,
                ..
            }) => {
                bounds.iter().any(|(p, _)| p == open) || trait_bounds.iter().any(|(p, _)| p == open)
            }
            _ => false,
        } || (self.call_bound_args.iter().flatten()).any(|(p, ..)| p == open);
        let elsewhere = (takes.iter())
            .filter(|p| !matches!(&p.ty, Ty::Param(n) if n == open))
            .any(|p| names_open(&p.ty));
        if bounded || elsewhere {
            return false;
        }
        let at = arg.value.span;
        let ints = vec![Ty::Int; params.len()];
        let checks = self.quietly(|c| {
            c.lambda_params.push((at, ints.clone()));
            c.infer(&arg.value);
            !(c.sink.diagnostics().iter()).any(|d| d.severity == cpl_diag::Severity::Error)
        });
        if !checks {
            let text = self.quote(at);
            let diag = msgs::check::FN_VALUE_NO_DEFAULT
                .error(at, &[&callee, open, &text])
                .with_help("write the parameters' types, as in `(a: Str, b: Str) => ..`");
            self.sink.push(diag);
            return false;
        }
        self.lambda_params.push((at, ints));
        true
    }

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
                self.report(
                    body.span,
                    msgs::check::TYPE_MISMATCH_EXPECTED_FOUND,
                    &[&dret, &ret],
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
            self.exprs.types.push((expr.span, sig));
            Ty::Unknown
        }
    }
}
