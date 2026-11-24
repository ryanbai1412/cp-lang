//! From the front end's syntax trees to a typed [`Program`].
//!
//! The front end's checker is deliberately forgiving: what it can't type is
//! `Unknown` and passes. A back end can't run `Unknown`, so this pass types
//! every expression for real, loads imported modules, monomorphizes generic
//! structs/enums/fns/consts, lowers traits and `impl`s, desugars `lends`
//! projections and generators, and emits one concrete [`Program`] shared by
//! `cpl-interp` and `cpl-codegen`.

mod alias;
mod anon_domain;
mod cache;
mod call;
mod calls;
mod capture;
mod coerce;
mod collect;
mod comprehension;
mod consts;
mod contains;
mod declare;
mod declared_keys;
mod do_block;
mod domain_impl;
mod expr;
mod float_fn;
mod fn_impl;
mod fn_value;
mod fresh;
mod function;
mod hoist;
mod impl_bounds;
mod index_set;
mod instance;
mod judge;
mod keyed;
mod lambda_infer;
mod later_calls;
mod lexical_ctx;
mod library_domain;
mod local_gen;
mod memo;
mod mono;
mod names;
mod op_value;
mod or_pattern;
mod ord_space;
mod prefix;
mod product;
mod program;
mod range;
mod range_coerce;
mod read_spec;
mod report;
mod resolve_ty;
mod shown;
mod signature;
mod stmt;
mod sum_product;
mod type_call;
mod typo;
mod unify_ast;
mod views;
mod yields;
pub use call::lower_sampling_laws;
pub(crate) use collect::is_cpp;

use crate::*;
use calls::*;
use cpl_check::Analysis;
use cpl_diag::msgs;
use cpl_diag::{codes, Diagnostic, Span};
use cpl_syntax::{ast, narrow};
use mono::*;
use range::*;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Out-of-line pushes of the two commonest statements: an inlined
/// `Vec::push` of a whole `Stmt` at each of hundreds of sites costs more.
trait PushStmt {
    fn push_let(&mut self, slot: Slot, value: Expr);
    fn push_give(&mut self, value: Expr);
}

impl PushStmt for Vec<Stmt> {
    fn push_let(&mut self, slot: Slot, value: Expr) {
        self.push(Stmt::Let(slot, Some(value)));
    }

    fn push_give(&mut self, value: Expr) {
        self.push(Stmt::Give(value));
    }
}

/// Lists the names of all `@test` functions declared at the top level of `analysis`.
pub fn list_tests(analysis: &Analysis) -> Vec<String> {
    let mut out = Vec::new();
    for item in &analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                if let ast::StmtKind::Fn(decl) = &stmt.kind {
                    out.push(decl.name.text.clone());
                }
            }
        }
    }
    out
}

/// Lowers a file that the front end reported no errors for.
pub fn lower(analysis: &Analysis) -> Result<Program, Vec<Diagnostic>> {
    lower_impl(analysis, None)
}

/// Lowers a file in `@cache` cold mode: read-`self` methods on structs with
/// `@cache` fields receive a copy of `self`, undoing cache writes at exit.
pub fn lower_cold_cache(analysis: &Analysis) -> Result<Program, Vec<Diagnostic>> {
    lower_entry_with_cold(analysis, None, false, call::Stress::ColdCache)
}

/// True when any struct in `analysis` (the file itself) declares a `@cache` field.
pub fn has_cached_fields(analysis: &Analysis) -> bool {
    fn in_stmts(stmts: &[ast::Stmt]) -> bool {
        stmts.iter().any(|s| match &s.kind {
            ast::StmtKind::Struct(sd) => {
                sd.fields.iter().any(|f| f.cached) || in_stmts(&sd.methods)
            }
            _ => false,
        })
    }
    analysis
        .items
        .iter()
        .any(|item| in_stmts(&item.syntax.parsed.stmts))
}

/// Lowers a single `@test` function of `analysis` as the program's entrypoint.
pub fn lower_test(analysis: &Analysis, test_name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_impl(analysis, Some(test_name))
}

/// The `@interactor` fn a file declares at its top level, if any.
/// True when the program's own file has `@!interactive`.
fn directed(analysis: &Analysis) -> bool {
    let stmts = analysis
        .items
        .iter()
        .flat_map(|item| &item.syntax.parsed.stmts);
    stmts
        .flat_map(|stmt| &stmt.decorators)
        .any(|d| d.directive && d.name.text == "interactive")
}

pub fn interactor(analysis: &Analysis) -> Option<String> {
    analysis
        .items
        .iter()
        .flat_map(|item| &item.syntax.parsed.stmts)
        .find_map(|stmt| match &stmt.kind {
            ast::StmtKind::Fn(decl)
                if stmt.decorators.iter().any(|d| d.name.text == "interactor") =>
            {
                Some(decl.name.text.clone())
            }
            _ => None,
        })
}

/// The fn a file decorates with `@name` at its top level, if any, and
/// the 1-based line of its name.
fn decorated(analysis: &Analysis, wanted: fn(&ast::Decorator) -> bool) -> Option<(String, u32)> {
    analysis.items.iter().find_map(|item| {
        item.syntax
            .parsed
            .stmts
            .iter()
            .find_map(|stmt| match &stmt.kind {
                ast::StmtKind::Fn(decl) if stmt.decorators.iter().any(wanted) => {
                    let at = decl.name.span.shift(item.offset).start;
                    let line = analysis.lines.line_of(at) + 1;
                    Some((decl.name.text.clone(), line))
                }
                _ => None,
            })
    })
}

/// `@alt(program)`, `@alt(program)`, or a bare `@alt` / `@alt`: a slow
/// program to compare the whole file with. `@alt(part)` / `@alt(part)`
/// pairs one fn with another and is not one of these.
fn is_whole_brute(decorator: &ast::Decorator) -> bool {
    matches!(decorator.name.text.as_str(), "brute" | "alt")
        && match decorator.args.first() {
            None => true,
            Some(arg) if arg.name.is_some() => true,
            Some(arg) => {
                matches!(&arg.value.kind, ast::ExprKind::Name(name) if name == "main" || name == "program")
            }
        }
}

/// The `@alt(program)` / `@alt(program)` fn a file declares, and the line it is on.
pub fn brute(analysis: &Analysis) -> Option<(String, u32)> {
    decorated(analysis, is_whole_brute)
}

/// The `@check` / `@check` fn a file declares, and the line it is on.
pub fn checker(analysis: &Analysis) -> Option<(String, u32)> {
    decorated(analysis, |d| {
        matches!(d.name.text.as_str(), "checker" | "check")
    })
}

/// A fn that runs in place of the solution, after the file's input
/// header: everything down to the last top-level `read()`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Aside {
    Brute,
    Checker,
    /// A `@gen` fn: it makes the input, so none of the header runs, and
    /// it is called with this size.
    Gen(i64),
    /// The `gen:` fn of the `read(gen: f)` on this 1-based line: the
    /// header above that line runs, then the fn's value is printed as
    /// `read()` reads it.
    Read(u32),
}

