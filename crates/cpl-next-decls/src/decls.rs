//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
pub use crate::cycles::{alias_cycle, field_holding_itself};
use crate::diag::Msg;
use crate::ty::*;
use cpl_diag::Span;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod collect;
mod impls;
mod inherent;
mod layer;
mod tyconv;
pub use collect::{collect, factor_name, prepare, Prepared};
pub use layer::Layer;
use tyconv::*;
pub use tyconv::{body_type, prim_type};
pub(crate) use tyconv::{names_in, unnamed, GETTER};

/// A module: the core, a library file or the program.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Default)]
pub struct ModId(pub usize);

/// The core's module: the language's own declarations, seen by every module.
pub const CORE: ModId = ModId(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelfMode {
    None,
    Value,
    Mut,
}

#[derive(Clone)]
pub struct FnDecl {
    pub name: String,
    /// The struct, trait or impl it sits in.
    pub owner: Option<Owner>,
    pub self_mode: SelfMode,
    /// What the body calls its receiver: `self`, or what an impl writes in its place
    /// (`fn lt(a, b)` for the trait's `fn lt(self, other: Self)`).
    pub self_name: String,
    /// The type of `self`, in `generics`, when it has one.
    pub self_ty: Option<Ty>,
    /// Type parameters: the owner's first, then its own.
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    /// Without `self`.
    pub params: Vec<Param>,
    /// The names of the parameters passed only by name.
    pub labels: Vec<String>,
    pub ret: Ty,
    /// Implicit set parameters: one per fn type in the parameters.
    pub n_sets: usize,
    /// How many function types its result holds.
    pub n_ret_sets: usize,
    /// A fn that runs the caller's code in the middle of its own: a generator (`yields T`,
    /// `false`) or a lender (`lends T`, `true`).
    pub region: Option<(bool, Ty)>,
    /// A read `lends T` result: the parameters it borrows, receiver first (R05.6.1).
    pub lends: Option<Vec<usize>>,
    pub body: FnBody,
    /// The name in `= @intrinsic(name)`, when that is its body.
    pub intrinsic: Option<String>,
    /// `@fast(op)` on a fn with a CP body: the operation the engines and C++ run in place
    /// of a call to it; the reference interpreter runs the body.
    pub fast: Option<String>,
    pub getter: bool,
    /// `uses io, opt`: contexts its caller's values flow into (R09.1.3).
    pub uses: Vec<String>,
    /// `@same_keys`: its arguments that are arrays over domains share one domain (R04b.1.3).
    pub same_keys: bool,
    /// `@aka(..)`: names another language calls it; a call by one names this fn (R06.4.11).
    pub aka: Vec<String>,
    /// `@was(..)`: names it had before a release renamed it; a call by one is E0503.
    pub was: Vec<String>,
    pub item: ItemId,
    /// Where it is written.
    pub span: SourceId,
    pub module: ModId,
    /// When the collector made it: the round, then the pass (traits, fns and types, impls,
    /// derived impls). Std's and a program's together are checked in this order.
    pub made: (u32, u8),
    /// What its signature uses that the slice does not build, said only when a call reaches it.
    pub errors: Vec<Msg>,
}
#[derive(Clone)]
pub struct Param {
    pub name: String,
    /// The name a call gives it by: its label, else its name.
    pub label: String,
    pub ty: Ty,
    pub mutable: bool,
    /// After `;`: passed by name only.
    pub named_only: bool,
    /// `..xs: Ts`: the arguments past the others, as a tuple, or a record when named (R06.1.5).
    pub variadic: bool,
    /// Read in the fn's module when a call leaves the parameter out (R06.1.5).
    pub default: Option<ast::Expr>,
}
#[derive(Clone)]
pub struct StructDecl {
    pub name: String,
    /// A record's struct, `(lo:, hi:)` (R02.5.2), shown and matched by its fields.
    pub record: bool,
    pub generics: Vec<String>,
    pub fields: Vec<(String, Ty)>,
    /// Per field: declared with `var`, so assignable after construction.
    pub var_fields: Vec<bool>,
    /// Per field: `@cache`, so a read-only method may write it (R05.10.10).
    pub cached: Vec<bool>,
    /// Per field: where its type is written, as an offset into the source.
    pub field_at: Vec<SourceId>,
    /// Per field: the value a constructor that leaves it out stores, read in `module` (R02.6.1).
    pub defaults: Vec<Option<ast::Expr>>,
    pub module: ModId,
    /// Where its declaration starts in the source: the spans of its defaults count from here.
    pub item: ItemId,
    pub n_sets: usize,
    pub methods: HashMap<String, FnId>,
    /// Its `const NAME = init`s, read as `Dir4.up`: each a const of its module, set once.
    pub consts: HashMap<String, ConstId>,
    /// The fns of its impls over a shape of it (`impl Rev<[D]T>:`), each `self` that shape.
    pub shaped: HashMap<String, Vec<FnId>>,
    /// Its static `new`, which `T(args)` calls outside its own body (R02.6.3).
    pub ctor: Option<FnId>,
    /// An enum's variants, each with the struct of its payload, named `Enum.Variant` (R02.7).
    pub variants: Vec<(String, StructId)>,
    /// What its fields use that the slice does not build, said only when the program reaches it.
    pub errors: Vec<Msg>,
    /// An enum's `Int` of each variant: its written value (`Neg = -1`), else its place from 0.
    pub values: Vec<i64>,
    /// The traits its parameters are bounded by (`struct Range<D: Domain>`).
    pub bounds: Vec<(usize, TraitId)>,
}
#[derive(Clone)]
pub struct TraitDecl {
    pub name: String,
    pub module: ModId,
    /// `trait Both = Sh + Nm`: met by meeting its supertraits, never implemented (R08.2.6).
    pub alias: bool,
    /// Its supertraits, in its own type parameters (R08.1.1).
    pub supers: Vec<TraitRef>,
    pub methods: HashMap<String, FnId>,
    /// What its library impls use that the slice does not build, said when one of its fns is reached.
    pub errors: Vec<Msg>,
    /// `trait Add<Rhs = Self>`: each parameter's default, in `Self` and the parameters before it.
    pub defaults: Vec<Option<Ty>>,
    /// `type Arc: Target<V>`: its associated types, its last parameters, which an impl writes
    /// in its body and a bound leaves out (R08.2.3).
    pub assoc: Vec<String>,
    /// The traits each associated type has, in `Self` and the trait's parameters.
    pub assoc_bounds: Vec<(usize, TraitRef)>,
    /// What a type lacking its impl is told (R08.1.8).
    pub unimplemented: Unimplemented,
    /// A row bound's trait, `(to:, ..)`: met by every record with those fields (R08.4.1).
    pub row: bool,
}
/// `@on_unimplemented(code: E0311, "...{T}...")` on a trait: the code a missing impl reports in
/// place of E0606, and the help it adds, `{T}` the type shown (R08.1.8).
#[derive(Clone, Default)]
pub struct Unimplemented {
    pub code: Option<cpl_diag::Code>,
    pub help: Option<String>,
}

impl TraitDecl {
    /// The arguments that choose an impl: all but the associated types, which the impl gives.
    pub fn inputs<'a, T>(&self, args: &'a [T]) -> &'a [T] {
        &args[..args.len().saturating_sub(self.assoc.len())]
    }
}
/// A trait with its arguments: `Seq<T>`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TraitRef {
    pub tr: TraitId,
    pub args: Vec<Ty>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConstId(pub usize);

/// A library's `const NAME[: T] = init`.
#[derive(Clone, Debug)]
pub struct LibConst {
    pub module: ModId,
    pub name: String,
    pub item: ItemId,
    pub ty: Option<ast::Type>,
    pub init: ast::Expr,
}

