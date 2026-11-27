//! The four views a layer fn reads instead of the whole program (handbook/design/next/
//! incremental.md, "The whole-program inputs, one by one"). The query database implements each
//! with per-item queries; today `Decls`, `SetAdds` and the lowered fns implement them, and a
//! test can with a map. A layer fn whose step is done takes these, never `&Decls` or a map of
//! every fn.
use crate::decls::{
    AliasDecl, AliasId, ConstId, Decls, FnDecl, ImplDecl, LibConst, ModId, Named, Roles,
    StructDecl, Sugar, TraitDecl,
};
use cpl_next_ir::ir::{Func, FuncKey};
use cpl_next_ir::ty::{FnId, SetAdds, SetTy, StructId, TraitId, Ty};
use std::collections::HashMap;

/// The part of the signatures a written type is converted against, each table read by key.
/// Decls reads it while it is still building the rest of the signatures.
pub trait TypeTables {
    fn alias(&self, id: AliasId) -> &AliasDecl;
    fn konst(&self, id: ConstId) -> &LibConst;
    /// Whether the program's top level binds `name` with `let`: a value, not an item.
    fn top_value(&self, name: &str) -> bool;
    /// Binary operator `op`'s fn, which `type_of(+)` names.
    fn operator(&self, op: &str) -> Option<FnId>;
    /// `Domain`'s getter `m`, which a range type's end `D.m` names.
    fn getter(&self, m: &str) -> Option<FnId>;
    /// How many set parameters struct `s` has, so a struct named in a type gets its own.
    fn n_sets(&self, s: StructId) -> usize;
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>>;
    /// Core's `@tuple` struct with `n` fields.
    fn tuple(&self, n: usize) -> Option<StructId>;
    /// The most fields a core `@tuple` struct has: a longer tuple nests its last parts.
    fn widest_tuple(&self) -> usize;
    /// The record struct with these field names.
    fn record(&self, fields: &[String]) -> Option<StructId>;
    fn record_fields(&self, s: StructId) -> Option<Vec<String>>;
    /// The `@member(m)` struct.
    fn member(&self, m: &str) -> Option<StructId>;
    fn sugar(&self) -> Sugar;
    fn roles(&self) -> Roles;
}

/// `signature(ItemId)`: what an item declares, never its body.
pub trait Signatures: TypeTables {
    fn f(&self, id: FnId) -> &FnDecl;
    fn s(&self, id: StructId) -> &StructDecl;
    fn t(&self, id: TraitId) -> &TraitDecl;
    /// Impl `i`, an index into the program's impls.
    fn imp(&self, i: usize) -> &ImplDecl;
    /// The impls of `tr` a lookup may find, by index, in order (`impls(TraitId)`).
    fn impls_of(&self, tr: TraitId) -> Vec<usize>;
    /// The impl's type arguments when it is for `t`.
    fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>>;
    /// Whether `t` has an impl of `tr`.
    fn has_impl(&self, t: &Ty, tr: TraitId) -> bool;
}

/// `scope(ModId)`: what a name means in a module.
pub trait Scope {
    fn named(&self, m: ModId, name: &str) -> Option<Named>;
    /// Each name the module's scope gives and what it means, in no fixed order.
    fn names(&self, m: ModId) -> Vec<(&str, Named)>;
}

/// `summary(ItemId)`: what a fn's body adds to its set parameters, read without checking it.
pub trait Summaries {
    /// What `f`'s body adds to its `slot`-th set parameter, if anything.
    fn adds(&self, f: FnId, slot: usize) -> Option<&SetTy>;
}

/// `generic(ItemId)`: one fn's generic IR.
pub trait Generics {
    fn generic(&self, key: &FuncKey) -> Option<&Func<Ty>>;
}

impl Signatures for Decls {
    fn f(&self, id: FnId) -> &FnDecl {
        &self.fns[id.0]
    }
    fn s(&self, id: StructId) -> &StructDecl {
        &self.structs[id.0]
    }
    fn t(&self, id: TraitId) -> &TraitDecl {
        &self.traits[id.0]
    }
    fn imp(&self, i: usize) -> &ImplDecl {
        &self.impls[i]
    }
    fn impls_of(&self, tr: TraitId) -> Vec<usize> {
        self.trait_impls(tr).map(|(k, _)| k).collect()
    }
    fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        Decls::impl_args(self, im, t)
    }
    fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        Decls::has_impl(self, t, tr)
    }
}

impl TypeTables for Decls {
    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
    }
    fn konst(&self, id: ConstId) -> &LibConst {
        &self.consts[id.0]
    }
    fn top_value(&self, name: &str) -> bool {
        self.top_values.contains(name)
    }
    fn operator(&self, op: &str) -> Option<FnId> {
        self.operator_fn(op, true)
    }
    fn getter(&self, m: &str) -> Option<FnId> {
        let tr = self.domain_trait?;
        (self.traits[tr.0].methods.get(m).copied()).filter(|f| self.fns[f.0].getter)
    }
    fn n_sets(&self, s: StructId) -> usize {
        self.struct_sets.get(&s).copied().unwrap_or(0)
    }
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>> {
        self.struct_defaults.get(&s)
    }
    fn tuple(&self, n: usize) -> Option<StructId> {
        self.tuples.get(&n).copied()
    }
    fn widest_tuple(&self) -> usize {
        self.tuples.keys().max().copied().unwrap_or(0)
    }
    fn record(&self, fields: &[String]) -> Option<StructId> {
        self.records.get(fields).copied()
    }
    fn record_fields(&self, s: StructId) -> Option<Vec<String>> {
        (self.records.iter())
            .find(|(_, id)| **id == s)
            .map(|(fs, _)| fs.clone())
    }
    fn member(&self, m: &str) -> Option<StructId> {
        self.member_types.get(m).copied()
    }
    fn sugar(&self) -> Sugar {
        self.sugar
    }
    fn roles(&self) -> Roles {
        self.roles
    }
}

impl Scope for Decls {
    fn named(&self, m: ModId, name: &str) -> Option<Named> {
        self.views[m.0].get(name).copied()
    }
    fn names(&self, m: ModId) -> Vec<(&str, Named)> {
        self.views[m.0]
            .iter()
            .map(|(n, x)| (n.as_str(), *x))
            .collect()
    }
}

impl Summaries for SetAdds {
    fn adds(&self, f: FnId, slot: usize) -> Option<&SetTy> {
        self.params.get(&(f, slot))
    }
}

impl Generics for HashMap<FuncKey, Func<Ty>> {
    fn generic(&self, key: &FuncKey) -> Option<&Func<Ty>> {
        self.get(key)
    }
}

#[cfg(test)]
mod tests;