/// Lowers the `@alt(program)` fn `name` as a program: the file's input
/// header, then the fn. A brute that calls `read()` itself reads the
/// whole input, and none of the header runs.
pub fn lower_brute(analysis: &Analysis, name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_aside(analysis, name, Aside::Brute)
}

/// Lowers the `@check` fn `name` as a program: the file's input
/// header, then the fn, whose `read_output()` reads what the solution
/// printed.
pub fn lower_checker(analysis: &Analysis, name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_aside(analysis, name, Aside::Checker)
}

/// The `@gen` fn a file declares, and the line it is on.
pub fn generator(analysis: &Analysis) -> Option<(String, u32)> {
    decorated(analysis, |d| d.name.text == "gen")
}

/// Lowers the `@gen` fn `name` as a program that prints one input:
/// the file's constants and fns, then `name(size)`. Nothing is read.
pub fn lower_gen(analysis: &Analysis, name: &str, size: i64) -> Result<Program, Vec<Diagnostic>> {
    lower_aside(analysis, name, Aside::Gen(size))
}

/// Lowers the `gen:` fn `name` of the `read(gen: name)` on `line` as a
/// program that reads the input above that line and prints the value
/// `name()` makes, written as `read()` reads it.
pub fn lower_gen_read(
    analysis: &Analysis,
    name: &str,
    line: u32,
) -> Result<Program, Vec<Diagnostic>> {
    lower_aside(analysis, name, Aside::Read(line))
}

fn lower_aside(analysis: &Analysis, name: &str, aside: Aside) -> Result<Program, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(analysis);
    lw.aside = Some(aside);
    lw.load_modules(analysis);
    lw.file_contexts();
    lw.mutators = narrow::Mutators::of(lw.mod_items(0).map(|(_, stmt)| stmt)).into();
    lw.steps = lw.read_steps().into();
    lw.collect_all();
    lw.signatures();
    lw.main_for_aside(name);
    lw.finish(analysis, false)
}

/// Lowers the judge's side of an interactive problem: a program that
/// runs the `@interactor` fn `name`. Its `read()` reads the hidden test,
/// `sol.recv()` reads what the solution prints, and `sol.send(..)` prints
/// what the solution reads.
pub fn lower_judge(analysis: &Analysis, name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_entry(analysis, Some(name), true)
}

fn lower_impl(analysis: &Analysis, test_name: Option<&str>) -> Result<Program, Vec<Diagnostic>> {
    lower_entry(analysis, test_name, false)
}

fn lower_entry(
    analysis: &Analysis,
    test_name: Option<&str>,
    judge: bool,
) -> Result<Program, Vec<Diagnostic>> {
    lower_entry_with_cold(analysis, test_name, judge, call::Stress::Off)
}

pub(in crate::lower) fn lower_entry_with_cold(
    analysis: &Analysis,
    test_name: Option<&str>,
    judge: bool,
    stress: call::Stress,
) -> Result<Program, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(analysis);
    lw.judge = judge;
    lw.stress = stress;
    // Whatever the judge sends goes out at once. A solution with a judge in its
    // file talks to it, so it reads as it goes and never all the input at once.
    // `@!interactive` asks for the same without a judge in the file.
    lw.interactive =
        judge || (test_name.is_none() && interactor(analysis).is_some()) || directed(analysis);
    lw.load_modules(analysis);
    lw.file_contexts();
    lw.mutators = narrow::Mutators::of(lw.mod_items(0).map(|(_, stmt)| stmt)).into();
    lw.steps = lw.read_steps().into();
    lw.collect_all();
    lw.signatures();
    if let Some(tname) = test_name {
        lw.main_for_test(tname);
    } else {
        lw.main();
    }
    lw.finish(analysis, test_name.is_none())
}

#[derive(Clone, Copy, Debug)]
enum Sym {
    Global(GlobalId),
    Fn(FnId),
    GenFn(GenFnId),
    Struct(StructId),
    GenStruct(GenStructId),
    Enum(EnumId),
    GenEnum(GenEnumId),
    Variant(EnumId, usize),
    Domain(DomainId),
    Alias(usize),
    GenAlias(GenAliasId),
    GenConst(GenConstId),
    ModConst(ModConstId),
    Trait(TraitId),
    Mod(ModId),
}

struct StructInfo<'a> {
    def: StructDef,
    decl: Option<&'a ast::StructDecl>,
    mod_id: ModId,
    offset: u32,
    origin_gen: Option<(GenStructId, Vec<GenArg>)>,
    env: GenEnv,
    methods: HashMap<String, FnId>,
    /// Methods by `(trait, name)`, for `Trait.method(x)`.
    trait_methods: HashMap<(String, String), FnId>,
    op_methods: Vec<(String, Ty, FnId)>,
    gen_methods: HashMap<String, GenFnId>,
    /// Impls whose params only the trait args name (`impl<T> Acts<T, T> for Plus`).
    open_impls: Vec<usize>,
    getters: HashSet<String>,
    projections: HashMap<String, ProjectionInfo<'a>>,
    consts: HashMap<String, StructConst<'a>>,
    owned_domains: HashMap<String, DomainId>,
    owned_domain_inits: Vec<OwnedDomainInit<'a>>,
    laws: Vec<LawInfo<'a>>,
    /// Backing domain id when `impl Index for S` is defined.
    index_domain: Option<DomainId>,
}

#[derive(Clone)]
struct SigParam<'a> {
    label: String,
    name: String,
    ty: Ty,
    mutable: bool,
    default: Option<&'a ast::Expr>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    InProgress,
    Done,
}

/// Whether a fn's parameter and return types have been read. Every reader
/// goes through `ensure_sig`, so declaration order doesn't matter.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SigState {
    Unresolved,
    Resolving,
    Resolved,
}

struct FnSig<'a> {
    name: String,
    /// Without `self`.
    params: Vec<SigParam<'a>>,
    /// `None` until the body has been read, for a fn without `->`.
    ret: Option<Ty>,
    /// A call of the fn from its own body took it to give nothing.
    assumed_unit: bool,
    /// The struct whose method this is, and whether it takes `self`
    /// (`Some(mutable)`).
    owner: Option<StructId>,
    receiver: Option<bool>,
    mod_id: ModId,
    env: GenEnv,
    decl: &'a ast::FnDecl,
    offset: u32,
    is_generator: bool,
    /// If the return type is `(domain C, ...)`, the allocated DomainId + name.
    new_domain_slots: Vec<(String, DomainId)>,
    /// The return type carries a domain made inside this fn (`compress`'s
    /// `Compressed<D, T>`): each call site gets its own copy and domain.
    returns_fresh_domain: bool,
    /// True once a call site has claimed this `returns_fresh_domain` fn.
    used_fresh_call: bool,
    state: State,
    sig: SigState,
    /// Where this fn was first asked for.
    asked_at: report::Asked,
}

/// What a block is for, which decides what happens to its last expression.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tail {
    Discard,
    Return,
    Give,
}

#[derive(Clone)]
struct DoFrame {
    ty: Option<Ty>,
    /// Loops opened before the block: `break` can't reach them.
    loops: usize,
    /// The block is written out as statements ahead of the statement
    /// that reads its value, so `return` works inside it.
    open: bool,
}

