//! The four views a layer fn reads instead of the whole program (handbook/design/next/
//! incremental.md, "The whole-program inputs, one by one"). The query database implements each
//! with per-item queries; today `Decls`, `SetAdds` and the lowered fns implement them, and a
//! test can with a map. A layer fn whose step is done takes these, never `&Decls` or a map of
//! every fn.
use crate::decls::{
    AliasDecl, AliasId, ConstId, Decls, FnDecl, ImplDecl, Impls, LibConst, ModId, Named, Role,
    Roles, StructDecl, Sugar, TraitDecl,
};
use crate::sym::Sym;
use cpl_next_ir::ir::{Func, FuncKey};
use cpl_next_ir::msg::{Deferred, Msg};
use cpl_next_ir::ty::{FnId, FnRef, SetAdds, SetTy, StructId, TraitId, Ty};
use cpl_next_ir::whole::{Dom, DomId, Global, GlobalId, Whole};
use cpl_next_syntax::grammar::{Decorator, RoleMember};
use std::collections::HashMap;

/// The part of the signatures a written type is converted against, each table read by key.
/// Decls reads it while it is still building the rest of the signatures.
pub trait TypeTables: Texts + Shapes + RoleStructs {
    fn alias(&self, id: AliasId) -> &AliasDecl;
    fn konst(&self, id: ConstId) -> &LibConst;
    /// The core type written `n`: `Int`, `Raw`.
    fn core_type(&self, n: crate::sym::Sym) -> Option<crate::decls::CoreType>;
    /// The value the program's top level binds as `name`, when it binds one: a value, not an item.
    fn top_value(
        &self,
        name: crate::sym::Sym,
    ) -> Option<(cpl_next_ir::ty::TopValueId, crate::decls::TopValue)>;
    /// Binary operator `op`'s fn, which `type_of(+)` names.
    fn operator(&self, op: crate::sym::Sym) -> Option<FnId>;
    /// `Domain`'s getter `m`, which a range type's end `D.m` names.
    fn getter(&self, m: crate::sym::Sym) -> Option<FnId>;
}

/// The declarations filling the language roles (R11.6.1) and the rows `@operator`, `@literal`
/// and the fn marks add, each read by key.
pub trait RoleRows: RoleStructs {
    /// The trait filling role `r`: iteration, positions, order, convert.
    fn role_trait(&self, r: Role) -> Option<TraitId>;
    /// The fn filling role `r`: `print`, `todo`, the string literal's builder.
    fn role_fn(&self, r: Role) -> Option<FnId>;
    /// The fns filling a role many fns fill: `@role(trace)`'s.
    fn role_fns(&self, r: Role) -> &[FnId];
    /// The fns marked `@d`: `@input`, `@index`, `@pattern`, `@from_op` or `@coerce`.
    fn marked_fns(&self, d: Decorator) -> &[FnId];
    /// Binary operator `op`'s fn, its `@operator` trait's fn of two operands.
    fn binary_operator(&self, op: Sym) -> Option<FnId>;
    /// Unary operator `op`'s fn, its `@operator` trait's fn of one operand.
    fn unary_operator(&self, op: Sym) -> Option<FnId>;
    /// The `@operator(op)` traits, in the order core declares them.
    fn operator_traits(&self, op: Sym) -> Vec<TraitId>;
    /// The fn an `@operator` trait's operator calls: its first declared (R06.4.12).
    fn operator_method(&self, tr: TraitId) -> Option<(Sym, FnId)>;
    /// The trait `@literal(op)` marks: `[]` a list literal builds through, `[;]` an array fill.
    fn literal_trait(&self, op: Sym) -> Option<TraitId>;
    /// Trait `tr`'s yielding method every impl writes: the iteration role's walk.
    fn walk(&self, tr: TraitId) -> Option<(Sym, FnId)>;
    /// Trait `tr`'s yielding method with a default body (`each`), which an impl may write.
    fn own_walk(&self, tr: TraitId) -> Option<(Sym, FnId)>;
    /// Role member `m`'s name and fn in its role's trait: `MemoTable`'s `get` and `set`.
    fn role_member(&self, m: RoleMember) -> Option<(Sym, FnId)>;
    /// Whether `tr` is an `@operator` trait.
    fn is_operator_trait(&self, tr: TraitId) -> bool;
    /// The operator `tr`'s `@operator(op)` gives: `**` for `Pow`.
    fn operator_of(&self, tr: TraitId) -> Option<Sym>;
}

