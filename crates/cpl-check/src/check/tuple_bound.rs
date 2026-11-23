//! A tuple annotation whose later part is bounded by an earlier one (R10.2.3).

use super::*;
use cpl_syntax::ast::{Pattern, PatternKind, Type, TypeKind};

impl Checker<'_> {
    /// Resolves a `let` annotation with each name of a tuple pattern in
    /// scope for the parts after it, in a block as at top level.
    pub(super) fn let_annotation(&mut self, pat: &Pattern, ty: &Type) -> Ty {
        let (PatternKind::Tuple(pats), TypeKind::Tuple(items)) = (&pat.kind, &ty.kind) else {
            return self.resolve_type(ty);
        };
        let binds = items
            .iter()
            .any(|i| matches!(i.kind, TypeKind::DomainBinder(_)));
        if pats.len() != items.len() || binds || self.scopes.is_empty() {
            return self.resolve_type(ty);
        }
        self.push_scope();
        let mut parts = Vec::new();
        for (part_pat, item) in pats.iter().zip(items) {
            let part = self.resolve_type(item);
            if let PatternKind::Name(name) = &part_pat.kind {
                let local = Local {
                    ty: part.clone(),
                    mutable: false,
                    span: part_pat.span,
                    keyword: "let",
                };
                self.scopes.last_mut().unwrap().insert(name.clone(), local);
            }
            parts.push(part);
        }
        self.pop_scope();
        Ty::Tuple(parts)
    }
}
