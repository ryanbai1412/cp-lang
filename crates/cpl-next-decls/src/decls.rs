//! Contract 2: syntax -> declarations. Answers "what does this name mean"
//! without looking inside a body.
pub use crate::cycles::{alias_cycle, field_holding_itself};
use crate::diag::Msg;
use crate::sym::{Sym, Syms};
use crate::ty::*;
use cpl_diag::Span;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::unbuilt::Unbuilt;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

pub mod attr_rules;
mod collect;
mod domains;
pub(crate) mod forms;
mod impls;
mod inherent;
mod layer;
mod names;
mod structs;
mod trait_impl;
mod tyconv;
use crate::views::Shapes;
pub use collect::{body_of, collect, factor_name, prepare, Prepared};
use domains::{domain_params, fresh_results};
pub use forms::TyForm;
pub use impls::{
    domain_rows, hidden_member, impl_rank, method_for, self_fits, shaped_member, with_defaults,
    Choice, ImplIdx, Impls,
};
pub use layer::Layer;
use tyconv::*;
pub use tyconv::{
    alias_type, body_type, literal_int, prim_type, range_type, static_axis, struct_args, BodyScope,
};
pub(crate) use tyconv::{by_arity, fields_of_record, fill_defaults, record_of, unnamed};

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

/// The body a fn has. The body itself is its item's, read per item, so an edit to one body leaves
/// every signature as it was.
#[derive(Clone, Debug)]
pub enum Body {
    Written,
    /// An impl's `let count = e`: the value its getter gives (R04.2.26).
    Value,
    /// A made fn's body, built as IR by lower from its variant.
    Synth(crate::synth::SynthBody),
    /// A trait's member with no default.
    None,
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
    pub generics: Vec<Sym>,
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
    pub body: Body,
    /// What `= @intrinsic(id)` is, when that is its body.
    pub intrinsic: Option<cpl_next_ir::ops::Intrinsic>,
    /// `@fast(op)` on a fn with a CP body: the operation the engines and C++ run in place
    /// of a call to it; the reference interpreter runs the body.
    pub fast: Option<String>,
    pub getter: bool,
    /// Declared `pub`: a struct's method named outside its module needs it (R11.1.4).
    pub public: bool,
    /// `uses io, opt`: contexts its caller's values flow into (R09.1.3).
    pub uses: Vec<String>,
    /// `@same_keys`: its arguments that are arrays over domains share one domain (R04b.1.3).
    pub same_keys: bool,
    /// `@aka(..)`: names another language calls it; a call by one names this fn (R06.4.11).
    pub aka: Vec<String>,
    /// `@was(..)`: names it had before a release renamed it; a call by one is E0503.
    pub was: Vec<String>,
    /// `@on_destructure("..{n}..", fix: "..")`: what a tuple pattern of `n` that can't open
    /// its result or item is told, `{n}` the pattern's count (R02.5.5).
    pub on_destructure: Option<Unimplemented>,
    pub item: ItemId,
    /// Where it is written.
    pub span: SourceId,
    pub module: ModId,
    /// When the collector made it: the round, then the pass (traits, fns and types, impls,
    /// derived impls). Std's and a program's together are checked in this order.
    pub made: (u32, u8),
    /// Its place among the fns its item writes, in written order: how `body(ItemId)` finds its
    /// body.
    pub at: u32,
    /// What its signature uses that the slice does not build, said only when a call reaches it.
    pub errors: Vec<Msg>,
}

impl FnDecl {
    /// Whether it is declared without a body, as a trait's required method is.
    pub fn bodiless(&self) -> bool {
        matches!(self.body, Body::None)
    }
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
    pub default: Option<cpl_next_syntax::ast::Expr>,
    /// `@reject(..)`: a call that takes the default where it holds is an error (R06.4.17).
    pub reject: Option<Reject>,
    /// A jagged parameter's written range, whose ends name earlier parameters (`k: 0..=n`); `ty`
    /// is the range those ends reach. A call checks the argument into it (R06.4.1).
    pub written: Option<cpl_next_syntax::ast::Type>,
}
#[derive(Clone)]
pub struct StructDecl {
    /// Its `law`s, in declaration order: fns giving `Bool`, never members (R08.3.1).
    pub laws: Vec<FnId>,
    pub name: String,
    /// A record's struct, `(lo:, hi:)` (R02.5.2), shown and matched by its fields.
    pub record: bool,
    pub generics: Vec<String>,
    pub fields: Vec<(Sym, Ty)>,
    /// Per field: declared with `var`, so assignable after construction.
    pub var_fields: Vec<bool>,
    /// Per field: declared `pub`, so it may be named outside `module` (R11.1.4).
    pub pub_fields: Vec<bool>,
    /// Per field: `@cache`, so a read-only method may write it (R05.10.10).
    pub cached: Vec<bool>,
    /// Per field: where its type is written, as an offset into the source.
    pub field_at: Vec<SourceId>,
    /// Per field: the value a constructor that leaves it out stores, read in `module` (R02.6.1).
    pub defaults: Vec<Option<cpl_next_syntax::ast::Expr>>,
    pub module: ModId,
    /// Where its declaration starts in the source: the spans of its defaults count from here.
    pub item: ItemId,
    pub n_sets: usize,
    pub methods: HashMap<Sym, FnId>,
    /// Its `const NAME = init`s, read as `Dir4.up`: each a const of its module, set once.
    pub consts: HashMap<Sym, ConstId>,
    /// The fns of its impls over a shape of it (`impl Rev<[D]T>:`), each `self` that shape.
    pub shaped: HashMap<Sym, Vec<FnId>>,
    /// Its static `new`, which `T(args)` calls outside its own body (R02.6.3).
    pub ctor: Option<FnId>,
    /// An enum's variants, each with the struct of its payload, named `Enum.Variant` (R02.7).
    pub variants: Vec<(Sym, StructId)>,
    /// What its fields use that the slice does not build, said only when the program reaches it.
    pub errors: Vec<Msg>,
    /// An enum's `Int` of each variant: its written value (`Neg = -1`), else its place from 0.
    pub values: Vec<i64>,
    /// The traits its parameters are bounded by (`struct Range<D: Domain>`).
    pub bounds: Vec<(usize, TraitId)>,
}
#[derive(Clone)]
pub struct TraitDecl {
    /// Its `law`s, in declaration order: fns giving `Bool`, never members (R08.3.1).
    pub laws: Vec<FnId>,
    pub name: String,
    pub module: ModId,
    /// `trait Both = Sh + Nm`: met by meeting its supertraits, never implemented (R08.2.6).
    pub alias: bool,
    /// Its supertraits, in its own type parameters (R08.1.1).
    pub supers: Vec<TraitRef>,
    pub methods: HashMap<Sym, FnId>,
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
    /// `@aka(pow)` on an `@operator` trait: other languages' names for its operator, which a
    /// call by that name is told to write as the operator (R12.13.2).
    pub aka: Vec<String>,
}
/// `@on_unimplemented(code: E0311, "...{T}...")` on a trait: the code a missing impl reports in
/// place of E0606, and the help it adds, `{T}` the type shown (R08.1.8).
#[derive(Clone, Default)]
pub struct Unimplemented {
    pub code: Option<cpl_diag::Code>,
    pub help: Option<String>,
    /// `@on_destructure`'s `fix:`: the text written after the call (R02.5.5).
    pub fix: Option<String>,
}

/// `@reject(when: T = Bool, "..", code: E0303)` on a defaulted parameter: a call that takes the
/// default where each `when` type parameter is its type is that error, said there (R06.4.17).
#[derive(Clone, Debug)]
pub struct Reject {
    /// Each condition: the fn's type parameter, by index, and the type it is.
    pub when: Vec<(usize, Ty)>,
    pub code: cpl_diag::Code,
    pub text: String,
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
/// A value the program's top level binds under one name, `let m = ..` or `var m = ..`.
#[derive(Clone, Debug)]
pub struct TopValue {
    pub name: Sym,
    pub item: ItemId,
    pub mutable: bool,
    /// The range its written type declares; all of `Int` when it writes none.
    pub within: (i64, i64),
}
/// The top-level value `name` is, the last one bound under it.
pub fn top_value(values: &[TopValue], name: Sym) -> Option<TopValueId> {
    values.iter().rposition(|v| v.name == name).map(TopValueId)
}

/// A library's `const NAME[: T] = init`.
#[derive(Clone, Debug)]
pub struct LibConst {
    pub module: ModId,
    pub name: String,
    pub item: ItemId,
    pub ty: Option<cpl_next_syntax::ast::Type>,
    pub init: cpl_next_syntax::ast::Expr,
}

/// A type core gives a name without a declaration: a primitive, or `Raw<T>`, storage with no CP
/// body (R02.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreType {
    Prim(cpl_next_ir::prim::Prim),
    Raw,
}

/// `context opt:` with its fields and their defaults (R09.1): values a `with` line sets for the
/// rest of a block, read as `opt.k`.
#[derive(Clone, Debug)]
pub struct ContextDecl {
    pub module: ModId,
    pub name: String,
    pub fields: Vec<forms::FieldForm>,
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
    pub generics: Vec<Sym>,
    pub ty: cpl_next_syntax::ast::Type,
    pub names: View,
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
    /// The next domain `item` declares: the same id each time `item` is collected.
    pub(crate) fn new_dom(&mut self, item: &ItemId) -> Ty {
        let nth = self.dom_made.entry(item.clone()).or_default();
        let key = (item.clone(), *nth);
        *nth += 1;
        let next = self.dom_ids.len();
        Ty::Dom(*self.dom_ids.entry(key).or_insert(next))
    }
}