/// `context opt:` with its fields and their defaults (R09.1): values a `with` line sets for the
/// rest of a block, read as `opt.k`.
#[derive(Clone, Debug)]
pub struct ContextDecl {
    pub module: ModId,
    pub name: String,
    pub fields: Vec<ast::FieldDecl>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ContextId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AliasId(pub usize);
/// `type Name<G> = T`: `T` is read in its module's names, where `Name` itself means what it means
/// outside (R02.6.2).
#[derive(Clone)]
pub struct AliasDecl {
    pub name: String,
    pub module: ModId,
    /// Where it is declared.
    pub at: SourceId,
    pub generics: Vec<String>,
    pub ty: ast::Type,
    pub names: HashMap<String, Named>,
    /// Declared in the program, so `T` may name the program's domains (`type Cell = (Row, Col)`).
    pub program: bool,
}
/// `impl Trait<A> for T`: `trait_` and `for_` are in the impl's own type parameters, which are
/// the names it writes that no declaration has (`impl Seq<T> for List<T>`) and its bounds' (R08.1.2).
impl ImplDecl {
    /// For a bare type parameter (`impl<D: Domain> Eq for D`): any impl for a written type is
    /// more specific and is chosen over it (R08.1.2).
    pub fn blanket(&self) -> bool {
        matches!(self.for_, Ty::Param(_))
    }
}
impl Decls {
    /// Which impl a type takes when several fit: one for its written type (0) before one for
    /// every record with some fields (1), before any other blanket one (2) (R08.1.2, R08.4.5).
    pub fn impl_rank(&self, im: &ImplDecl) -> u8 {
        self.rank_of(&im.for_, &im.bounds)
    }
    /// `impl_rank` of an impl for `for_` with `bounds`.
    pub fn rank_of(&self, for_: &Ty, bounds: &[(usize, TraitRef)]) -> u8 {
        let Ty::Param(i) = *for_ else { return 0 };
        match (bounds.iter()).any(|(k, b)| *k == i && self.traits[b.tr.0].row) {
            true => 1,
            false => 2,
        }
    }
}
#[derive(Clone)]
pub struct ImplDecl {
    pub trait_: TraitRef,
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    pub for_: Ty,
    pub methods: HashMap<String, FnId>,
    /// Where it is written.
    pub at: SourceId,
    /// Where it says each of its trait's associated types, or `at` for a default.
    pub assoc_at: Vec<SourceId>,
    pub module: ModId,
    /// Made from the fields (R08.1.6): its `S(..)` is the generated constructor, never a `new`.
    pub derived: bool,
    /// A std-derived impl a program's written one replaces (R08.1.6): no lookup finds it.
    pub replaced: bool,
}
/// What a top-level name declares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Named {
    Fn(FnId),
    Struct(StructId),
    Trait(TraitId),
    Alias(AliasId),
    /// A library item the slice does not build: an index into `Decls::refused`, said only
    /// when a program names it.
    Refused(usize),
    /// A library's `const`: a global its initializer sets once, at program start (R01.6.1).
    Const(ConstId),
    /// `context io:`: compile-time settings a `with` line sets (R09.1).
    Context(ContextId),
    /// An inline `mod m:` to the module it sits in, or a module a `use` names without names:
    /// `m.f` names its `pub f` (R11.1.3, R11.2.1).
    Module(ModId),
}
/// What a method sits in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Struct(StructId),
    Trait(TraitId),
    /// An index into `Decls::impls`.
    Impl(usize),
}

/// The declarations std marks as what a literal or a type's syntax is: `[a, b]` is a `@list`,
/// `"s"` is a `@str` made by the `@str_literal` fn, `?T` is the `@optional` struct, `[D]T` the
/// `@array` one, and `lo..hi` the `@range_literal` fn's result.
#[derive(Default, Clone, Copy)]
pub struct Sugar {
    pub list: Option<StructId>,
    pub str: Option<StructId>,
    pub optional: Option<StructId>,
    pub map: Option<StructId>,
    pub array: Option<StructId>,
    pub str_literal: Option<FnId>,
    pub range_literal: Option<FnId>,
}

#[derive(Default, Clone)]
pub struct Decls {
    pub fns: Layer<FnDecl>,
    pub structs: Layer<StructDecl>,
    pub traits: Layer<TraitDecl>,
    pub impls: Layer<ImplDecl>,
    /// Per module, what each name means there: the core's names, then the prelude's (the
    /// program only, R11.3.1), then its own and what its `use` lines import (R11.2.3).
    pub views: Layer<Arc<HashMap<String, Named>>>,
    /// Per module, the names two of its `use` lines import as different items: a use of one
    /// is ambiguous (R08.1.4).
    pub ambiguous: Vec<HashSet<String>>,
    pub module_names: Vec<String>,
    /// Per module, the module an inline `mod` of it sits in (R11.1.3).
    pub within: Vec<Option<ModId>>,
    /// Per module, what each name it declares or exports means, and whether it is
    /// `pub`: `m.f` outside it.
    pub exports: Vec<Arc<HashMap<String, (Named, bool)>>>,
    pub program: ModId,
    /// Per library module, what it uses outside its fns that the slice does not build.
    pub module_errors: Vec<Vec<Msg>>,
    /// Why each `Named::Refused` library item is not built.
    pub refused: Vec<Unbuilt>,
    pub consts: Layer<LibConst>,
    pub contexts: Layer<ContextDecl>,
    pub struct_sets: Arc<HashMap<StructId, usize>>,
    /// `struct SegTree<D, T, F = Int>`: each struct's default type arguments, in its parameters.
    pub struct_defaults: Arc<HashMap<StructId, Vec<Option<Ty>>>>,
    /// Core's `@tuple` structs by arity: `(A, B)` is the one with two fields.
    pub tuples: Arc<HashMap<usize, StructId>>,
    /// A library's `@recipe` struct: the value of a `-> yields T` call made outside a `for`
    /// (R07.6.2). Its one field is the closure that walks the call.
    pub recipe: Option<StructId>,
    /// Core's `@member(m)` structs by `m`: `D.m` is the struct at `D` (R06).
    pub member_types: Arc<HashMap<String, StructId>>,
    /// The struct core marks `@cuts`: a half-open literal range of its values, where a
    /// `Range<D>` is wanted, is the piece of positions between them (R04.4.4).
    pub cut_struct: Option<StructId>,
    /// What literals and type syntax build, each found by its std marker.
    pub sugar: Sugar,
    /// Each record struct by its field names, in order: `(lo: A, hi: B)` is the one keyed
    /// `["lo", "hi"]` (R02.5.2).
    pub records: Arc<HashMap<Vec<String>, StructId>>,
    pub aliases: Layer<AliasDecl>,
    pub errors: Vec<Msg>,
    /// Declarations the program gets wrong: where, the code, what is wrong.
    pub misdeclared: Vec<(SourceId, cpl_diag::Code, String)>,
    /// Each library's public free fns with a `self`, by name: a method call finds them with no
    /// import (R11.3.4).
    pub methods: HashMap<String, Vec<FnId>>,
    /// The program's top-level domains, which its declarations may name: each is the
    /// `Ty::Dom` of its place among them, filled when its statement runs (R02.9).
    pub top_doms: HashMap<String, Ty>,
    /// The `domain C` a fn's result binds, `-> (domain C, [C]T)`: each is a domain of its own
    /// among `top_doms`, with the name the trace shows (R02.9.3).
    pub fresh_doms: Vec<(usize, String)>,
    /// The fns whose result's slots are `domain C`s, till those slots are made `Range<C>`s.
    pub fresh_slots: Vec<(FnId, Vec<usize>)>,
    /// `[0..2]T` in a declaration: the domain of each constant range an array is over, one for
    /// all that write it, as `(domain, lo, hi)` with `hi` inclusive (R04.3.4).
    pub anon_doms: Vec<(usize, i64, i64)>,
    /// The names the program's top level binds with `let`.
    pub top_values: Arc<HashSet<String>>,
    /// The trait core marks `@positions`: every domain's position type has it, and nothing else.
    pub domain_trait: Option<TraitId>,
    /// Core's `@operator("+")` traits by the operator they give: `a + b` on a type that is not
    /// a number is its impl's fn (R08.1.5).
    pub operators: Vec<(String, TraitId)>,
    /// `@literal("[]")` and `@literal("[;]")`: the trait a list literal and an array fill build
    /// through (R01.8.8).
    pub literals: Vec<(String, TraitId)>,
    /// The trait core marks `@convert`: `T(x)` for a core type `T` is `x`'s impl of it at `T`.
    pub convert_trait: Option<TraitId>,
    /// `@range_bounds`: the trait whose fns give a range's first and past-the-end cuts (R04b.1.5).
    pub bounds_trait: Option<TraitId>,
    /// Per fn, the type parameters that key an array it names: domain parameters (R04.2).
    pub domain_params: HashMap<FnId, Vec<usize>>,
    /// Core's `@coerce` fns: each converts a value of its parameter's type where its result's
    /// type is wanted (R04.8.10).
    pub coercions: Vec<FnId>,
    /// The structs marked `@memo_table(word)`, by word: the table `@memo(word)` keeps (R06.4.1).
    pub memo_tables: HashMap<String, String>,
    /// Static fns marked `@from_op`: what a bare operator becomes where their struct is wanted (R08.1.9).
    pub from_op: Vec<FnId>,
    /// Core's `@pattern` fns: `f(p)` as a pattern matches the optional `f` makes when it is
    /// present and its value matches `p` (R01.7.1).
    pub pattern_fns: Vec<FnId>,
    /// Core's `@input` fns: a value they give is input, so a range slot it fills is the
    /// input's contract (R10.2.2).
    pub input_fns: Vec<FnId>,
    /// Core's `@interpolation` fn: `put(mut out, x)` appends each piece of an interpolated string.
    pub interpolate: Option<FnId>,
    /// The `@index` fns, in declaration order.
    pub index_fns: Vec<FnId>,
    /// Per module, whether the program loads it: std's others are declared but out of sight.
    pub seen: Vec<bool>,
    /// The records std writes and no module the program loads does: out of sight with their impls.
    pub unseen_records: HashSet<StructId>,
    /// Each trait's impls, and each member name's (its trait's or its own), in order.
    pub impls_by_trait: HashMap<TraitId, Vec<usize>>,
    pub impls_by_member: HashMap<String, Vec<usize>>,
}

