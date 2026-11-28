//! The four views a layer fn reads instead of the whole program (handbook/design/next/
//! incremental.md, "The whole-program inputs, one by one"). The query database implements each
//! with per-item queries; today `Decls`, `SetAdds` and the lowered fns implement them, and a
//! test can with a map. A layer fn whose step is done takes these, never `&Decls` or a map of
//! every fn.
use crate::decls::{
    AliasDecl, AliasId, ConstId, Decls, FnDecl, ImplDecl, Impls, LibConst, ModId, Named, Roles,
    StructDecl, Sugar, TraitDecl,
};
use crate::sym::Sym;
use cpl_next_ir::ir::{Func, FuncKey};
use cpl_next_ir::msg::{Deferred, Msg};
use cpl_next_ir::ty::{FnId, FnRef, SetAdds, SetTy, StructId, TraitId, Ty};
use cpl_next_ir::whole::{Dom, DomId, Global, GlobalId, Whole};
use std::collections::HashMap;

/// The part of the signatures a written type is converted against, each table read by key.
/// Decls reads it while it is still building the rest of the signatures.
pub trait TypeTables: Texts {
    /// The id of a name written `text`, if any table holds it.
    fn sym(&self, text: &str) -> Option<crate::sym::Sym>;
    fn alias(&self, id: AliasId) -> &AliasDecl;
    fn konst(&self, id: ConstId) -> &LibConst;
    /// The core type written `n`: `Int`, `Raw`.
    fn core_type(&self, n: crate::sym::Sym) -> Option<crate::decls::CoreType>;
    /// The value the program's top level binds as `name`, when it binds one: a value, not an item.
    fn top_value(
        &self,
        name: &str,
    ) -> Option<(cpl_next_ir::ty::TopValueId, crate::decls::TopValue)>;
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

/// `text(Sym)`: the text a name was written with, for showing it.
pub trait Texts {
    fn text(&self, s: Sym) -> &str;
}

/// One declared item by its id: a layer that reads only these holds no whole-program value.
pub trait Items: Texts {
    fn f(&self, id: FnId) -> &FnDecl;
    fn s(&self, id: StructId) -> &StructDecl;
    fn t(&self, id: TraitId) -> &TraitDecl;
    /// Impl `i`, an index into the program's impls.
    fn imp(&self, i: usize) -> &ImplDecl;
    /// The name of top-level value `g`, for showing a range type that ends at it.
    fn top_name(&self, g: cpl_next_ir::ty::TopValueId) -> Sym;
    /// The impls of `tr`, by index, in order, found or not (see `live`).
    fn impls_by_trait(&self, tr: TraitId) -> &[usize];
    /// The impls of `tr` in the order a type takes them, once they are ranked.
    fn ranked(&self, tr: TraitId) -> Option<&[usize]>;
    /// The impls that write a fn named `name`, by index.
    fn impls_named(&self, name: Sym) -> &[usize];
    /// Whether a lookup may find impl `i` (R11.3.4).
    fn live(&self, i: usize) -> bool;
}

/// `signature(ItemId)`: what an item declares, never its body.
pub trait Signatures: TypeTables + Items {
    /// The impl lookup over these signatures (`Impls`): the one place impl questions go.
    fn impls(&self) -> Impls<'_>;
    /// The trait `@literal(op)` marks: `"[]"` a list literal builds through.
    fn literal_trait(&self, op: &str) -> Option<TraitId>;
    /// The traits that declare a method `name`, by id (`traits(name)`, indexed from the
    /// signatures).
    fn traits_with(&self, name: &str) -> Vec<TraitId>;
    /// The fns that `@aka` or `@was` give `name`, by id.
    fn renamed(&self, name: &str) -> Vec<FnId>;
    /// Which impl a type takes when several fit, lowest first (R08.1.2).
    fn impl_rank(&self, im: &ImplDecl) -> u8;
    /// The fn of `members` named `name`.
    fn member_named(&self, members: &HashMap<Sym, FnId>, name: &str) -> Option<FnId>;
}

/// `names(ModId)`: what a module-level name means, keyed by the module and the interned name
/// (C's name resolution reads it; text becomes a `Sym` only through `Syms`). Every answer is an
/// id, rendered through `Texts` for display only.
pub trait Names {
    fn named(&self, m: ModId, name: Sym) -> Option<Named>;
    /// What module `m` exports as `name`, and whether it is `pub`.
    fn export(&self, m: ModId, name: Sym) -> Option<(Named, bool)>;
    /// Whether two of module `m`'s `use` lines import `name` as different items (R08.1.4).
    fn is_ambiguous(&self, m: ModId, name: Sym) -> bool;
    fn fn_named(&self, m: ModId, name: Sym) -> Option<FnId> {
        match self.named(m, name) {
            Some(Named::Fn(f)) => Some(f),
            _ => None,
        }
    }
    fn struct_named(&self, m: ModId, name: Sym) -> Option<StructId> {
        match self.named(m, name) {
            Some(Named::Struct(s)) => Some(s),
            _ => None,
        }
    }
    /// The program's top-level value `name` binds, the last one bound under it.
    fn top_value(&self, m: ModId, name: Sym) -> Option<cpl_next_ir::ty::TopValueId>;
    /// Module `m`'s names, for closest-name candidates: one module's answer.
    fn names(&self, m: ModId) -> Vec<(Sym, Named)>;
}

/// What a fn's signature says beyond its decl, keyed by the fn.
pub trait FnFacts {
    /// A read-only method that may still write its receiver's `@cache` fields.
    fn writes_cache(&self, f: FnId) -> bool;
    /// Of `first` and its overloads, the one whose labels `given` fit.
    fn overload(&self, first: FnId, given: &[Sym]) -> FnId;
    /// The params of `f` whose type is a domain.
    fn domain_params(&self, f: FnId) -> &[usize];
    /// The params of `f` whose type is ordered.
    fn ordered_params(&self, f: FnId) -> &[usize];
    /// The `.clear` of `@memo` fn `f`.
    fn memo_clear(&self, f: FnId) -> Option<FnId>;
    /// Whether `f`'s return type has holes its body fills.
    fn ret_hole(&self, f: FnId) -> bool;
    /// Whether trait `tr`'s argument `j` may be a domain in some impl.
    fn arg_may_be_domain(&self, tr: TraitId, j: usize) -> bool;
}

/// What the program's domains are, keyed by name or id.
pub trait DomFacts {
    /// The domain a top-level `domain` line names `name`.
    fn top_dom(&self, name: Sym) -> Option<&Ty>;
    /// Whether domain `c` is one a record's field makes fresh per value.
    fn fresh_dom(&self, c: usize) -> bool;
    /// Whether module `m` is reached by the program.
    fn seen(&self, m: ModId) -> bool;
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

/// The impl one trait's call takes at one type: its index and the type arguments its head binds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chosen {
    pub imp: crate::decls::ImplIdx,
    pub binds: Vec<Ty>,
}

/// `impl_for(TraitId, Ty)`: which impl a type takes, as the checker chose it. Keyed by the type
/// itself, compared by structure, so an instance's type that no item names has its answer too.
pub trait ImplChoice {
    /// The impl of `tr` that `t` takes: the head ranked first (R08.1.9), a domain's row as
    /// `ImplIdx::Row`. `None` when no impl's head fits or the chosen one's bounds fail.
    fn impl_for(&self, tr: TraitId, t: &Ty) -> Option<Chosen>;
    /// Each trait `t` implements with the impl it takes, by `TraitId`, then `ImplIdx`.
    fn impls_on(&self, t: &Ty) -> Vec<(TraitId, Chosen)>;
}

/// `checked(ItemId)`: what the checker answered about one item that link reads by id. No
/// method lists a whole table, so link reads only the items it instantiates.
pub trait Checked {
    /// `diagnostics(FnId)`: what the checker leaves to say once `f` is instantiated.
    fn deferred(&self, f: FnId) -> &[Msg];
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
    pub deferred: &'a Deferred,
    pub globals: &'a [Ty],
}

/// `generic(ItemId)`: one fn's generic IR.
pub trait Generics {
    fn generic(&self, key: &FuncKey) -> Option<&Func<Ty>>;
}

impl Items for Decls {
    fn f(&self, id: FnId) -> &FnDecl {
        &self.fns[id.0]
    }
    fn top_name(&self, g: cpl_next_ir::ty::TopValueId) -> Sym {
        self.top_values[g.0].name
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
    fn impls_by_trait(&self, tr: TraitId) -> &[usize] {
        self.impls_by_trait.get(&tr).map_or(&[], Vec::as_slice)
    }
    fn ranked(&self, tr: TraitId) -> Option<&[usize]> {
        (self.impls_ranked.as_ref()).map(|r| r.get(&tr).map_or(&[][..], Vec::as_slice))
    }
    fn impls_named(&self, name: Sym) -> &[usize] {
        self.impls_by_member.get(&name).map_or(&[], Vec::as_slice)
    }
    fn live(&self, i: usize) -> bool {
        Decls::live(self, &self.impls[i])
    }
}

impl Signatures for Decls {
    fn literal_trait(&self, op: &str) -> Option<TraitId> {
        Decls::literal_trait(self, op)
    }
    fn impls(&self) -> Impls<'_> {
        Impls::of(self)
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

impl Texts for Decls {
    fn text(&self, s: Sym) -> &str {
        self.syms.text(s)
    }
}

impl TypeTables for Decls {
    fn sym(&self, text: &str) -> Option<crate::sym::Sym> {
        self.syms.get(text)
    }
    fn core_type(&self, n: crate::sym::Sym) -> Option<crate::decls::CoreType> {
        self.core_types.get(&n).copied()
    }