#[derive(Clone, Default)]
struct FnCx {
    locals: Vec<Local>,
    /// Inside an alternative of a `|` pattern: the names it binds, declared once after all.
    or_binds: Option<Vec<(String, Expr)>>,
    /// Local fns an earlier fn in the same block calls, by name: their slots.
    predeclared: HashMap<String, usize>,
    /// The optionals a test has shown to hold a value here, by name.
    present: Vec<String>,
    /// The name about to be lowered stands where an optional is asked for.
    as_written: bool,
    uses_io: bool,
    /// A top-level fn's `uses`: the contexts it follows its caller in.
    lexical_uses: Option<Vec<String>>,
    scopes: Vec<HashMap<String, Slot>>,
    /// Block-scoped type aliases, local domains, etc. (e.g. `domain V = 1..=6`, `type M = Mod<7>`).
    local_syms: Vec<HashMap<String, Sym>>,
    /// Block-scoped generic local functions (`fn id<T>(x: T) -> T = x`).
    local_gen_fns: HashMap<String, ast::FnDecl>,
    /// Parameters bound to compile-time `BareOp` values (`+`, `min`, `max`, `gcd`, etc.).
    op_locals: HashMap<String, BareOp>,
    /// Named mutable aliases (`let st = mut a[i]`): resolved to their underlying `Place`.
    mut_aliases: HashMap<String, Place>,
    /// `let g = gen(n)` with `gen` a generator: `g` is the recipe, and each
    /// read of it walks the generator afresh. By name, with the scope depth.
    recipes: HashMap<String, (usize, ast::Expr)>,
    /// Local zero-copy range views over `SortedBag<K>` / `SortedSet<K>`.
    sorted_views: HashMap<Slot, SortedView>,
    /// Local lazy `xs.map(f)` views (v0.3): each read `m[i]` calls `f(xs[i])`.
    map_views: HashMap<Slot, MapView>,
    /// Block-scoped `type Alias = ...` resolved to concrete types (e.g. `type Cell = (Row, Col)`).
    local_type_aliases: HashMap<String, Ty>,
    /// Local slots declared `var` or mutated, so closures capture them by reference.
    mut_slots: HashSet<Slot>,
    /// Annotated array `Spec`s for local variables/parameters, used by `a.inverse()`
    /// when the element type is a bounded `Int` range (e.g. `[Pos]?(0..Pos.count)`).
    local_specs: HashMap<Slot, ast::Type>,
    /// Declared integer range bounds for local variables, parameters, and range-typed arrays/lists.
    local_ranges: HashMap<Slot, RangeInfo>,
    /// Constant bounds of an immutable `let x = e` with no declared type, from `e`.
    let_bounds: HashMap<Slot, (i64, i64)>,
    view_lo: HashMap<Slot, i64>,
    view_rev: HashSet<Slot>,
    ret: Option<Ty>,
    new_domain_slots: Vec<DomainId>,
    /// `self`'s slot and struct inside a method.
    receiver: Option<(Slot, StructId)>,
    /// Inlined `self` place when desugaring a `lends` / `lends mut` projection.
    inline_self: Option<(Place, StructId)>,
    /// Hidden list accumulator when lowering a `-> yields T` generator.
    yield_slot: Option<(Slot, Ty)>,
    /// While probing what a generic generator yields: each `yield`'s type.
    yield_seen: Option<Vec<Ty>>,
    loops: Vec<(Option<String>, LoopId)>,
    /// The outermost loop a `break` out of a nested loop's innermost
    /// one leaves, for one source loop lowered as several.
    break_out: HashMap<LoopId, LoopId>,
    /// The factors `nested_for` is about to give the position it binds.
    loop_parts: Option<Vec<Expr>>,
    /// A loop's product position, by slot: the factors its nested loops walk,
    /// so `p.0` reads a loop variable instead of dividing.
    product_parts: HashMap<Slot, Vec<Expr>>,
    /// A loop with a trailing `else:` (decision 125): its break target's
    /// `Bool` flag, `true` unless a `break` targets that loop.
    loop_else: HashMap<LoopId, Slot>,
    dos: Vec<DoFrame>,
    /// The loop a `?` leaves when it is inside the left side of `??`
    /// instead of leaving the fn.
    try_extent: Option<LoopId>,
    /// The statement right-hand side that is the extent of its `?`s.
    rhs_extent: Option<Span>,
    /// The `x ?? return` that is the whole right side of the statement
    /// being lowered, and the test it puts ahead of that statement.
    way_out: Option<Span>,
    /// The `match` or `if` the statement being lowered is the value of,
    /// when one of its arms leaves with `return`, `break` or `continue`.
    exits: Option<Span>,
    pending: Vec<Stmt>,
    /// What runs after the statement being lowered: a lend's tail (R05.6.3).
    after: Vec<Stmt>,
    is_main: bool,
    is_closure: bool,
    captures: Vec<Capture>,
    captured_names: HashMap<String, Slot>,
}

#[derive(Clone, Debug)]
pub(super) enum SortedViewKind {
    Bag(StructId),
    Set,
    Map,
}

#[derive(Clone, Debug)]
pub(super) struct MapView {
    pub base: ViewBase,
    pub func: ast::Expr,
    pub span: Span,
}

/// Where a map view keeps the sequence it maps: a local, or a global when the
/// view is a top-level `let` the file's fns read too.
#[derive(Clone, Copy, Debug)]
pub(super) enum ViewBase {
    Local(Slot),
    Global(GlobalId),
}