impl Decls {
    /// Which impl a type takes when several fit: one for its written type (0) before one for
    /// every record with some fields (1), before any other blanket one (2) (R08.1.2, R08.4.5).
    pub fn impl_rank(&self, im: &ImplDecl) -> u8 {
        impls::impl_rank(self, im)
    }
    /// `impl_rank` of an impl for `for_` with `bounds`.
    pub fn rank_of(&self, for_: &Ty, bounds: &[(usize, TraitRef)]) -> u8 {
        impls::rank_of(self, for_, bounds)
    }
}
#[derive(Clone)]
pub struct ImplDecl {
    pub trait_: TraitRef,
    pub generics: Vec<String>,
    pub bounds: Vec<(usize, TraitRef)>,
    pub for_: Ty,
    pub methods: HashMap<Sym, FnId>,
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

/// The declarations std marks as what a literal or a type's syntax is: `[a, b]` is a `@role(list)`,
/// `"s"` is a `@role(str)` made by the `@role(str_literal)` fn, `?T` is the `@optional` struct, `[D]T` the
/// `@role(array)` one, and `lo..hi` the `@role(range_literal)` fn's result.
#[derive(Default, Clone, Copy)]
pub struct Sugar {
    pub list: Option<StructId>,
    pub str: Option<StructId>,
    pub map: Option<StructId>,
    pub array: Option<StructId>,
    pub str_literal: Option<FnId>,
    pub range_literal: Option<FnId>,
}

/// The declarations filling the language roles, each found by its `@role(name)` (R11.6.1).
#[derive(Default, Clone)]
pub struct Roles {
    /// `?T`, `Some` and `None`: the struct `Optional<T>`.
    pub option: Option<StructId>,
    /// `print(...)`: shows its values and ends the line, reading `io`'s settings.
    pub print: Option<FnId>,
    /// `write(...)`: `print` without the line end.
    pub write: Option<FnId>,
    /// What an event shows of a value (contract 8): each fn writes the text for the values its
    /// parameter takes, and link picks the one that takes a value's type.
    pub trace: Vec<FnId>,
    /// `for x in s`: the trait whose one yielding method walks `s`.
    pub iteration: Option<TraitId>,
    /// The `@role(recipe)` struct: the value of a `-> yields T` call made outside a `for`
    /// (R07.6.2). Its one field is the closure that walks the call.
    pub recipe: Option<StructId>,
    /// The struct core marks `@role(cuts)`: a half-open literal range of its values, where a
    /// `Range<D>` is wanted, is the piece of positions between them (R04.4.4).
    pub cut: Option<StructId>,
    /// The `@role(positions)` trait: every domain's position type has it, and nothing else.
    pub domain: Option<TraitId>,
    /// The `@role(order)` trait: stepping and order, which every domain the compiler declares
    /// has and a library domain lacks (R04.2.31).
    pub order: Option<TraitId>,
    /// The `@role(convert)` trait: `T(x)` for a core type `T` is `x`'s impl of it at `T`.
    pub convert: Option<TraitId>,
    /// The `@role(counted)` trait: the one whose one fn counts `domain V = xs`'s positions (R04.2.29).
    pub counted: Option<TraitId>,
    /// The `@role(range_bounds)` trait: the one whose fns give a range's first and past-the-end cuts (R04b.1.5).
    pub bounds: Option<TraitId>,
    /// The `@role(endpoint)` trait: its associated type is the struct `lo..hi` builds from two
    /// ends of the implementing type, `Range<D>` for a domain and `RealRange` for `F64`.
    pub endpoint: Option<TraitId>,
    /// Core's `@role(interpolation)` fn: `put(mut out, x)` appends each piece of an interpolated string.
    pub interpolate: Option<FnId>,
    /// The `@role(pack)` struct: the head-and-tail pair a `@role(derive_fields)` trait's derived
    /// impl packs a struct's fields into (R08.1.6).
    pub pack: Option<StructId>,
    /// The `@role(derive_fields)` trait: every struct without its own impl gets it through its
    /// fields packed as `pack` pairs.
    pub derive_fields: Option<TraitId>,
    /// The `@role(variant_check)` fn: the check an `Int` converted to an enum goes through.
    pub variant_check: Option<FnId>,
    /// The `@role(todo)` fn: the expression `todo`, of any type, is a trap through it (R01.6.7).
    pub todo: Option<FnId>,
    /// The `@role(memo_table)` trait: a `@memo_table(word)` struct's impl of it is the table a
    /// `@memo(word)` fn reads with `get` and keeps results in with `set` (R06.4.1).
    pub memo_table: Option<TraitId>,
    /// The `@role(lawful)` trait: each struct with `law`s and each impl of a trait with them
    /// gets an impl of it, whose `laws` lists them (R08.3.5).
    pub lawful: Option<TraitId>,
}

/// A language role of R11.6.1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Option,
    Show,
    Print,
    Write,
    Iteration,
    Todo,
    Prelude,
    Recipe,
    Trace,
    Positions,
    Order,
    Convert,
    Counted,
    RangeBounds,
    Endpoint,
    Tuple,
    List,
    Str,
    Map,
    Array,
    Cuts,
    StrLiteral,
    RangeLiteral,
    Interpolation,
    Pack,
    DeriveFields,
    VariantCheck,
    MemoTable,
    Lawful,
}

impl Role {
    pub const ALL: [Role; 29] = [
        Role::Option,
        Role::Show,
        Role::Print,
        Role::Write,
        Role::Iteration,
        Role::Todo,
        Role::Prelude,
        Role::Recipe,
        Role::Trace,
        Role::Positions,
        Role::Order,
        Role::Convert,
        Role::Counted,
        Role::RangeBounds,
        Role::Endpoint,
        Role::Tuple,
        Role::List,
        Role::Str,
        Role::Map,
        Role::Array,
        Role::Cuts,
        Role::StrLiteral,
        Role::RangeLiteral,
        Role::Interpolation,
        Role::Pack,
        Role::DeriveFields,
        Role::VariantCheck,
        Role::MemoTable,
        Role::Lawful,
    ];
    /// The name `@role(name)` gives it.
    pub fn name(self) -> &'static str {
        match self {
            Role::Option => "option",
            Role::Show => "show",
            Role::Print => "print",
            Role::Write => "write",
            Role::Iteration => "iteration",
            Role::Todo => "todo",
            Role::Prelude => "prelude",
            Role::Recipe => "recipe",
            Role::Trace => "trace",
            Role::Positions => "positions",
            Role::Order => "order",
            Role::Convert => "convert",
            Role::Counted => "counted",
            Role::RangeBounds => "range_bounds",
            Role::Endpoint => "endpoint",
            Role::Tuple => "tuple",
            Role::List => "list",
            Role::Str => "str",
            Role::Map => "map",
            Role::Array => "array",
            Role::Cuts => "cuts",
            Role::StrLiteral => "str_literal",
            Role::RangeLiteral => "range_literal",
            Role::Interpolation => "interpolation",
            Role::Pack => "pack",
            Role::DeriveFields => "derive_fields",
            Role::VariantCheck => "variant_check",
            // The role is named for the decorator that marks its tables.
            Role::MemoTable => crate::attrs::Decorator::MemoTable.name(),
            Role::Lawful => "lawful",
        }
    }
}

