//! The four views a layer fn reads instead of the whole program (handbook/design/next/
//! incremental.md, "The whole-program inputs, one by one"). The query database implements each
//! with per-item queries; today `Decls`, `SetAdds` and the lowered fns implement them, and a
//! test can with a map. A layer fn whose step is done takes these, never `&Decls` or a map of
//! every fn.
use crate::decls::{
    AliasDecl, AliasId, ConstId, Decls, FnDecl, ImplDecl, LibConst, ModId, Named, Roles,
    StructDecl, Sugar, TraitDecl,
};
use crate::sym::Sym;
use cpl_next_ir::ir::{Func, FuncKey};
use cpl_next_ir::ty::{FnId, FnRef, SetAdds, SetTy, StructId, TraitId, Ty};
use cpl_next_ir::whole::{Dom, DomId, Global, GlobalId, Whole};
use std::collections::HashMap;

/// The part of the signatures a written type is converted against, each table read by key.
/// Decls reads it while it is still building the rest of the signatures.
pub trait TypeTables {
    /// The text a name was written with, for showing it.
    fn text(&self, s: crate::sym::Sym) -> &str;
    /// The id of a name written `text`, if any table holds it.
    fn sym(&self, text: &str) -> Option<crate::sym::Sym>;

    fn alias(&self, id: AliasId) -> &AliasDecl;
    fn konst(&self, id: ConstId) -> &LibConst;
    /// Whether the program's top level binds `name` with `let`: a value, not an item.
    fn top_value(&self, name: &str) -> bool;
    /// Whether the program's top level binds `name` with `var`, which no type may end at.
    fn top_var(&self, name: &str) -> bool;
    /// Binary operator `op`'s fn, which `type_of(+)` names.
    fn operator(&self, op: &str) -> Option<FnId>;
    /// `Domain`'s getter `m`, which a range type's end `D.m` names.
    fn getter(&self, m: &str) -> Option<FnId>;
    /// How many set parameters struct `s` has, so a struct named in a type gets its own.
    fn n_sets(&self, s: StructId) -> usize;
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>>;
    /// The struct of `s`'s name with `n` type parameters, else `s` (R02.11.4).
    fn by_arity(&self, s: StructId, n: usize) -> StructId;
    /// Core's `@role(tuple)` struct with `n` fields.
    fn tuple(&self, n: usize) -> Option<StructId>;
    /// The most fields a the `@role(tuple)` struct has: a longer tuple nests its last parts.
    fn widest_tuple(&self) -> usize;
    /// The record struct with these field names.
    fn record(&self, fields: &[String]) -> Option<StructId>;
    fn record_fields(&self, s: StructId) -> Option<Vec<String>>;
    /// The `@member(m)` struct.
    fn member(&self, m: &str) -> Option<StructId>;
    fn sugar(&self) -> Sugar;
    fn roles(&self) -> &Roles;
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
    /// The trait `@literal(op)` marks: `"[]"` a list literal builds through.
    fn literal_trait(&self, op: &str) -> Option<TraitId>;
    /// The traits that declare a method `name`, by id (`traits(name)`, indexed from the
    /// signatures).
    fn traits_with(&self, name: &str) -> Vec<TraitId>;
    /// The fns that `@aka` or `@was` give `name`, by id.
    fn renamed(&self, name: &str) -> Vec<FnId>;
    /// Which impl a type takes when several fit, lowest first (R08.1.2).
    fn impl_rank(&self, im: &ImplDecl) -> u8;
    /// Whether `im` is for `t` as far as a range type and `Int` go.
    fn head_fits(&self, im: &ImplDecl, t: &Ty) -> bool;
    /// Whether `tr` is one every domain meets with no impl.
    fn core_domain_trait(&self, tr: TraitId) -> bool;
    /// The fn of `members` named `name`.
    fn member_named(&self, members: &HashMap<Sym, FnId>, name: &str) -> Option<FnId>;
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
    /// What the top level puts in the `k`-th set of global `g`'s type.
    fn global_set(&self, g: GlobalId, k: usize) -> Option<&SetTy>;
    /// The call and result slot whose sets a `SetTy::AddedAt(c)` names; none for a call in
    /// code that never runs.
    fn added_at(&self, c: usize) -> Option<&(FnRef, usize)>;
}

/// `checked(ItemId)`: what the checker answered about one item that link reads by id. No
/// method lists a whole table, so link reads only the items it instantiates.
pub trait Checked {
    /// `domain(DomId)`.
    fn domain(&self, d: DomId) -> &Dom;
    /// `global(GlobalId)`.
    fn global(&self, g: GlobalId) -> Option<&Global>;
    /// The type of global `g`.
    fn global_ty(&self, g: GlobalId) -> &Ty;
    /// Whether `f` is written in the user's file.
    fn in_program(&self, f: FnId) -> bool;
    /// The context copy of `f` that runs under context copy `ctx` of a trait's fn.
    fn ctx_impl(&self, ctx: u32, f: FnId) -> Option<u32>;
}

