//! Binding a trait alias's parameters from what the alias says: `G: WeightedNext<V, W>` binds
//! `V` through `Next<V>` and `W` through the arc's `Weighted<W>`, as its
//! `where Self.Arc: Weighted<W>` states.

use super::*;

impl<'a> Lowerer<'a> {
    /// The trait alias `name` names in `tpl_mod`, if it names one.
    pub(super) fn alias_trait(&self, name: &str, tpl_mod: ModId) -> Option<usize> {
        match self.mod_syms[tpl_mod].get(name) {
            Some(Sym::Trait(tid)) if self.traits[*tid].alias_of.is_some() => Some(*tid),
            _ => None,
        }
    }

    /// Unifies alias `tid` written with `args` against `concrete`: each aliased trait, then
    /// each `where Self.X: Bound` against `concrete`'s associated type `X`.
    pub(super) fn unify_alias(
        &self,
        tid: usize,
        args: &[ast::Type],
        concrete: &Ty,
        gen_names: &[String],
        env: &mut GenEnv,
    ) {
        let info = &self.traits[tid];
        let subst: Vec<(&str, &ast::Type)> = info.generics.iter().copied().zip(args).collect();
        for sup in &info.alias_tys {
            self.unify_ast_ty(
                &subst_ty(sup, &subst),
                concrete,
                info.mod_id,
                gen_names,
                env,
            );
        }
        let traits = info.alias_of.as_deref().unwrap_or_default();
        for (assoc, bound) in &info.alias_where {
            if let Some(item) = self.assoc_of(traits, assoc, concrete) {
                self.unify_ast_ty(&subst_ty(bound, &subst), &item, info.mod_id, gen_names, env);
            }
        }
    }

    /// `concrete`'s associated type `assoc` of one of `traits`: a struct's impl names it
    /// among its parameters; an adjacency table or `V -> Seq` fn yields it.
    fn assoc_of(&self, traits: &[String], assoc: &str, concrete: &Ty) -> Option<Ty> {
        match concrete {
            Ty::Struct(sid) => (0..self.impls.len()).find_map(|idx| {
                let imp = &self.impls[idx];
                if !traits.iter().any(|t| imp.trait_name.as_deref() == Some(t)) {
                    return None;
                }
                let ty = imp.body.stmts.iter().find_map(|s| match &s.kind {
                    ast::StmtKind::TypeAlias { name, ty, .. } if name.text == assoc => Some(ty),
                    _ => None,
                })?;
                match self
                    .struct_impl_env(idx, *sid)?
                    .get(super::unify_ast::bare_name(ty)?)?
                {
                    GenArg::Ty(t) => Some(t.clone()),
                    GenArg::Domain(d) => Some(Ty::Pos(*d)),
                    _ => None,
                }
            }),
            Ty::Array { elem, .. } => elem.seq_elem(),
            Ty::Fn(_, ret) => ret.seq_elem(),
            _ => None,
        }
    }
}

/// `t` with the alias's parameters replaced by the args it was written with.
fn subst_ty(t: &ast::Type, subst: &[(&str, &ast::Type)]) -> ast::Type {
    match &t.kind {
        ast::TypeKind::Named { name, args } if args.is_empty() => subst
            .iter()
            .rfind(|(param, _)| *param == name.text)
            .map_or_else(|| t.clone(), |(_, a)| (*a).clone()),
        ast::TypeKind::Named { name, args } => ast::Type {
            kind: ast::TypeKind::Named {
                name: name.clone(),
                args: args.iter().map(|a| subst_ty(a, subst)).collect(),
            },
            span: t.span,
        },
        _ => t.clone(),
    }
}
