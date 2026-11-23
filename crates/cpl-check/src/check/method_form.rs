//! `xs.f(args)` on a library `fn f(self: S, ...)` checks as `f(xs, args)`.

use super::*;
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl<'a> Checker<'a> {
    /// The type of `base.name(args)` when `name` is a top-level fn taking
    /// `self`, found by checking the free form `name(base, args)`, when that
    /// checks cleanly to a plain value type. Its resolution, and its hover
    /// where the member has none, are the free form's in any case.
    pub(super) fn method_form_call(
        &mut self,
        call: &Expr,
        callee: &Expr,
        args: &[Arg],
    ) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let module = matches!(&base.kind, ExprKind::Name(m) if self.local(m).is_none() && self.env.is_module(m));
        if module || self.local(&name.text).is_some() {
            return None;
        }
        let takes_self = |def: &Def| {
            matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
                if params.first().is_some_and(|p| p.name == "self" && !p.named_only))
        };
        if !self.global(&name.text).is_some_and(|d| takes_self(&d)) {
            return None;
        }
        let mut free_args = vec![Arg {
            name: None,
            mutable: false,
            value: (**base).clone(),
        }];
        free_args.extend(args.iter().cloned());
        let free = Expr {
            kind: ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Name(name.text.clone()),
                    span: name.span,
                }),
                args: free_args,
            },
            span: call.span,
        };
        let (ty, hover, resolved) = self.quietly(|c| {
            let ty = c.infer(&free);
            let erred = c
                .sink
                .diagnostics()
                .iter()
                .any(|d| d.severity == cpl_diag::Severity::Error);
            let ty = if erred { Ty::Unknown } else { ty };
            let hover = c.hovers.iter().rev().find(|h| h.span == name.span).cloned();
            let resolved = c.resolved.iter().rev().find(|r| r.0 == name.span).cloned();
            (ty, hover, resolved)
        });
        if let Some(hover) = hover.filter(|_| !self.hovers.iter().any(|h| h.span == name.span)) {
            self.hovers.push(hover);
        }
        self.resolved.extend(resolved);
        if plain_value(&ty) {
            return Some(ty);
        }
        self.shapes.push((call.span, ty));
        None
    }
}

impl Checker<'_> {
    /// The type of a call, index or other compound expression, for tools.
    /// An untyped call keeps the library type `method_form_call` gave it.
    pub(super) fn record_shape(&mut self, expr: &Expr, ty: &Ty) {
        let compound = matches!(
            expr.kind,
            ExprKind::Call { .. }
                | ExprKind::Index { .. }
                | ExprKind::Paren(_)
                | ExprKind::Range { .. }
                | ExprKind::Unwrap(_)
                | ExprKind::Try(_)
        );
        let kept = *ty == Ty::Unknown && self.shapes.last().is_some_and(|s| s.0 == expr.span);
        if compound && !kept {
            self.shapes.push((expr.span, ty.clone()));
        }
    }
}

/// A type the checker models in full: no library type (`Slice<S>`, a
/// generator) whose members and traits only lowering knows, nothing vague.
fn plain_value(ty: &Ty) -> bool {
    match ty {
        Ty::Optional(inner) => plain_value(inner),
        Ty::Tuple(parts) => parts.iter().all(plain_value),
        Ty::Record(fields) => fields.iter().all(|(_, t)| plain_value(t)),
        Ty::Array { index, elem } => index.iter().all(plain_value) && plain_value(elem),
        Ty::Named { .. }
        | Ty::TypeName(_)
        | Ty::Fn { .. }
        | Ty::Param(_)
        | Ty::RowWithout(..)
        | Ty::Unknown
        | Ty::Error => false,
        _ => true,
    }
}