#[derive(Clone, Debug)]
pub(super) struct SortedView {
    pub base: Place,
    pub kind: SortedViewKind,
    pub key_ty: Ty,
    pub elem_ty: Ty,
    pub lo: Slot,
    pub lo_inc: Slot,
    pub hi: Slot,
    pub hi_inc: Slot,
    pub empty: Option<Slot>,
    pub rev: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ProductFactor {
    Domain(DomainId),
    Enum {
        enum_id: EnumId,
        len: i64,
    },
    Bool,
    /// An `impl Index` type: its values are the factor, through `ord` and `nth`.
    Library {
        sid: StructId,
        dom: DomainId,
        ord: FnId,
        nth: FnId,
    },
}

impl ProductFactor {
    pub(super) fn ty(&self) -> Ty {
        match self {
            ProductFactor::Domain(d) => Ty::Pos(*d),
            ProductFactor::Enum { enum_id, .. } => Ty::Enum(*enum_id),
            ProductFactor::Bool => Ty::Bool,
            ProductFactor::Library { sid, .. } => Ty::Struct(*sid),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct SumEnumInfo {
    pub(super) enum_id: EnumId,
    pub(super) variants: Vec<(String, Vec<ProductFactor>)>,
}

/// The length of a library domain, from the domains it is built on.
enum LibrarySize {
    /// An `impl Index` instance: its `let count`, lowered with the type
    /// arguments bound, and the user's line that asked for the instance.
    Count(Expr, Span),
}

/// What a library domain's size reads: it is sized after a statement sets
/// one of these, once each that its block sets has been set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum SizeInput {
    Domain(DomainId),
    Local(Slot),
    Global(GlobalId),
}

struct Lowerer<'a> {
    mods: Vec<&'a Analysis>,
    path_to_mod: HashMap<String, ModId>,
    /// The module `std.mod`, when the program uses it.
    std_mod: Option<ModId>,
    /// The module `std.prefix`, when the program uses `prefix(...)`.
    mod_syms: Vec<HashMap<String, Sym>>,
    /// Each inline `mod`: the module it sits in and its body.
    inline_mods: HashMap<ModId, (ModId, &'a [ast::Stmt])>,
    cur_mod: ModId,
    diags: Vec<Diagnostic>,
    /// What is allowed but almost surely a mistake.
    warnings: Vec<Diagnostic>,
    /// How many `unchecked:` blocks the code being lowered is inside.
    unchecked: usize,
    /// The `with` settings in force, innermost last: the module that
    /// wrote it, the context, the field and its value. (R09.1.1)
    contexts: Vec<(ModId, String, String, ast::Expr)>,
    file_contexts_len: usize,
    /// Domains declared by a count alone, `domain E = [m]`: the ones
    /// `io.base` shifts. (R10.2.2)
    counted: HashSet<DomainId>,
    overflow_checks_off: usize,
    io_sep: Option<String>,
    io_precision: Option<u32>,
    io_base: Option<i64>,
    counted_domains: HashSet<DomainId>,
    range_keyed: HashSet<DomainId>,
    view_fns: HashMap<(FnId, Vec<(usize, Ty)>), FnId>,
    io_global: Option<GlobalId>,
    lexical_call_at: Option<Span>,
    context_defs: HashMap<String, (GlobalId, Vec<(String, Ty)>)>,
    /// Top-level `let m = xs.map(f)` views, by module and name.
    global_map_views: HashMap<(ModId, String), MapView>,
    /// Inside `g[r, ..]`: the loop variable that stands where the `..` is.
    whole_axes: Vec<Slot>,

    structs: Vec<StructInfo<'a>>,
    gen_structs: Vec<GenStructTemplate<'a>>,
    /// The empty struct each bare operator value has as its type (`op_value`).
    op_types: Vec<(BareOp, StructId)>,
    struct_insts: HashMap<(GenStructId, Vec<GenArg>), StructId>,
    /// `w.Idx` by the place of `w` and the struct's own `Idx`.
    value_domain_ids: HashMap<((bool, usize), DomainId), DomainId>,
    /// The domains made for arrays indexed by a plain range (`[0..n]T`,
    /// `[1..=n]T`) where a fn asks for a `[D]T`, with their first index.
    /// One per call and generic name; its size is set where the array is
    /// passed.
    range_domains: HashMap<DomainId, i64>,
    range_domain_sites: HashMap<(ModId, u32, String), DomainId>,
    /// Those one fn has for all its calls: a call puts the size back.
    range_domains_shared: HashSet<DomainId>,
    /// Inside the arguments of one call: the domains an argument before
    /// has sized. The next array given for the same `D` must be as long.
    call_sized: Option<HashSet<DomainId>>,
    /// Those that stand for the positions of a `List<T>` given where a
    /// `[D]T` is asked for: a `[D]T` that comes back is a list again.
    list_domains: HashSet<DomainId>,

    enums: Vec<EnumDef>,
    enum_discriminants: Vec<Vec<i64>>,
    enum_decls: Vec<(ModId, u32, &'a ast::EnumDecl)>,
    gen_enums: Vec<GenEnumTemplate<'a>>,
    enum_insts: HashMap<(GenEnumId, Vec<GenArg>), EnumId>,

    domains: Vec<String>,
    /// How each library domain is sized, by its `DomainId`: right after
    /// every definition of a domain its size reads.
    library_sizes: BTreeMap<DomainId, LibrarySize>,
    /// Factor definitions for product domains (`domain State = (V, K)`).
    pub(super) product_domains: HashMap<DomainId, Vec<(Option<String>, ProductFactor)>>,
    /// Finite-index payload enums lowered as dense `DomainId`s (§2.16).
    pub(super) sum_enums: HashMap<DomainId, SumEnumInfo>,
    pub(super) enum_to_sum_domain: HashMap<EnumId, DomainId>,
    /// Interned anonymous product domains for `(Row, Col)` / `(r: Row, c: Col)` generic args (§2.16).
    pub(super) anon_product_domains: HashMap<Vec<(Option<String>, ProductFactor)>, DomainId>,
    pub(super) anon_product_domain_set: HashSet<DomainId>,
    anon_range_domains: HashMap<(ModId, String), DomainId>,
    /// `domain S = (a: 0..=k, b: 0..2)`: each range factor's own domain, by where it is written.
    range_factors: HashMap<(ModId, u32, u32), DomainId>,
    aliases: Vec<(ModId, &'a ast::Type, u32)>,
    alias_stack: Vec<usize>,
    alias_shadows: HashMap<usize, Sym>,
    gen_aliases: Vec<GenAliasTemplate<'a>>,

    globals: Vec<Global>,
    mut_globals: HashSet<GlobalId>,
    global_view_lo: HashMap<GlobalId, i64>,
    global_view_rev: HashSet<GlobalId>,
    mut_slice_inits: HashSet<Span>,
    /// Annotated array `Spec`s for top-level globals (for `a.inverse()` on bounded-range elements).
    global_specs: HashMap<GlobalId, ast::Type>,
    /// The least and greatest value of each top-level `let` of a range
    /// with constant ends, and of each domain's length.
    global_bounds: HashMap<GlobalId, (i64, i64)>,
    domain_lens: HashMap<DomainId, (i64, i64)>,
    /// Domains whose ends are `Char`s: `Char(l)` turns a position back into its char.
    char_domains: HashSet<DomainId>,
    /// Declared integer range bounds for globals and struct fields.
    global_ranges: HashMap<GlobalId, RangeInfo>,
    /// Declared ranges of each top-level fn's parameters and result, for
    /// the types a `@grader` fn gives the grader.
    sig_ranges: HashMap<FnId, (Vec<Option<RangeInfo>>, Option<RangeInfo>)>,
    judge_tys: Vec<(FnId, Vec<JudgeTy>, JudgeTy)>,
    struct_field_ranges: HashMap<(StructId, usize), RangeInfo>,
    /// Declared 32-bit element storage layouts for ranged-`Int` arrays/grids/lists/tuples.
    narrow_elems: std::collections::BTreeMap<ElemLoc, NarrowLayout>,
    const_ints: HashMap<String, i64>,
    /// The `const`s that are a `Bool` known when compiling, for `if const`.
    const_bools: HashMap<String, bool>,
    /// What the fns of the file assign outside themselves: a call to one
    /// ends what a test showed about an optional it assigns.
    mutators: std::rc::Rc<narrow::Mutators>,
    /// Where a step along a domain stays inside it, for each module
    /// ([`cpl_syntax::order`]).
    steps: std::rc::Rc<Vec<cpl_syntax::order::Steps>>,
    mod_consts: Vec<ModConstInfo<'a>>,
    gen_consts: Vec<GenConstTemplate<'a>>,

    traits: Vec<TraitInfo<'a>>,
    impls: Vec<ImplInfo<'a>>,
    /// Methods an `impl` gives a built-in type (`impl Hash for Int`).
    prim_methods: Vec<(Ty, String, FnId)>,
    /// The `self` type of each of those methods.
    prim_self: HashMap<FnId, Ty>,

    sigs: Vec<FnSig<'a>>,
    bodies: Vec<Option<Function>>,
    gen_fns: Vec<GenFnTemplate<'a>>,
    /// Block-local generators lifted to fns of their own.
    lifted: HashMap<FnId, local_gen::Lifted>,
    fn_insts: HashMap<(GenFnId, Vec<GenArg>, Vec<String>, bool), FnId>,
    growth: instance::Growth,

    closures: Vec<ClosureDef>,
    queued_fns: Vec<FnId>,
    queued_set: HashSet<FnId>,
    /// Instance methods whose signature is read on first use, quietly.
    quiet_sigs: HashSet<FnId>,
    preamble_stmts: Vec<Stmt>,
    /// The top-level fns marked `@memo`.
    memo_fns: HashSet<FnId>,
    /// Each `@grader fn` over its own domains (`p: [C]Int`), with the
    /// entry the judge's grader calls in its place, over lists.
    grader_wrappers: Vec<(&'a ast::FnDecl, &'a ast::FnDecl)>,
    memo_count: usize,
    /// The lookup of the `@memo` wrapper being built, for a short fn of
    /// its own (see [`memo::MemoFast`]).
    memo_fast: Option<memo::MemoFast>,
    raw_lists: Vec<GlobalId>,
    /// The domains `[domain S]T` declared: each is sized by its array.
    own_domains: HashSet<DomainId>,
    /// The `ready` flag of each file-level cache: cleared as a test starts.
    memo_resets: Vec<Place>,
    /// The table of each `@memo(into: dp)` fn, by name.
    memo_into: HashMap<FnId, String>,
    main_body: Block,
    main_locals: Vec<Local>,
    /// What is above the `@!multitest` divider.
    setup_body: Block,
    multitest: bool,
    multitest_span: Span,
    until_eof: bool,
    interactive: bool,
    /// Lowering an `@interactor` as the program, not the solution.
    judge: bool,
    /// What a stress build adds: cold `@cache` runs, or sampled laws.
    stress: call::Stress,
    /// Lowering a `@alt(program)` or a `@check` as the program.
    aside: Option<Aside>,
    /// Top-level names set below the input header: the solution's own
    /// work, which a brute or a checker does not run.
    tail_names: HashSet<String>,
    /// The judge's side under `@!multitest`: it tells the solution how
    /// many test cases there are.
    announce_tests: bool,

    cx: FnCx,
    open_impl_fns: HashMap<(StructId, usize, String, Vec<GenArg>), FnId>,
    outer_cxs: Vec<FnCx>,
    gen_env: GenEnv,
    /// The type wanted of the generic call being lowered, which fills a
    /// type parameter only its return type names: `let x: Int = gen(r)`.
    call_hint: (Option<Ty>, Option<lambda_infer::WantBound>),
    self_struct: Option<StructId>,
    /// The trait fns a law being lowered names unqualified (`LawInfo::trait_fns`).
    law_fns: Vec<String>,
    /// A trait default copied into an impl: the trait's fns, which its body calls unqualified.
    default_trait_fns: HashMap<FnId, Vec<String>>,
    /// Start of the item being lowered; its spans are relative to it.
    offset: u32,
    /// Declared types shown as their source writes them, innermost last.
    written: Vec<(Ty, String)>,
    /// The user's expression being lowered: where an error inside the
    /// library code it asks for is shown.
    asked_at: report::Asked,
    /// The parameter type each signature is resolving, for E0209.
    sig_param_at: HashMap<FnId, Span>,
    /// The messages of the errors found inside library code.
    in_library: HashSet<String>,
    next_loop: LoopId,
    /// The places a `(place ??= init)` filled: module and span.
    filled: HashSet<(ModId, u32, u32)>,
    /// The structs `counts(xs)` makes, by the type counted, and back.
    counter_structs: HashMap<Ty, StructId>,
    counter_key: HashMap<StructId, Ty>,
    later_calls: Option<later_calls::LaterCalls>,
}

impl<'a> Lowerer<'a> {
    fn new(_analysis: &'a Analysis) -> Lowerer<'a> {
        Lowerer {
            mods: Vec::new(),
            path_to_mod: HashMap::new(),
            std_mod: None,
            mod_syms: Vec::new(),
            inline_mods: HashMap::new(),
            cur_mod: 0,
            diags: Vec::new(),
            warnings: Vec::new(),
            structs: Vec::new(),
            gen_structs: Vec::new(),
            op_types: Vec::new(),
            struct_insts: HashMap::new(),
            value_domain_ids: HashMap::new(),
            range_domains: HashMap::new(),
            range_domain_sites: HashMap::new(),
            range_domains_shared: HashSet::new(),
            call_sized: None,
            list_domains: HashSet::new(),
            enums: Vec::new(),
            enum_discriminants: Vec::new(),
            enum_decls: Vec::new(),
            gen_enums: Vec::new(),
            enum_insts: HashMap::new(),
            domains: Vec::new(),
            library_sizes: BTreeMap::new(),
            product_domains: HashMap::new(),
            sum_enums: HashMap::new(),
            enum_to_sum_domain: HashMap::new(),
            anon_product_domains: HashMap::new(),
            anon_product_domain_set: HashSet::new(),
            anon_range_domains: HashMap::new(),
            range_factors: HashMap::new(),
            aliases: Vec::new(),
            alias_stack: Vec::new(),
            alias_shadows: HashMap::new(),
            gen_aliases: Vec::new(),
            globals: Vec::new(),
            mut_globals: HashSet::new(),
            global_view_lo: HashMap::new(),
            global_view_rev: HashSet::new(),
            mut_slice_inits: HashSet::new(),
            global_specs: HashMap::new(),
            global_bounds: HashMap::new(),
            domain_lens: HashMap::new(),
            char_domains: HashSet::new(),
            global_ranges: HashMap::new(),
            sig_ranges: HashMap::new(),
            judge_tys: Vec::new(),
            struct_field_ranges: HashMap::new(),
            narrow_elems: std::collections::BTreeMap::new(),
            const_ints: HashMap::new(),
            const_bools: HashMap::new(),
            mutators: Default::default(),
            steps: Default::default(),
            mod_consts: Vec::new(),
            gen_consts: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            prim_methods: Vec::new(),
            prim_self: HashMap::new(),
            sigs: Vec::new(),
            bodies: Vec::new(),
            gen_fns: Vec::new(),
            lifted: HashMap::new(),
            fn_insts: HashMap::new(),
            growth: instance::Growth::default(),
            closures: Vec::new(),
            queued_fns: Vec::new(),
            queued_set: HashSet::new(),
            quiet_sigs: HashSet::new(),
            preamble_stmts: Vec::new(),
            memo_fns: HashSet::new(),
            grader_wrappers: Vec::new(),
            memo_count: 0,
            memo_fast: None,
            raw_lists: Vec::new(),
            unchecked: 0,
            contexts: Vec::new(),
            file_contexts_len: 0,
            counted: HashSet::new(),
            overflow_checks_off: 0,
            io_sep: None,
            io_precision: None,
            io_base: None,
            counted_domains: HashSet::new(),
            range_keyed: HashSet::new(),
            view_fns: HashMap::new(),
            io_global: None,
            lexical_call_at: None,
            context_defs: HashMap::new(),
            global_map_views: HashMap::new(),
            whole_axes: Vec::new(),
            own_domains: HashSet::new(),
            memo_resets: Vec::new(),
            memo_into: HashMap::new(),
            main_body: Block::default(),
            main_locals: Vec::new(),
            setup_body: Block::default(),
            multitest: false,
            multitest_span: Span::default(),
            until_eof: false,
            interactive: false,
            judge: false,
            stress: call::Stress::Off,
            aside: None,
            tail_names: HashSet::new(),
            written: Vec::new(),
            announce_tests: false,
            cx: FnCx::default(),
            open_impl_fns: HashMap::new(),
            outer_cxs: Vec::new(),
            gen_env: HashMap::new(),
            call_hint: (None, None),
            self_struct: None,
            law_fns: Vec::new(),
            default_trait_fns: HashMap::new(),
            offset: 0,
            asked_at: report::Asked::default(),
            sig_param_at: HashMap::new(),
            in_library: HashSet::new(),
            next_loop: 0,
            filled: HashSet::new(),
            counter_structs: HashMap::new(),
            counter_key: HashMap::new(),
            later_calls: None,
        }
    }

    // ----- reporting -----

    /// The checker accepted every name, so a name lowering does not know
    /// after an earlier lowering error lost its binding to that error.
    fn reported_an_error(&self) -> bool {
        self.diags
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error)
    }

    fn help(&mut self, help: impl Into<String>) {
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help.into());
        }
    }

    /// Reports something the language has but the back ends don't yet.
    fn unsupported(&mut self, span: Span, what: &str) {
        self.report(span, msgs::ir::UNSUPPORTED_CPL_RUN_AND_CPL, &[&what]);
    }

    fn mismatch(&mut self, span: Span, want: &Ty, found: &Ty) {
        if want.mentions_error() || found.mentions_error() {
            return;
        }
        let said = crate::show::mismatch(&self.show(want), &self.show(found), want, found);
        self.type_mismatch(span, want, found, said);
    }

    /// Each domain by the name it is written with: one made for a `D`
    /// of a generic fn is `D`, not the `D_0` it is in the C++.
    pub(super) fn domain_said(&self, d: DomainId) -> String {
        let name = &self.domains[d];
        match name.rsplit_once('_') {
            Some((written, _)) if self.range_domains.contains_key(&d) => written.to_string(),
            _ => name.clone(),
        }
    }

    fn err_expr(&self, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Error,
            ty: Ty::Error,
            span: self.span(span),
        }
    }

    // ----- declarations across all modules -----

    fn collect_all(&mut self) {
        let num_mods = self.mods.len();
        // Pass 1: collect declarations in every module.
        for m in 0..num_mods {
            self.cur_mod = m;
            let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(m).collect();
            for (offset, stmt) in items {
                self.offset = offset;
                if m == 0 {
                    for decorator in &stmt.decorators {
                        match decorator.name.text.as_str() {
                            // The divider: `main` splits the file at it.
                            "multitest" => {}
                            // Read once, before lowering: see `directed`.
                            "interactive" if decorator.directive => {}
                            "get" | "brute" | "alt" | "test" | "solution" | "tags" | "memo"
                            | "law" | "gen" | "checker" | "check" | "interactor" | "grader"
                            | "on_unimplemented" | "droppable" | "" => {}
                            // For `cpl lint` alone: they change nothing a run does.
                            "allow" | "lint" => {}
                            other => self
                                .unsupported(decorator.span, &format!("the `@{other}` decorator")),
                        }
                    }
                }
                self.collect_item(m, offset, stmt);
            }
        }

        // Pass 2: wire `use` imports and module aliases across all modules.
        // Run two rounds in case a module re-exports or chains imports.
        for _ in 0..2 {
            for m in 0..num_mods {
                let analysis = self.mods[m];
                let mods_list = analysis.modules();
                for (local_name, imp) in analysis.imports() {
                    if let Some(target_mod) =
                        self.path_to_mod.get(&mods_list[imp.module].path).copied()
                    {
                        if let Some(&sym) = self.mod_syms[target_mod].get(&imp.name) {
                            self.note_alias_shadow(m, local_name, sym);
                            self.mod_syms[m].entry(local_name.clone()).or_insert(sym);
                        }
                    }
                }
                for (alias, &mod_idx) in analysis.aliases() {
                    if let Some(target_mod) =
                        self.path_to_mod.get(&mods_list[mod_idx].path).copied()
                    {
                        self.mod_syms[m]
                            .entry(alias.clone())
                            .or_insert(Sym::Mod(target_mod));
                    }
                }
            }
        }
        // Pass 3: register top-level product domain factors so fns using `State` know them.
        for m in 0..num_mods {
            self.cur_mod = m;
            let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(m).collect();
            for (offset, item) in items {
                if let ast::StmtKind::Domain { name, def } = &item.kind {
                    if let Some(Sym::Domain(id)) = self.mod_syms[m].get(&name.text).copied() {
                        self.offset = offset;
                        if let Some(factors) = self.resolve_product_factors(def) {
                            self.product_domains.insert(id, factors);
                        }
                    }
                }
            }
        }
        self.cur_mod = 0;
    }

    // ----- types -----

    /// The source text of a span of the current item.
    fn source(&self, span: Span) -> String {
        let abs = span.shift(self.offset);
        self.mods[self.cur_mod]
            .text
            .get(abs.start as usize..abs.end as usize)
            .unwrap_or("")
            .to_string()
    }

    fn line(&self, span: Span) -> u32 {
        let abs = span.shift(self.offset);
        self.mods[self.cur_mod].lines.line_of(abs.start) + 1
    }

    // ----- scopes -----

    fn new_slot(&mut self, name: &str, ty: Ty) -> Slot {
        self.cx.locals.push(Local {
            name: name.to_string(),
            ty,
            binds: None,
        });
        self.cx.locals.len() - 1
    }

    /// A local the user didn't name.
    fn temp(&mut self, ty: Ty) -> Slot {
        self.new_slot("", ty)
    }

    fn local(&self, slot: Slot, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Local(slot),
            ty: self.cx.locals[slot].ty.clone(),
            span: self.span(span),
        }
    }

