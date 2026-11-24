//! What each call calls, and the type arguments it binds: the facts
//! lowering reads instead of resolving the call again.

use super::*;
use crate::ty::Bindings;
use crate::{CallFact, CalleeKind};
use cpl_diag::msgs;
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
            || self.global(&name.text).is_some_and(|d| takes_an_arc(&d))
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
        if let Some(diag) = self.report(
            whole.span,
            msgs::check::REMOVED_SYNTAX_A_LIST_OF_LISTS,
            &[&name.text],
        ) {
            diag.help = Some(format!("write `{new}`"));
            diag.fix(format!("write `{new}`"), whole.span, new);
        }
        Some(Ty::Bool)
    }

    pub(crate) fn call(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) -> Ty {
        if let Some(ty) = self.adjacency_contains(call, callee, args) {
            return ty;
        }
        let from = std::mem::replace(&mut self.call_types_from, self.exprs.types.len());
        let mut ty = self.call_unrecorded(call, callee, args);
        self.call_types_from = from;
        // `rand()` and `rand(lo, hi)` are an `Int`.
        if ty == Ty::Unknown
            && matches!(args.len(), 0 | 2)
            && matches!(&callee.kind, ExprKind::Name(n) if n == "rand" && self.local(n).is_none())
        {
            ty = Ty::Int;
        }
        if !self.exprs.calls.iter().any(|c| c.call == call.span) {
            if let Some((name, kind)) = self.callee_form(callee) {
                let builtin = kind == CalleeKind::Method && self.builtin_method(callee, &name.0);
                self.exprs.calls.push(CallFact {
                    call: call.span,
                    name_span: name.1,
                    name: name.0,
                    kind,
                    builtin,
                });
            }
        }
        ty
    }

    /// `p.norm` called as a getter.
    pub(super) fn record_getter(&mut self, call: &Expr, callee: &Expr, builtin: bool) {
        if let ExprKind::Field { name, .. } = &callee.kind {
            self.exprs.calls.push(CallFact {
                call: call.span,
                name_span: name.span,
                name: name.text.clone(),
                kind: CalleeKind::Getter,
                builtin,
            });
        }
    }

    /// `xs.push(x)`: a method the compiler gives the receiver's built-in type.
    fn builtin_method(&self, callee: &Expr, name: &str) -> bool {
        use crate::builtins::{builtin_member, MemberKind};
        let ExprKind::Field { base, .. } = &callee.kind else {
            return false;
        };
        // `List.filled(n, x)`: a built-in type's own constructor.
        if let ExprKind::Name(t) = &base.kind {
            if crate::builtins::is_catalog_type(t) && self.local(t).is_none() {
                return true;
            }
        }
        let owner = match self.exprs.last_type(base.span) {
            Some(Ty::Named { name, .. }) => name.as_str(),
            Some(Ty::Str) => "Str",
            Some(Ty::Array { .. }) => "[D]T",
            _ => return false,
        };
        builtin_member(owner, name).is_some_and(|m| m.kind == MemberKind::Method)
    }

    /// The type each generic parameter of the callee took at `call`.
    pub(super) fn record_instantiation(&mut self, call: &Expr, bindings: &Bindings) {
        if bindings.is_empty() {
            return;
        }
        self.settle_recorded(call, bindings);
        let mut named: Vec<(&String, &Ty)> = bindings.iter().collect();
        named.sort_by(|a, b| a.0.cmp(b.0));
        let bound: Vec<(String, Ty)> = (named.into_iter())
            .map(|(name, ty)| (name.clone(), self.settle(ty)))
            .collect();
        self.exprs.instantiations.retain(|(at, _)| *at != call.span);
        self.exprs.instantiations.push((call.span, bound));
    }

    /// The types recorded inside `call` before its bindings were known, such
    /// as a lambda param's `T`, as the call binds them. A name the code here
    /// declares itself stays its own.
    fn settle_recorded(&mut self, call: &Expr, bindings: &Bindings) {
        if bindings.iter().any(|(name, _)| self.generics.contains(name)) {
            return;
        }
        let from = self.call_types_from.min(self.exprs.types.len());
        for (at, ty) in &mut self.exprs.types[from..] {
            if call.span.start <= at.start && at.end <= call.span.end && ty.mentions_param() {
                *ty = ty.subst(bindings);
            }
        }
    }

    /// The name a call is made through and what kind of thing it names.
    fn callee_form(&self, callee: &Expr) -> Option<((String, Span), CalleeKind)> {
        match &callee.kind {
            ExprKind::Name(n) => {
                let ty = self.exprs.last_type(callee.span);
                let ctor = ty.is_some_and(|t| {
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

/// A global `contains`/`has` that `g.contains(u, v)` could call as written:
/// anything but a fn whose parameters can't hold the list, `u` and `v`
/// (std's `contains(self: S, x: T)` is one element's test).
fn takes_an_arc(def: &Def) -> bool {
    !matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. } if params.len() < 3)
}