/// Where a method call resolved to.
pub enum Member {
    Field(usize, Ty),
    /// A method, with the receiver's type and set arguments.
    Method(FnId, Vec<Ty>, Vec<SetTy>),
}

impl Decls {
    /// `Tagged<Str>` written in a body: its arguments, the defaults filling those left off.
    pub fn with_defaults_of(&self, d: StructId, mut given: Vec<Ty>) -> Vec<Ty> {
        if given.is_empty() {
            return given;
        }
        fill_defaults(self.struct_defaults.get(&d), &mut given);
        given
    }
    /// The domains a declaration in module `m` may name.
    fn doms_in(&self, m: ModId) -> HashMap<String, Ty> {
        match m == self.program {
            true => self.top_doms.clone(),
            false => (self.anon_doms.iter())
                .map(|&(i, lo, hi)| (anon_key(lo, hi), Ty::Dom(i)))
                .collect(),
        }
    }
    /// What a declaration uses that the slice does not build: a library's is said when the program
    /// reaches it, the program's at once.
    fn module_or_program_errors(&mut self, m: ModId, errors: Vec<Msg>) {
        if self.is_library(m) {
            self.module_errors[m.0].extend(errors);
        } else {
            self.errors.extend(errors);
        }
    }
    /// The fn a bare operator names: its core `@operator` trait's fn of that many operands.
    pub fn operator_fn(&self, text: &str, binary: bool) -> Option<FnId> {
        (self.operators.iter().filter(|(o, _)| o == text)).find_map(|(_, tr)| {
            let (_, m) = self.operator_method(*tr)?;
            (self.f(m).params.len() == usize::from(binary)).then_some(m)
        })
    }
    /// The trait `@literal(op)` marks: `"[]"` a list literal builds through, `"[;]"` an array fill.
    pub fn literal_trait(&self, op: &str) -> Option<TraitId> {
        (self.literals.iter()).find_map(|(o, tr)| (o == op).then_some(*tr))
    }
    /// A trait's fns in the order it declares them, the role each plays in a literal (R01.8.8).
    pub fn trait_fns(&self, tr: TraitId) -> Vec<(&String, FnId)> {
        let mut fns: Vec<_> = (self.traits[tr.0].methods.iter())
            .map(|(n, &m)| (n, m))
            .collect();
        fns.sort_by_key(|&(_, m)| m);
        fns
    }
    /// The fn an `@operator` trait's operator calls: its first declared, `lt` of `Ord`'s
    /// `lt` and `cmp` (R06.4.12).
    pub fn operator_method(&self, tr: TraitId) -> Option<(&String, FnId)> {
        (self.traits[tr.0].methods.iter())
            .min_by_key(|&(_, &m)| m)
            .map(|(n, &m)| (n, m))
    }
    /// Each binary operator's fn, for `type_of(+)`.
    pub(crate) fn operator_fns(&self) -> Arc<Vec<(String, FnId)>> {
        Arc::new(
            (self.operators.iter())
                .filter_map(|(o, _)| Some((o.clone(), self.operator_fn(o, true)?)))
                .collect(),
        )
    }
    pub fn f(&self, d: FnId) -> &FnDecl {
        &self.fns[d.0]
    }
    pub fn s(&self, d: StructId) -> &StructDecl {
        &self.structs[d.0]
    }
    pub fn named(&self, m: ModId, name: &str) -> Option<Named> {
        self.views[m.0].get(name).copied()
    }
    pub fn fn_named(&self, m: ModId, name: &str) -> Option<FnId> {
        match self.named(m, name) {
            Some(Named::Fn(f)) => Some(f),
            _ => None,
        }
    }
    pub fn struct_named(&self, m: ModId, name: &str) -> Option<StructId> {
        struct_named(&self.views[m.0], name)
    }
    /// The core's fn of this name, whatever the module calls its own.
    pub fn core_fn(&self, name: &str) -> Option<FnId> {
        self.fn_named(CORE, name)
    }
    /// A read-only method that may still write its receiver's `@cache` fields, so a call
    /// lends the receiver for writing (R05.10.10).
    pub fn writes_cache(&self, f: FnId) -> bool {
        let fd = self.f(f);
        fd.self_mode == SelfMode::Value
            && matches!(fd.owner, Some(Owner::Struct(s)) if self.s(s).cached.contains(&true))
    }
    /// A library declaration's place is not the program's: its module's name.
    pub fn is_library(&self, m: ModId) -> bool {
        m != CORE && !self.of_program(m)
    }
    /// Whether `m` holds only items: a library, or an inline `mod` (R11.1.2, R11.1.3).
    pub fn items_only(&self, m: ModId) -> bool {
        self.is_library(m) || self.within.get(m.0).copied().flatten().is_some()
    }
    /// Whether `m` is the program or an inline module inside it.
    pub fn of_program(&self, mut m: ModId) -> bool {
        loop {
            if m == self.program {
                return true;
            }
            match self.within.get(m.0).copied().flatten() {
                Some(w) => m = w,
                None => return false,
            }
        }
    }
    /// Whether the program loads module `m`.
    pub fn seen(&self, m: ModId) -> bool {
        self.seen.get(m.0).copied().unwrap_or(true)
    }
    /// Whether a lookup may find impl `im`: no written impl replaces it, and its module is loaded
    /// or declares the type it is for, whose methods are found wherever it came from (R11.3.4).
    pub fn live(&self, im: &ImplDecl) -> bool {
        let own = matches!(im.for_, Ty::Struct(s, ..) if self.s(s).module == im.module);
        !im.replaced
            && (own || self.seen(im.module))
            && !matches!(im.for_, Ty::Struct(s, ..) if self.unseen_records.contains(&s))
    }
    /// The impls of `tr` a lookup may find, in order.
    pub fn trait_impls(&self, tr: TraitId) -> impl Iterator<Item = (usize, &ImplDecl)> + Clone {
        (self.impls_by_trait.get(&tr).into_iter().flatten())
            .map(|&k| (k, &self.impls[k]))
            .filter(|(_, im)| self.live(im))
    }
    fn push_impl(&mut self, im: ImplDecl) {
        let k = self.impls.len();
        self.impls_by_trait.entry(im.trait_.tr).or_default().push(k);
        let mut names: Vec<&String> = im.methods.keys().collect();
        names.extend(self.traits[im.trait_.tr.0].methods.keys());
        names.sort();
        names.dedup();
        for n in names {
            self.impls_by_member.entry(n.clone()).or_default().push(k);
        }
        self.impls.push(im);
    }
}