#[derive(Default, Clone)]
pub struct Decls {
    pub fns: Layer<FnDecl>,
    /// The fns each item writes, so a per-item query finds them: a struct's item holds its
    /// methods, and `FnDecl::item` is the way back.
    pub item_fns: HashMap<ItemId, Vec<FnId>>,
    pub structs: Layer<StructDecl>,
    pub traits: Layer<TraitDecl>,
    pub impls: Layer<ImplDecl>,
    /// Per module, what each name means there: the core's names, then the prelude's (the
    /// program only, R11.3.1), then its own and what its `use` lines import (R11.2.3).
    pub views: Layer<Arc<View>>,
    /// Per module, the names two of its `use` lines import as different items: a use of one
    /// is ambiguous (R08.1.4).
    pub ambiguous: Vec<HashSet<Sym>>,
    pub module_names: Vec<String>,
    /// Per module, whether std's own layer declares it: the modules a `use std.` line names.
    pub of_std: Vec<bool>,
    /// Per module, the module an inline `mod` of it sits in (R11.1.3).
    pub within: Vec<Option<ModId>>,
    /// Per module, what each name it declares or exports means, and whether it is
    /// `pub`: `m.f` outside it.
    pub exports: Vec<Arc<HashMap<Sym, (Named, bool)>>>,
    pub program: ModId,
    /// The program's top-level items in order: its statements, constants and `@memo(word)` fns.
    pub statements: Vec<ItemId>,
    /// Per library module, what it uses outside its fns that the slice does not build.
    pub module_errors: Vec<Vec<Msg>>,
    /// Why each `Named::Refused` library item is not built.
    pub refused: Vec<Unbuilt>,
    pub consts: Layer<LibConst>,
    pub contexts: Layer<ContextDecl>,
    pub struct_sets: Arc<HashMap<StructId, usize>>,
    /// `struct SegTree<D, T, F = Int>`: each struct's default type arguments, in its parameters.
    pub struct_defaults: Arc<HashMap<StructId, Vec<Option<Ty>>>>,
    /// A module's structs of one name, each with its count of type parameters, under the first
    /// declared, which the name means (R02.11.4).
    pub struct_arity: Arc<HashMap<StructId, Vec<(StructId, usize)>>>,
    /// Core's `@role(tuple)` structs by arity: `(A, B)` is the one with two fields.
    pub tuples: Arc<HashMap<usize, StructId>>,
    /// Core's `@member(m)` structs by `m`: `D.m` is the struct at `D` (R06).
    pub member_types: Arc<HashMap<Sym, StructId>>,
    /// What literals and type syntax build, each found by its std marker.
    pub sugar: Sugar,
    /// The language roles, each found by its `@role` (R11.6.1).
    pub roles: Roles,
    /// Each record struct by its field names, in order: `(lo: A, hi: B)` is the one keyed
    /// `["lo", "hi"]` (R02.5.2).
    pub records: Arc<HashMap<Vec<String>, StructId>>,
    pub aliases: Layer<AliasDecl>,
    pub errors: Vec<Msg>,
    /// Declarations the program gets wrong: where, the code, what is wrong.
    pub misdeclared: Vec<(SourceId, cpl_diag::Code, String)>,
    /// The fns whose body decls rejected, as an intrinsic not in the table: its error is said,
    /// and check reads no body for them.
    pub rejected_bodies: std::collections::BTreeSet<FnId>,
    /// The fns refused as declared (a decorator the slice lacks): the refusal is the error, so
    /// check reads neither their body nor their calls.
    pub refused_fns: std::collections::BTreeSet<FnId>,
    /// Each module's public free fns with a `self`, by name: a method call finds them with no
    /// import (R11.3.4).
    pub methods: HashMap<Sym, Vec<FnId>>,
    /// The free fns of one name in one module past its first, keyed by the first: each has its
    /// own required labels, and a call takes the one whose labels it gives (R06.1.6).
    pub overloads: HashMap<FnId, Vec<FnId>>,
    /// The program's top-level domains, which its declarations may name: each is the
    /// `Ty::Dom` of its place among them, filled when its statement runs (R02.9).
    pub top_doms: HashMap<Sym, Ty>,
    /// The `domain C` a fn's result binds, `-> (domain C, [C]T)`: each is a domain of its own
    /// among `top_doms`, with the name the trace shows (R02.9.3).
    pub fresh_doms: Vec<(usize, String)>,
    /// Each domain decls makes, keyed by the item that declares it and which of that item's
    /// domains it is: its place here is its `Ty::Dom`, so a domain's id never reads the others.
    pub dom_ids: HashMap<(ItemId, u32), usize>,
    /// How many of `dom_ids` have their `Domain` and `Ordered` rows in `impls`.
    pub dom_rows_made: usize,
    /// The structs and impls `add_law_rows` has read.
    pub law_rows_made: (usize, usize),
    /// How many domains each item has made.
    dom_made: HashMap<ItemId, u32>,
    /// The fns whose result's slots are `domain C`s, till those slots are made `Range<C>`s.
    pub fresh_slots: Vec<(FnId, Vec<usize>)>,
    /// The fns whose result leaves a struct's parameter as `_`, `-> Rooted<V, _, _>` (R02.9.3).
    pub ret_holes: Vec<FnId>,
    /// What the program's top level binds under one name, in order: `TopValueId` indexes it.
    pub top_values: Arc<Vec<TopValue>>,
    /// Core's `@operator("+")` traits by the operator they give: `a + b` on a type that is not
    /// a number is its impl's fn (R08.1.5).
    pub operators: Vec<(Sym, TraitId)>,
    /// `@literal("[]")` and `@literal("[;]")`: the trait a list literal and an array fill build
    /// through (R01.8.8).
    pub literals: Vec<(Sym, TraitId)>,
    /// Per fn, the type parameters that key an array it names: domain parameters (R04.2).
    pub domain_params: HashMap<FnId, Vec<usize>>,
    /// The domain parameters whose body steps on them, so they meet `Ordered` (R04.2.31).
    pub ordered_params: HashMap<FnId, Vec<usize>>,
    /// Core's `@coerce` fns: each converts a value of its parameter's type where its result's
    /// type is wanted (R04.8.10).
    pub coercions: Vec<FnId>,
    /// Each `@memo_table(word)` struct's module and name there, by word: the table `@memo(word)`
    /// keeps (R06.4.1).
    pub memo_homes: HashMap<Sym, (usize, Sym)>,
    /// Each role trait's members, by the grammar's role-member table (R11.6.1).
    pub role_members: HashMap<cpl_next_syntax::grammar::RoleMember, (Sym, FnId)>,
    /// Each `@memo` fn's `f.clear`, by the fn (R06.4.1).
    pub memo_clears: HashMap<FnId, FnId>,
    /// Static fns marked `@from_op`: what a bare operator becomes where their struct is wanted (R08.1.9).
    pub from_op: Vec<FnId>,
    /// The fns marked `@operator(op)`, by the operator they give as a value: only one with no
    /// `@operator` trait, `and` and `or` (R06.4.12).
    pub op_fns: Vec<(Sym, FnId)>,
    /// Core's `@pattern` fns: `f(p)` as a pattern matches the optional `f` makes when it is
    /// present and its value matches `p` (R01.7.1).
    pub pattern_fns: Vec<FnId>,
    /// Core's `@input` fns: a value they give is input, so a range slot it fills is the
    /// input's contract (R10.2.2).
    pub input_fns: Vec<FnId>,
    /// The `@index` fns, in declaration order.
    pub index_fns: Vec<FnId>,
    /// Each `@free_form` trait method, with its trait and name: `f(x, ..)` is `x.f(..)` in a
    /// module whose view has the trait (R01.10.1).
    pub free_forms: BTreeMap<FnId, (TraitId, Sym)>,
    /// Per module, whether the program loads it: std's others are declared but out of sight.
    pub seen: Vec<bool>,
    /// The records std writes and no module the program loads does: out of sight with their impls.
    pub unseen_records: HashSet<StructId>,
    /// Each trait's impls, and each member name's (its trait's or its own), in order.
    pub impls_by_trait: HashMap<TraitId, Vec<usize>>,
    pub impls_by_member: HashMap<Sym, Vec<usize>>,
    /// The traits each struct has an impl of headed by it, and those of every other impl:
    /// which traits `impls_on` asks at a type.
    pub impls_by_head: HashMap<StructId, Vec<TraitId>>,
    pub impls_unheaded: Vec<TraitId>,
    /// Every name a member of a struct, trait or impl has, as an id.
    pub syms: Syms,
    /// Each core type by the name it is written with.
    pub core_types: Arc<HashMap<Sym, CoreType>>,
    /// Each trait's live impls in the order a type takes them (`impls_of`), made once a
    /// program's impls are all known: a lookup walks them thousands of times.
    pub impls_ranked: Option<HashMap<TraitId, Vec<usize>>>,
}

/// Where a method call resolved to.
pub enum Member {
    Field(usize, Ty),
    /// A method, with the receiver's type and set arguments.
    Method(FnId, Vec<Ty>, Vec<SetTy>),
}

