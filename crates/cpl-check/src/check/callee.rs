//! What each call calls, and the type arguments it binds: the facts
//! lowering reads instead of resolving the call again.

use super::*;
use crate::ty::Bindings;
use crate::{CallFact, CalleeKind};
use cpl_syntax::ast::{Arg, Expr, ExprKind};

impl Checker<'_> {
    /// `g.contains(u, v)` or `g.has(u, v)` on a list of lists: the arc test is
    /// `u`'s own list's `contains` (decision 525).
    fn adjacency_contains(&mut self, whole: &Expr, callee: &Expr, args: &[Arg]) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let [u, v] = args else { return None };
        if !matches!(name.text.as_str(), "contains" | "has")
            || u.name.is_some()
            || v.name.is_some()
            || self.global(&name.text).is_some()
        {
            return None;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        if !matches!(base_ty.seq_elem()?, Ty::Named { name, .. } if name == "List") {
            return None;
        }
        self.infer(&u.value);
        self.infer(&v.value);
        let text = |s: Span| &self.src[s.start as usize..s.end as usize];
        let new = format!(
            "{}[{}].contains({})",
            text(base.span),
            text(u.value.span),
            text(v.value.span)
        );
        let message = format!(
            "a list of lists has no `{}(u, v)`; an arc test asks `u`'s list",
            name.text
        );
        if let Some(diag) = self.error(codes::REMOVED_SYNTAX, whole.span, message) {
            diag.help = Some(format!("write `{new}`"));
            diag.fix(format!("write `{new}`"), whole.span, new);
        }
        Some(Ty::Bool)
    }

    pub(crate) fn call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Ty {
        if let Some(ty) = self.adjacency_contains(call, callee, args) {
            return ty;
        }
        let ty = self.call_unrecorded(call, callee, args);
        if !self.exprs.calls.iter().any(|c| c.call == call.span) {
            if let Some((name, kind)) = self.callee_form(callee) {
                self.exprs.calls.push(CallFact {
                    call: call.span,
                    name_span: name.1,
                    name: name.0,
                    kind,
                });
            }
        }
        ty
    }

    /// `p.norm` called as a getter.
    pub(super) fn record_getter(&mut self, call: &Expr, callee: &Expr) {
        if let ExprKind::Field { name, .. } = &callee.kind {
            self.exprs.calls.push(CallFact {
                call: call.span,
                name_span: name.span,
                name: name.text.clone(),
                kind: CalleeKind::Getter,
            });
        }
    }

    /// The type each generic parameter of the callee took at `call`.
    pub(super) fn record_instantiation(&mut self, call: &Expr, bindings: &Bindings) {
        if bindings.is_empty() {
            return;
        }
        let mut bound: Vec<(String, Ty)> = (bindings.iter())
            .map(|(name, ty)| (name.clone(), self.settle(ty)))
            .collect();
        bound.sort_by(|a, b| a.0.cmp(&b.0));
        self.exprs.instantiations.retain(|(at, _)| *at != call.span);
        self.exprs.instantiations.push((call.span, bound));
    }

    /// The name a call is made through and what kind of thing it names.
    fn callee_form(&self, callee: &Expr) -> Option<((String, Span), CalleeKind)> {
        match &callee.kind {
            ExprKind::Name(n) => {
                let ty = self.exprs.types.iter().rev().find(|t| t.0 == callee.span);
                let ctor = ty.is_some_and(|(_, t)| {
                    matches!(t, Ty::TypeName(_) | Ty::Named { .. } | Ty::Domain(_))
                });
                let kind = if ctor {
                    CalleeKind::Ctor
                } else {
                    CalleeKind::Fn
                };
                Some(((n.clone(), callee.span), kind))
            }
            ExprKind::Generic { base, .. } => self.callee_form(base),
            ExprKind::Field { name, .. } => {
                let kind = if self.exprs.module_members.contains(&name.span) {
                    CalleeKind::Fn
                } else {
                    CalleeKind::Method
                };
                Some(((name.text.clone(), name.span), kind))
            }
            ExprKind::Paren(inner) => self.callee_form(inner),
            _ => None,
        }
    }
}