    /// Reads every module for the steps that stay inside their domain.
    fn read_steps(&self) -> Vec<cpl_syntax::order::Steps> {
        let mutators = self.mutators.clone();
        (0..self.mods.len())
            .filter(|m| !self.inline_mods.contains_key(m))
            .map(|m| {
                let mut steps = cpl_syntax::order::Steps::default();
                for (offset, stmt) in self.mod_items(m) {
                    steps.read(std::slice::from_ref(stmt), offset, &mut |f, x| {
                        mutators.writes(f, x)
                    });
                }
                steps
            })
            .collect()
    }

    /// The generator call `name` stands for, unless a local hides it.
    fn recipe(&self, name: &str) -> Option<ast::Expr> {
        let (depth, init) = self.cx.recipes.get(name)?;
        let local = self
            .cx
            .scopes
            .iter()
            .rposition(|s| s.contains_key(name))
            .map(|i| i + 1);
        (*depth <= self.cx.scopes.len() && local.is_none_or(|l| l < *depth)).then(|| init.clone())
    }

    fn lookup(&self, name: &str) -> Option<Slot> {
        self.cx
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
    }

    // ----- fns -----

    // ----- small builders -----

    fn int_lit(&self, value: i64, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Int(value),
            ty: Ty::Int,
            span: self.span(span),
        }
    }

