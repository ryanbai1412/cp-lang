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
        let mut ty = self.call_unrecorded(call, callee, args);
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
        use crate::builtins::{builtin_members, MemberKind, Receiver};
        let ExprKind::Field { base, .. } = &callee.kind else {
            return false;
        };
        // `List.filled(n, x)`: a built-in type's own constructor.
        if let ExprKind::Name(t) = &base.kind {
            const TYPES: [&str; 6] = ["List", "Map", "HashMap", "Set", "HashSet", "Str"];
            if TYPES.contains(&t.as_str()) && self.local(t).is_none() {
                return true;
            }
        }
        let ty = self.exprs.types.iter().rev().find(|t| t.0 == base.span);
        let receiver = match ty.map(|t| &t.1) {
            Some(Ty::Named { name, args }) => match (name.as_str(), args.len()) {
                ("List", 1) => Receiver::List,
                ("Map" | "HashMap", 2) => Receiver::Map,
                ("Set" | "HashSet", 1) => Receiver::Set,
                _ => return false,
            },
            Some(Ty::Str) => Receiver::Str,
            Some(Ty::Array { .. }) => Receiver::Array,
            _ => return false,
        };
        builtin_members(receiver).any(|m| m.kind == MemberKind::Method && m.name == name)
    }

    /// The type each generic parameter of the callee took at `call`.
    pub(super) fn record_instantiation(&mut self, call: &Expr, bindings: &Bindings) {
        if bindings.is_empty() {
            return;
        }
        let mut names: Vec<String> = bindings.keys().cloned().collect();
        names.sort();
        let bound: Vec<(String, Ty)> = (names.into_iter())
            .map(|name| {
                let ty = self.settle(&bindings[&name]);
                (name, ty)
            })
            .collect();
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

/// A global `contains`/`has` that `g.contains(u, v)` could call as written:
/// anything but a fn whose parameters can't hold the list, `u` and `v`
/// (std's `contains(self: S, x: T)` is one element's test).
fn takes_an_arc(def: &Def) -> bool {
    !matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. } if params.len() < 3)
}