impl Decls {
    /// `Tagged<Str>` written in a body: its arguments, the defaults filling those left off.
    pub fn with_defaults_of(&self, d: StructId, given: Vec<Ty>) -> Vec<Ty> {
        Shapes::with_defaults_of(self, d, &given)
    }
    /// The domains a declaration in module `m` may name.
    fn doms_in(&self, m: ModId) -> HashMap<Sym, Ty> {
        match m == self.program {
            true => self.top_doms.clone(),
            false => HashMap::new(),
        }
    }
    /// What a declaration uses that the slice does not build, kept with its module: said when
    /// the program reaches the module, at once for the program's own.
    fn module_or_program_errors(&mut self, m: ModId, errors: Vec<Msg>) {
        self.module_errors[m.0].extend(errors);
    }
    /// The fn a bare operator names: its core `@operator` trait's fn of that many operands.
    pub fn operator_fn(&self, text: &str, binary: bool) -> Option<FnId> {
        self.operator_fn_of(self.sym(text)?, binary)
    }
    pub(crate) fn operator_fn_of(&self, op: Sym, binary: bool) -> Option<FnId> {
        if let Some(&(_, f)) = self.op_fns.iter().find(|(o, _)| *o == op) {
            return (self.f(f).params.len() == usize::from(binary) + 1).then_some(f);
        }
        self.operator_traits_of(op).find_map(|tr| {
            let (_, m) = self.operator_method_of(tr)?;
            (self.f(m).params.len() == usize::from(binary)).then_some(m)
        })
    }
    /// The trait `@literal(op)` marks: `"[]"` a list literal builds through, `"[;]"` an array fill.
    pub fn literal_trait(&self, op: &str) -> Option<TraitId> {
        self.literal_trait_of(self.sym(op)?)
    }
    pub(crate) fn literal_trait_of(&self, op: Sym) -> Option<TraitId> {
        (self.literals.iter()).find_map(|(o, tr)| (*o == op).then_some(*tr))
    }
    /// The `@operator(text)` traits, in the order core declares them.
    pub fn operator_traits(&self, text: &str) -> impl Iterator<Item = TraitId> + '_ {
        (self.sym(text).into_iter()).flat_map(|op| self.operator_traits_of(op))
    }
    pub(crate) fn operator_traits_of(&self, op: Sym) -> impl Iterator<Item = TraitId> + '_ {
        (self.operators.iter())
            .filter(move |(o, _)| *o == op)
            .map(|&(_, tr)| tr)
    }
    /// The `Domain` getter `m` a range type's end `D.m` names.
    pub fn domain_getter(&self, m: &str) -> Option<FnId> {
        self.domain_getters().get(&self.sym(m)?).copied()
    }
    /// A trait's fns in the order it declares them, the role each plays in a literal (R01.8.8).
    pub fn trait_fns(&self, tr: TraitId) -> Vec<(Sym, FnId)> {
        let mut fns: Vec<_> = (self.traits[tr.0].methods.iter())
            .map(|(&n, &m)| (n, m))
            .collect();
        fns.sort_by_key(|&(_, m)| m);
        fns
    }
    /// The fn an `@operator` trait's operator calls: its first declared, `lt` of `Ord`'s
    /// `lt` and `cmp` (R06.4.12).
    pub fn operator_method(&self, tr: TraitId) -> Option<(&str, FnId)> {
        (self.operator_method_of(tr)).map(|(n, m)| (self.text(n), m))
    }
    pub(crate) fn operator_method_of(&self, tr: TraitId) -> Option<(Sym, FnId)> {
        (self.traits[tr.0].methods.iter())
            .min_by_key(|&(_, &m)| m)
            .map(|(&n, &m)| (n, m))
    }
    /// Each binary operator's fn, for `type_of(+)`.
    /// `Domain`'s getters by name, which a range type's end `D.m` names.
    pub fn domain_getters(&self) -> Arc<HashMap<Sym, FnId>> {
        let Some(tr) = self.roles.domain else {
            return Arc::default();
        };
        let ms = self.traits[tr.0].methods.iter();
        Arc::new(
            // A range type ends at a count: a static getter giving an `Int`.
            ms.filter(|(_, f)| {
                let f = self.f(**f);
                f.getter && f.self_ty.is_none() && f.ret == Ty::Int
            })
            .map(|(n, f)| (*n, *f))
            .collect(),
        )
    }
    pub(crate) fn operator_fns(&self) -> Arc<Vec<(Sym, FnId)>> {
        Arc::new(
            (self.operators.iter())
                .filter_map(|(o, _)| Some((*o, self.operator_fn(self.text(*o), true)?)))
                .chain(self.op_fns.iter().copied())
                .collect(),
        )
    }
    /// The iteration role's walk (R11.6.1): the yielding method of the trait filling it that
    /// every impl writes, the one without a default body.
    pub fn iteration_walk(&self) -> Option<(&str, FnId)> {
        (self.walk_of(self.roles.iteration?, false)).map(|(n, f)| (self.text(n), f))
    }
    /// Trait `tr`'s yielding method every impl writes, or with `own` the one with a default
    /// body that an impl may write its own of.
    pub(crate) fn walk_of(&self, tr: TraitId, own: bool) -> Option<(Sym, FnId)> {
        (self.traits[tr.0].methods.iter())
            .find(|(_, f)| self.f(**f).region.is_some() && self.f(**f).bodiless() != own)
            .map(|(&n, &f)| (n, f))
    }

    /// The iteration role's yielding method with a default body (`each`), which a type may
    /// write its own of; a type without one walks through `iteration_walk`.
    pub fn iteration_own_walk(&self) -> Option<(&str, FnId)> {
        (self.walk_of(self.roles.iteration?, true)).map(|(n, f)| (self.text(n), f))
    }
    /// The id of a name some declaration gave, or `None` when none did.
    pub fn sym(&self, text: &str) -> Option<Sym> {
        self.syms.get(text)
    }
    pub fn text(&self, s: Sym) -> &str {
        self.syms.text(s)
    }
    /// The member `name` of a struct's, trait's or impl's members.
    /// Whether some module declares a `@memo_table(word)` for `word`.
    pub fn memo_word(&self, word: &str) -> bool {
        self.sym(word)
            .is_some_and(|w| self.memo_homes.contains_key(&w))
    }
    /// Which of struct `s`'s fields is named `name`.
    pub fn field(&self, s: StructId, name: &str) -> Option<usize> {
        let name = self.sym(name)?;
        (self.structs[s.0].fields.iter()).position(|(f, _)| *f == name)
    }
    /// The names of struct `s`'s fields, in order.
    pub fn field_names(&self, s: StructId) -> Vec<String> {
        let fields = Shapes::field_names(self, s).into_iter();
        fields.map(|f| self.text(f).to_string()).collect()
    }
    /// Which of enum `e`'s variants is named `name`.
    pub fn variant(&self, e: StructId, name: &str) -> Option<usize> {
        Shapes::variant(self, e, self.sym(name)?)
    }
    pub fn member(&self, members: &HashMap<Sym, FnId>, name: &str) -> Option<FnId> {
        members.get(&self.sym(name)?).copied()
    }
    /// The fns item `item` writes, in the order they were declared.
    pub fn fns_of(&self, item: &ItemId) -> &[FnId] {
        self.item_fns.get(item).map_or(&[], Vec::as_slice)
    }
    pub fn f(&self, d: FnId) -> &FnDecl {
        &self.fns[d.0]
    }
    /// Its required labels: those of the named-only parameters with no default, sorted.
    pub fn required_labels(&self, d: FnId) -> Vec<&str> {
        required_labels(self, d)
    }
    pub fn overload(&self, first: FnId, given: &[&str]) -> Option<FnId> {
        overload(self, self, first, given)
    }
    pub fn s(&self, d: StructId) -> &StructDecl {
        &self.structs[d.0]
    }
    /// E0207 and its fix, when member `m` (named `name`) of a struct is private to a module
    /// other than `at` (R11.1.4).
    pub fn hidden_member(
        &self,
        m: &Member,
        rt: &Ty,
        name: &str,
        at: ModId,
    ) -> Option<(Msg, String)> {
        hidden_member(self, self, m, rt, name, at)
    }
    pub fn named(&self, m: ModId, name: &str) -> Option<Named> {
        look(&self.syms, &self.views[m.0], name)
    }
    /// What module `m` exports as `name`, and whether it is `pub`.
    pub fn export(&self, m: ModId, name: &str) -> Option<(Named, bool)> {
        self.exports[m.0].get(&self.sym(name)?).copied()
    }
    /// Whether two of module `m`'s `use` lines import `name` as different items (R08.1.4).
    pub fn is_ambiguous(&self, m: ModId, name: &str) -> bool {
        self.sym(name)
            .is_some_and(|s| self.ambiguous[m.0].contains(&s))
    }
    /// The trait and method a plain call `name(x, ..)` in module `m` is, when `name` is a
    /// `@free_form` method of a trait in `m`'s view: the call is `x.name(..)` (R01.10.1).
    pub fn free_form(&self, m: ModId, name: Sym) -> Option<(TraitId, Sym)> {
        match self.views[m.0].get(&name)? {
            Named::Fn(f) => self.free_forms.get(f).copied(),
            _ => None,
        }
    }
    pub fn fn_named(&self, m: ModId, name: &str) -> Option<FnId> {
        match self.named(m, name) {
            Some(Named::Fn(f)) => Some(f),
            _ => None,
        }
    }
    pub fn struct_named(&self, m: ModId, name: &str) -> Option<StructId> {
        match self.named(m, name) {
            Some(Named::Struct(s)) => Some(s),
            _ => None,
        }
    }
    /// A read-only method that may still write its receiver's `@cache` fields, so a call
    /// lends the receiver for writing (R05.10.10).
    pub fn writes_cache(&self, f: FnId) -> bool {
        let fd = self.f(f);
        fd.self_mode == SelfMode::Value
            && matches!(fd.owner, Some(Owner::Struct(s)) if self.s(s).cached.contains(&true))
    }
    /// Whether a body in `m` that does not check is the pipeline's to fix rather than the
    /// program's: until every std fn checks (`next_std_unchecked` 0), a library's is a refusal.
    pub fn is_library(&self, m: ModId) -> bool {
        !self.of_program(m)
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
    /// Whether an impl of `tr` takes a domain for its argument `j` (`impl<V: Domain, ..>
    /// Next<V> for [V]S`), so a type a bound of `tr` has there may be one.
    pub fn arg_may_be_domain(&self, tr: TraitId, j: usize) -> bool {
        let Some(dom) = self.roles.domain else {
            return false;
        };
        self.trait_impls(tr).any(|(_, im)| {
            matches!(im.trait_.args.get(j), Some(Ty::Param(i))
                if im.bounds.iter().any(|(b, r)| b == i && r.tr == dom))
        })
    }
    pub(crate) fn push_impl(&mut self, im: ImplDecl) {
        self.impls_ranked = None;
        let k = self.impls.len();
        self.impls_by_trait.entry(im.trait_.tr).or_default().push(k);
        match im.for_ {
            Ty::Struct(s, ..) => self.impls_by_head.entry(s).or_default().push(im.trait_.tr),
            _ => self.impls_unheaded.push(im.trait_.tr),
        }
        let mut names: Vec<Sym> = im.methods.keys().copied().collect();
        names.extend(self.traits[im.trait_.tr.0].methods.keys());
        names.sort();
        names.dedup();
        for n in names {
            self.impls_by_member.entry(n).or_default().push(k);
        }
        self.impls.push(im);
    }
}

