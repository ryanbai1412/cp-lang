//! Domains a value holds: `t.Idx` of a rebase, or of a struct that sizes
//! its domain from its fields (R04b.1.7).

use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    /// `view.member` when `view` holds the domain `member`: that value's own
    /// domain, which no other value's keys index.
    pub(crate) fn value_domain(&mut self, view: &str, member: &str) -> Option<String> {
        let ty = match self.local(view) {
            Some(local) => local.ty.clone(),
            None => self.global(view)?.value_ty(view),
        };
        let own = format!("{view}.{member}");
        match ty {
            // `let t = w.rebase` names its fresh domain `t.Idx`.
            Ty::Array { index, .. } => {
                matches!(index.as_slice(), [Ty::Pos(d)] if *d == own).then_some(own)
            }
            Ty::Named { name: owner, args } => {
                (self.value_domains(&owner).iter().any(|d| d == member)
                    || self
                        .open_domains(&owner, &args)
                        .iter()
                        .any(|(_, d)| d == member))
                .then_some(own)
            }
            _ => None,
        }
    }

    /// `Rooted<V, _, _>` leaves `Pre` open: a parameter the type's own fns return as `_` is
    /// fresh for each value, so the one `t` holds is `t.Pre`. A parameter that merely failed
    /// to infer (`Paths<_, Int, A>`) is not.
    fn open_domains(&mut self, owner: &str, args: &[Ty]) -> Vec<(usize, String)> {
        let Some(members) = self.members_of(owner) else {
            return Vec::new();
        };
        // With no argument known the type failed to infer: nothing says which are open.
        if members.generics.len() != args.len() || args.iter().all(|a| *a == Ty::Unknown) {
            return Vec::new();
        }
        let fresh = |i: usize| {
            members.members.iter().any(|(_, ty)| {
                matches!(ty, Ty::Fn { ret, .. } if matches!(&**ret,
                    Ty::Named { name, args } if name == owner && args.get(i) == Some(&Ty::Unknown)))
            })
        };
        members
            .generics
            .iter()
            .enumerate()
            .filter(|(i, _)| args[*i] == Ty::Unknown && fresh(*i))
            .map(|(i, g)| (i, g.clone()))
            .collect()
    }

    /// `t.pos(v)` on a `t: Rooted<V, _, _>`: the domains `t` owns are read as `t.Pre`.
    pub(crate) fn own_open_domains(&mut self, base: &Expr, base_ty: Ty) -> Ty {
        let (ExprKind::Name(view), Ty::Named { name: owner, args }) = (&base.kind, &base_ty) else {
            return base_ty;
        };
        if self.local(view).is_none() && self.global(view).is_none() {
            return base_ty;
        }
        let open = self.open_domains(owner, args);
        if open.is_empty() {
            return base_ty;
        }
        let mut args = args.clone();
        for (i, g) in open {
            args[i] = Ty::Pos(format!("{view}.{g}"));
        }
        Ty::Named {
            name: owner.clone(),
            args,
        }
    }

    /// A member's type read through the value `base`: the domains its
    /// struct sizes from fields become `base`'s own, so `t.at(u.Idx(0))`
    /// mixes two values' keys (E0301).
    pub(crate) fn through_value(&mut self, base: &Expr, base_ty: &Ty, ty: Ty) -> Ty {
        let (ExprKind::Name(view), Ty::Named { name: owner, .. }) = (&base.kind, base_ty) else {
            return ty;
        };
        self.value_domains(owner).iter().fold(ty, |ty, d| {
            ty.rename_domain(&format!("{owner}.{d}"), &format!("{view}.{d}"))
        })
    }

    fn value_domains(&mut self, owner: &str) -> Vec<String> {
        match self.type_def(owner).as_deref() {
            Some(Def::Struct { value_domains, .. }) => value_domains.clone(),
            _ => Vec::new(),
        }
    }

    /// A scope where a struct body names its `fields` without `self.`.
    pub(crate) fn field_scope(&mut self, fields: &[crate::ty::Param], span: Span) {
        self.push_scope();
        let scope = self.scopes.innermost();
        for field in fields {
            let local = Local {
                ty: field.ty.clone(),
                mutable: false,
                span,
                keyword: "self",
            };
            scope.insert(field.name.clone(), local);
        }
    }
}