/// The structs filling the language roles, which a written type converts through: `?T` the
/// option role's, `[D]T` the array role's.
pub trait RoleStructs {
    /// The struct filling role `r`: `Optional`, `List`, the `@role(cuts)` struct.
    fn role_struct(&self, r: Role) -> Option<StructId>;
}

/// The struct `roles` and `sugar` say fills role `r`.
pub(crate) fn role_struct_of(roles: &Roles, sugar: &Sugar, r: Role) -> Option<StructId> {
    match r {
        Role::Option => roles.option,
        Role::Recipe => roles.recipe,
        Role::Cuts => roles.cut,
        Role::Pack => roles.pack,
        Role::List => sugar.list,
        Role::Str => sugar.str,
        Role::Map => sugar.map,
        Role::Array => sugar.array,
        _ => None,
    }
}

/// The shapes of the structs, each read by key: a struct's sets, defaults, fields and variants,
/// its siblings by arity, and the tuple, record and `@member` structs a written type names.
pub trait Shapes: Texts {
    /// How many set parameters struct `s` has, so a struct named in a type gets its own.
    fn n_sets(&self, s: StructId) -> usize;
    /// Each of struct `s`'s type parameters' default, when it declares any.
    fn struct_defaults(&self, s: StructId) -> Option<&Vec<Option<Ty>>>;
    /// The struct of `s`'s name with `n` type parameters, else `s` (R02.11.4).
    fn by_arity(&self, s: StructId, n: usize) -> StructId;
    /// Core's `@role(tuple)` struct with `n` fields; `None` past the widest, where a longer tuple
    /// nests its last parts.
    fn tuple(&self, n: usize) -> Option<StructId>;
    /// The record struct with these field names, in this order.
    fn record(&self, fields: &[Sym]) -> Option<StructId>;
    /// Record `s`'s field names, or `None` when `s` is not a record.
    fn record_fields(&self, s: StructId) -> Option<Vec<Sym>>;
    /// The names of struct `s`'s fields, in order.
    fn field_names(&self, s: StructId) -> Vec<Sym>;
    /// The `@member(m)` struct.
    fn member(&self, m: Sym) -> Option<StructId>;
    /// Which of enum `e`'s variants is named `name`.
    fn variant(&self, e: StructId, name: Sym) -> Option<usize>;
    /// `Tagged<Str>` written in a body: its arguments, the defaults filling those left off.
    fn with_defaults_of(&self, s: StructId, given: &[Ty]) -> Vec<Ty> {
        let mut given = given.to_vec();
        if !given.is_empty() {
            crate::decls::fill_defaults(self.struct_defaults(s), &mut given);
        }
        given
    }
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
    /// Context `id`, declared by `context` (R09.1.1).
    fn context(&self, id: crate::decls::ContextId) -> &crate::decls::ContextDecl;
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
    /// The fns item `item` writes, in the order they were declared.
    fn fns_of(&self, item: &cpl_next_ir::ids::ItemId) -> &[FnId];
    /// The traits with an impl whose head may fit `t`, ascending, each once: a struct's own
    /// impls and the impls headed by anything else (a type not known yet may fit every impl).
    fn impl_traits(&self, t: &Ty) -> Vec<TraitId>;
}

/// `signature(ItemId)`: what an item declares, never its body.
pub trait Signatures: TypeTables + Items {
    /// The id of a name written `text`, if any table holds it.
    fn sym(&self, text: &str) -> Option<crate::sym::Sym>;
    /// The names' ids, which a type written in a body is read through.
    fn syms(&self) -> &crate::sym::Syms;
    fn sugar(&self) -> Sugar;
    fn roles(&self) -> &Roles;
    /// The impl lookup over these signatures (`Impls`): the one place impl questions go.
    fn impls(&self) -> Impls<'_>;
    /// The trait `@literal(op)` marks: `"[]"` a list literal builds through.
    fn literal_trait(&self, op: &str) -> Option<TraitId>;
    /// The traits that declare a method `name`, by id (`traits(name)`, indexed from the
    /// signatures).
    fn traits_with(&self, name: Sym) -> Vec<TraitId>;
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
    /// The trait and method a plain call `name(x, ..)` in module `m` is, when `name` is a
    /// `@free_form` method of a trait in `m`'s view: the call is `x.name(..)` (R01.10.1).
    fn free_form(&self, m: ModId, name: Sym) -> Option<(TraitId, Sym)>;
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
    /// The std modules that export `name` as `pub`, in module order: where a `use` fix
    /// finds it.
    fn exporters(&self, name: Sym) -> Vec<ModId>;
    /// Module `m`'s dotted name (`std.seq`).
    fn module_name(&self, m: ModId) -> &str;
    /// Whether `m` is the program or an inline module inside it.
    fn of_program(&self, m: ModId) -> bool;
    /// What module `m` says of itself, once wherever its items are reached.
    fn module_errors(&self, m: ModId) -> &[Msg];
}