fn field_names(fs: &[(ast::Name, ast::Type)]) -> Vec<String> {
    fs.iter().map(|(n, _)| n.text.clone()).collect()
}

fn struct_named(names: &HashMap<String, Named>, name: &str) -> Option<StructId> {
    match names.get(name) {
        Some(Named::Struct(s)) => Some(*s),
        _ => None,
    }
}

/// A result's `domain C` slot holds the fresh domain's positions, a `Range<C>` (R02.9.3).
fn fresh_results(d: &mut Decls) {
    let range = (d.sugar.range_literal).and_then(|r| match &d.fns[r.0].ret {
        Ty::Struct(r, ..) => Some(*r),
        _ => None,
    });
    let Some(range) = range else {
        return;
    };
    for (f, slots) in std::mem::take(&mut d.fresh_slots) {
        if let Ty::Struct(_, args, _) = &mut d.fns[f.0].ret {
            for k in slots {
                if let Some(a) = args.get_mut(k) {
                    *a = Ty::Struct(range, vec![a.clone()], Vec::new());
                }
            }
        }
    }
}

/// A type parameter in a place whose struct bounds it by `Domain`, in a fn's signature or in its
/// owner's fields, is a domain parameter (`struct Dsu<V>: parent: [V]V`, `Range<D>`): its body
/// may use what every domain has (R04.2). So is one passed where a trait bound's parameter is
/// one (`S: Indexed<D, T>`, whose `keys` is a `Range<D>`). A caller is not held to `Domain`,
/// since a list keyed by `Int` passes as `[Int]T`.
fn domain_params(d: &mut Decls, from: usize) {
    let Some(domain) = d.domain_trait else {
        return;
    };
    // Also through a struct whose own parameter in that place is a domain: `self: Dsu<V>`.
    fn keys(d: &Decls, t: &Ty, domain: TraitId, i: usize, depth: usize) -> bool {
        let go = |t: &Ty| depth < 8 && keys(d, t, domain, i, depth + 1);
        match t {
            Ty::Struct(s, ts, _) => {
                ts.iter().any(go)
                    || (ts.iter().enumerate()).any(|(k, a)| {
                        (d.structs[s.0].bounds.contains(&(k, domain))
                            && domain_at(d, a, domain, i, depth))
                            || (*a == Ty::Param(i)
                                && depth < 8
                                && (d.structs[s.0].fields.iter())
                                    .any(|(_, f)| keys(d, f, domain, k, depth + 1)))
                    })
            }
            Ty::Raw(e) | Ty::Ref(e) => go(e),
            Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).any(go),
            _ => false,
        }
    }
    // `P` in a domain's place is one, and so is a part of it that `P`'s `Domain` impl bounds by
    // `Domain`: `V` in `[(V, V)]T` (R04.2.11).
    fn domain_at(d: &Decls, t: &Ty, domain: TraitId, i: usize, depth: usize) -> bool {
        let Ty::Struct(s, ts, _) = t else {
            return *t == Ty::Param(i);
        };
        depth < 8
            && (d.impls.iter()).any(|m| {
                let Ty::Struct(f, ps, _) = &m.for_ else {
                    return false;
                };
                m.trait_.tr == domain && f == s && (ps.iter().zip(ts)).any(|(p, a)| {
                matches!(p, Ty::Param(j) if m.bounds.iter().any(|(b, r)| b == j && r.tr == domain))
                    && domain_at(d, a, domain, i, depth + 1)
            })
            })
    }
    // A parameter whose body names one of `Domain`'s fns on it (`D.nth(k)`) is one (R04.4.30).
    struct DomUse<'a> {
        generics: &'a [String],
        fns: Vec<&'a String>,
        found: Vec<usize>,
    }
    impl cpl_syntax::visit::Visitor for DomUse<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            let ast::ExprKind::Field { base, name } = &e.kind else {
                return;
            };
            let ast::ExprKind::Name(n) = &base.kind else {
                return;
            };
            let i = self.generics.iter().position(|g| g == n);
            if let Some(i) = i.filter(|i| self.fns.contains(&&name.text) && !self.found.contains(i))
            {
                self.found.push(i);
            }
        }
    }
    let dom_fns: Vec<&String> = d.traits[domain.0].methods.keys().collect();
    let used: Vec<Vec<usize>> = (d.fns.iter().skip(from))
        .map(|f| {
            let mut v = DomUse {
                generics: &f.generics,
                fns: dom_fns.clone(),
                found: Vec::new(),
            };
            match &f.body {
                FnBody::Block(b) => cpl_syntax::visit::walk_block(b, &mut v),
                FnBody::Expr(e) => cpl_syntax::visit::walk_expr(e, &mut v),
                FnBody::None => {}
            }
            v.found
        })
        .collect();
    for k in from..d.fns.len() {
        let f = &d.fns[k];
        let fields: &[(String, Ty)] = match f.owner {
            Some(Owner::Struct(s)) => &d.structs[s.0].fields,
            _ => &[],
        };
        let added: Vec<usize> = (0..f.generics.len())
            .filter(|&i| {
                (f.params.iter().map(|p| &p.ty))
                    .chain([&f.ret])
                    .chain(&f.self_ty)
                    .chain(fields.iter().map(|(_, t)| t))
                    .any(|t| keys(d, t, domain, i, 0))
                    || used[k - from].contains(&i)
            })
            .collect();
        if !added.is_empty() {
            d.domain_params.insert(FnId(k), added);
        }
    }
    // A struct's parameter that one of its methods treats as a domain is one in all of them.
    let mut of_struct: HashMap<StructId, Vec<usize>> = HashMap::new();
    for (f, ds) in &d.domain_params {
        if let Some(Owner::Struct(s)) = d.fns[f.0].owner {
            let n = d.structs[s.0].generics.len();
            let js = of_struct.entry(s).or_default();
            js.extend(
                ds.iter()
                    .filter(|&&j| j < n && !js.contains(&j))
                    .collect::<Vec<_>>(),
            );
        }
    }
    for k in from..d.fns.len() {
        let Some(Owner::Struct(s)) = d.fns[k].owner else {
            continue;
        };
        let Some(js) = of_struct.get(&s) else {
            continue;
        };
        let ds = d.domain_params.entry(FnId(k)).or_default();
        for &j in js {
            if !ds.contains(&j) {
                ds.push(j);
            }
        }
    }
    // Through bounds, until no trait's or fn's domain parameters grow.
    loop {
        // A trait's parameter is a domain when one of its methods uses it as one, or when it is
        // passed to a supertrait's domain parameter. A supertrait's args are in the trait's own
        // parameters, without `Self`: `IndexedMut<K, T>: Indexed<K, T>`.
        fn trait_dom(d: &Decls, tr: TraitId, p: usize, depth: usize) -> bool {
            let t = &d.traits[tr.0];
            (t.methods.values())
                .any(|m| (d.domain_params.get(m)).is_some_and(|ds| ds.contains(&(p + 1))))
                || (depth < 8
                    && (t.supers.iter()).any(|sup| {
                        (sup.args.iter().enumerate())
                            .any(|(q, a)| *a == Ty::Param(p) && trait_dom(d, sup.tr, q, depth + 1))
                    }))
        }
        let mut grew = false;
        for k in 0..d.fns.len() {
            let more: Vec<usize> = (d.fns[k].bounds.iter())
                .flat_map(|(_, r)| {
                    (r.args.iter().enumerate()).filter_map(move |(p, a)| match a {
                        Ty::Param(j) => Some((r.tr, p, *j)),
                        _ => None,
                    })
                })
                .filter(|&(tr, p, _)| trait_dom(d, tr, p, 0))
                .map(|(_, _, j)| j)
                .collect();
            let ds = d.domain_params.entry(FnId(k)).or_default();
            for j in more {
                if !ds.contains(&j) {
                    ds.push(j);
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    d.domain_params.retain(|_, ds| !ds.is_empty());
}

/// The name a top-level item declares, if it declares one.
pub fn item_name(s: &ast::Stmt) -> Option<&str> {
    match &s.kind {
        StmtKind::Context { name, .. } => Some(&name.text),
        StmtKind::Struct(sd) => Some(&sd.name.text),
        StmtKind::Trait { name, .. } => Some(&name.text),
        StmtKind::Fn(f) => Some(&f.name.text),
        StmtKind::TypeAlias { name, .. } => Some(&name.text),
        StmtKind::Const { name, .. } => Some(&name.text),
        _ => None,
    }
}

/// What each name means in module `mi`: the prelude's for the program, then the core's, then
/// what its `use` lines import, then its own (R11.2.3). `errors` is `None` while fns are not
/// declared yet, so a name not found is not yet known to be missing.
/// What every module declares and imports, from which each module's view is built.
struct Scopes<'a> {
    own: &'a Layer<HashMap<String, Named>>,
    public: &'a Layer<HashSet<String>>,
    names: &'a [String],
    imports: &'a [(ModId, &'a ast::Stmt)],
    prelude: &'a HashMap<String, String>,
    within: &'a [Option<ModId>],
}

