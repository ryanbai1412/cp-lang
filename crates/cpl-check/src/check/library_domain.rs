//! Library domains: a type with `impl Index` is a domain (R04.2.26), with
//! the members user code calls, `T.count`, `x.ord` and `T.nth(k)`, and the
//! walk's `Seq` members.

use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind, Name, Stmt, StmtKind};

/// Members of an ordered core domain that a library domain does not derive.
pub(super) const CORE_DOMAIN_ONLY: [&str; 10] = [
    "next", "prev", "l", "r", "before", "after", "cuts", "at", "wrap", "offset",
];

impl Checker<'_> {
    /// R04.2.27: a type with `impl Index` has the domain members that need no
    /// order: `T.count`, `T.first`, `T.last`, `T.rev`, and every `Seq` method
    /// over its walk. Its own members (`nth`, constructors) come first.
    pub(super) fn library_domain_member(
        &mut self,
        expr: &Expr,
        base: &Expr,
        name: &Name,
    ) -> Option<Ty> {
        let ExprKind::Name(type_name) = &base.kind else {
            return None;
        };
        if self.local(type_name).is_some()
            || !self.has_impl(type_name, "Index")
            || CORE_DOMAIN_ONLY.contains(&name.text.as_str())
        {
            return None;
        }
        // `T.count(f)` is the `Seq` count; the impl's own `let count` is `T.count`.
        let seq_count = name.text == "count" && self.in_call_callee;
        let own = self
            .members_of(type_name)
            .is_some_and(|m| m.members.iter().any(|(m, _)| *m == name.text));
        if own && !seq_count {
            return None;
        }
        let elem = Ty::Named {
            name: type_name.to_string(),
            args: Vec::new(),
        };
        Some(match name.text.as_str() {
            "count" if !seq_count => Ty::Int,
            "first" | "last" => Ty::Optional(Box::new(elem)),
            _ => self.member_ty(expr, name, Ty::list(elem)),
        })
    }

    /// The `count` of an `impl Index`: how many values the type has. `const count` is known at
    /// compile time; `let count` is evaluated once, where the impl is declared, and frozen, so it
    /// can't read a `var` (plan/19 §3.1). The impl's members take the names user code calls:
    /// `count`, `ord` and `nth`.
    pub(super) fn index_size(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.index_member_name(stmt);
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } if name.text == "count" => {
                    let known = matches!(self.bounds(init), (Some(lo), Some(hi)) if lo == hi);
                    if !known {
                        let value = self.quote(init.span);
                        let message = format!(
                            "`const count = {value}`: `const` is compile-time only, and `{value}` is known only at run time"
                        );
                        if let Some(diag) = self.error(codes::BAD_INDEX_IMPL, stmt.span, message) {
                            diag.help = Some(format!(
                                "write `let count = {value}`; an associated `let` is evaluated once, where the impl is declared, and is frozen from then on"
                            ));
                        }
                    }
                }
                StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } if index_binding(pat) == Some("count") => {
                    for name in super::lends::names_in(init) {
                        let mutable = match self.local(&name) {
                            Some(local) => local.mutable,
                            None => matches!(
                                self.global(&name).as_deref(),
                                Some(Def::Value { mutable: true, .. })
                            ),
                        };
                        if mutable {
                            let message = format!(
                                "`count` can't be sized by the var `{name}`: it could change after arrays are made"
                            );
                            if let Some(diag) =
                                self.error(codes::BAD_INDEX_IMPL, init.span, message)
                            {
                                diag.help = Some(format!("freeze it first: `let n = {name}`"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// An `impl Index` member under a pre-0.3 name (`K`, `to_idx`, `from_idx`).
    fn index_member_name(&mut self, stmt: &Stmt) {
        let (old, span) = match &stmt.kind {
            StmtKind::Fn(f) => (f.name.text.as_str(), f.name.span),
            StmtKind::Const { name, .. } => (name.text.as_str(), name.span),
            StmtKind::Let { pat, .. } => match index_binding(pat) {
                Some(name) => (name, pat.span),
                None => return,
            },
            _ => return,
        };
        let new = match old {
            "K" => "count",
            "to_idx" => "ord",
            "from_idx" => "nth",
            _ => return,
        };
        let message = format!("`impl Index` names this member `{new}`, not `{old}`");
        if let Some(diag) = self.error(codes::BAD_INDEX_IMPL, span, message) {
            diag.help = Some(format!(
                "rename `{old}` to `{new}`; user code reads it as `T.count`, `x.ord` or `T.nth(k)`"
            ));
        }
    }
}

/// The name a `let` in an impl body binds: `let count = ..` or `let K = ..`.
fn index_binding(pat: &ast::Pattern) -> Option<&str> {
    match &pat.kind {
        ast::PatternKind::Name(name) => Some(name.as_str()),
        _ => super::pattern::domain_binding(pat),
    }
}