/// What a fn's signature says beyond its decl, keyed by the fn.
pub trait FnFacts {
    /// A read-only method that may still write its receiver's `@cache` fields.
    fn writes_cache(&self, f: FnId) -> bool;
    /// The other fns of `first`'s name, which a call's labels choose between (R06.1.6).
    fn overloads(&self, first: FnId) -> &[FnId];
    /// The params of `f` whose type is a domain.
    fn domain_params(&self, f: FnId) -> &[usize];
    /// The params of `f` whose type is ordered.
    fn ordered_params(&self, f: FnId) -> &[usize];
    /// The `.clear` of `@memo` fn `f`.
    fn memo_clear(&self, f: FnId) -> Option<FnId>;
    /// Whether `f`'s return type has holes its body fills.
    fn ret_hole(&self, f: FnId) -> bool;
    /// Whether fn `f` was refused as declared, so a call of it is downstream of that error.
    fn refused(&self, f: FnId) -> bool;
    /// Whether fn `f` was rejected as declared, so its body is not checked.
    fn body_rejected(&self, f: FnId) -> bool;
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

/// `impl_for(TraitId, args, Ty)`: which impl a type takes, as the checker chose it. Keyed by the trait
/// with its arguments (`Int` has `Into<Char>` and `Into<U64>`) and the type itself, compared by
/// structure, so an instance's type that no item names has its answer too.
pub trait ImplChoice {
    /// The impl of `r` that `t` takes: the head ranked first (R08.1.9), a domain's row as
    /// `ImplIdx::Row`. `None` when no impl's head fits, two do, or the chosen one's bounds fail.
    fn impl_for(&self, tr: TraitId, args: &[Ty], t: &Ty) -> Option<Chosen>;
    /// Each trait `t` implements with the impl it takes, by `TraitId`, then `ImplIdx`.
    fn impls_on(&self, t: &Ty) -> Vec<(TraitId, Chosen)>;
}

/// Methods by name, each keyed by its `Sym`: the candidates a member call picks among. It lists
/// candidates only, never fits or ranks them; which one a receiver takes is `ImplChoice`'s.
pub trait Methods {
    /// The library methods named `name`, from any std module: a method call needs no import
    /// (R11.3.4).
    fn methods_named(&self, name: Sym) -> &[FnId];
    /// The fns named `name` of impls over a shape of struct `s` (`impl Rev<[D]T>:`, R08.1.3).
    fn shaped(&self, s: StructId, name: Sym) -> &[FnId];
    /// The traits that declare a fn named `name`.
    fn traits_declaring(&self, name: Sym) -> Vec<TraitId>;
    /// The fns once, or also, called `name` (`@aka`, `@was`).
    fn renamed_from(&self, name: Sym) -> Vec<FnId>;
    /// Trait `tr`'s fns in the order it declares them (R01.8.8).
    fn trait_fns(&self, tr: TraitId) -> Vec<(Sym, FnId)>;
}

/// `item(ItemId)` and the module text a `SourceId` lies in: what a body's checker reads of the
/// source. `Loaded` serves it until the db answers `ItemOf`/`Parse` per query.
pub trait Bodies {
    /// Item `id`'s syntax, its spans counted from its start.
    fn item(&self, id: &cpl_next_ir::ids::ItemId) -> Option<cpl_next_syntax::ast::Item>;
    /// The text of the module `at` lies in.
    fn source(&self, at: &cpl_next_ir::ids::SourceId) -> &crate::modules::Text;
    /// The source text `at` covers.
    fn text(&self, at: &cpl_next_ir::ids::SourceId) -> &str;
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
    fn context(&self, id: crate::decls::ContextId) -> &crate::decls::ContextDecl {
        &self.contexts[id.0]
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
    fn fns_of(&self, item: &cpl_next_ir::ids::ItemId) -> &[FnId] {
        Decls::fns_of(self, item)
    }
    fn impl_traits(&self, t: &Ty) -> Vec<TraitId> {
        let mut out: Vec<TraitId> = match t {
            Ty::Var(_) | Ty::Error => self.impls_by_trait.keys().copied().collect(),
            Ty::Struct(s, ..) => (self.impls_by_head.get(s).into_iter().flatten())
                .chain(&self.impls_unheaded)
                .copied()
                .collect(),
            _ => self.impls_unheaded.clone(),
        };
        out.sort();
        out.dedup();
        out
    }
}

impl Signatures for Decls {
    fn sym(&self, text: &str) -> Option<crate::sym::Sym> {
        self.syms.get(text)
    }
    fn syms(&self) -> &crate::sym::Syms {
        &self.syms
    }
    fn sugar(&self) -> Sugar {
        self.sugar
    }
    fn roles(&self) -> &Roles {
        &self.roles
    }
    fn literal_trait(&self, op: &str) -> Option<TraitId> {
        Decls::literal_trait(self, op)
    }
    fn impls(&self) -> Impls<'_> {
        Impls::of(self)
    }
    fn traits_with(&self, name: Sym) -> Vec<TraitId> {
        (0..self.traits.len())
            .map(TraitId)
            .filter(|tr| self.traits[tr.0].methods.contains_key(&name))
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
        name: crate::sym::Sym,
    ) -> Option<(cpl_next_ir::ty::TopValueId, crate::decls::TopValue)> {
        let g = crate::decls::top_value(&self.top_values, name)?;
        Some((g, self.top_values[g.0].clone()))
    }
    fn operator(&self, op: crate::sym::Sym) -> Option<FnId> {
        self.operator_fn_of(op, true)
    }
    fn getter(&self, m: crate::sym::Sym) -> Option<FnId> {
        let tr = self.roles.domain?;
        (self.member(&self.traits[tr.0].methods, self.syms.text(m)))
            .filter(|f| self.fns[f.0].getter)
    }
}

impl RoleStructs for Decls {
    fn role_struct(&self, r: Role) -> Option<StructId> {
        role_struct_of(&self.roles, &self.sugar, r)
    }
}

impl RoleRows for Decls {
    fn role_trait(&self, r: Role) -> Option<TraitId> {
        match r {
            Role::Iteration => self.roles.iteration,
            Role::Positions => self.roles.domain,
            Role::Order => self.roles.order,
            Role::Convert => self.roles.convert,
            Role::Counted => self.roles.counted,
            Role::RangeBounds => self.roles.bounds,
            Role::Endpoint => self.roles.endpoint,
            Role::DeriveFields => self.roles.derive_fields,
            Role::MemoTable => self.roles.memo_table,
            _ => None,
        }
    }
    fn role_fn(&self, r: Role) -> Option<FnId> {
        match r {
            Role::Print => self.roles.print,
            Role::Write => self.roles.write,
            Role::Todo => self.roles.todo,
            Role::Interpolation => self.roles.interpolate,
            Role::VariantCheck => self.roles.variant_check,
            Role::StrLiteral => self.sugar.str_literal,
            Role::RangeLiteral => self.sugar.range_literal,
            _ => None,
        }
    }
    fn role_fns(&self, r: Role) -> &[FnId] {
        match r {
            Role::Trace => &self.roles.trace,
            _ => &[],
        }
    }
    fn marked_fns(&self, d: Decorator) -> &[FnId] {
        match d {
            Decorator::Input => &self.input_fns,
            Decorator::Index => &self.index_fns,
            Decorator::Pattern => &self.pattern_fns,
            Decorator::FromOp => &self.from_op,
            Decorator::Coerce => &self.coercions,
            _ => &[],
        }
    }
    fn binary_operator(&self, op: Sym) -> Option<FnId> {
        self.operator_fn_of(op, true)
    }
    fn unary_operator(&self, op: Sym) -> Option<FnId> {
        self.operator_fn_of(op, false)
    }
    fn operator_traits(&self, op: Sym) -> Vec<TraitId> {
        self.operator_traits_of(op).collect()
    }
    fn operator_method(&self, tr: TraitId) -> Option<(Sym, FnId)> {
        self.operator_method_of(tr)
    }
    fn literal_trait(&self, op: Sym) -> Option<TraitId> {
        self.literal_trait_of(op)
    }
    fn walk(&self, tr: TraitId) -> Option<(Sym, FnId)> {
        self.walk_of(tr, false)
    }
    fn own_walk(&self, tr: TraitId) -> Option<(Sym, FnId)> {
        self.walk_of(tr, true)
    }
    fn role_member(&self, m: RoleMember) -> Option<(Sym, FnId)> {
        self.role_members.get(&m).copied()
    }
    fn is_operator_trait(&self, tr: TraitId) -> bool {
        self.operators.iter().any(|&(_, t)| t == tr)
    }
    fn operator_of(&self, tr: TraitId) -> Option<Sym> {
        (self.operators.iter()).find_map(|&(o, t)| (t == tr).then_some(o))
    }
}

impl Shapes for Decls {
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
    fn record(&self, fields: &[Sym]) -> Option<StructId> {
        crate::decls::record_of(&self.records, &self.syms, fields)
    }
    fn record_fields(&self, s: StructId) -> Option<Vec<Sym>> {
        crate::decls::fields_of_record(&self.records, &self.syms, s)
    }
    fn field_names(&self, s: StructId) -> Vec<Sym> {
        self.structs[s.0].fields.iter().map(|(f, _)| *f).collect()
    }
    fn member(&self, m: Sym) -> Option<StructId> {
        self.member_types.get(&m).copied()
    }
    fn variant(&self, e: StructId, name: Sym) -> Option<usize> {
        (self.structs[e.0].variants.iter()).position(|(v, _)| *v == name)
    }
}

impl FnFacts for Decls {
    fn writes_cache(&self, f: FnId) -> bool {
        self.writes_cache(f)
    }
    fn overloads(&self, first: FnId) -> &[FnId] {
        self.overloads.get(&first).map_or(&[], Vec::as_slice)
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
    fn refused(&self, f: FnId) -> bool {
        self.refused_fns.contains(&f)
    }
    fn body_rejected(&self, f: FnId) -> bool {
        self.rejected_bodies.contains(&f)
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
    fn free_form(&self, m: ModId, name: Sym) -> Option<(TraitId, Sym)> {
        Decls::free_form(self, m, name)
    }
    fn top_value(&self, m: ModId, name: Sym) -> Option<cpl_next_ir::ty::TopValueId> {
        (m == self.program)
            .then(|| crate::decls::top_value(&self.top_values, name))
            .flatten()
    }
    fn names(&self, m: ModId) -> Vec<(Sym, Named)> {
        self.views[m.0].iter().map(|(n, x)| (*n, *x)).collect()
    }
    fn exporters(&self, name: Sym) -> Vec<ModId> {
        (0..self.module_names.len())
            .filter(|&i| ModId(i) != crate::decls::CORE && self.of_std[i])
            .filter(|&i| self.exports[i].get(&name).is_some_and(|e| e.1))
            .map(ModId)
            .collect()
    }
    fn module_name(&self, m: ModId) -> &str {
        &self.module_names[m.0]
    }
    fn of_program(&self, m: ModId) -> bool {
        Decls::of_program(self, m)
    }
    fn module_errors(&self, m: ModId) -> &[Msg] {
        &self.module_errors[m.0]
    }
}

impl Bodies for crate::modules::Loaded {
    fn item(&self, id: &cpl_next_ir::ids::ItemId) -> Option<cpl_next_syntax::ast::Item> {
        (self.modules.iter())
            .filter(|m| m.name == *id.module)
            .flat_map(|m| m.items.iter())
            .find(|i| i.id == *id)
            .map(|i| i.item.clone())
    }
    fn source(&self, at: &cpl_next_ir::ids::SourceId) -> &crate::modules::Text {
        self.text_of(at)
    }
    fn text(&self, at: &cpl_next_ir::ids::SourceId) -> &str {
        self.text(at)
    }
}

impl Methods for Decls {
    fn methods_named(&self, name: Sym) -> &[FnId] {
        self.methods.get(&name).map_or(&[], Vec::as_slice)
    }
    fn shaped(&self, s: StructId, name: Sym) -> &[FnId] {
        self.structs[s.0]
            .shaped
            .get(&name)
            .map_or(&[], Vec::as_slice)
    }
    fn traits_declaring(&self, name: Sym) -> Vec<TraitId> {
        Signatures::traits_with(self, name)
    }
    fn renamed_from(&self, name: Sym) -> Vec<FnId> {
        Signatures::renamed(self, self.syms.text(name))
    }
    fn trait_fns(&self, tr: TraitId) -> Vec<(Sym, FnId)> {
        self.trait_fns(tr)
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