/// What each name means in one module, by the name's id.
pub type View = HashMap<Sym, Named>;

/// What `name` means in view `v`.
pub(crate) fn look(syms: &Syms, v: &View, name: &str) -> Option<Named> {
    v.get(&syms.get(name)?).copied()
}

/// What each name means in module `mi`: the prelude's for the program, then the core's, then
/// what its `use` lines import, then its own (R11.2.3). `errors` is `None` while fns are not
/// declared yet, so a name not found is not yet known to be missing.
/// What every module declares and imports, from which each module's view is built.
struct Scopes<'a> {
    own: &'a Layer<HashMap<String, Named>>,
    public: &'a Layer<HashSet<String>>,
    names: &'a [String],
    imports: &'a [(ModId, forms::UseForm)],
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
        let (path, ns, glob, alias) = (&s.path, &s.names, &s.glob, &s.alias);
        let module = path
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(".");
        // `use std.rand.Rng` names one item of `std.rand` (R11.2.1).
        let (module, ns) = match (index(&module), path.split_last()) {
            (None, Some((last, init))) if ns.is_empty() && !*glob => (
                init.iter()
                    .map(String::as_str)
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
                .map_or(module.rsplit('.').next().unwrap_or(""), |a| a.as_str());
            v.insert(m.to_string(), Named::Module(ModId(t)));
            for n in exports(sc, t) {
                if let Some(x) = export(t, &n) {
                    v.insert(format!("{m}.{n}"), x);
                }
            }
            continue;
        }
        for n in ns {
            match export(t, n) {
                Some(_) if own[mi].contains_key(n) => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(Unbuilt::UseOfOwn { name: n.clone() }.into());
                    }
                }
                // Two imports of one name stay ambiguous where it is used (R08.1.4).
                Some(x) if imported.get(n).is_some_and(|y| *y != x) => {
                    v.remove(n);
                    if let Some(a) = ambiguous.as_deref_mut() {
                        a.insert(n.clone());
                    }
                }
                Some(x) => {
                    v.insert(n.clone(), x);
                    imported.insert(n.clone(), x);
                }
                None => {
                    if let Some(e) = errors.as_deref_mut() {
                        e.push(
                            Unbuilt::UseMissing {
                                module: module.to_string(),
                                name: n.clone(),
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
        out.extend(s.names.iter().cloned());
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
                let (path, names, glob) = (&s.path, &s.names, &s.glob);
                let module = path
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(".");
                let named = *glob || names.iter().any(|x| x == n);
                named
                    .then(|| sc.names.iter().position(|m| *m == module))
                    .flatten()
            })?;
    }
}

/// The item a method sits in: its type parameters and set parameters come first.
struct Within<'a> {
    owner: Option<Owner>,
    generics: &'a [Sym],
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

/// The trait a bound names, with how many arguments it writes.
fn bound_trait(syms: &Syms, t: &forms::TyForm) -> Option<(Sym, usize)> {
    use cpl_next_syntax::ast::Type;
    match t {
        forms::TyForm::Named(n, args) => Some((*n, args.len())),
        forms::TyForm::Written(Type::Path(p)) => {
            let text: Vec<_> = p.segments().map(|s| s.text().to_string()).collect();
            let n = p.generic_args().map_or(0, |a| a.args().count());
            Some((syms.get(&text.join("."))?, n))
        }
        _ => None,
    }
}

/// A parameter typed by a trait (`b: Big<Int>`) has a type parameter of its own bounded by it, as
/// `<B: Big<Int>>` would be (R08.2.1); its name `b.impl` can't be written.
fn trait_typed(
    f: &forms::FnForm,
    outer: &[Sym],
    names: &dyn Fn(Sym) -> Option<Named>,
    syms: &mut Syms,
) -> Option<forms::FnForm> {
    use cpl_next_syntax::ast::Type;
    let mut out: Option<forms::FnForm> = None;
    for k in 0..f.params.len() {
        let Some(t @ forms::TyForm::Written(Type::Path(_))) = &f.params[k].ty else {
            continue;
        };
        let Some((n, _)) = bound_trait(syms, t) else {
            continue;
        };
        let shadowed = outer.contains(&n) || f.generics.contains(&n);
        if shadowed || !matches!(names(n), Some(Named::Trait(_))) {
            continue;
        }
        let g = syms.intern(&format!("{}.impl", syms.text(f.params[k].name)));
        let o = out.get_or_insert_with(|| f.clone());
        o.generics.push(g);
        o.bounds.push(forms::BoundForm {
            param: g,
            tr: t.clone(),
        });
        o.params[k].ty = Some(forms::TyForm::Named(g, Vec::new()));
    }
    out
}

fn add_fn(d: &mut Decls, m: ModId, item: ItemId, f: &forms::FnForm, within: Within) -> FnId {
    use crate::attrs::{DeclKind, Decorator};
    use forms::{BodyForm, TyForm};
    let Within {
        owner,
        generics: outer,
        sets: outer_sets,
        self_ty: outer_self,
        sig,
    } = within;
    let names = d.views[m.0].clone();
    let named = |n: Sym| names.get(&n).copied();
    let desugared = trait_typed(f, outer, &named, &mut d.syms);
    let f = desugared.as_ref().unwrap_or(f);
    let fname = d.syms.text(f.name).to_string();
    let mut generics = outer.to_vec();
    generics.extend(f.generics.iter().copied());
    // The bounds' traits, each the `j`th of its parameter's, then each `without` a row-bound
    // parameter's record takes, whose trait's associated type is a parameter too (R08.4.1).
    let mut heads: Vec<(Sym, usize, TraitId, usize)> = Vec::new();
    let nth = |p: Sym, heads: &Vec<(Sym, usize, TraitId, usize)>| {
        heads.iter().filter(|h| h.0 == p).count()
    };
    for b in &f.bounds {
        if let Some((n, k)) = bound_trait(&d.syms, &b.tr) {
            if let Some(Named::Trait(tr)) = named(n) {
                let j = nth(b.param, &heads);
                heads.push((b.param, j, tr, k));
            }
        }
    }
    let written_heads = heads.len();
    let rests = rest_forms(d, &names, f);
    let mut rest_traits = Vec::new();
    for (r, form) in &rests {
        let tr = d
            .syms
            .get(&crate::records::without_name(form))
            .and_then(named);
        if let Some(Named::Trait(tr)) = tr {
            let j = nth(*r, &heads);
            heads.push((*r, j, tr, form.0.len()));
            rest_traits.push(((*r, form.clone()), (tr, j)));
        }
    }
    let hidden = hidden_params(d, &heads, &mut generics);
    let doms: HashMap<Sym, Ty> = d.doms_in(m);
    let jag_in: Vec<(String, Option<cpl_next_syntax::ast::Type>)> = (f.params.iter())
        .map(|p| {
            let t = match &p.ty {
                Some(TyForm::Written(t)) => Some(t.clone()),
                _ => None,
            };
            (d.syms.text(p.name).to_string(), t)
        })
        .collect();
    let jag = crate::synth::jagged(&jag_in);
    let tables = d.tables();
    let rows = std::cell::RefCell::new(Vec::new());
    let mut cx = TyCx {
        axes: None,
        names: &named,
        env: &tables,
        syms: &tables.syms,
        item: Some(item.clone()),
        generics: &generics,
        outer: &generics,
        args: None,
        expanding: Vec::new(),
        doms: &doms,
        scoped: &tyconv::unscoped,
        self_ty: outer_self.clone(),
        rows: Some(&rows),
        ret_slots: None,
        next_set: Some(outer_sets),
    };
    let mut errors = Vec::new();
    let mut self_mode = SelfMode::None;
    let mut self_name = d.syms.intern("self");
    let mut self_ty = None;
    let mut params = Vec::new();
    let mut param_syms = Vec::new();
    let sig_takes_self = sig.as_ref().is_some_and(|s| s.self_mode != SelfMode::None);
    for (k, p) in f.params.iter().enumerate() {
        if p.is_self || (k == 0 && sig_takes_self) {
            self_name = p.name;
            self_mode = if p.mutable || sig.as_ref().is_some_and(|s| s.self_mode == SelfMode::Mut) {
                SelfMode::Mut
            } else {
                SelfMode::Value
            };
            // A free fn's `self` says what it extends (R08.1.4).
            self_ty = match (&outer_self, &p.ty) {
                (Some(t), _) => Some(t.clone()),
                (None, Some(t)) => Some(conv_form(&mut cx, t, &mut errors)),
                (None, None) => {
                    errors.push(Msg::at(
                        item.at(p.name_span),
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
        let jagged = jag.get(k).cloned().flatten();
        let ty = match (&p.ty, from_trait, &jagged) {
            // `k: 0..=n`: the range its ends reach over the earlier parameters' (R06.4.1).
            (_, _, Some(((lo, hi), _))) => Ty::Range(*lo, *hi),
            (Some(t), _, None) => conv_form(&mut cx, t, &mut errors),
            (None, Some(t), None) => t,
            (None, None, None) => {
                errors.push(Msg::at(
                    item.at(p.name_span),
                    Msg::verdict(
                        cpl_diag::codes::UNKNOWN_TYPE,
                        format!("parameter `{}` needs a type", d.syms.text(p.name)),
                    ),
                ));
                Ty::Error
            }
        };
        param_syms.push(p.name);
        params.push(Param {
            name: d.syms.text(p.name).to_string(),
            label: d.syms.text(p.label).to_string(),
            ty,
            mutable: p.mutable,
            named_only: p.named_only,
            variadic: p.variadic,
            default: p.default.clone(),
            reject: reject(&mut cx, &item, p, &mut errors),
            written: jagged.map(|(_, t)| t),
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
        let t = d.new_dom(&item);
        if let Ty::Dom(i) = t {
            d.fresh_doms.push((i, format!("{fname}.{c}")));
        }
        let k = d.syms.intern(&format!("${}", d.fresh_doms.len()));
        d.top_doms.insert(k, t.clone());
        ret_doms.insert(d.syms.intern(c), t);
    }
    cx.doms = &ret_doms;
    let mut region = None;
    let mut lends = None;
    let ret = match (&f.ret, &f.lends) {
        (Some(t), Some((true, _))) => {
            region = Some((true, conv_form(&mut cx, t, &mut errors)));
            Ty::Unit
        }
        // A read lend is the fn's value, which borrows every source it names, or every parameter.
        (Some(t), Some((false, sources))) => {
            let receiver = (self_mode != SelfMode::None).then_some(self_name);
            let names: Vec<Sym> = receiver
                .into_iter()
                .chain(param_syms.iter().copied())
                .collect();
            let mut from = Vec::new();
            for (s, at) in sources {
                match names.iter().position(|n| n == s) {
                    Some(i) => from.push(i),
                    None => errors.push(Msg::at(
                        item.at(*at),
                        Msg::verdict(
                            cpl_diag::codes::UNRESOLVED_NAME,
                            format!("`{}` is not a parameter", d.syms.text(*s)),
                        ),
                    )),
                }
            }
            if sources.is_empty() {
                from = (0..names.len()).collect();
            }
            lends = Some(from);
            conv_form(&mut cx, t, &mut errors)
        }
        (Some(t), None) if f.yields => {
            region = Some((false, conv_form(&mut cx, t, &mut errors)));
            Ty::Unit
        }
        (Some(t), None) => conv_form(&mut cx, t, &mut errors),
        (None, _) => sig.as_ref().map_or(Ty::Unit, |s| s.ret.clone()),
    };
    let n_ret_sets = cx.ret_slots.unwrap_or(0);
    let mut bounds = Vec::new();
    let mut counted: Vec<Sym> = Vec::new();
    for b in &f.bounds {
        let j = counted.iter().filter(|p| **p == b.param).count();
        counted.push(b.param);
        let Some(i) = generics.iter().position(|g| *g == b.param) else {
            continue;
        };
        if let Some(r) = trait_ref(&mut cx, &b.tr, &mut errors) {
            bound_ref(d, r, i, (b.param, j), &hidden, &mut bounds);
        }
    }
    // Each `without` the signature takes of a row-bound parameter bounds it by that form's trait.
    let mut seen_rows = Vec::new();
    for (i, form, args) in rows.take() {
        let Some(((r, _), (tr, j))) = rest_traits
            .iter()
            .find(|((r, g), _)| generics.get(i) == Some(r) && *g == form)
        else {
            continue;
        };
        if seen_rows.contains(&(*r, *j)) {
            continue;
        }
        seen_rows.push((*r, *j));
        bound_ref(
            d,
            TraitRef { tr: *tr, args },
            i,
            (*r, *j),
            &hidden,
            &mut bounds,
        );
    }
    debug_assert!(written_heads <= heads.len());
    // A decorator changes meaning, so one the slice does not implement refuses the program; it is
    // never skipped.
    let kind = DeclKind::Fn {
        params: f.params.len(),
    };
    let built = |a: Decorator| {
        matches!(
            a,
            Decorator::Coerce
                | Decorator::Pattern
                | Decorator::Input
                | Decorator::Role
                | Decorator::Get
                | Decorator::SameKeys
                | Decorator::Index
                | Decorator::Aka
                | Decorator::Was
                | Decorator::FromOp
                | Decorator::Operator
                | Decorator::Fast
                | Decorator::Lint
                | Decorator::Stress
                | Decorator::OnUnimplemented
                | Decorator::OnDestructure
                | Decorator::Memo
                | Decorator::FreeForm
        )
    };
    let attr_at = |x: &forms::AttrForm| {
        use cpl_next_syntax::ast::AstNode;
        item.at(x.node.as_ref().map_or(f.span, |n| n.syntax().span()))
    };
    for x in &f.attrs {
        // `@memo` spells its table: the word names which, never picked for it (R06.4.1).
        let bare = x
            .node
            .as_ref()
            .and_then(|n| n.args())
            .is_none_or(|a| a.args().next().is_none());
        if x.attr == Decorator::Memo && bare {
            let msg = Msg::verdict(
                cpl_diag::codes::BAD_MEMO,
                "`@memo` spells its table: write `@memo(array)` or `@memo(map)`",
            );
            errors.push(Msg::at(attr_at(x), msg));
            continue;
        }
        attr_rules::check(x, kind, &item, &mut errors);
        if x.attr == Decorator::Lint {
            let body = match &f.body {
                BodyForm::Written(cpl_next_syntax::ast::FnBody::Expr(e)) => {
                    use cpl_next_syntax::ast::AstNode;
                    let toks = e.syntax().tokens().filter(|t| !t.kind().is_trivia());
                    Some(toks.map(|t| (t.kind(), t.text().to_string())).collect())
                }
                _ => None,
            };
            let receiver = (self_mode != SelfMode::None).then_some(self_name);
            let lint_params = (receiver.into_iter().chain(param_syms.iter().copied()))
                .map(|p| d.syms.text(p).to_string())
                .collect();
            let lf = attr_rules::LintedFn {
                name: &fname,
                params: lint_params,
                yields: f.yields,
                body,
            };
            attr_rules::lint(x, &lf, &item, &mut errors);
        }
        if !built(x.attr) {
            let name = x.attr.name().to_string();
            errors.push(Msg::at(attr_at(x), Unbuilt::Decorator { name }));
        }
    }
    let has = |a: Decorator| f.attrs.iter().any(|x| x.attr == a);
    // A getter is read like a field, so nothing but `self` may need an argument (R02.6.6).
    let getter = has(Decorator::Get);
    let same_keys = has(Decorator::SameKeys);
    let words = |a: Decorator| -> Vec<String> {
        use cpl_next_syntax::ast::Expr;
        (f.attrs.iter().filter(|x| x.attr == a))
            .filter_map(|x| x.node.as_ref()?.args())
            .flat_map(|args| args.args().collect::<Vec<_>>())
            .filter_map(|arg| match arg.value()? {
                Expr::Name(n) => Some(n.ident()?.text().to_string()),
                _ => None,
            })
            .collect()
    };
    if has(Decorator::Index) {
        d.index_fns.push(FnId(d.fns.len()));
    }
    for x in f.attrs.iter().filter(|x| x.attr == Decorator::FreeForm) {
        match owner {
            Some(Owner::Trait(tr)) if self_mode != SelfMode::None => {
                d.free_forms.insert(FnId(d.fns.len()), (tr, f.name));
            }
            _ => errors.push(Msg::at(attr_at(x), attr_rules::free_form_misplaced())),
        }
    }
    if has(Decorator::FromOp) {
        d.from_op.push(FnId(d.fns.len()));
    }
    for x in f.attrs.iter().filter(|x| x.attr == Decorator::Operator) {
        let Some(text) = x.str_arg() else {
            continue;
        };
        let op = d.syms.intern(&text);
        let msg = if d.operators.iter().any(|(o, _)| *o == op) {
            Some(Msg {
                help: Some(format!(
                    "`{text}` is its `@operator` trait's: impl that trait"
                )),
                ..Msg::verdict(
                    cpl_diag::codes::BAD_LIBRARY_LINT,
                    format!("`@operator(\"{text}\")` on a fn, but a trait already gives `{text}`"),
                )
            })
        } else if d.op_fns.iter().any(|(o, _)| *o == op) {
            Some(Msg::verdict(
                cpl_diag::codes::OVERLOAD_BY_TYPE,
                format!("a second fn marked `@operator(\"{text}\")`"),
            ))
        } else {
            d.op_fns.push((op, FnId(d.fns.len())));
            None
        };
        errors.extend(msg.map(|m| Msg::at(attr_at(x), m)));
    }
    let coerce = has(Decorator::Coerce);
    if coerce {
        d.coercions.push(FnId(d.fns.len()));
    }
    if has(Decorator::Pattern) {
        d.pattern_fns.push(FnId(d.fns.len()));
    }
    let id = FnId(d.fns.len());
    for x in f.attrs.iter().filter(|x| x.attr == Decorator::Role) {
        let role = form_role(x);
        if let Some(Role::Trace) = role {
            d.roles.trace.push(id);
            continue;
        }
        let slot = match role {
            Some(Role::Print) => &mut d.roles.print,
            Some(Role::Write) => &mut d.roles.write,
            Some(Role::StrLiteral) => &mut d.sugar.str_literal,
            Some(Role::RangeLiteral) => &mut d.sugar.range_literal,
            Some(Role::Interpolation) => &mut d.roles.interpolate,
            Some(Role::VariantCheck) => &mut d.roles.variant_check,
            Some(Role::Todo) => &mut d.roles.todo,
            Some(_) => {
                let name = x.attr.name().to_string();
                errors.push(Msg::at(attr_at(x), Unbuilt::Decorator { name }));
                continue;
            }
            None => {
                d.errors
                    .push(collect::misplaced_role(&item, attr_at(x), None, "fn"));
                continue;
            }
        };
        let dup = Msg::at(
            item.at(f.span),
            Msg::verdict(
                cpl_diag::codes::DUPLICATE_DEFINITION,
                "a second declaration filling one `@role`",
            ),
        );
        collect::once(slot, id, dup, &mut d.errors);
    }
    if has(Decorator::Input) {
        d.input_fns.push(FnId(d.fns.len()));
    }
    if getter && params.iter().any(|p| p.default.is_none()) {
        let msg = format!(
            "getter `{fname}` is read like a field, so each parameter but `self` needs a default"
        );
        let code = cpl_diag::codes::GETTER_NEEDS_DEFAULTS;
        d.misdeclared
            .push((item.at(Span::new(f.span.start, f.span.start)), code, msg));
    }
    if !binders.is_empty() {
        let slots = binders.iter().map(|b| b.0).collect();
        d.fresh_slots.push((FnId(d.fns.len()), slots));
    }
    if matches!(&f.ret, Some(TyForm::Written(t)) if tyconv::has_hole(t)) {
        d.ret_holes.push(FnId(d.fns.len()));
    }
    let intrinsic =
        match &f.body {
            BodyForm::Written(cpl_next_syntax::ast::FnBody::Expr(
                cpl_next_syntax::ast::Expr::Intrinsic(e),
            )) => {
                use cpl_next_syntax::ast::AstNode;
                use cpl_next_syntax::literal::{string_parts, StrPart};
                let tok = e.id();
                let text = tok.as_ref().map_or("", |t| t.text());
                let n: String = (string_parts(text).into_iter())
                    .filter_map(|p| match p {
                        StrPart::Text(s) => Some(&text[s.start as usize..s.end as usize]),
                        StrPart::Expr(_) => None,
                    })
                    .collect();
                let span = e.syntax().span();
                let known = cpl_next_ir::ops::Intrinsic::named(&n);
                // The table grows only by a decision entry; a program names one of its ids
                // (R12.14.1, R12.14.5).
                if known.is_none() {
                    let ids: Vec<&str> =
                        (cpl_next_ir::ops::INTRINSICS.iter()).map(|e| e.0).collect();
                    let msg = cpl_diag::msgs::check::UNKNOWN_INTRINSIC_NO_INTRINSIC_THE_TABLE
                        .say(&[&n, &ids.join(", ")]);
                    let code = cpl_diag::codes::UNKNOWN_INTRINSIC;
                    d.misdeclared.push((item.at(span), code, msg));
                    d.rejected_bodies.insert(FnId(d.fns.len()));
                }
                // A type parameter stands for the entry's types; each call's are checked.
                let fits = |ty: &Ty, want: &str| {
                    matches!(ty, Ty::Param(_)) || cpl_next_ir::ops::entry_fits(ty, want)
                };
                if let Some((want, res)) = cpl_next_ir::ops::entry_sig(&n) {
                    let receiver = self_ty.iter().filter(|_| self_mode != SelfMode::None);
                    let taken: Vec<&Ty> = receiver.chain(params.iter().map(|p| &p.ty)).collect();
                    let params_fit = taken.len() == want.len()
                        && (taken.iter().zip(want)).all(|(t, w)| fits(t, w));
                    if !params_fit || !fits(&ret, res) {
                        let msg = cpl_diag::msgs::check::UNKNOWN_INTRINSIC_IS_MUST_HAVE_THAT
                            .say(&[&n, &want.join(", "), &res, &fname]);
                        let code = cpl_diag::codes::UNKNOWN_INTRINSIC;
                        d.misdeclared.push((item.at(span), code, msg));
                    }
                }
                known
            }
            _ => None,
        };
    let on_destructure = on_destructure(&item, f, &ret, &region, &mut errors);
    let fns = d.item_fns.entry(item.clone()).or_default();
    let at = fns.len() as u32;
    fns.push(FnId(d.fns.len()));
    let body = match &f.body {
        BodyForm::Written(_) => Body::Written,
        BodyForm::Value(_) => Body::Value,
        BodyForm::Synth(b) => Body::Synth(b.clone()),
        BodyForm::None => Body::None,
    };
    d.fns.push(FnDecl {
        made: (0, 0),
        at,
        name: fname.clone(),
        owner,
        self_mode,
        self_name: d.syms.text(self_name).to_string(),
        self_ty,
        generics,
        bounds,
        params,
        labels: (f.params.iter())
            .filter(|p| p.named_only)
            .map(|p| d.syms.text(p.label).to_string())
            .collect(),
        ret,
        n_sets,
        n_ret_sets,
        region,
        lends,
        uses: f.uses.iter().map(|n| d.syms.text(*n).to_string()).collect(),
        body,
        intrinsic,
        fast: words(Decorator::Fast).into_iter().next(),
        getter,
        public: f.public,
        same_keys,
        aka: words(Decorator::Aka),
        was: words(Decorator::Was),
        on_destructure,
        span: item.at(f.span),
        item,
        module: m,
        errors,
    });
    FnId(d.fns.len() - 1)
}

/// The parameters a bound that leaves its trait's arguments out takes for each, at `generics`'
/// end, by bound and trait. An associated type is named through the bounded parameter: `G.Arc` of
/// `G: Next<V>` (R08.2.3); any other is `S.1`, which can't be written, fixed when the bound is met
/// (R08.1.2). An associated type's default is the impl's to take, never the bound's.
type Hidden = Vec<((Sym, usize), std::ops::Range<usize>)>;

/// The `(to: X, ..R without (u, v))` a signature writes, through its aliases, of a parameter `R`
/// whose row bound has those fields: `R`'s record without them is then a parameter of the
/// signature, bounded by that form's trait (R08.4.1).
fn rest_forms(d: &Decls, names: &View, f: &forms::FnForm) -> Vec<(Sym, crate::records::Form)> {
    use cpl_next_syntax::ast::{GenericArg, Type};
    /// What an alias's parameters stand for while its body is walked: each one's argument,
    /// with where that argument is written.
    struct AliasScope<'e> {
        names: &'e View,
        sub: HashMap<Sym, (Type, &'e AliasScope<'e>)>,
    }
    fn bare(d: &Decls, t: &Type) -> Option<Sym> {
        let Type::Path(p) = t else { return None };
        if p.generic_args().is_some_and(|a| a.args().next().is_some()) {
            return None;
        }
        let text: Vec<_> = p.segments().map(|s| s.text().to_string()).collect();
        d.syms.get(&text.join("."))
    }
    fn walk(
        d: &Decls,
        f: &forms::FnForm,
        env: &AliasScope,
        t: &Type,
        out: &mut Vec<(Sym, crate::records::Form)>,
    ) {
        let types = |args: Vec<GenericArg>| -> Vec<Type> {
            (args.into_iter())
                .filter_map(|a| match a {
                    GenericArg::Type(t) => Some(t),
                    GenericArg::Param(_) => None,
                })
                .collect()
        };
        match t {
            Type::Path(p) => {
                let args = types(
                    p.generic_args()
                        .map(|a| a.args().collect())
                        .unwrap_or_default(),
                );
                args.iter().for_each(|a| walk(d, f, env, a, out));
                let text: Vec<_> = p.segments().map(|s| s.text().to_string()).collect();
                let Some(n) = d.syms.get(&text.join(".")) else {
                    return;
                };
                if env.sub.contains_key(&n) || f.generics.contains(&n) {
                    return;
                }
                if let Some(Named::Alias(id)) = env.names.get(&n).copied() {
                    let a = &d.aliases[id.0];
                    if a.generics.len() == args.len() {
                        let sub = (a.generics.iter().copied())
                            .zip(args.iter().map(|t| (t.clone(), env)))
                            .collect();
                        walk(
                            d,
                            f,
                            &AliasScope {
                                names: &a.names,
                                sub,
                            },
                            &a.ty,
                            out,
                        );
                    }
                }
            }
            Type::Record(r) => {
                let fields: Vec<_> = r.fields().collect();
                fields
                    .iter()
                    .filter_map(|x| x.ty())
                    .for_each(|t| walk(d, f, env, &t, out));
                let Some(rest) = fields.iter().find(|x| x.is_rest()) else {
                    return;
                };
                let Some(mut base) = rest.ty() else { return };
                // `R`, through the aliases that pass it on.
                let mut at = env;
                let r = loop {
                    let Some(n) = bare(d, &base) else { return };
                    match at.sub.get(&n) {
                        Some((t, outer)) => {
                            base = t.clone();
                            at = outer;
                        }
                        None => break n,
                    }
                };
                let row = (f.bounds.iter().filter(|b| b.param == r)).find_map(|b| match &b.tr {
                    forms::TyForm::Written(Type::Record(rr))
                        if rr.fields().any(|x| x.is_rest() && x.ty().is_none()) =>
                    {
                        Some(
                            (rr.fields().filter_map(|x| x.name()))
                                .map(|t| t.text().to_string())
                                .collect::<Vec<_>>(),
                        )
                    }
                    _ => None,
                });
                let removed: Vec<String> = rest
                    .removed()
                    .iter()
                    .map(|t| t.text().to_string())
                    .collect();
                if !row.is_some_and(|row| removed.iter().all(|n| row.contains(n))) {
                    return;
                }
                let prefix = (fields.iter().filter(|x| !x.is_rest()))
                    .map(|x| x.name().map_or(String::new(), |t| t.text().to_string()))
                    .collect();
                let form = (prefix, removed);
                if !out.iter().any(|(x, g)| *x == r && *g == form) {
                    out.push((r, form));
                }
            }
            Type::Array(a) => {
                let dims = types(a.dims().map(|x| x.args().collect()).unwrap_or_default());
                dims.iter()
                    .chain(a.elem().iter())
                    .for_each(|t| walk(d, f, env, t, out));
            }
            Type::Optional(o) => o.inner().iter().for_each(|t| walk(d, f, env, t, out)),
            Type::Tuple(ts) => ts.elems().iter().for_each(|(_, t)| walk(d, f, env, t, out)),
            Type::Fn(ft) => {
                ft.param().iter().for_each(|t| walk(d, f, env, t, out));
                ft.ret()
                    .and_then(|r| r.ty())
                    .iter()
                    .for_each(|t| walk(d, f, env, t, out));
            }
            _ => {}
        }
    }
    let env = AliasScope {
        names,
        sub: HashMap::new(),
    };
    let mut out = Vec::new();
    let sig = (f.params.iter().filter_map(|p| p.ty.as_ref())).chain(f.ret.as_ref());
    for t in sig {
        if let forms::TyForm::Written(t) = t {
            walk(d, f, &env, t, &mut out);
        }
    }
    out
}

/// The parameters each bound that leaves its trait's arguments out takes, at `generics`' end:
/// `heads` holds each bound's parameter, its place among that parameter's bounds, its trait and
/// how many arguments it writes.
fn hidden_params(
    d: &mut Decls,
    heads: &[(Sym, usize, TraitId, usize)],
    generics: &mut Vec<Sym>,
) -> Hidden {
    let mut hidden = Vec::new();
    for &(param, j, tr, written) in heads {
        let n = d.traits[tr.0].defaults.len();
        let first_assoc = n - d.traits[tr.0].assoc.len();
        if written < n && (written >= first_assoc || d.traits[tr.0].defaults[written].is_none()) {
            let k = generics.len();
            let assoc = d.traits[tr.0].assoc.clone();
            for p in written..n {
                let name = match p.checked_sub(n - assoc.len()) {
                    Some(a) => format!("{}.{}", d.syms.text(param), assoc[a]),
                    None => format!("{}.hidden", k + p - written),
                };
                generics.push(d.syms.intern(&name));
            }
            hidden.push(((param, j), k..generics.len()));
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
    key: (Sym, usize),
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
fn fresh_binders(ret: Option<&forms::TyForm>) -> Vec<(usize, String)> {
    use cpl_next_syntax::ast::Type;
    let binder = |t: Option<Type>| match t {
        Some(Type::DomainBinder(b)) => b.name().map(|n| n.text().to_string()),
        _ => None,
    };
    match ret {
        Some(forms::TyForm::Written(Type::Record(r))) => (r.fields().enumerate())
            .filter_map(|(k, x)| Some((k, binder(x.ty())?)))
            .collect(),
        Some(forms::TyForm::Written(Type::Tuple(ts))) => (ts.elems().into_iter().enumerate())
            .filter_map(|(k, (_, t))| Some((k, binder(Some(t))?)))
            .collect(),
        _ => Vec::new(),
    }
}

/// The role a `@role(name)` attribute names, when `name` is one of R11.6.1's.
pub(crate) fn form_role(x: &forms::AttrForm) -> Option<Role> {
    use cpl_next_syntax::ast::Expr;
    if x.attr != crate::attrs::Decorator::Role {
        return None;
    }
    let arg = x.node.as_ref()?.args()?.args().next()?.value()?;
    let Expr::Name(n) = arg else {
        return None;
    };
    let n = n.ident()?;
    Role::ALL.into_iter().find(|r| r.name() == n.text())
}

/// Whether a declaration with these attributes fills role `r`.
pub(crate) fn form_fills(attrs: &[forms::AttrForm], r: Role) -> bool {
    attrs.iter().any(|x| form_role(x) == Some(r))
}

pub fn required_labels(items: &dyn crate::views::Items, d: FnId) -> Vec<&str> {
    let ps = &items.f(d).params;
    // A parameter after `..xs` is given only by its label, like one after `;`.
    let after = ps
        .iter()
        .position(|p| p.variadic)
        .map_or(ps.len(), |i| i + 1);
    let mut ls: Vec<&str> = (ps.iter().enumerate())
        .filter(|(i, p)| (p.named_only || *i >= after) && p.default.is_none())
        .map(|(_, p)| p.label.as_str())
        .collect();
    ls.sort_unstable();
    ls
}

/// The fn of `first`'s name a call giving the labels `given` takes: the one whose required
/// labels it gives and that has every label it gives (R06.1.6). None when no overload fits:
/// the call is the named-argument error (E0310).
pub fn overload(
    items: &dyn crate::views::Items,
    facts: &dyn crate::views::FnFacts,
    first: FnId,
    given: &[&str],
) -> Option<FnId> {
    let rest = facts.overloads(first);
    let fits = |f: FnId| {
        let has = |l: &&str| items.f(f).params.iter().any(|p| p.label == *l);
        required_labels(items, f).iter().all(|l| given.contains(l)) && given.iter().all(has)
    };
    (std::iter::once(first).chain(rest.iter().copied())).find(|f| fits(*f))
}

/// A fn's `@on_destructure("..", fix: "..")` (R02.5.5): a message, an optional `code:` and
/// `fix:`. On a fn with no result to take apart, or without a message, it is E0505.
fn on_destructure(
    item: &ItemId,
    f: &forms::FnForm,
    ret: &Ty,
    region: &Option<(bool, Ty)>,
    errors: &mut Vec<Msg>,
) -> Option<Unimplemented> {
    use cpl_next_syntax::ast::AstNode;
    let x = (f.attrs.iter()).find(|a| a.attr == crate::attrs::Decorator::OnDestructure)?;
    let node = x.node.as_ref()?;
    let bad = |why: &str| {
        Msg::at(
            item.at(node.syntax().span()),
            Msg::verdict(cpl_diag::codes::BAD_LIBRARY_LINT, why.to_string()),
        )
    };
    let said = collect::said(item, node, crate::attrs::Decorator::OnDestructure, errors);
    let gives = !matches!(ret, Ty::Unit) || matches!(region, Some((false, _)));
    match said.help {
        _ if !gives => errors.push(bad(
            "`@on_destructure` sits on a fn with a result or a generator: a pattern takes apart what it gives",
        )),
        Some(_) => return Some(said),
        None => errors.push(bad("`@on_destructure` takes a message")),
    }
    None
}

/// A parameter's `@reject(when: T = Bool, "..", code: C)` (R06.4.17). Any other decorator on a
/// parameter, or a `@reject` with no default to guard, no message or code, or a `when` that is
/// not type parameters equal to types, is E0505.
fn reject(
    cx: &mut TyCx,
    item: &ItemId,
    p: &forms::ParamForm,
    errors: &mut Vec<Msg>,
) -> Option<Reject> {
    use cpl_next_syntax::ast::AstNode;
    use cpl_next_syntax::grammar::SaidArg;
    let mut out = None;
    for x in &p.decorators {
        let at = item.at(x.syntax().span());
        let bad = |why: &str| {
            Msg::at(
                at.clone(),
                Msg::verdict(cpl_diag::codes::BAD_LIBRARY_LINT, why.to_string()),
            )
        };
        if x.name().and_then(|n| crate::attrs::Decorator::of(n.text()))
            != Some(crate::attrs::Decorator::Reject)
        {
            errors.push(bad("only `@reject` goes on a parameter"));
            continue;
        }
        let said = collect::said(item, x, crate::attrs::Decorator::Reject, errors);
        let (mut when, mut typed) = (Vec::new(), true);
        let args: Vec<_> = x.args().map(|l| l.args().collect()).unwrap_or_default();
        for a in args
            .iter()
            .filter(|a| a.label().and_then(|l| SaidArg::of(l.text())) == Some(SaidArg::When))
        {
            let sides: Vec<_> = a.types().collect();
            typed &= !sides.is_empty() && sides.len() % 2 == 0;
            for pair in sides.chunks(2) {
                let [l, r] = pair else { continue };
                let l = conv_form(cx, &TyForm::Written(l.clone()), errors);
                let r = conv_form(cx, &TyForm::Written(r.clone()), errors);
                match l {
                    Ty::Param(i) => when.push((i, r)),
                    _ => typed = false,
                }
            }
        }
        match (said.code, said.help) {
            _ if p.default.is_none() => {
                errors.push(bad("`@reject` guards a parameter's default, and this one has none"))
            }
            _ if !typed || when.is_empty() => errors.push(bad(
                "`@reject` takes `when: T = Type`, each `T` a type parameter of the fn, joined by `and`",
            )),
            (Some(code), Some(text)) => out = Some(Reject { when, code, text }),
            _ => errors.push(bad("`@reject` takes a message and `code:` a diagnostic code")),
        }
    }
    out
}