    /// `V.at(x)`: the position labelled `x`, `None` outside `V` (R04.8.9).
    /// The label of `V`'s first position, even when `V` is empty.
    pub(super) fn domain_first_label(&mut self, id: DomainId, span: Span) -> Expr {
        let zero = self.int_lit(0, span);
        let first = self.mk(
            ExprKind::Cast(Cast::Trust, Box::new(zero)),
            Ty::Pos(id),
            span,
        );
        self.mk(
            ExprKind::Cast(Cast::Label(id), Box::new(first)),
            Ty::Int,
            span,
        )
    }

    fn domain_try_label(&mut self, id: DomainId, label: Expr, span: Span) -> Expr {
        let lo = self.domain_first_label(id, span);
        let ord = self.mk_binary(BinOp::Sub, label, lo, Ty::Int, span);
        self.mk_builtin(Builtin::PosTryOrd(id), [ord], Ty::opt(Ty::Pos(id)), span)
    }

    /// `a + b` on two `Int` expressions.
    fn int_sum(&self, a: Expr, b: Expr) -> Expr {
        let span = a.span;
        Expr {
            kind: ExprKind::Binary(BinOp::Add, Box::new(a), Box::new(b)),
            ty: Ty::Int,
            span,
        }
    }

    /// `e + by`, folded when `e` is a literal.
    fn int_add(&self, e: Expr, by: i64) -> Expr {
        if let ExprKind::Int(v) = e.kind {
            if let Some(sum) = v.checked_add(by) {
                return Expr {
                    kind: ExprKind::Int(sum),
                    ..e
                };
            }
        }
        let span = e.span;
        let rhs = Expr {
            kind: ExprKind::Int(by),
            ty: Ty::Int,
            span,
        };
        Expr {
            kind: ExprKind::Binary(BinOp::Add, Box::new(e), Box::new(rhs)),
            ty: Ty::Int,
            span,
        }
    }