fn view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    errors: Option<&mut Vec<Msg>>,
    ambiguous: &mut HashSet<String>,
) -> HashMap<String, Named> {
    let mut v = outer_view(sc, mi, program, errors, Some(ambiguous));
    v.extend(sc.own[mi].iter().map(|(n, x)| (n.clone(), *x)));
    v
}

/// What each name means in module `mi` apart from what it declares itself.
fn outer_view(
    sc: &Scopes,
    mi: usize,
    program: bool,
    mut errors: Option<&mut Vec<Msg>>,
    mut ambiguous: Option<&mut HashSet<String>>,
) -> HashMap<String, Named> {
    let Scopes {
        own,
        public,
        names,
        imports,
        prelude,
        within,
    } = *sc;
    let index = |m: &str| names.iter().position(|n| n == m);
    let export = |t: usize, n: &str| exported(sc, t, n);
    // The core holds the slice's own forms of some std names; a prelude name never hides one.
    let mut v = HashMap::new();
    if program {
        for (n, m) in prelude {
            if let Some(x) = index(m).and_then(|t| export(t, n)) {
                v.insert(n.clone(), x);
            }
        }
    }
    // Every other module uses what the core exports, as if by `use core.*`.
    if mi != CORE.0 {
        for n in exports(sc, CORE.0) {
            if let Some(x) = export(CORE.0, &n) {
                v.insert(n, x);
            }
        }
    }
    let mut imported: HashMap<String, Named> = HashMap::new();
    for (_, s) in imports.iter().filter(|(m, _)| m.0 == mi) {
        let StmtKind::Use {
            path,
            names: ns,
            glob,
            alias,
        } = &s.kind
        else {
            continue;
        };
        let module = path
            .iter()
            .map(|n| n.text.as_str())
            .collect::<Vec<_>>()
            .join(".");
        // `use std.rand.Rng` names one item of `std.rand` (R11.2.1).
        let (module, ns) = match (index(&module), path.split_last()) {
            (None, Some((last, init))) if ns.is_empty() && !*glob => (
                init.iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                std::slice::from_ref(last),
            ),
            _ => (module, &ns[..]),
        };
        // An unknown module is the loader's to report.
        let Some(t) = index(&module) else { continue };
        if *glob {
            for n in &public[t] {
                if let Some(x) = export(t, n) {
                    v.insert(n.clone(), x);
                }
            }
            continue;
        }
        // `use std.segtree` names the module: `segtree.SegTree` is its `pub SegTree` (R11.2.1).
        if ns.is_empty() {
            let m = alias
                .as_ref()
                .map_or(module.rsplit('.').next().unwrap_or(""), |a| a.text.as_str());
            v.insert(m.to_string(), Named::Module(ModId(t)));
            for n in exports(sc, t) {
                if let Some(x) = export(t, &n) {
                    v.insert(format!("{m}.{n}"), x);
                }
            }
            continue;
        }
        for n in ns {
            match export(t, &n.text) {
                Some(_) if own[mi].contains_key(&n.text) => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(
                            Unbuilt::UseOfOwn {
                                name: n.text.clone(),
                            }
                            .into(),
                        );
                    }
                }
                // Two imports of one name stay ambiguous where it is used (R08.1.4).
                Some(x) if imported.get(&n.text).is_some_and(|y| *y != x) => {
                    v.remove(&n.text);
                    if let Some(a) = ambiguous.as_deref_mut() {
                        a.insert(n.text.clone());
                    }
                }
                Some(x) => {
                    v.insert(n.text.clone(), x);
                    imported.insert(n.text.clone(), x);
                }
                None => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(
                            Unbuilt::UseMissing {
                                module: module.to_string(),
                                name: n.text.clone(),
                            }
                            .into(),
                        );
                    }
                }
            }
        }
    }
    // An inline module is a name of the module it sits in, and `m.T` its `pub T` (R11.1.3).
    for (t, w) in within.iter().enumerate() {
        if *w == Some(ModId(mi)) {
            let m = names[t].rsplit('.').next().unwrap_or("");
            v.insert(m.to_string(), Named::Module(ModId(t)));
            for n in exports(sc, t) {
                if let Some(x) = export(t, &n) {
                    v.insert(format!("{m}.{n}"), x);
                }
            }
        }
    }
    v
}

/// What each name module `t` declares or exports means, and whether another module may name it.
fn export_map(sc: &Scopes, t: usize) -> HashMap<String, (Named, bool)> {
    let mut m: HashMap<String, (Named, bool)> = (sc.own[t].iter())
        .map(|(n, x)| (n.clone(), (*x, sc.public[t].contains(n))))
        .collect();
    for n in exports(sc, t) {
        if let Some(x) = exported(sc, t, &n) {
            m.insert(n, (x, true));
        }
    }
    m
}

/// The names module `t` exports: its `pub` declarations and what its `pub use` lines name.
fn exports(sc: &Scopes, t: usize) -> Vec<String> {
    let mut out: Vec<String> = sc.public[t].iter().cloned().collect();
    for (_, s) in sc.imports.iter().filter(|(m, s)| m.0 == t && s.public) {
        if let StmtKind::Use { names, .. } = &s.kind {
            out.extend(names.iter().map(|n| n.text.clone()));
        }
    }
    out
}

/// What `n` means to a module that imports it from module `t`: a `pub` declaration of `t`'s,
/// or what a `pub use` line of `t`'s names it, followed to the module that declares it.
fn exported(sc: &Scopes, t: usize, n: &str) -> Option<Named> {
    let mut seen = HashSet::new();
    let mut t = t;
    loop {
        if sc.public[t].contains(n) {
            return sc.own[t].get(n).copied();
        }
        if !seen.insert(t) {
            return None;
        }
        t = sc
            .imports
            .iter()
            .filter(|(m, s)| m.0 == t && s.public)
            .find_map(|(_, s)| {
                let StmtKind::Use {
                    path, names, glob, ..
                } = &s.kind
                else {
                    return None;
                };
                let module = path
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                let named = *glob || names.iter().any(|x| x.text == n);
                named
                    .then(|| sc.names.iter().position(|m| *m == module))
                    .flatten()
            })?;
    }
}