    fn alias(&self, id: AliasId) -> &AliasDecl {
        &self.aliases[id.0]
    }
    fn konst(&self, id: ConstId) -> &LibConst {
        &self.consts[id.0]
    }
    fn top_value(
        &self,
        name: &str,
    ) -> Option<(cpl_next_ir::ty::TopValueId, crate::decls::TopValue)> {
        let g = crate::decls::top_value(&self.top_values, self.sym(name)?)?;
        Some((g, self.top_values[g.0].clone()))
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

impl FnFacts for Decls {
    fn writes_cache(&self, f: FnId) -> bool {
        self.writes_cache(f)
    }
    fn overload(&self, first: FnId, given: &[Sym]) -> FnId {
        let given: Vec<&str> = given.iter().map(|s| self.syms.text(*s)).collect();
        self.overload(first, &given)
    }
    fn domain_params(&self, f: FnId) -> &[usize] {
        self.domain_params.get(&f).map_or(&[], Vec::as_slice)
    }
    fn ordered_params(&self, f: FnId) -> &[usize] {
        self.ordered_params.get(&f).map_or(&[], Vec::as_slice)
    }
    fn memo_clear(&self, f: FnId) -> Option<FnId> {
        self.memo_clears.get(&f).copied()
    }
    fn ret_hole(&self, f: FnId) -> bool {
        self.ret_holes.contains(&f)
    }
    fn arg_may_be_domain(&self, tr: TraitId, j: usize) -> bool {
        self.arg_may_be_domain(tr, j)
    }
}

impl DomFacts for Decls {
    fn top_dom(&self, name: Sym) -> Option<&Ty> {
        self.top_doms.get(&name)
    }
    fn fresh_dom(&self, c: usize) -> bool {
        self.fresh_doms.iter().any(|f| f.0 == c)
    }
    fn seen(&self, m: ModId) -> bool {
        self.seen(m)
    }
}

impl Names for Decls {
    fn named(&self, m: ModId, name: Sym) -> Option<Named> {
        self.views[m.0].get(&name).copied()
    }
    fn export(&self, m: ModId, name: Sym) -> Option<(Named, bool)> {
        self.exports[m.0].get(&name).copied()
    }
    fn is_ambiguous(&self, m: ModId, name: Sym) -> bool {
        self.ambiguous[m.0].contains(&name)
    }
    fn top_value(&self, m: ModId, name: Sym) -> Option<cpl_next_ir::ty::TopValueId> {
        (m == self.program)
            .then(|| crate::decls::top_value(&self.top_values, name))
            .flatten()
    }
    fn names(&self, m: ModId) -> Vec<(Sym, Named)> {
        self.views[m.0].iter().map(|(n, x)| (*n, *x)).collect()
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
    fn deferred(&self, f: FnId) -> &[Msg] {
        self.deferred.of(f)
    }
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