    pub(super) fn ensure_io_global(&mut self) -> GlobalId {
        if let Some(gid) = self.io_global {
            return gid;
        }
        let ty = Ty::Record(vec![
            ("sep".into(), Ty::Str),
            ("base".into(), Ty::Int),
            ("precision".into(), Ty::Int),
            ("answers".into(), Ty::Tuple(vec![Ty::Str, Ty::Str])),
        ]);
        let gid = self.globals.len();
        self.globals.push(Global {
            name: "_ctx_io".to_string(),
            ty: ty.clone(),
            span: Span::new(0, 0),
        });
        self.mut_globals.insert(gid);
        let init = Expr {
            kind: ExprKind::Tuple(vec![
                Expr {
                    kind: ExprKind::Str(b" ".to_vec()),
                    ty: Ty::Str,
                    span: Span::new(0, 0),
                },
                Expr {
                    kind: ExprKind::Int(1),
                    ty: Ty::Int,
                    span: Span::new(0, 0),
                },
                Expr {
                    kind: ExprKind::Int(crate::FLOAT_DIGITS as i64),
                    ty: Ty::Int,
                    span: Span::new(0, 0),
                },
                Expr {
                    kind: ExprKind::Tuple(vec![
                        Expr {
                            kind: ExprKind::Str(b"YES".to_vec()),
                            ty: Ty::Str,
                            span: Span::new(0, 0),
                        },
                        Expr {
                            kind: ExprKind::Str(b"NO".to_vec()),
                            ty: Ty::Str,
                            span: Span::new(0, 0),
                        },
                    ]),
                    ty: Ty::Tuple(vec![Ty::Str, Ty::Str]),
                    span: Span::new(0, 0),
                },
            ]),
            ty,
            span: Span::new(0, 0),
        };
        self.preamble_stmts.push(Stmt::Assign(
            Place {
                kind: PlaceKind::Global(gid),
                ty: self.globals[gid].ty.clone(),
                span: Span::new(0, 0),
            },
            AssignOp::Set,
            init,
        ));
        self.io_global = Some(gid);
        gid
    }

    /// Inside `unchecked:`, a position made from an ordinal or a label is
    /// vouched for like a list index (decision 43.8): the C++ drops its
    /// `pos` check, `cpl run` keeps it.
    pub(super) fn vouch_pos(&self, kind: ExprKind, ty: &Ty, span: Span) -> ExprKind {
        match kind {
            ExprKind::Cast(Cast::FromOrd(_) | Cast::FromLabel(_), _) if self.unchecked > 0 => {
                ExprKind::Cast(
                    Cast::Raw,
                    Box::new(Expr {
                        kind,
                        ty: ty.clone(),
                        span,
                    }),
                )
            }
            kind => kind,
        }
    }

    fn mk(&self, kind: ExprKind, ty: Ty, span: Span) -> Expr {
        Expr {
            kind: self.vouch_pos(kind, &ty, self.span(span)),
            ty,
            span: self.span(span),
        }
    }

