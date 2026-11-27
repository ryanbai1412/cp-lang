//! The four views a layer fn reads instead of the whole program (handbook/design/next/
//! incremental.md, "The whole-program inputs, one by one"). The query database implements each
//! with per-item queries; today `Decls`, `SetAdds` and the lowered fns implement them, and a
//! test can with a map. A layer fn whose step is done takes these, never `&Decls` or a map of
//! every fn.
use crate::decls::{
    AliasDecl, AliasId, Decls, FnDecl, ImplDecl, ModId, Named, StructDecl, TraitDecl,
};
use cpl_next_ir::ir::{Func, FuncKey};
use cpl_next_ir::ty::{FnId, SetAdds, SetTy, StructId, TraitId, Ty};
use std::collections::HashMap;

/// `signature(ItemId)`: what an item declares, never its body.
pub trait Signatures {
    fn f(&self, id: FnId) -> &FnDecl;
    fn s(&self, id: StructId) -> &StructDecl;
    fn t(&self, id: TraitId) -> &TraitDecl;
    fn alias(&self, id: AliasId) -> &AliasDecl;
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
    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
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

impl Scope for Decls {
    fn named(&self, m: ModId, name: &str) -> Option<Named> {
        self.views[m.0].get(name).copied()
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