/// The item a method sits in: its type parameters and set parameters come first.
struct Within<'a> {
    owner: Option<Owner>,
    generics: &'a [String],
    sets: usize,
    /// The type of `self` in what it sits in, in `generics`.
    self_ty: Option<Ty>,
    /// For an impl's fn, the trait's signature for it in the impl's terms: what an
    /// untyped parameter or a missing result means (R08.1.2).
    sig: Option<TraitSig>,
}
struct TraitSig {
    self_mode: SelfMode,
    params: Vec<Ty>,
    ret: Ty,
}
const FREE: Within = Within {
    owner: None,
    sig: None,
    generics: &[],
    sets: 0,
    self_ty: None,
};

/// A parameter typed by a trait (`b: Big<Int>`) has a type parameter of its own bounded by it, as
/// `<B: Big<Int>>` would be (R08.2.1); its name `b.impl` can't be written.
fn trait_typed(
    f: &ast::FnDecl,
    outer: &[String],
    names: &HashMap<String, Named>,
) -> Option<ast::FnDecl> {
    let mut out: Option<ast::FnDecl> = None;
    for (k, p) in f.params.iter().enumerate() {
        let Some(t) = &p.ty else { continue };
        let TypeKind::Named { name, .. } = &t.kind else {
            continue;
        };
        let shadowed = outer.contains(&name.text) || f.generics.iter().any(|g| g.text == name.text);
        if shadowed || !matches!(names.get(&name.text), Some(Named::Trait(_))) {
            continue;
        }
        let g = out.get_or_insert_with(|| f.clone());
        let n = ast::Name {
            text: format!("{}.impl", p.name.text),
            span: name.span,
        };
        g.generics.push(n.clone());
        g.bounds.push(ast::Bound {
            name: n.clone(),
            traits: vec![t.clone()],
        });
        g.params[k].ty = Some(ast::Type {
            kind: TypeKind::Named {
                name: n,
                args: Vec::new(),
            },
            span: t.span,
        });
    }
    out
}

fn add_fn(
    d: &mut Decls,
    m: ModId,
    item: ItemId,
    s: &ast::Stmt,
    f: &ast::FnDecl,
    within: Within,
) -> FnId {
    let Within {
        owner,
        generics: outer,
        sets: outer_sets,
        self_ty: outer_self,
        sig,
    } = within;
    let names = d.views[m.0].clone();
    let desugared = trait_typed(f, outer, &names);
    let f = desugared.as_ref().unwrap_or(f);
    let mut generics = outer.to_vec();
    generics.extend(f.generics.iter().map(|g| g.text.clone()));
    let rests = without_bounds(d, &names, f);
    let with_rests;
    let f = match rests.is_empty() {
        true => f,
        false => {
            let mut g = f.clone();
            g.bounds.extend(rests);
            with_rests = g;
            &with_rests
        }
    };
    let hidden = hidden_params(d, &names, &f.bounds, &mut generics);
    let doms = d.doms_in(m);
    let aliases = d.aliases.clone();
    let sets = d.struct_sets.clone();
    let defaults = d.struct_defaults.clone();
    let tuples = d.tuples.clone();
    let (sugar, records) = (d.sugar, d.records.clone());
    let members = d.member_types.clone();
    let mut cx = TyCx {
        names: &names,
        values: d.top_values.clone(),
        ops: d.operator_fns(),
        item: Some(item.clone()),
        struct_sets: &sets,
        struct_defaults: &defaults,
        tuples: &tuples,
        sugar,
        records: &records,
        members: &members,
        generics: &generics,
        outer: &generics,
        args: None,
        aliases: &aliases,
        expanding: Vec::new(),
        doms: &doms,
        ret_slots: None,
        next_set: Some(outer_sets),
    };
    let mut errors = Vec::new();
    let mut self_mode = SelfMode::None;
    let mut self_name = "self".to_string();
    let mut self_ty = None;
    let mut params = Vec::new();
    let sig_takes_self = sig.as_ref().is_some_and(|s| s.self_mode != SelfMode::None);
    for (k, p) in f.params.iter().enumerate() {
        if p.name.text == "self" || (k == 0 && sig_takes_self) {
            self_name = p.name.text.clone();
            self_mode = if p.mutable || sig.as_ref().is_some_and(|s| s.self_mode == SelfMode::Mut) {
                SelfMode::Mut
            } else {
                SelfMode::Value
            };
            // A free fn's `self` says what it extends (R08.1.4).
            self_ty = match (&outer_self, &p.ty) {
                (Some(t), _) => Some(t.clone()),
                (None, Some(t)) => Some(conv(&mut cx, t, &mut errors)),
                (None, None) => {
                    errors.push(Msg::at(
                        item.at(p.name.span),
                        Msg::verdict(
                            cpl_diag::codes::UNKNOWN_TYPE,
                            "`self` of a free fn needs a type",
                        ),
                    ));
                    None
                }
            };
            continue;
        }
        let from_trait = (sig.as_ref()).and_then(|s| s.params.get(params.len()).cloned());
        let ty = match (&p.ty, from_trait) {
            (Some(t), _) => conv(&mut cx, t, &mut errors),
            (None, Some(t)) => t,
            (None, None) => {
                errors.push(Msg::at(
                    item.at(p.name.span),
                    Msg::verdict(
                        cpl_diag::codes::UNKNOWN_TYPE,
                        format!("parameter `{}` needs a type", p.name.text),
                    ),
                ));
                Ty::Error
            }
        };
        if p.lazy {
            errors.push(
                Unbuilt::LazyParam {
                    name: p.name.text.clone(),
                }
                .into(),
            );
        }
        params.push(Param {
            name: p.name.text.clone(),
            label: p.label.as_ref().unwrap_or(&p.name).text.clone(),
            ty,
            mutable: p.mutable,
            named_only: p.named_only,
            variadic: p.variadic,
            default: p.default.clone(),
        });
    }
    let n_sets = cx.next_set.unwrap_or(0);
    // A fn type in a result is "whatever this fn returns": filled in after the body is checked.
    cx.next_set = None;
    cx.ret_slots = Some(0);
    // `-> (domain C, [C]T)`: each `domain C` is a domain of its own, which the result names.
    let mut ret_doms = doms.clone();
    let binders = fresh_binders(f.ret.as_ref());
    for (_, c) in &binders {
        let t = collect::fresh_dom(d);
        if let Ty::Dom(i) = t {
            d.fresh_doms.push((i, format!("{}.{c}", f.name.text)));
        }
        d.top_doms
            .insert(format!("${}", d.fresh_doms.len()), t.clone());
        ret_doms.insert(c.clone(), t);
    }
    cx.doms = &ret_doms;
    let mut region = None;
    let mut lends = None;
    let ret = match (&f.ret, &f.lends) {
        (_, Some(l)) if l.mutable => {
            region = Some((true, conv(&mut cx, &l.ty, &mut errors)));
            Ty::Unit
        }
        // A read lend is the fn's value, which borrows every source it names, or every parameter.
        (_, Some(l)) => {
            let receiver = (self_mode != SelfMode::None).then_some(self_name.as_str());
            let names: Vec<&str> = (receiver.into_iter())
                .chain(params.iter().map(|p| p.name.as_str()))
                .collect();
            let mut from = Vec::new();
            for s in &l.sources {
                match names.iter().position(|n| *n == s.text) {
                    Some(i) => from.push(i),
                    None => errors.push(Msg::at(
                        item.at(s.span),
                        Msg::verdict(
                            cpl_diag::codes::UNRESOLVED_NAME,
                            format!("`{}` is not a parameter", s.text),
                        ),
                    )),
                }
            }
            if l.sources.is_empty() {
                from = (0..names.len()).collect();
            }
            lends = Some(from);
            conv(&mut cx, &l.ty, &mut errors)
        }
        (Some(t), None) => match &t.kind {
            TypeKind::Yields(inner) => {
                region = Some((false, conv(&mut cx, inner, &mut errors)));
                Ty::Unit
            }
            _ => conv(&mut cx, t, &mut errors),
        },
        (None, None) => sig.as_ref().map_or(Ty::Unit, |s| s.ret.clone()),
    };
    let n_ret_sets = cx.ret_slots.unwrap_or(0);
    let mut bounds = Vec::new();
    for b in &f.bounds {
        let Some(i) = generics.iter().position(|g| *g == b.name.text) else {
            continue;
        };
        for (j, t) in b.traits.iter().enumerate() {
            if let Some(r) = trait_ref(&mut cx, t, &mut errors) {
                bound_ref(d, r, i, (b.name.text.clone(), j), &hidden, &mut bounds);
            }
        }
    }
    // A decorator changes meaning, so one the slice does not implement refuses the program; it is
    // never skipped. A library's `@lint` declares a rewrite for the linter, which the slice does not
    // run, so the program's own is refused: its expected result may be that lint (R06.4.9).
    for x in &s.decorators {
        if let Some(msg) = crate::synth::unspelled_memo(x) {
            errors.push(Msg::at(item.at(x.span), msg));
            continue;
        }
        let elsewhere = d.is_library(m) && read_elsewhere(&x.name.text);
        let marker = matches!(
            x.name.text.as_str(),
            "coerce"
                | "variant_check"
                | "pattern"
                | "str_literal"
                | "range_literal"
                | "interpolation"
                | "input"
        );
        if !elsewhere
            && !marker
            && x.name.text != GETTER
            && !matches!(
                x.name.text.as_str(),
                "multitest" | "same_keys" | "index" | "aka" | "was" | "from_op" | "fast"
            )
        {
            let at = item.at(x.span);
            errors.push(Msg::at(
                at,
                Unbuilt::Decorator {
                    name: x.name.text.clone(),
                },
            ));
        }
    }
    // A program's are said now; its refusals stay on the fn, which is then the error.
    let mut errors = if d.is_library(m) {
        errors
    } else {
        d.errors.extend(errors.iter().cloned());
        errors.retain(|e| e.refusal.is_some());
        errors
    };
    let has = |n: &str| s.decorators.iter().any(|x| x.name.text == n);
    // A getter is read like a field, so nothing but `self` may need an argument (R02.6.6).
    let getter = has(GETTER);
    let same_keys = has("same_keys");
    let words = |n: &str| -> Vec<String> {
        (s.decorators.iter().filter(|x| x.name.text == n))
            .flat_map(|x| &x.args)
            .filter_map(|a| match &a.value.kind {
                cpl_syntax::ast::ExprKind::Name(w) => Some(w.clone()),
                _ => None,
            })
            .collect()
    };
    if has("index") {
        d.index_fns.push(FnId(d.fns.len()));
    }
    if has("from_op") {
        d.from_op.push(FnId(d.fns.len()));
    }
    let coerce = has("coerce");
    if coerce {
        d.coercions.push(FnId(d.fns.len()));
    }
    if has("pattern") {
        d.pattern_fns.push(FnId(d.fns.len()));
    }
    let id = FnId(d.fns.len());
    for (n, slot) in [
        ("str_literal", &mut d.sugar.str_literal),
        ("range_literal", &mut d.sugar.range_literal),
    ] {
        if has(n) {
            collect::once(slot, id, collect::again(&item, s, n), &mut d.errors);
        }
    }
    if has("input") {
        d.input_fns.push(FnId(d.fns.len()));
    }
    if has("interpolation") {
        d.interpolate = Some(FnId(d.fns.len()));
    }
    if getter && params.iter().any(|p| p.default.is_none()) {
        let msg = format!(
            "getter `{}` is read like a field, so each parameter but `self` needs a default",
            f.name.text
        );
        if d.is_library(m) {
            errors.push(Msg::internal(msg));
        } else {
            let code = cpl_diag::codes::GETTER_NEEDS_DEFAULTS;
            d.misdeclared
                .push((item.at(Span::new(s.span.start, s.span.start)), code, msg));
        }
    }
    if !binders.is_empty() {
        let slots = binders.iter().map(|b| b.0).collect();
        d.fresh_slots.push((FnId(d.fns.len()), slots));
    }
    d.fns.push(FnDecl {
        made: (0, 0),
        name: f.name.text.clone(),
        owner,
        self_mode,
        self_name,
        self_ty,
        generics,
        bounds,
        params,
        labels: f
            .params
            .iter()
            .filter(|p| p.named_only)
            .map(|p| p.label.as_ref().unwrap_or(&p.name).text.clone())
            .collect(),
        ret,
        n_sets,
        n_ret_sets,
        region,
        lends,
        uses: f.uses.iter().map(|n| n.text.clone()).collect(),
        body: f.body.clone(),
        intrinsic: match &f.body {
            FnBody::Expr(e) => match &e.kind {
                cpl_syntax::ast::ExprKind::Intrinsic(n) => Some(n.clone()),
                _ => None,
            },
            _ => None,
        },
        fast: words("fast").into_iter().next(),
        getter,
        same_keys,
        aka: words("aka"),
        was: words("was"),
        span: item.at(s.span),
        item,
        module: m,
        errors,
    });
    FnId(d.fns.len() - 1)
}