    // One out-of-line builder per boxed node: each boxing call site
    // otherwise carries its own allocation and copy.
    fn mk_binary(&self, op: BinOp, l: Expr, r: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span)
    }

    fn mk_cmp(&self, op: CmpOp, l: Expr, r: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Cmp(op, Box::new(l), Box::new(r)), ty, span)
    }

    fn mk_if(&self, c: Expr, t: Expr, e: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(
            ExprKind::If(Box::new(c), Box::new(t), Box::new(e)),
            ty,
            span,
        )
    }

    fn mk_is_some(&self, e: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::IsSome(Box::new(e)), ty, span)
    }

    fn mk_some(&self, e: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Some(Box::new(e)), ty, span)
    }

    fn mk_unwrap(&self, e: Expr, checked: bool, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Unwrap(Box::new(e), checked), ty, span)
    }

    fn mk_field(&self, e: Expr, idx: usize, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Field(Box::new(e), idx), ty, span)
    }

    fn mk_index(&self, e: Expr, at: Vec<Expr>, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Index(Box::new(e), at), ty, span)
    }

    fn mk_unary(&self, op: UnOp, e: Expr, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Unary(op, Box::new(e)), ty, span)
    }

    fn mk_mutate(&self, m: Mutation, place: Place, args: Vec<Expr>, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Mutate(m, Box::new(place), args), ty, span)
    }

    fn mk_call_expr(&self, f: Expr, args: Vec<Arg>, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::CallExpr(Box::new(f), args), ty, span)
    }

    fn mk_builtin<const N: usize>(&self, b: Builtin, args: [Expr; N], ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Builtin(b, args.into()), ty, span)
    }

    fn mk_read(&self, spec: Spec, ty: Ty, span: Span) -> Expr {
        self.mk(ExprKind::Read(Box::new(spec)), ty, span)
    }

    /// Whether a struct is a number: it has a `zero`, a `one` and a
    /// `new(x: Int)`, so an `Int` is made into it where it is wanted.
    /// `Mod<P>` is one, and so is any struct written the same way.
    pub(super) fn is_number(&mut self, sid: StructId) -> bool {
        let made_from_int = self.own_method(sid, "new").is_some_and(
            |new_fn| matches!(self.sigs[new_fn].params.as_slice(), [only] if only.ty == Ty::Int),
        );
        made_from_int
            && ["zero", "one"]
                .iter()
                .all(|c| self.structs[sid].consts.contains_key(*c))
    }

    /// Resolves a domain or type alias pointing to a domain (e.g. `type Group = Subset<Rabbit>`).
    pub(super) fn resolve_domain_name(&mut self, name: &str) -> Option<DomainId> {
        if let Some(GenArg::Domain(id)) = self.gen_env.get(name) {
            return Some(*id);
        }
        if let Some(Ty::Pos(id)) = self.cx.local_type_aliases.get(name) {
            return Some(*id);
        }
        match self.get_sym(name) {
            Some(Sym::Domain(id)) => Some(id),
            Some(Sym::Alias(aid)) => match self.in_alias(aid, |lw, ty| lw.resolve_ty(ty)) {
                Ty::Pos(id) => Some(id),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn fresh_loop(&mut self) -> LoopId {
        let id = self.next_loop;
        self.next_loop += 1;
        id
    }

    pub(super) fn push_synth_struct(
        &mut self,
        name: String,
        fields: Vec<(String, Ty)>,
    ) -> StructId {
        let id = self.structs.len();
        self.structs.push(StructInfo {
            def: StructDef {
                name,
                fields,
                ..Default::default()
            },
            decl: None,
            mod_id: 0,
            offset: 0,
            origin_gen: None,
            env: HashMap::new(),
            methods: HashMap::new(),
            trait_methods: HashMap::new(),
            op_methods: Vec::new(),
            gen_methods: HashMap::new(),
            open_impls: Vec::new(),
            getters: HashSet::new(),
            projections: HashMap::new(),
            consts: HashMap::new(),
            owned_domains: HashMap::new(),
            owned_domain_inits: Vec::new(),
            laws: Vec::new(),
            index_domain: None,
        });
        id
    }
}

/// Whether `a` and `b` differ only in naming the fields of tuples, so a
/// value of one is a value of the other, held the same way: `[n](V, V)`
/// is a `[n](u: V, v: V)`.
fn same_cells(a: &Ty, b: &Ty) -> bool {
    let fields = |ty: &Ty| -> Option<Vec<Ty>> {
        match ty {
            Ty::Tuple(items) => Some(items.clone()),
            Ty::Record(fields) => Some(fields.iter().map(|(_, ty)| ty.clone()).collect()),
            _ => None,
        }
    };
    match (a, b) {
        _ if a == b => true,
        (Ty::Tuple(_) | Ty::Record(_), Ty::Tuple(_)) | (Ty::Tuple(_), Ty::Record(_)) => {
            let (a, b) = (fields(a).unwrap_or_default(), fields(b).unwrap_or_default());
            a.len() == b.len() && a.iter().zip(&b).all(|(a, b)| same_cells(a, b))
        }
        (Ty::Opt(a), Ty::Opt(b)) | (Ty::List(a), Ty::List(b)) => same_cells(a, b),
        (Ty::Array { dims: ad, elem: ae }, Ty::Array { dims: bd, elem: be }) => {
            ad == bd && same_cells(ae, be)
        }
        _ => false,
    }
}

/// Whether `e` is a call of `name`: `name(...)` or `x.name(...)`.
fn calls_fn(e: &ast::Expr, name: &str) -> bool {
    let ast::ExprKind::Call { callee, .. } = &e.kind else {
        return false;
    };
    match &callee.kind {
        ast::ExprKind::Name(called) => called == name,
        ast::ExprKind::Field { name: called, .. } => called.text == name,
        _ => false,
    }
}

/// Whether a `return` under `block` gives a value. A nested fn's returns
/// are its own.
fn returns_a_value(block: &ast::Block) -> bool {
    struct Returns(bool);
    impl cpl_syntax::visit::Visitor for Returns {
        fn stmt(&mut self, stmt: &ast::Stmt) {
            if matches!(&stmt.kind, ast::StmtKind::Return(Some(_))) {
                self.0 = true;
            }
        }
    }
    let mut returns = Returns(false);
    for stmt in &block.stmts {
        if !matches!(stmt.kind, ast::StmtKind::Fn(_)) {
            cpl_syntax::visit::walk_stmt(stmt, &mut returns);
        }
    }
    returns.0
}

/// The type of a field written `total = 0`, from its default.
fn literal_ty(e: &ast::Expr) -> Ty {
    match &e.kind {
        ast::ExprKind::Int(_) => Ty::Int,
        ast::ExprKind::Float(_) => Ty::Float,
        ast::ExprKind::Bool(_) => Ty::Bool,
        ast::ExprKind::Char(_) => Ty::Char,
        ast::ExprKind::Str(_) => Ty::Str,
        ast::ExprKind::Unary { expr, .. } | ast::ExprKind::Paren(expr) => literal_ty(expr),
        _ => Ty::Error,
    }
}

fn implicit_type_names(ty: &ast::Type, out: &mut Vec<String>) {
    match &ty.kind {
        ast::TypeKind::Named { name, args } => {
            if args.is_empty()
                && name.text.len() == 1
                && name.text.as_bytes()[0].is_ascii_uppercase()
                && !out.contains(&name.text)
            {
                out.push(name.text.clone());
            }
            args.iter().for_each(|a| implicit_type_names(a, out));
        }
        ast::TypeKind::Array { index, elem } => {
            index.iter().for_each(|a| implicit_type_names(a, out));
            implicit_type_names(elem, out);
        }
        ast::TypeKind::OwnDomain { elem: t, .. }
        | ast::TypeKind::Optional(t)
        | ast::TypeKind::Mut(t)
        | ast::TypeKind::Yields(t) => implicit_type_names(t, out),
        ast::TypeKind::Tuple(ts) => ts.iter().for_each(|a| implicit_type_names(a, out)),
        ast::TypeKind::Record(fs) | ast::TypeKind::Row(fs) => {
            fs.iter().for_each(|(_, a)| implicit_type_names(a, out))
        }
        ast::TypeKind::Fn { params, ret } => {
            params.iter().for_each(|a| implicit_type_names(a, out));
            implicit_type_names(ret, out);
        }
        _ => {}
    }
}

fn type_has_hole(ty: &ast::Type) -> bool {
    match &ty.kind {
        ast::TypeKind::Value(e) => matches!(&e.kind, ast::ExprKind::Name(n) if n == "_"),
        ast::TypeKind::Named { args, .. } => args.iter().any(type_has_hole),
        ast::TypeKind::Optional(t) | ast::TypeKind::Mut(t) => type_has_hole(t),
        ast::TypeKind::Tuple(ts) => ts.iter().any(type_has_hole),
        ast::TypeKind::Record(fs) => fs.iter().any(|(_, t)| type_has_hole(t)),
        _ => false,
    }
}
