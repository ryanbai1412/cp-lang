//! The types a tool reads (contract 9): each item's table of the types its body and signature
//! hold, and the view that answers a tool's questions about one by its id alone. No `Ty`
//! leaves the checker through it, and a tool never matches a type's variants.
use super::*;
use cpl_contract::Prim;
use cpl_next_decls::views::{ImplChoice, Items};
use decls::ImplIdx;

/// A type in an item's table: `n` is its place in that item's walk, so it does not depend on
/// the order anything is asked in.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TyId {
    pub item: ItemId,
    pub n: u32,
}

/// The types one item's body or signature holds, each once, in the order its walk meets them;
/// a type's arguments come before it.
#[derive(Clone, Default, Debug)]
pub struct ItemTypes {
    /// The item's own type parameters, which its types name.
    pub generics: Vec<Sym>,
    types: Vec<Ty>,
}

impl ItemTypes {
    /// The place of `ty`, a solved type, adding it and each of its arguments the first time.
    pub fn add(&mut self, ty: &Ty) -> u32 {
        if let Ty::Struct(_, args, _) = ty {
            for a in args {
                self.add(a);
            }
        }
        match self.types.iter().position(|t| t == ty) {
            Some(n) => n as u32,
            None => {
                self.types.push(ty.clone());
                (self.types.len() - 1) as u32
            }
        }
    }

    pub fn get(&self, n: u32) -> Option<&Ty> {
        self.types.get(n as usize)
    }

    /// The places of type `n`'s arguments: the declaration's, for an instance of one.
    pub fn args(&self, n: u32) -> Vec<u32> {
        let Some(Ty::Struct(_, args, _)) = self.get(n).map(shown) else {
            return Vec::new();
        };
        let at = |a: &Ty| self.types.iter().position(|t| t == a).map(|k| k as u32);
        args.iter().filter_map(at).collect()
    }
}

/// What a tool asks about a type, by its id.
pub trait TypeFacts {
    /// As a diagnostic writes it.
    fn text(&self, t: &TyId) -> Option<String>;
    /// An integer's ends, `None` for an end the checker does not know.
    fn range(&self, t: &TyId) -> Option<(Option<i64>, Option<i64>)>;
    /// The core primitive it is.
    fn prim(&self, t: &TyId) -> Option<Prim>;
    /// The struct or enum it is an instance of.
    fn decl(&self, t: &TyId) -> Option<StructId>;
    /// That declaration's type arguments.
    fn args(&self, t: &TyId) -> Vec<TyId>;
    /// Whether it implements trait `tr`, as the checker's impl lookup answers.
    fn implements(&self, t: &TyId, tr: TraitId) -> bool;
    /// What a `.` after a value of it can name: its fields, its own fns, and the fns of each
    /// impl found for it, by name, each once.
    fn members(&self, t: &TyId) -> Vec<Sym>;
}

/// The view over the checked program's item tables: every part answers one key.
pub struct TypeView<'a> {
    pub items: &'a dyn Items,
    pub impls: &'a dyn ImplChoice,
    /// The `?T` struct, which a type shows as `?T`.
    pub option: Option<StructId>,
    pub domains: &'a [DomInfo],
    pub of: &'a dyn Fn(&ItemId) -> Option<&'a ItemTypes>,
}

/// A type past a lend, which a value's type does not show.
fn shown(mut ty: &Ty) -> &Ty {
    while let Ty::Lent(inner) = ty {
        ty = inner;
    }
    ty
}

impl<'a> TypeView<'a> {
    fn ty(&self, t: &TyId) -> Option<(&'a ItemTypes, &'a Ty)> {
        let table = (self.of)(&t.item)?;
        Some((table, shown(table.get(t.n)?)))
    }
}

impl TypeFacts for TypeView<'_> {
    fn text(&self, t: &TyId) -> Option<String> {
        let (table, ty) = self.ty(t)?;
        let shown = show_ty(
            self.items,
            self.option,
            self.domains,
            &table.generics,
            &|t| t.clone(),
            ty,
        );
        Some(shown)
    }

    fn range(&self, t: &TyId) -> Option<(Option<i64>, Option<i64>)> {
        match self.ty(t)?.1 {
            Ty::Range(lo, hi) => Some((Some(*lo), Some(*hi))),
            t if *t == Ty::Int || t.is_int_range() || matches!(t, Ty::AnyRange(_)) => {
                Some((None, None))
            }
            _ => None,
        }
    }

    fn prim(&self, t: &TyId) -> Option<Prim> {
        self.ty(t)?.1.prim()
    }

    fn decl(&self, t: &TyId) -> Option<StructId> {
        match self.ty(t)?.1 {
            Ty::Struct(id, ..) => Some(*id),
            _ => None,
        }
    }

    fn args(&self, t: &TyId) -> Vec<TyId> {
        let Some((table, _)) = self.ty(t) else {
            return Vec::new();
        };
        let id = |n| TyId {
            item: t.item.clone(),
            n,
        };
        table.args(t.n).into_iter().map(id).collect()
    }

    fn implements(&self, t: &TyId, tr: TraitId) -> bool {
        self.ty(t)
            .is_some_and(|(_, ty)| self.impls.impl_for(tr, ty).is_some())
    }

    fn members(&self, t: &TyId) -> Vec<Sym> {
        let Some((_, ty)) = self.ty(t) else {
            return Vec::new();
        };
        let items = self.items;
        let mut out = Vec::new();
        if let Ty::Struct(id, ..) = ty {
            let s = items.s(*id);
            out.extend(s.fields.iter().map(|(name, _)| *name));
            out.extend(s.methods.keys());
        }
        for (tr, chosen) in self.impls.impls_on(ty) {
            out.extend(items.t(tr).methods.keys());
            if let ImplIdx::Decl(i) = chosen.imp {
                out.extend(items.imp(i).methods.keys());
            }
        }
        out.sort_by(|a, b| items.text(*a).cmp(items.text(*b)));
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_follows_its_args_once_each_in_walk_order() {
        let pair = |a: Ty, b: Ty| Ty::Struct(StructId(0), vec![a, b], Vec::new());
        let mut t = ItemTypes::default();
        let outer = pair(Ty::Int, pair(Ty::Bool, Ty::Int));
        assert_eq!(t.add(&outer), 3);
        assert_eq!(t.get(0), Some(&Ty::Int));
        assert_eq!(t.get(1), Some(&Ty::Bool));
        assert_eq!(t.args(3), vec![0, 2]);
        assert_eq!(t.args(2), vec![1, 0]);
        assert_eq!(t.add(&Ty::Bool), 1);
    }
}