/// The ends of a range type when both are literals, as an inclusive pair.
pub fn static_range(
    lo: &Option<Box<ast::Expr>>,
    hi: &Option<Box<ast::Expr>>,
    inclusive: bool,
) -> Option<(i64, i64)> {
    let lo = match lo {
        Some(e) => e.int_literal()?,
        None => i64::MIN + 1,
    };
    let hi = match hi {
        Some(e) if inclusive => e.int_literal()?,
        Some(e) => e.int_literal()?.checked_sub(1)?,
        None => i64::MAX,
    };
    Some((lo, hi))
}

/// The name a constant range's domain is kept under: none a program can write.
pub fn anon_key(lo: i64, hi: i64) -> String {
    format!("${lo}..={hi}")
}

/// `lo..hi` with both ends constant, as an array's axis: its first and last integer.
pub fn const_axis(t: &ast::Type) -> Option<(i64, i64)> {
    match &t.kind {
        TypeKind::Range {
            lo: lo @ Some(_),
            hi: hi @ Some(_),
            inclusive,
        } => static_range(lo, hi, *inclusive),
        _ => None,
    }
}

/// The parameters a bound that leaves its trait's arguments out takes for each, at `generics`'
/// end, by bound and trait. An associated type is named through the bounded parameter: `G.Arc` of
/// `G: Next<V>` (R08.2.3); any other is `S.1`, which can't be written, fixed when the bound is met
/// (R08.1.2). An associated type's default is the impl's to take, never the bound's.
type Hidden = Vec<((String, usize), std::ops::Range<usize>)>;
/// `R: (to:, ..without u, v)<X>` for each `(to: X, ..R without (u, v))` the signature writes,
/// through its aliases, of a parameter `R` whose row bound has those fields: the record it names
/// is then a parameter of the signature (R08.4.1).
fn without_bounds(d: &Decls, names: &HashMap<String, Named>, f: &ast::FnDecl) -> Vec<ast::Bound> {
    let mut out = Vec::new();
    let sig = f.params.iter().filter_map(|p| p.ty.as_ref()).chain(&f.ret);
    sig.for_each(|t| rest_bounds(d, names, f, t, &mut out));
    out
}