/// The lowered program's checker answers, until the query database answers each per item.
pub struct Answers<'a> {
    pub whole: &'a Whole,
    pub globals: &'a [Ty],
}

/// `generic(ItemId)`: one fn's generic IR.
pub trait Generics {
    fn generic(&self, key: &FuncKey) -> Option<&Func<Ty>>;
}

impl Signatures for Decls {
    fn literal_trait(&self, op: &str) -> Option<TraitId> {
        Decls::literal_trait(self, op)
    }
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
        self.ranked_impls(tr).map(|(k, _)| k).collect()
    }
    fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        Decls::impl_args(self, im, t)
    }
    fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        Decls::has_impl(self, t, tr)
    }
    fn traits_with(&self, name: &str) -> Vec<TraitId> {
        (0..self.traits.len())
            .map(TraitId)
            .filter(|tr| self.member(&self.traits[tr.0].methods, name).is_some())
            .collect()
    }
    fn impl_rank(&self, im: &ImplDecl) -> u8 {
        Decls::impl_rank(self, im)
    }
    fn head_fits(&self, im: &ImplDecl, t: &Ty) -> bool {
        Decls::head_fits(self, im, t)
    }
    fn core_domain_trait(&self, tr: TraitId) -> bool {
        Decls::core_domain_trait(self, tr)
    }
    fn member_named(&self, members: &HashMap<Sym, FnId>, name: &str) -> Option<FnId> {
        Decls::member(self, members, name)
    }
    fn renamed(&self, name: &str) -> Vec<FnId> {
        let named = |f: &FnDecl| f.aka.iter().chain(&f.was).any(|n| n == name);
        (0..self.fns.len())
            .map(FnId)
            .filter(|f| named(&self.fns[f.0]))
            .collect()
    }
}

impl TypeTables for Decls {
    fn text(&self, s: crate::sym::Sym) -> &str {
        self.syms.text(s)
    }
    fn sym(&self, text: &str) -> Option<crate::sym::Sym> {
        self.syms.get(text)
    }

    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
    }
    fn konst(&self, id: ConstId) -> &LibConst {
        &self.consts[id.0]
    }
    fn top_value(&self, name: &str) -> bool {
        self.sym(name).is_some_and(|s| self.top_values.contains(&s))
    }
    fn top_var(&self, name: &str) -> bool {
        self.sym(name).is_some_and(|s| self.top_vars.contains(&s))
    }
    fn operator(&self, op: &str) -> Option<FnId> {
        self.operator_fn(op, true)
    }
    fn getter(&self, m: &str) -> Option<FnId> {
        let tr = self.roles.domain?;
        (self.member(&self.traits[tr.0].methods, m)).filter(|f| self.fns[f.0].getter)
    }
    fn n_sets(&self, s: StructId) -> usize {
        self.struct_sets.get(&s).copied().unwrap_or(0)
    }
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>> {
        self.struct_defaults.get(&s)
    }
    fn by_arity(&self, s: StructId, n: usize) -> StructId {
        crate::decls::by_arity(&self.struct_arity, s, n)
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
        self.member_types.get(&self.sym(m)?).copied()
    }
    fn sugar(&self) -> Sugar {
        self.sugar
    }
    fn roles(&self) -> &Roles {
        &self.roles
    }
}

impl Scope for Decls {
    fn named(&self, m: ModId, name: &str) -> Option<Named> {
        self.named(m, name)
    }
    fn names(&self, m: ModId) -> Vec<(&str, Named)> {
        self.views[m.0]
            .iter()
            .map(|(n, x)| (self.syms.text(*n), *x))
            .collect()
    }
}

impl Summaries for SetAdds {
    fn adds(&self, f: FnId, slot: usize) -> Option<&SetTy> {
        self.params.get(&(f, slot))
    }
    fn global_set(&self, g: GlobalId, k: usize) -> Option<&SetTy> {
        self.globals.get(&(g.0, k))
    }
    fn added_at(&self, c: usize) -> Option<&(FnRef, usize)> {
        self.live
            .get(c)
            .copied()
            .unwrap_or(true)
            .then(|| &self.calls[c])
    }
}

impl Checked for Answers<'_> {
    fn domain(&self, d: DomId) -> &Dom {
        self.whole.dom(d)
    }
    fn global(&self, g: GlobalId) -> Option<&Global> {
        self.whole.global(g)
    }
    fn global_ty(&self, g: GlobalId) -> &Ty {
        &self.globals[g.0]
    }
    fn in_program(&self, f: FnId) -> bool {
        self.whole.program_fns.contains(&f)
    }
    fn ctx_impl(&self, ctx: u32, f: FnId) -> Option<u32> {
        self.whole.ctx_impls.get(&(ctx, f)).copied()
    }
}

impl Generics for HashMap<FuncKey, Func<Ty>> {
    fn generic(&self, key: &FuncKey) -> Option<&Func<Ty>> {
        self.get(key)
    }
}

#[cfg(test)]
mod tests;
