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
    /// The impls of `tr`, by index (`impls(TraitId)`).
    fn impls_of(&self, tr: TraitId) -> Vec<usize>;
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
        self.impls_by_trait.get(&tr).cloned().unwrap_or_default()
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