fn rest_bounds(
    d: &Decls,
    names: &HashMap<String, Named>,
    f: &ast::FnDecl,
    t: &ast::Type,
    out: &mut Vec<ast::Bound>,
) {
    let mut go = |t: &ast::Type| rest_bounds(d, names, f, t, out);
    match &t.kind {
        TypeKind::Named { name, args } => {
            args.iter().for_each(&mut go);
            let generic = f.generics.iter().any(|g| g.text == name.text);
            if let (Some(Named::Alias(id)), false) = (names.get(&name.text), generic) {
                let a = &d.aliases[id.0];
                if a.generics.len() == args.len() {
                    let sub = a.generics.iter().map(String::as_str).zip(args).collect();
                    rest_bounds(d, &a.names, f, &subst(&a.ty, &sub), out);
                }
            }
        }
        TypeKind::RowWithout {
            prefix,
            base,
            removed,
        } => {
            prefix.iter().for_each(|(_, t)| go(t));
            go(base);
            let TypeKind::Named { name: r, args } = &base.kind else {
                return;
            };
            let row = (f.bounds.iter())
                .filter(|b| b.name.text == r.text)
                .flat_map(|b| &b.traits)
                .find_map(|t| match &t.kind {
                    TypeKind::Row(fs) => Some(fs),
                    _ => None,
                });
            let fits = |row: &Vec<(ast::Name, ast::Type)>| {
                (removed.iter()).all(|n| row.iter().any(|(f, _)| f.text == n.text))
            };
            if !args.is_empty() || !row.is_some_and(fits) {
                return;
            }
            let form = (
                prefix.iter().map(|(n, _)| n.text.clone()).collect(),
                removed.iter().map(|n| n.text.clone()).collect(),
            );
            let tr = crate::records::without_name(&form);
            let known = |b: &ast::Bound| {
                b.name.text == r.text
                    && matches!(&b.traits[..], [ast::Type { kind: TypeKind::Named { name, .. }, .. }] if name.text == tr)
            };
            if out.iter().any(known) {
                return;
            }
            let kind = TypeKind::Named {
                name: ast::Name {
                    text: tr,
                    span: t.span,
                },
                args: prefix.iter().map(|(_, t)| t.clone()).collect(),
            };
            out.push(ast::Bound {
                name: r.clone(),
                traits: vec![ast::Type { kind, span: t.span }],
            });
        }
        TypeKind::Array { index, elem } => index.iter().chain([&**elem]).for_each(go),
        TypeKind::OwnDomain { elem: x, .. }
        | TypeKind::Optional(x)
        | TypeKind::Mut(x)
        | TypeKind::Yields(x) => go(x),
        TypeKind::Tuple(ts) => ts.iter().for_each(go),
        TypeKind::Record(fs) | TypeKind::Row(fs) => fs.iter().for_each(|(_, t)| go(t)),
        TypeKind::Fn { params, ret } => params.iter().chain([&**ret]).for_each(go),
        _ => {}
    }
}

/// `t` with each bare name `sub` has replaced by its type: an alias's body at its arguments.
fn subst(t: &ast::Type, sub: &HashMap<&str, &ast::Type>) -> ast::Type {
    let s = |x: &ast::Type| subst(x, sub);
    let b = |x: &ast::Type| Box::new(subst(x, sub));
    let fs = |v: &[(ast::Name, ast::Type)]| -> Vec<(ast::Name, ast::Type)> {
        v.iter().map(|(n, x)| (n.clone(), subst(x, sub))).collect()
    };
    let kind = match &t.kind {
        TypeKind::Named { name, args } if args.is_empty() => match sub.get(name.text.as_str()) {
            Some(x) => return (*x).clone(),
            None => t.kind.clone(),
        },
        TypeKind::Named { name, args } => TypeKind::Named {
            name: name.clone(),
            args: args.iter().map(s).collect(),
        },
        TypeKind::Array { index, elem } => TypeKind::Array {
            index: index.iter().map(s).collect(),
            elem: b(elem),
        },
        TypeKind::OwnDomain { name, elem } => TypeKind::OwnDomain {
            name: name.clone(),
            elem: b(elem),
        },
        TypeKind::Optional(x) => TypeKind::Optional(b(x)),
        TypeKind::Mut(x) => TypeKind::Mut(b(x)),
        TypeKind::Yields(x) => TypeKind::Yields(b(x)),
        TypeKind::Tuple(ts) => TypeKind::Tuple(ts.iter().map(s).collect()),
        TypeKind::Record(v) => TypeKind::Record(fs(v)),
        TypeKind::Row(v) => TypeKind::Row(fs(v)),
        TypeKind::Fn { params, ret } => TypeKind::Fn {
            params: params.iter().map(s).collect(),
            ret: b(ret),
        },
        TypeKind::RowWithout {
            prefix,
            base,
            removed,
        } => TypeKind::RowWithout {
            prefix: fs(prefix),
            base: b(base),
            removed: removed.clone(),
        },
        other => other.clone(),
    };
    ast::Type { kind, span: t.span }
}

fn hidden_params(
    d: &Decls,
    names: &HashMap<String, Named>,
    bounds: &[ast::Bound],
    generics: &mut Vec<String>,
) -> Hidden {
    let mut hidden = Vec::new();
    for b in bounds {
        for (j, t) in b.traits.iter().enumerate() {
            let TypeKind::Named { name, args } = &t.kind else {
                continue;
            };
            let Some(Named::Trait(tr)) = names.get(&name.text).copied() else {
                continue;
            };
            let n = d.traits[tr.0].defaults.len();
            let first_assoc = n - d.traits[tr.0].assoc.len();
            if args.len() < n
                && (args.len() >= first_assoc || d.traits[tr.0].defaults[args.len()].is_none())
            {
                let k = generics.len();
                let assoc = &d.traits[tr.0].assoc;
                generics.extend(
                    (args.len()..n).map(|p| match p.checked_sub(n - assoc.len()) {
                        Some(a) => format!("{}.{}", b.name.text, assoc[a]),
                        None => format!("{}.hidden", k + p - args.len()),
                    }),
                );
                hidden.push(((b.name.text.clone(), j), k..generics.len()));
            }
        }
    }
    hidden
}

/// Bound `r` of parameter `i`, the `key`th of its bound: its left-out arguments the parameters
/// [`hidden_params`] gave them, then its defaults, pushed to `out` with what its trait says of
/// its associated types, which holds of the parameters naming them.
fn bound_ref(
    d: &Decls,
    mut r: TraitRef,
    i: usize,
    key: (String, usize),
    hidden: &Hidden,
    out: &mut Vec<(usize, TraitRef)>,
) {
    if let Some((_, ks)) = hidden.iter().find(|(h, _)| *h == key) {
        r.args.extend(ks.clone().map(Ty::Param));
    }
    let r = d.with_defaults(r, &Ty::Param(i));
    let mut tys = vec![Ty::Param(i)];
    tys.extend(r.args.iter().cloned());
    let sub = Subst {
        tys: &tys,
        sets: &[],
    };
    for (a, ab) in &d.traits[r.tr.0].assoc_bounds {
        if let Ty::Param(ai) = tys[*a] {
            let ab = TraitRef {
                tr: ab.tr,
                args: ab.args.iter().map(|t| sub.ty(t)).collect(),
            };
            out.push((ai, ab));
        }
    }
    out.push((i, r));
}

/// The `domain C` slots of a result `(domain C, [C]T)` or `(domain C, comp: [V]C)`: each one's
/// place and name.
fn fresh_binders(ret: Option<&ast::Type>) -> Vec<(usize, String)> {
    match ret.map(|t| &t.kind) {
        Some(TypeKind::Record(fs)) => (fs.iter().enumerate())
            .filter(|(_, (_, t))| t.kind == TypeKind::NewDomain)
            .map(|(k, (n, _))| (k, n.text.clone()))
            .collect(),
        Some(TypeKind::Tuple(ts)) => (ts.iter().enumerate())
            .filter_map(|(k, t)| match &t.kind {
                TypeKind::DomainBinder(n) => Some((k, n.text.clone())),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
