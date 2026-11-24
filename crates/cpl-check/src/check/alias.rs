//! `type` aliases: the right side sees the alias's own name as it is outside
//! the alias, and an alias reached again through other aliases is E0323
//! (R02.6.2).

use super::*;
use cpl_syntax::ast;

impl<'a> Checker<'a> {
    /// The right side of `type name<generics> = ty`. In it, `name` means
    /// what it means outside the alias (`type Mod = Mod<P>` names std's
    /// `Mod`), and an alias reached again on the way is a cycle (E0323).
    pub(crate) fn resolve_alias(
        &mut self,
        name: &ast::Name,
        generics: &[ast::Name],
        ty: &ast::Type,
    ) -> Ty {
        if let Some(span) = self.alias_reaches_itself(&name.text, ty) {
            return self.alias_cycle(&name.text, span);
        }
        let outer = self.generics.len();
        self.generics
            .extend(generics.iter().map(|g| g.text.clone()));
        self.defining_aliases.push(name.text.clone());
        let ty = self.resolve_type(ty);
        self.defining_aliases.pop();
        self.generics.truncate(outer);
        ty
    }

    /// Where `type start = ty` reaches `start` again through this file's
    /// other aliases (`type A = B`, `type B = A`). An alias's own name on its
    /// right side means the outer one, so it is not followed.
    fn alias_reaches_itself(&mut self, start: &str, ty: &ast::Type) -> Option<Span> {
        let mut work = Vec::new();
        type_names(ty, &mut |n| {
            if n.text != start {
                work.push((n.text.clone(), n.span));
            }
        });
        let mut seen = std::collections::HashSet::new();
        while let Some((alias, origin)) = work.pop() {
            if alias == start {
                return Some(origin);
            }
            if !seen.insert(alias.clone()) {
                continue;
            }
            if let Some(rhs) = self.env.alias_rhs(&alias) {
                type_names(&rhs, &mut |n| {
                    if n.text != alias {
                        work.push((n.text.clone(), origin));
                    }
                });
            }
        }
        None
    }

    pub(super) fn alias_cycle(&mut self, name: &str, span: Span) -> Ty {
        let message = format!("`type {name}` is defined by itself, so it never names a type");
        if let Some(diag) = self.error(codes::ALIAS_CYCLE, span, message) {
            diag.help = Some(format!(
                "write the type `{name}` stands for, or give the alias a name of its own (R02.6.2)"
            ));
        }
        Ty::Error
    }
}

/// Every type name written in `ty`, arguments included.
fn type_names(ty: &ast::Type, f: &mut impl FnMut(&ast::Name)) {
    use ast::TypeKind as K;
    match &ty.kind {
        K::Named { name, args } => {
            f(name);
            args.iter().for_each(|a| type_names(a, f));
        }
        K::Array { index, elem } => {
            index.iter().for_each(|i| type_names(i, f));
            type_names(elem, f);
        }
        K::OwnDomain { elem, .. } | K::Optional(elem) | K::Mut(elem) | K::Yields(elem) => {
            type_names(elem, f)
        }
        K::Tuple(items) => items.iter().for_each(|t| type_names(t, f)),
        K::Record(fields) | K::Row(fields) => fields.iter().for_each(|(_, t)| type_names(t, f)),
        K::Fn { params, ret } => {
            params.iter().for_each(|p| type_names(p, f));
            type_names(ret, f);
        }
        K::RowWithout { prefix, base, .. } => {
            prefix.iter().for_each(|(_, t)| type_names(t, f));
            type_names(base, f);
        }
        _ => {}
    }
}
