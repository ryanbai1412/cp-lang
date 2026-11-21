//! From the front end's syntax trees to a typed [`Program`].
//!
//! The front end's checker is deliberately forgiving: what it can't type is
//! `Unknown` and passes. A back end can't run `Unknown`, so this pass types
//! every expression for real, loads imported modules, monomorphizes generic
//! structs/enums/fns/consts, lowers traits and `impl`s, desugars `lends`
//! projections and generators, and emits one concrete [`Program`] shared by
//! `cpl-interp` and `cpl-codegen`.

mod cache;
mod call;
mod expr;
mod memo;
mod mono;
mod range;
mod stmt;

use crate::*;
use cpl_check::Analysis;
use cpl_diag::{codes, Diagnostic, Span};
use cpl_syntax::ast;
use cpl_syntax::narrow;
use mono::*;
use range::*;
use std::collections::{HashMap, HashSet};

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

/// Lowers a single `@test` function of `analysis` as the program's entrypoint.
pub fn lower_test(analysis: &Analysis, test_name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_impl(analysis, Some(test_name))
}

/// The `@interactor` fn a file declares at its top level, if any.
/// True when the program's own file has `@!interactive`.
fn directed(analysis: &Analysis) -> bool {
    let stmts = analysis.items.iter().flat_map(|item| &item.syntax.parsed.stmts);
    stmts.flat_map(|stmt| &stmt.decorators).any(|d| d.directive && d.name.text == "interactive")
}

pub fn interactor(analysis: &Analysis) -> Option<String> {
    analysis
        .items
        .iter()
        .flat_map(|item| &item.syntax.parsed.stmts)
        .find_map(|stmt| match &stmt.kind {
            ast::StmtKind::Fn(decl) if stmt.decorators.iter().any(|d| d.name.text == "interactor") => {
                Some(decl.name.text.clone())
            }
            _ => None,
        })
}

/// The fn a file decorates with `@name` at its top level, if any, and
/// the 1-based line of its name.
fn decorated(analysis: &Analysis, wanted: fn(&ast::Decorator) -> bool) -> Option<(String, u32)> {
    analysis.items.iter().find_map(|item| {
        item.syntax.parsed.stmts.iter().find_map(|stmt| match &stmt.kind {
            ast::StmtKind::Fn(decl) if stmt.decorators.iter().any(wanted) => {
                let at = decl.name.span.shift(item.offset).start;
                let line = analysis.lines.line_of(at) + 1;
                Some((decl.name.text.clone(), line))
            }
            _ => None,
        })
    })
}

/// `@brute(main)`, or a bare `@brute`: a slow program to compare the
/// whole file with. `@brute(part)` pairs one fn with another and is not
/// one of these.
fn is_whole_brute(decorator: &ast::Decorator) -> bool {
    decorator.name.text == "brute"
        && match decorator.args.as_slice() {
            [] => true,
            [arg] => matches!(&arg.value.kind, ast::ExprKind::Name(name) if name == "main"),
            _ => false,
        }
}

/// The `@brute(main)` fn a file declares, and the line it is on.
pub fn brute(analysis: &Analysis) -> Option<(String, u32)> {
    decorated(analysis, is_whole_brute)
}

/// The `@checker` fn a file declares, and the line it is on.
pub fn checker(analysis: &Analysis) -> Option<(String, u32)> {
    decorated(analysis, |d| d.name.text == "checker")
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

/// Lowers the `@brute(main)` fn `name` as a program: the file's input
/// header, then the fn. A brute that calls `read()` itself reads the
/// whole input, and none of the header runs.
pub fn lower_brute(analysis: &Analysis, name: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_aside(analysis, name, Aside::Brute)
}

/// Lowers the `@checker` fn `name` as a program: the file's input
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
    lw.mutators = narrow::Mutators::of(lw.mod_items(0).map(|(_, stmt)| stmt)).into();
    lw.steps = lw.read_steps().into();
    lw.collect_all(Some(name));
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
    let mut lw = Lowerer::new(analysis);
    lw.judge = judge;
    // Whatever the judge sends goes out at once. A solution with a judge
    // in its file talks to it, so it reads as it goes and never all the
    // input at once.
    // `@!interactive` asks for the same without a judge in the file.
    lw.interactive = judge || (test_name.is_none() && interactor(analysis).is_some()) || directed(analysis);
    lw.load_modules(analysis);
    lw.mutators = narrow::Mutators::of(lw.mod_items(0).map(|(_, stmt)| stmt)).into();
    lw.steps = lw.read_steps().into();
    lw.collect_all(test_name);
    lw.signatures();
    if let Some(tname) = test_name {
        lw.main_for_test(tname);
    } else {
        lw.main();
    }
    lw.finish(analysis, test_name.is_none())
}

impl Lowerer<'_> {
/// Lowers every fn the program reaches and gathers the [`Program`].
/// `whole` is a run of the file itself: every fn of the file is lowered,
/// called or not, so each one's errors are reported.
fn finish(self, analysis: &Analysis, whole: bool) -> Result<Program, Vec<Diagnostic>> {
    let mut lw = self;
    if whole {
        for id in 0..lw.sigs.len() {
            // A fn with no body is the judge's: there is nothing to lower
            // until something calls it.
            let bodyless = matches!(lw.sigs[id].decl.body, ast::FnBody::None);
            if lw.sigs[id].mod_id == 0 && lw.sigs[id].owner.is_none() && !bodyless {
                lw.queue_fn(id);
            }
        }
    }
    loop {
        while let Some(id) = lw.queued_fns.pop() {
            lw.function(id);
        }
        let mut newly_called = Vec::new();
        collect_block_calls(&lw.main_body, &mut newly_called);
        collect_block_calls(&lw.setup_body, &mut newly_called);
        for s in &lw.preamble_stmts {
            collect_stmt_calls(s, &mut newly_called);
        }
        for c in &lw.closures {
            collect_block_calls(&c.body, &mut newly_called);
        }
        for body in lw.bodies.iter().flatten() {
            collect_block_calls(&body.body, &mut newly_called);
        }
        for s in &lw.structs {
            for hook in [s.def.eq_fn, s.def.lt_fn, s.def.show_fn].into_iter().flatten() {
                newly_called.push(hook);
            }
        }
        let mut added = false;
        for fid in newly_called {
            if lw.queued_set.insert(fid) {
                lw.queued_fns.push(fid);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    // A `@memo` table starts unset, and anew in each test case
    // (decision 20.2).
    if !lw.memo_resets.is_empty() {
        let unset: Vec<Stmt> = lw
            .memo_resets
            .iter()
            .map(|ready| {
                let no = lw.mk(ExprKind::Bool(false), Ty::Bool, Span::default());
                Stmt::Assign(ready.clone(), AssignOp::Set, no)
            })
            .collect();
        if lw.multitest {
            lw.setup_body.stmts.splice(0..0, unset.iter().cloned());
        }
        lw.main_body.stmts.splice(0..0, unset);
    }
    // The initializers of imported modules' consts run first, once.
    if !lw.preamble_stmts.is_empty() {
        let preamble = std::mem::take(&mut lw.preamble_stmts);
        let first = if lw.multitest {
            &mut lw.setup_body
        } else {
            &mut lw.main_body
        };
        first.stmts.splice(0..0, preamble);
    }
    // A mistake in the user's code often breaks the library code it asks
    // for as well. The user's own mistake is the one to show.
    if lw.diags.iter().any(|d| !lw.in_library.contains(&d.message)) {
        let in_library = std::mem::take(&mut lw.in_library);
        lw.diags.retain(|d| !in_library.contains(&d.message));
    }
    if !lw.diags.is_empty() {
        lw.diags.sort_by_key(|d| (d.span.start, d.span.end));
        lw.diags.dedup();
        return Err(lw.diags);
    }
    let (judge_fns, entry_fns) = lw.grader_fns(analysis);
    if !lw.diags.is_empty() {
        return Err(lw.diags);
    }
    // Ensure any unlowered non-generic fns from imported modules (unused) don't panic:
    // keep only lowered fns or fill unused bodies with empty blocks, and deduplicate names.
    let mut used_fn_names: HashSet<String> = HashSet::new();
    let library_fns: Vec<bool> = lw.sigs.iter().take(lw.bodies.len()).map(|sig| sig.mod_id != 0).collect();
    let fns = lw
        .bodies
        .into_iter()
        .enumerate()
        .map(|(i, body)| {
            let mut f = body.unwrap_or_else(|| Function {
                name: lw.sigs[i].name.clone(),
                params: Vec::new(),
                ret: Ty::Unit,
                locals: Vec::new(),
                body: Block::default(),
            });
            if !used_fn_names.insert(f.name.clone()) {
                f.name = format!("{}_{i}", f.name);
                used_fn_names.insert(f.name.clone());
            }
            f
        })
        .collect();
    lw.warnings.sort_by_key(|d| (d.span.start, d.span.end));
    lw.warnings.dedup();
    Ok(Program {
        warnings: lw.warnings,
        unordered_maps: Vec::new(),
        unordered_sets: Vec::new(),
        structs: lw.structs.iter().map(|s| s.def.clone()).collect(),
        enums: lw.enums,
        domains: lw.domains,
        subsets: lw.subsets,
        globals: lw.globals,
        fns,
        library_fns,
        closures: lw.closures,
        main: lw.main_body,
        main_locals: lw.main_locals,
        setup: lw.setup_body,
        multitest: lw.multitest,
        until_eof: lw.until_eof,
        interactive: lw.interactive,
        announce_tests: lw.announce_tests,
        judge_fns,
        entry_fns,
        raw_lists: lw.raw_lists,
    })
}
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
    op_methods: Vec<(String, Ty, FnId)>,
    gen_methods: HashMap<String, GenFnId>,
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
    state: State,
    /// The user's expression that first asked for this fn.
    asked_at: Span,
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
    /// The optionals a test has shown to hold a value here, by name.
    present: Vec<String>,
    /// The name about to be lowered stands where an optional is asked for.
    as_written: bool,
    scopes: Vec<HashMap<String, Slot>>,
    /// Block-scoped type aliases, local domains, etc. (e.g. `domain V = 1..=6`, `type M = Mod<7>`).
    local_syms: Vec<HashMap<String, Sym>>,
    /// Parameters bound to compile-time `BareOp` values (`+`, `min`, `max`, `gcd`, etc.).
    op_locals: HashMap<String, BareOp>,
    /// Named mutable aliases (`let st = mut a[i]`): resolved to their underlying `Place`.
    mut_aliases: HashMap<String, Place>,
    /// Local zero-copy range views over `SortedBag<K>` / `SortedSet<K>`.
    sorted_views: HashMap<Slot, SortedView>,
    /// Block-scoped `type Alias = ...` resolved to concrete types (e.g. `type Cell = (Row, Col)`).
    local_type_aliases: HashMap<String, Ty>,
    /// Local slots declared `var` or mutated, so closures capture them by reference.
    mut_slots: HashSet<Slot>,
    /// Annotated array `Spec`s for local variables/parameters, used by `a.inverse()`
    /// when the element type is a bounded `Int` range (e.g. `[Pos]?(0..Pos.count)`).
    local_specs: HashMap<Slot, ast::Type>,
    /// Declared integer range bounds for local variables, parameters, and range-typed arrays/lists.
    local_ranges: HashMap<Slot, RangeInfo>,
    ret: Option<Ty>,
    new_domain_slots: Vec<DomainId>,
    /// `self`'s slot and struct inside a method.
    receiver: Option<(Slot, StructId)>,
    /// Inlined `self` place when desugaring a `lends` / `lends mut` projection.
    inline_self: Option<(Place, StructId)>,
    /// Hidden list accumulator when lowering a `-> yields T` generator.
    yield_slot: Option<(Slot, Ty)>,
    loops: Vec<(Option<String>, LoopId)>,
    dos: Vec<DoFrame>,
    /// The `x ?? return` that is the whole right side of the statement
    /// being lowered, and the test it puts ahead of that statement.
    way_out: Option<Span>,
    /// The `match` or `if` the statement being lowered is the value of,
    /// when one of its arms leaves with `return`, `break` or `continue`.
    exits: Option<Span>,
    pending: Vec<Stmt>,
    is_main: bool,
    is_closure: bool,
    captures: Vec<Capture>,
    captured_names: HashMap<String, Slot>,
}

#[derive(Clone, Debug)]
pub(super) struct SortedView {
    pub base: Place,
    pub sid: StructId,
    pub key_ty: Ty,
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
    Enum { enum_id: EnumId, len: i64 },
    Bool,
}

#[derive(Clone, Debug)]
pub(super) struct SumEnumInfo {
    pub(super) enum_id: EnumId,
    pub(super) variants: Vec<(String, Vec<ProductFactor>)>,
}

struct Lowerer<'a> {
    mods: Vec<&'a Analysis>,
    path_to_mod: HashMap<String, ModId>,
    /// The module `std.mod`, when the program uses it.
    std_mod: Option<ModId>,
    /// The module `std.prefix`, when the program uses `prefix(...)`.
    mod_syms: Vec<HashMap<String, Sym>>,
    cur_mod: ModId,
    diags: Vec<Diagnostic>,
    /// What is allowed but almost surely a mistake.
    warnings: Vec<Diagnostic>,
    /// How many `unchecked:` blocks the code being lowered is inside.
    unchecked: usize,
    /// Inside `g[r, ..]`: the loop variable that stands where the `..` is.
    whole_axis: Option<Slot>,

    structs: Vec<StructInfo<'a>>,
    gen_structs: Vec<GenStructTemplate<'a>>,
    struct_insts: HashMap<(GenStructId, Vec<GenArg>), StructId>,
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
    /// `Subset<D>` domains: `(subset, base)`.
    subsets: Vec<(DomainId, DomainId)>,
    /// Factor definitions for product domains (`domain State = (V, K)`).
    pub(super) product_domains: HashMap<DomainId, Vec<(Option<String>, ProductFactor)>>,
    /// Finite-index payload enums lowered as dense `DomainId`s (§2.16).
    pub(super) sum_enums: HashMap<DomainId, SumEnumInfo>,
    pub(super) enum_to_sum_domain: HashMap<EnumId, DomainId>,
    /// Interned anonymous product domains for `(Row, Col)` / `(r: Row, c: Col)` generic args (§2.16).
    pub(super) anon_product_domains: HashMap<Vec<(Option<String>, ProductFactor)>, DomainId>,
    pub(super) anon_product_domain_set: HashSet<DomainId>,
    aliases: Vec<(ModId, &'a ast::Type, u32)>,
    gen_aliases: Vec<GenAliasTemplate<'a>>,

    globals: Vec<Global>,
    /// Annotated array `Spec`s for top-level globals (for `a.inverse()` on bounded-range elements).
    global_specs: HashMap<GlobalId, ast::Type>,
    /// The least and greatest value of each top-level `let` of a range
    /// with constant ends, and of each domain's length.
    global_bounds: HashMap<GlobalId, (i64, i64)>,
    domain_lens: HashMap<DomainId, (i64, i64)>,
    /// Declared integer range bounds for globals and struct fields.
    global_ranges: HashMap<GlobalId, RangeInfo>,
    struct_field_ranges: HashMap<(StructId, usize), RangeInfo>,
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
    fn_insts: HashMap<(GenFnId, Vec<GenArg>), FnId>,

    closures: Vec<ClosureDef>,
    queued_fns: Vec<FnId>,
    queued_set: HashSet<FnId>,
    preamble_stmts: Vec<Stmt>,
    /// The top-level fns marked `@memo`.
    memo_fns: HashSet<FnId>,
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
    until_eof: bool,
    interactive: bool,
    /// Lowering an `@interactor` as the program, not the solution.
    judge: bool,
    /// Lowering a `@brute(main)` or a `@checker` as the program.
    aside: Option<Aside>,
    /// Top-level names set below the input header: the solution's own
    /// work, which a brute or a checker does not run.
    tail_names: HashSet<String>,
    /// The judge's side under `@!multitest`: it tells the solution how
    /// many test cases there are.
    announce_tests: bool,

    cx: FnCx,
    outer_cxs: Vec<FnCx>,
    gen_env: GenEnv,
    /// The type wanted of the generic call being lowered, which fills a
    /// type parameter only its return type names: `let x: Int = gen(r)`.
    call_hint: Option<Ty>,
    self_struct: Option<StructId>,
    /// Start of the item being lowered; its spans are relative to it.
    offset: u32,
    /// The user's expression being lowered: where an error inside the
    /// library code it asks for is shown.
    asked_at: Span,
    /// The messages of the errors found inside library code.
    in_library: HashSet<String>,
    next_loop: LoopId,
    /// The places a `(place ??= init)` filled: module and span.
    filled: HashSet<(ModId, u32, u32)>,
    /// The structs `counts(xs)` makes, by the type counted, and back.
    counter_structs: HashMap<Ty, StructId>,
    counter_key: HashMap<StructId, Ty>,
}

impl<'a> Lowerer<'a> {
    fn new(_analysis: &'a Analysis) -> Lowerer<'a> {
        Lowerer {
            mods: Vec::new(),
            path_to_mod: HashMap::new(),
            std_mod: None,
            mod_syms: Vec::new(),
            cur_mod: 0,
            diags: Vec::new(),
            warnings: Vec::new(),
            structs: Vec::new(),
            gen_structs: Vec::new(),
            struct_insts: HashMap::new(),
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
            subsets: Vec::new(),
            product_domains: HashMap::new(),
            sum_enums: HashMap::new(),
            enum_to_sum_domain: HashMap::new(),
            anon_product_domains: HashMap::new(),
            anon_product_domain_set: HashSet::new(),
            aliases: Vec::new(),
            gen_aliases: Vec::new(),
            globals: Vec::new(),
            global_specs: HashMap::new(),
            global_bounds: HashMap::new(),
            domain_lens: HashMap::new(),
            global_ranges: HashMap::new(),
            struct_field_ranges: HashMap::new(),
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
            fn_insts: HashMap::new(),
            closures: Vec::new(),
            queued_fns: Vec::new(),
            queued_set: HashSet::new(),
            preamble_stmts: Vec::new(),
            memo_fns: HashSet::new(),
            memo_count: 0,
            memo_fast: None,
            raw_lists: Vec::new(),
            unchecked: 0,
            whole_axis: None,
            own_domains: HashSet::new(),
            memo_resets: Vec::new(),
            memo_into: HashMap::new(),
            main_body: Block::default(),
            main_locals: Vec::new(),
            setup_body: Block::default(),
            multitest: false,
            until_eof: false,
            interactive: false,
            judge: false,
            aside: None,
            tail_names: HashSet::new(),
            announce_tests: false,
            cx: FnCx::default(),
            outer_cxs: Vec::new(),
            gen_env: HashMap::new(),
            call_hint: None,
            self_struct: None,
            offset: 0,
            asked_at: Span::default(),
            in_library: HashSet::new(),
            next_loop: 0,
            filled: HashSet::new(),
            counter_structs: HashMap::new(),
            counter_key: HashMap::new(),
        }
    }

    // ----- reporting -----

    /// Where an error at `span` is shown. One inside library code is
    /// shown at the user's expression that asked for that code, since the
    /// file on screen is the user's.
    fn span(&self, span: Span) -> Span {
        if self.cur_mod == 0 {
            span.shift(self.offset)
        } else {
            self.asked_at
        }
    }

    fn error(&mut self, code: cpl_diag::Code, span: Span, message: impl Into<String>) {
        let span = self.span(span);
        let message = message.into();
        if self.cur_mod != 0 {
            self.in_library.insert(message.clone());
        }
        self.diags.push(Diagnostic::error(code, span, message));
    }

    /// The fns that cross to the judge's grader: those it supplies (no
    /// body) and those it calls (`@grader fn` with a body), each checked
    /// to take and give only what a grader can.
    fn grader_fns(&mut self, analysis: &'a Analysis) -> (Vec<FnId>, Vec<FnId>) {
        let entries: Vec<&ast::FnDecl> = analysis
            .items
            .iter()
            .flat_map(|item| &item.syntax.parsed.stmts)
            .filter_map(|stmt| match &stmt.kind {
                ast::StmtKind::Fn(decl)
                    if stmt.decorators.iter().any(|d| d.name.text == "grader") =>
                {
                    Some(decl)
                }
                _ => None,
            })
            .collect();
        let (mut judge_fns, mut entry_fns) = (Vec::new(), Vec::new());
        for id in 0..self.sigs.len() {
            let Some(f) = &self.bodies[id] else { continue };
            let decl = self.sigs[id].decl;
            let top_level = self.sigs[id].mod_id == 0 && self.sigs[id].owner.is_none();
            if matches!(decl.body, ast::FnBody::None) {
                judge_fns.push(id);
            } else if top_level && entries.iter().any(|entry| std::ptr::eq(*entry, decl)) {
                entry_fns.push(id);
            } else {
                continue;
            }
            let name = &decl.name.text;
            let changed = f.params.iter().position(|p| p.mutable);
            let odd = (f
                .locals
                .iter()
                .take(f.params.len())
                .map(|l| &l.ty)
                .chain([&f.ret]))
            .find(|ty| !ty.crosses_to_judge())
            .cloned();
            let saved = (self.cur_mod, self.offset);
            (self.cur_mod, self.offset) = (self.sigs[id].mod_id, self.sigs[id].offset);
            if let Some(at) = changed {
                let param = decl.params[at].name.text.clone();
                self.error(
                    codes::UNSUPPORTED,
                    decl.params[at].name.span,
                    format!("the judge's grader can't lend `{param}` to `{name}` to change"),
                );
                self.help(format!("take `{param}` by value and return what changed"));
            } else if let Some(ty) = odd {
                let shown = self.show(&ty);
                self.error(
                    codes::UNSUPPORTED,
                    decl.name.span,
                    format!("a `{shown}` can't cross to the judge's grader in `{name}`"),
                );
                self.help("a `@grader fn` takes and gives `Int`, `F64`, `Bool`, `Char`, `Str`, and lists and tuples of them");
            }
            (self.cur_mod, self.offset) = saved;
        }
        (judge_fns, entry_fns)
    }

    fn help(&mut self, help: impl Into<String>) {
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help.into());
        }
    }

    /// Reports something the language has but the back ends don't yet.
    fn unsupported(&mut self, span: Span, what: &str) {
        self.error(
            codes::UNSUPPORTED,
            span,
            format!("`cpl run` and `cpl build` can't handle {what} yet"),
        );
    }

    fn mismatch(&mut self, span: Span, want: &Ty, found: &Ty) {
        let message = format!(
            "expected `{}`, found `{}`",
            self.show(want),
            self.show(found)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
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

    fn show(&self, ty: &Ty) -> String {
        let domains: Vec<String> = (0..self.domains.len())
            .map(|d| self.domain_said(d))
            .collect();
        // A struct made from a generic one is shown as it is written,
        // `Mod<998244353>`, not by the name its copy has in the C++.
        let mut structs: Vec<StructDef> = self.structs.iter().map(|s| s.def.clone()).collect();
        for sid in 0..structs.len() {
            let (Some(decl), Some((_, args))) = (self.structs[sid].decl, &self.structs[sid].origin_gen) else {
                continue;
            };
            let args: Vec<String> = args
                .iter()
                .map(|arg| match arg {
                    GenArg::Int(v) => v.to_string(),
                    GenArg::Domain(d) => domains[*d].clone(),
                    GenArg::Ty(ty) => crate::show::ty_name(ty, &domains, &structs, &self.enums),
                    GenArg::RuntimeInt(_) => "_".to_string(),
                    GenArg::Op(op) => format!("{op:?}"),
                })
                .collect();
            structs[sid].name = format!("{}<{}>", decl.name.text, args.join(", "));
        }
        crate::show::ty_name(ty, &domains, &structs, &self.enums)
    }

    fn err_expr(&self, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Error,
            ty: Ty::Error,
            span: self.span(span),
        }
    }

    // ----- declarations across all modules -----

    fn collect_all(&mut self, active_test: Option<&str>) {
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
                            "get" | "brute" | "test" | "solution" | "tags" | "memo" | "law" | "gen" | "checker"
                            | "interactor" | "grader" | "on_unimplemented" | "" => {}
                            // For `cpl lint` alone: it changes nothing a run does.
                            "allow" => {}
                            other => self.unsupported(
                                decorator.span,
                                &format!("the `@{other}` decorator"),
                            ),
                        }
                    }
                }
                match &stmt.kind {
                    ast::StmtKind::Struct(decl) => {
                        if !decl.generics.is_empty() {
                            let gid = self.gen_structs.len();
                            self.gen_structs.push(GenStructTemplate {
                                mod_id: m,
                                offset,
                                decl,
                            });
                            self.mod_syms[m]
                                .insert(decl.name.text.clone(), Sym::GenStruct(gid));
                        } else {
                            let id = self.structs.len();
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Struct(id));
                            let sname = if m == 0 {
                                decl.name.text.clone()
                            } else {
                                format!("m{m}_{}", decl.name.text)
                            };
                            self.structs.push(StructInfo {
                                def: StructDef {
                                    name: sname,
                                    fields: Vec::new(),
                                    fixed: Default::default(),
                                    narrow_ints: Default::default(),
                                    show_fn: None,
                                    eq_fn: None,
                                    lt_fn: None,
                                },
                                decl: Some(decl),
                                mod_id: m,
                                offset,
                                origin_gen: None,
                                env: HashMap::new(),
                                methods: HashMap::new(),
                                op_methods: Vec::new(),
                                gen_methods: HashMap::new(),
                                getters: HashSet::new(),
                                projections: HashMap::new(),
                                consts: HashMap::new(),
                                owned_domains: HashMap::new(),
                                owned_domain_inits: Vec::new(),
                                laws: Vec::new(),
                                index_domain: None,
                            });
                            for item in &decl.methods {
                                let is_get = item.decorators.iter().any(|d| d.name.text == "get");
                                match &item.kind {
                                    ast::StmtKind::Fn(f) => {
                                        if is_get {
                                            self.structs[id].getters.insert(f.name.text.clone());
                                        }
                                        if f.lends.is_some() {
                                            self.structs[id].projections.insert(
                                                f.name.text.clone(),
                                                ProjectionInfo {
                                                    mod_id: m,
                                                    offset,
                                                    owner: id,
                                                    env: HashMap::new(),
                                                    decl: f,
                                                },
                                            );
                                        } else if !f.generics.is_empty() {
                                            let gfid = self.gen_fns.len();
                                            self.gen_fns.push(GenFnTemplate {
                                                mod_id: m,
                                                offset,
                                                owner: Some(id),
                                                owner_env: HashMap::new(),
                                                decl: f,
                                            });
                                            self.structs[id]
                                                .gen_methods
                                                .insert(f.name.text.clone(), gfid);
                                        } else {
                                            let fn_id = self.declare_fn_in(
                                                f,
                                                m,
                                                offset,
                                                Some(id),
                                                HashMap::new(),
                                            );
                                            self.structs[id]
                                                .methods
                                                .insert(f.name.text.clone(), fn_id);
                                        }
                                    }
                                    ast::StmtKind::Let {
                                        pat,
                                        ty,
                                        init: Some(init),
                                        ..
                                    } if mono::let_const_name(pat).is_some() => {
                                        let cname = mono::let_const_name(pat).expect("guarded");
                                        self.structs[id].consts.insert(
                                            cname.to_string(),
                                            StructConst {
                                                mod_id: m,
                                                offset,
                                                ty: ty.as_ref(),
                                                init,
                                            },
                                        );
                                    }
                                    ast::StmtKind::Const {
                                        name,
                                        ty,
                                        init: Some(init),
                                        ..
                                    } => {
                                        self.structs[id].consts.insert(
                                            name.text.clone(),
                                            StructConst {
                                                mod_id: m,
                                                offset,
                                                ty: ty.as_ref(),
                                                init,
                                            },
                                        );
                                    }
                                    ast::StmtKind::Law { name, params, body } => {
                                        self.structs[id].laws.push(LawInfo {
                                            mod_id: m,
                                            offset,
                                            name: name.text.clone(),
                                            params,
                                            body,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    ast::StmtKind::Enum(decl) => {
                        if !decl.generics.is_empty() {
                            let gid = self.gen_enums.len();
                            self.gen_enums.push(GenEnumTemplate {
                                mod_id: m,
                                offset,
                                decl,
                            });
                            self.mod_syms[m]
                                .insert(decl.name.text.clone(), Sym::GenEnum(gid));
                        } else {
                            let id = self.enums.len();
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Enum(id));
                            for (tag, variant) in decl.variants.iter().enumerate() {
                                self.mod_syms[m]
                                    .insert(variant.name.text.clone(), Sym::Variant(id, tag));
                            }
                            let ename = if m == 0 {
                                decl.name.text.clone()
                            } else {
                                format!("m{m}_{}", decl.name.text)
                            };
                            self.enums.push(EnumDef {
                                name: ename,
                                variants: Vec::new(),
                                values: Vec::new(),
                            });
                            self.enum_discriminants.push(Vec::new());
                            self.enum_decls.push((m, offset, decl));
                        }
                    }
                    ast::StmtKind::Domain { name, .. } => {
                        let d = self.domains.len();
                        let dname = if m == 0 {
                            name.text.clone()
                        } else {
                            format!("m{m}_{}", name.text)
                        };
                        self.domains.push(dname);
                        self.mod_syms[m].insert(name.text.clone(), Sym::Domain(d));
                    }
                    ast::StmtKind::Let { pat, .. } => {
                        fn register_pat_domains(
                            pat: &ast::Pattern,
                            m: usize,
                            domains: &mut Vec<String>,
                            mod_syms: &mut [HashMap<String, Sym>],
                        ) {
                            match &pat.kind {
                                ast::PatternKind::Domain(name) => {
                                    if !mod_syms[m].contains_key(&name.text) {
                                        let d = domains.len();
                                        let dname = if m == 0 {
                                            name.text.clone()
                                        } else {
                                            format!("m{m}_{}", name.text)
                                        };
                                        domains.push(dname);
                                        mod_syms[m].insert(name.text.clone(), Sym::Domain(d));
                                    }
                                }
                                ast::PatternKind::Tuple(pats) => {
                                    for p in pats {
                                        register_pat_domains(p, m, domains, mod_syms);
                                    }
                                }
                                _ => {}
                            }
                        }
                        register_pat_domains(pat, m, &mut self.domains, &mut self.mod_syms);
                    }
                    ast::StmtKind::TypeAlias { name, generics, ty } => {
                        if !generics.is_empty() {
                            let gaid = self.gen_aliases.len();
                            self.gen_aliases.push(GenAliasTemplate {
                                mod_id: m,
                                offset,
                                generics,
                                ty,
                            });
                            self.mod_syms[m]
                                .insert(name.text.clone(), Sym::GenAlias(gaid));
                        } else {
                            let aid = self.aliases.len();
                            self.aliases.push((m, ty, offset));
                            self.mod_syms[m].insert(name.text.clone(), Sym::Alias(aid));
                        }
                    }
                    ast::StmtKind::Fn(decl) => {
                        if stmt.decorators.iter().any(|d| d.name.text == "test")
                            && (m != 0 || active_test != Some(decl.name.text.as_str()))
                        {
                            continue;
                        }
                        if !decl.generics.is_empty() {
                            let gfid = self.gen_fns.len();
                            self.gen_fns.push(GenFnTemplate {
                                mod_id: m,
                                offset,
                                owner: None,
                                owner_env: HashMap::new(),
                                decl,
                            });
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::GenFn(gfid));
                        } else {
                            let id = self.declare_fn_in(decl, m, offset, None, HashMap::new());
                            self.mod_syms[m].insert(decl.name.text.clone(), Sym::Fn(id));
                            if stmt.decorators.iter().any(|d| d.name.text == "memo") {
                                self.memo_fns.insert(id);
                                if let Some(table) = memo::into_table(stmt) {
                                    self.memo_into.insert(id, table.to_string());
                                }
                            }
                        }
                    }
                    ast::StmtKind::Const {
                        name,
                        generics,
                        ty,
                        init: Some(init),
                        ..
                    } => {
                        if !generics.is_empty() {
                            let gcid = self.gen_consts.len();
                            self.gen_consts.push(GenConstTemplate {
                                mod_id: m,
                                offset,
                                generics,
                                ty: ty.as_ref(),
                                init,
                            });
                            self.mod_syms[m]
                                .insert(name.text.clone(), Sym::GenConst(gcid));
                        } else {
                            if let Some(v) = self.eval_const_int(init) {
                                self.const_ints.insert(name.text.clone(), v);
                            }
                            if let Some(v) = self.eval_const_bool(init) {
                                self.const_bools.insert(name.text.clone(), v);
                            }
                            if m > 0 {
                                let mcid = self.mod_consts.len();
                                self.mod_consts.push(ModConstInfo {
                                    mod_id: m,
                                    offset,
                                    name: name.text.clone(),
                                    ty: ty.as_ref(),
                                    init,
                                    global: None,
                                });
                                self.mod_syms[m]
                                    .insert(name.text.clone(), Sym::ModConst(mcid));
                            }
                        }
                    }
                    ast::StmtKind::Trait { name, body, .. } => {
                        let tid = self.traits.len();
                        let mut methods = Vec::new();
                        for s in &body.stmts {
                            if let ast::StmtKind::Fn(f) = &s.kind {
                                methods.push(f);
                            }
                        }
                        let unimplemented = stmt
                            .decorators
                            .iter()
                            .filter(|d| d.name.text == "on_unimplemented")
                            .find_map(|d| {
                                let arg = d.args.first()?;
                                let ast::ExprKind::Str(_) = &arg.value.kind else { return None };
                                let abs = arg.value.span.shift(offset);
                                let raw = self.mods[m].text.get(abs.start as usize..abs.end as usize)?;
                                Some(raw.trim_matches('"').to_string())
                            });
                        self.traits.push(TraitInfo {
                            mod_id: m,
                            offset,
                            name: name.text.clone(),
                            methods,
                            unimplemented,
                        });
                        self.mod_syms[m].insert(name.text.clone(), Sym::Trait(tid));
                    }
                    ast::StmtKind::Impl {
                        trait_name,
                        target,
                        body,
                        ..
                    } => {
                        let (tname, targs) = match trait_name.as_ref().map(|t| &t.kind) {
                            Some(ast::TypeKind::Named { name, args }) => {
                                (Some(name.text.clone()), args.iter().collect())
                            }
                            _ => (None, Vec::new()),
                        };
                        self.impls.push(ImplInfo {
                            mod_id: m,
                            offset,
                            trait_name: tname,
                            trait_args: targs,
                            target,
                            body,
                        });
                    }
                    ast::StmtKind::Use { .. } => {}
                    _ => {}
                }
            }
        }

        // Pass 2: wire `use` imports and module aliases across all modules.
        // Run two rounds in case a module re-exports or chains imports.
        for _ in 0..2 {
            for m in 0..num_mods {
                let analysis = self.mods[m];
                let mods_list = analysis.modules();
                for (local_name, imp) in analysis.imports() {
                    if let Some(target_mod) = self.path_to_mod.get(&mods_list[imp.module].path).copied() {
                        if let Some(&sym) = self.mod_syms[target_mod].get(&imp.name) {
                            self.mod_syms[m].entry(local_name.clone()).or_insert(sym);
                        }
                    }
                }
                for (alias, &mod_idx) in analysis.aliases() {
                    if let Some(target_mod) = self.path_to_mod.get(&mods_list[mod_idx].path).copied() {
                        self.mod_syms[m].entry(alias.clone()).or_insert(Sym::Mod(target_mod));
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

    pub(super) fn resolve_product_factor(&mut self, e: &ast::Expr) -> Option<ProductFactor> {
        let ast::ExprKind::Name(name) = &e.kind else {
            return None;
        };
        if name == "Bool" {
            return Some(ProductFactor::Bool);
        }
        if let Some(d) = self.resolve_domain_name(name) {
            return Some(ProductFactor::Domain(d));
        }
        if let Some(Sym::Enum(eid)) = self.get_sym(name) {
            if self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) {
                return Some(ProductFactor::Enum {
                    enum_id: eid,
                    len: self.enums[eid].variants.len() as i64,
                });
            }
        }
        None
    }

    pub(super) fn resolve_product_factors(
        &mut self,
        def: &ast::Expr,
    ) -> Option<Vec<(Option<String>, ProductFactor)>> {
        match &def.kind {
            ast::ExprKind::Tuple(items) if items.len() >= 2 => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push((None, self.resolve_product_factor(item)?));
                }
                Some(out)
            }
            ast::ExprKind::Record(fields) if fields.len() >= 2 => {
                let mut out = Vec::with_capacity(fields.len());
                for (fname, fval) in fields {
                    out.push((
                        Some(fname.text.clone()),
                        self.resolve_product_factor(fval)?,
                    ));
                }
                Some(out)
            }
            _ => None,
        }
    }

    pub(super) fn ty_to_product_factor(&self, ty: &Ty) -> Option<ProductFactor> {
        match ty {
            Ty::Pos(d) => Some(ProductFactor::Domain(*d)),
            Ty::Bool => Some(ProductFactor::Bool),
            Ty::Enum(eid)
                if !self.enums[*eid].variants.is_empty()
                    && self.enums[*eid].variants.iter().all(|(_, fs)| fs.is_empty()) =>
            {
                Some(ProductFactor::Enum {
                    enum_id: *eid,
                    len: self.enums[*eid].variants.len() as i64,
                })
            }
            _ => None,
        }
    }

    pub(super) fn expr_to_product_factor(&mut self, e: &ast::Expr) -> Option<ProductFactor> {
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.expr_to_product_factor(inner),
            ast::ExprKind::Name(name) if name == "Bool" && self.lookup(name).is_none() => {
                Some(ProductFactor::Bool)
            }
            _ => match self.sym(e) {
                Some(Sym::Domain(id)) => Some(ProductFactor::Domain(id)),
                Some(Sym::Enum(eid)) => {
                    if let Some(&sum_dom) = self.enum_to_sum_domain.get(&eid) {
                        Some(ProductFactor::Domain(sum_dom))
                    } else if self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) {
                        Some(ProductFactor::Enum {
                            enum_id: eid,
                            len: self.enums[eid].variants.len() as i64,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            },
        }
    }

    pub(super) fn intern_anon_product_domain(
        &mut self,
        factors: Vec<(Option<String>, ProductFactor)>,
    ) -> DomainId {
        if let Some(&d) = self.anon_product_domains.get(&factors) {
            return d;
        }
        let d = self.domains.len();
        self.domains.push(format!("Prod{d}"));
        self.product_domains.insert(d, factors.clone());
        self.anon_product_domains.insert(factors, d);
        self.anon_product_domain_set.insert(d);
        d
    }

    pub(super) fn def_anon_product_domain(&self, d: DomainId, span: Span, out: &mut Vec<Stmt>) {
        let Some(factors) = self.product_domains.get(&d) else {
            return;
        };
        let mut total = self.product_factor_len(&factors[0].1, span);
        for (_, f) in &factors[1..] {
            let flen = self.product_factor_len(f, span);
            total = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(total), Box::new(flen)),
                Ty::Int,
                span,
            );
        }
        out.push(Stmt::DefDomain {
            id: d,
            lo: self.int_lit(0, span),
            len: total,
        });
    }

    pub(super) fn sum_variant_count(&self, factors: &[ProductFactor], span: Span) -> Expr {
        if factors.is_empty() {
            return self.int_lit(1, span);
        }
        let mut prod = self.product_factor_len(&factors[0], span);
        for f in &factors[1..] {
            let flen = self.product_factor_len(f, span);
            prod = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(prod), Box::new(flen)),
                Ty::Int,
                span,
            );
        }
        prod
    }

    pub(super) fn sum_variant_offset(&self, d: DomainId, tag: usize, span: Span) -> Expr {
        let variants = &self.sum_enums[&d].variants;
        if tag == 0 {
            return self.int_lit(0, span);
        }
        let mut acc = self.sum_variant_count(&variants[0].1, span);
        for (_, fs) in &variants[1..tag] {
            let c = self.sum_variant_count(fs, span);
            acc = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(acc), Box::new(c)),
                Ty::Int,
                span,
            );
        }
        acc
    }

    pub(super) fn def_sum_enum_domain(&self, d: DomainId, span: Span, out: &mut Vec<Stmt>) {
        let variants = &self.sum_enums[&d].variants;
        let mut total = self.sum_variant_count(&variants[0].1, span);
        for (_, fs) in &variants[1..] {
            let c = self.sum_variant_count(fs, span);
            total = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(total), Box::new(c)),
                Ty::Int,
                span,
            );
        }
        out.push(Stmt::DefDomain {
            id: d,
            lo: self.int_lit(0, span),
            len: total,
        });
    }

    pub(super) fn encode_sum_variant(
        &mut self,
        d: DomainId,
        tag: usize,
        parts: Vec<Expr>,
        span: Span,
    ) -> Expr {
        let factors = self.sum_enums[&d].variants[tag].1.clone();
        let off = self.sum_variant_offset(d, tag, span);
        let ord = if factors.is_empty() {
            off
        } else {
            let mut inner = self.product_factor_ord(&factors[0], parts[0].clone(), span);
            for (f, part) in factors[1..].iter().zip(parts.into_iter().skip(1)) {
                let len = self.product_factor_len(f, span);
                let ford = self.product_factor_ord(f, part, span);
                let scaled = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(inner), Box::new(len)),
                    Ty::Int,
                    span,
                );
                inner = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(scaled), Box::new(ford)),
                    Ty::Int,
                    span,
                );
            }
            if tag == 0 {
                inner
            } else {
                self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(off), Box::new(inner)),
                    Ty::Int,
                    span,
                )
            }
        };
        self.cast(Cast::Trust, ord, Ty::Pos(d))
    }

    pub(super) fn is_sum_variant(&self, d: DomainId, tag: usize, val: Expr, span: Span) -> Expr {
        let factors = &self.sum_enums[&d].variants[tag].1;
        let ord = self.cast(Cast::Ord, val, Ty::Int);
        let lo = self.sum_variant_offset(d, tag, span);
        if factors.is_empty() {
            return self.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(ord), Box::new(lo)),
                Ty::Bool,
                span,
            );
        }
        let count = self.sum_variant_count(factors, span);
        let hi = self.mk(
            ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(count)),
            Ty::Int,
            span,
        );
        let ge = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)),
            Ty::Bool,
            span,
        );
        let lt = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(ord), Box::new(hi)),
            Ty::Bool,
            span,
        );
        self.mk(
            ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(lt)),
            Ty::Bool,
            span,
        )
    }

    pub(super) fn decode_sum_variant_field(
        &self,
        d: DomainId,
        tag: usize,
        field_idx: usize,
        val: Expr,
        span: Span,
    ) -> Expr {
        let factors = &self.sum_enums[&d].variants[tag].1;
        let ord = self.cast(Cast::Ord, val, Ty::Int);
        let mut rel = if tag == 0 {
            ord
        } else {
            let lo = self.sum_variant_offset(d, tag, span);
            self.mk(
                ExprKind::Binary(BinOp::Sub, Box::new(ord), Box::new(lo)),
                Ty::Int,
                span,
            )
        };
        if field_idx + 1 < factors.len() {
            let mut stride = self.product_factor_len(&factors[field_idx + 1], span);
            for f in &factors[field_idx + 2..] {
                let flen = self.product_factor_len(f, span);
                stride = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(stride), Box::new(flen)),
                    Ty::Int,
                    span,
                );
            }
            rel = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(rel), Box::new(stride)),
                Ty::Int,
                span,
            );
        }
        let ford = if field_idx == 0 {
            rel
        } else {
            let flen = self.product_factor_len(&factors[field_idx], span);
            self.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(rel), Box::new(flen)),
                Ty::Int,
                span,
            )
        };
        self.product_factor_from_ord(&factors[field_idx], ford, span)
    }

    pub(super) fn product_factor_ty(&self, f: &ProductFactor) -> Ty {
        match f {
            ProductFactor::Domain(d) => Ty::Pos(*d),
            ProductFactor::Enum { enum_id, .. } => Ty::Enum(*enum_id),
            ProductFactor::Bool => Ty::Bool,
        }
    }

    pub(super) fn product_factor_len(&self, f: &ProductFactor, span: Span) -> Expr {
        match f {
            ProductFactor::Domain(d) => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            ProductFactor::Enum { len, .. } => self.int_lit(*len, span),
            ProductFactor::Bool => self.int_lit(2, span),
        }
    }

    pub(super) fn product_factor_ord(&mut self, f: &ProductFactor, val: Expr, span: Span) -> Expr {
        match f {
            ProductFactor::Domain(d) => {
                let v = self.coerce(val, &Ty::Pos(*d), span);
                self.cast(Cast::Ord, v, Ty::Int)
            }
            ProductFactor::Enum { enum_id, .. } => {
                let v = self.coerce(val, &Ty::Enum(*enum_id), span);
                self.cast(Cast::EnumOrd(*enum_id), v, Ty::Int)
            }
            ProductFactor::Bool => {
                let v = self.coerce(val, &Ty::Bool, span);
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::If(Box::new(v), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                )
            }
        }
    }

    pub(super) fn product_factor_from_ord(
        &self,
        f: &ProductFactor,
        ord: Expr,
        span: Span,
    ) -> Expr {
        match f {
            ProductFactor::Domain(d) => self.cast(Cast::Trust, ord, Ty::Pos(*d)),
            ProductFactor::Enum { enum_id, .. } => {
                self.cast(Cast::OrdToEnum(*enum_id), ord, Ty::Enum(*enum_id))
            }
            ProductFactor::Bool => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cmp(CmpOp::Ne, Box::new(ord), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
            }
        }
    }

    pub(super) fn encode_product(&mut self, id: DomainId, parts: Vec<Expr>, span: Span) -> Expr {
        let factors = self.product_domains[&id].clone();
        let mut acc = self.product_factor_ord(&factors[0].1, parts[0].clone(), span);
        for ((_, f), part) in factors[1..].iter().zip(parts.into_iter().skip(1)) {
            let len = self.product_factor_len(f, span);
            let ord = self.product_factor_ord(f, part, span);
            let scaled = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(acc), Box::new(len)),
                Ty::Int,
                span,
            );
            acc = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(scaled), Box::new(ord)),
                Ty::Int,
                span,
            );
        }
        self.cast(Cast::Trust, acc, Ty::Pos(id))
    }

    pub(super) fn decode_product(
        &mut self,
        id: DomainId,
        val: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> Vec<Expr> {
        let factors = self.product_domains[&id].clone();
        let held = self.hold(val, out);
        let mut ord = self.cast(Cast::Ord, held, Ty::Int);
        let mut parts = vec![self.int_lit(0, span); factors.len()];
        for i in (1..factors.len()).rev() {
            let ord_held = self.hold(ord, out);
            let len = self.product_factor_len(&factors[i].1, span);
            let rem = self.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(ord_held.clone()), Box::new(len.clone())),
                Ty::Int,
                span,
            );
            parts[i] = self.product_factor_from_ord(&factors[i].1, rem, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(ord_held), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        parts[0] = self.product_factor_from_ord(&factors[0].1, ord, span);
        parts
    }

    pub(super) fn decode_product_field(
        &mut self,
        id: DomainId,
        val: Expr,
        idx: usize,
        span: Span,
    ) -> Expr {
        let factors = self.product_domains[&id].clone();
        let mut ord = self.cast(Cast::Ord, val, Ty::Int);
        for (_, f) in &factors[idx + 1..] {
            let len = self.product_factor_len(f, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(ord), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        if idx > 0 {
            let len = self.product_factor_len(&factors[idx].1, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(ord), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        self.product_factor_from_ord(&factors[idx].1, ord, span)
    }

    /// `impl T for Int`: a built-in type has no struct to hang methods
    /// on, so each method is a fn whose `self` is the type, found by
    /// `x.method()` on a value of it.
    fn attach_impls_for_builtins(&mut self) {
        for idx in 0..self.impls.len() {
            let (m, offset, target, body) =
                (self.impls[idx].mod_id, self.impls[idx].offset, self.impls[idx].target, self.impls[idx].body);
            let ast::TypeKind::Named { name, args } = &target.kind else {
                continue;
            };
            if !args.is_empty() || self.mod_syms[m].contains_key(&name.text) {
                continue;
            }
            let trait_name = self.impls[idx].trait_name.clone();
            let saved = (std::mem::replace(&mut self.cur_mod, m), std::mem::replace(&mut self.offset, offset));
            let ty = self.resolve_ty(target);
            if !matches!(ty, Ty::Struct(_) | Ty::Error) {
                for item in &body.stmts {
                    if let ast::StmtKind::Fn(f) = &item.kind {
                        if !f.generics.is_empty() || f.lends.is_some() {
                            continue;
                        }
                        let fn_id = self.declare_fn_in(f, m, offset, None, GenEnv::default());
                        self.sigs[fn_id].name = format!("{}_{}_{}", trait_name.as_deref().unwrap_or("impl"), name.text, f.name.text);
                        self.prim_self.insert(fn_id, ty.clone());
                        self.prim_methods.push((ty.clone(), f.name.text.clone(), fn_id));
                        self.resolve_fn_sig(fn_id, trait_name.as_deref());
                    }
                }
            }
            (self.cur_mod, self.offset) = saved;
        }
    }

    pub(super) fn declare_fn_in(
        &mut self,
        decl: &'a ast::FnDecl,
        mod_id: ModId,
        offset: u32,
        owner: Option<StructId>,
        env: GenEnv,
    ) -> FnId {
        let base = match owner {
            Some(id) => format!("{}.{}", self.structs[id].def.name, decl.name.text),
            None if mod_id == 0 => decl.name.text.clone(),
            None => format!("m{mod_id}_{}", decl.name.text),
        };
        let is_generator = matches!(
            decl.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        );
        self.sigs.push(FnSig {
            name: base,
            params: Vec::new(),
            ret: None,
            assumed_unit: false,
            owner,
            receiver: None,
            mod_id,
            env,
            decl,
            offset,
            is_generator,
            new_domain_slots: Vec::new(),
            state: State::Pending,
            asked_at: self.asked_at,
        });
        self.bodies.push(None);
        self.sigs.len() - 1
    }

    /// Second pass: field, payload and parameter types for non-generic declarations.
    fn signatures(&mut self) {
        for id in 0..self.enum_decls.len() {
            let (m, offset, decl) = self.enum_decls[id];
            self.cur_mod = m;
            self.offset = offset;
            let mut next_disc: i64 = 0;
            let mut discs = Vec::with_capacity(decl.variants.len());
            let variants: Vec<(String, Vec<Ty>)> = decl
                .variants
                .iter()
                .map(|v| {
                    let disc = if let Some(val) = &v.value {
                        self.eval_const_int(val).unwrap_or(next_disc)
                    } else {
                        next_disc
                    };
                    discs.push(disc);
                    next_disc = disc.saturating_add(1);
                    let fields = v.fields.iter().map(|(_, t)| self.resolve_ty(t)).collect();
                    (v.name.text.clone(), fields)
                })
                .collect();
            self.enum_discriminants[id] = discs;
            let values = decl
                .variants
                .iter()
                .map(|v| v.value.as_ref().and_then(|e| self.eval_const_int(e)))
                .collect();
            self.enums[id].variants = variants;
            self.enums[id].values = values;
            // §2.16: a payload enum whose every variant carries only finite-index
            // payloads is a finite index domain (sum domain), lowered as a flat `DomainId`.
            let has_payload = self.enums[id].variants.iter().any(|(_, fs)| !fs.is_empty());
            if has_payload && !self.enums[id].variants.is_empty() {
                let sum_variants: Option<Vec<(String, Vec<ProductFactor>)>> = self.enums[id]
                    .variants
                    .iter()
                    .map(|(vname, fs)| {
                        let pfs: Option<Vec<ProductFactor>> =
                            fs.iter().map(|t| self.ty_to_product_factor(t)).collect();
                        pfs.map(|pfs| (vname.clone(), pfs))
                    })
                    .collect();
                if let Some(sum_variants) = sum_variants {
                    let d = self.domains.len();
                    self.domains.push(self.enums[id].name.clone());
                    self.sum_enums.insert(
                        d,
                        SumEnumInfo {
                            enum_id: id,
                            variants: sum_variants,
                        },
                    );
                    self.enum_to_sum_domain.insert(id, d);
                    self.mod_syms[m].insert(decl.name.text.clone(), Sym::Domain(d));
                }
            }
        }
        let num_structs = self.structs.len();
        for id in 0..num_structs {
            let Some(decl) = self.structs[id].decl else {
                continue;
            };
            let (m, offset) = (
                self.structs[id].mod_id,
                self.structs[id].offset,
            );
            self.cur_mod = m;
            self.offset = offset;
            self.self_struct = Some(id);
            let fields = decl
                .fields
                .iter()
                .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
                .collect();
            self.structs[id].def.fields = fields;
            self.structs[id].def.fixed = self.fixed_rows(decl);
            self.structs[id].def.narrow_ints = self.narrow_int_fields(decl);
            self.collect_struct_field_ranges(id, decl);
            self.self_struct = None;
        }
        // Attach `impl` blocks for non-generic structs.
        for id in 0..num_structs {
            self.attach_impls_for_struct(id);
        }
        self.attach_impls_for_builtins();
        let num_sigs = self.sigs.len();
        for id in 0..num_sigs {
            if self.sigs[id].params.is_empty() && !self.sigs[id].decl.params.is_empty()
                || self.sigs[id].ret.is_none()
            {
                self.resolve_fn_sig(id, None);
            }
        }
        self.cur_mod = 0;
        self.offset = 0;
    }

    pub(super) fn resolve_fn_sig(&mut self, id: FnId, trait_name: Option<&str>) {
        self.resolve_fn_sig_with(id, trait_name, None, None);
    }

    pub(super) fn resolve_fn_sig_with(
        &mut self,
        id: FnId,
        trait_name: Option<&str>,
        rhs_ty: Option<&Ty>,
        out_ty: Option<&Ty>,
    ) {
        let decl = self.sigs[id].decl;
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);

        let owner_ty = self.sigs[id].owner.map(Ty::Struct).or_else(|| self.prim_self.get(&id).cloned());
        let mut params = Vec::new();
        for (i, param) in decl.params.iter().enumerate() {
            if i == 0 && param.name.text == "self" && self.sigs[id].owner.is_some() {
                // A method that grows a cache works on the caller's struct.
                let owner = self.sigs[id].owner.expect("checked above");
                let cached = self.structs[owner]
                    .decl
                    .is_some_and(|sd| cache::grows_cache(sd, decl));
                self.sigs[id].receiver = Some(param.mutable || cached);
                continue;
            }
            let ty = match &param.ty {
                Some(ty) => self.resolve_ty(ty),
                None => {
                    if i == 1 {
                        if let Some(rt) = rhs_ty {
                            rt.clone()
                        } else if let Some(ot) = owner_ty.clone() {
                            ot
                        } else {
                            self.error(
                                codes::UNKNOWN_TYPE,
                                param.span,
                                format!("parameter `{}` needs a type", param.name.text),
                            );
                            self.help(format!(
                                "write `{}: Int`, or the type it has",
                                param.name.text
                            ));
                            Ty::Error
                        }
                    } else {
                        match (trait_name, owner_ty.clone()) {
                            (
                                Some(
                                    "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Neg" | "Show" | "Eq"
                                    | "Ord",
                                ),
                                Some(ot),
                            ) => ot,
                            (_, Some(ot)) => ot,
                            _ => {
                                self.error(
                                    codes::UNKNOWN_TYPE,
                                    param.span,
                                    format!("parameter `{}` needs a type", param.name.text),
                                );
                                self.help(format!(
                                    "write `{}: Int`, or the type it has",
                                    param.name.text
                                ));
                                Ty::Error
                            }
                        }
                    }
                }
            };
            params.push(SigParam {
                name: param.name.text.clone(),
                ty,
                mutable: param.mutable,
                default: param.default.as_ref(),
            });
        }
        self.sigs[id].params = params;
        self.sigs[id].ret = match (&decl.ret, &decl.body) {
            (
                Some(ast::Type {
                    kind: ast::TypeKind::Yields(inner),
                    ..
                }),
                _,
            ) => {
                self.sigs[id].is_generator = true;
                Some(Ty::list(self.resolve_ty(inner)))
            }
            (Some(ty), _) => {
                // Check for `-> (domain C, comp: [V]C)` or `-> (domain P, [P]Int)` returning fresh domain(s).
                let mut binders: Vec<String> = Vec::new();
                match &ty.kind {
                    ast::TypeKind::Record(fields) => {
                        for (dname, ft) in fields {
                            if matches!(ft.kind, ast::TypeKind::NewDomain) {
                                binders.push(dname.text.clone());
                            }
                        }
                    }
                    ast::TypeKind::Tuple(items) => {
                        for item in items {
                            if let ast::TypeKind::DomainBinder(dname) = &item.kind {
                                binders.push(dname.text.clone());
                            }
                        }
                    }
                    ast::TypeKind::DomainBinder(dname) => {
                        binders.push(dname.text.clone());
                    }
                    _ => {}
                }
                for dname in binders {
                    let dom_id = self.domains.len();
                    self.domains.push(format!("{}_{}", self.sigs[id].name, dname));
                    self.gen_env.insert(dname.clone(), GenArg::Domain(dom_id));
                    self.sigs[id].env.insert(dname.clone(), GenArg::Domain(dom_id));
                    self.sigs[id].new_domain_slots.push((dname, dom_id));
                }
                Some(self.resolve_ty(ty))
            }
            (None, ast::FnBody::Block(block)) => match block.stmts.last() {
                // A body that ends by calling its own fn gives what that
                // call gives, which is nothing unless a `return` says so.
                Some(ast::Stmt {
                    kind: ast::StmtKind::Expr(last),
                    ..
                }) if calls_fn(last, &decl.name.text) && !returns_a_value(block) => Some(Ty::Unit),
                Some(ast::Stmt {
                    kind: ast::StmtKind::Expr(_),
                    ..
                }) => None,
                _ => match (trait_name, out_ty, owner_ty) {
                    (_, Some(o), _) => Some(o.clone()),
                    (Some("Show"), _, _) => Some(Ty::Str),
                    (Some("Eq" | "Ord"), _, _) => Some(Ty::Bool),
                    (Some("Add" | "Sub" | "Mul" | "Div" | "Rem" | "Neg"), _, Some(ot)) => Some(ot),
                    _ => Some(Ty::Unit),
                },
            },
            (None, _) => match (trait_name, out_ty, owner_ty) {
                (_, Some(o), _) => Some(o.clone()),
                (Some("Show"), _, _) => Some(Ty::Str),
                (Some("Eq" | "Ord"), _, _) => Some(Ty::Bool),
                (Some("Add" | "Sub" | "Mul" | "Div" | "Rem" | "Neg"), _, Some(ot)) => Some(ot),
                _ => None,
            },
        };

        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
    }

    // ----- types -----

    fn resolve_ty(&mut self, ty: &ast::Type) -> Ty {
        self.resolve(ty, false).ty()
    }

    /// The type `ty` names. With `values`, also the lengths and ranges
    /// written in it, lowered where the type stands.
    fn resolve(&mut self, ty: &ast::Type, values: bool) -> Spec {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => self.named(name, args, values),
            ast::TypeKind::Array { index, elem } => {
                // `[V]T` where `V` stands for `(R, C)`: one dimension for
                // each part, as `[R, C]T`.
                let mut dims: Vec<Option<DimSpec>> = Vec::new();
                for d in index {
                    let parts = match &d.kind {
                        ast::TypeKind::Named { name, args } if args.is_empty() => {
                            match self.gen_env.get(&name.text) {
                                Some(GenArg::Ty(Ty::Tuple(items))) => items
                                    .iter()
                                    .map(|t| match t {
                                        Ty::Pos(id) => Some(DimSpec::Pos(*id)),
                                        _ => None,
                                    })
                                    .collect::<Option<Vec<_>>>(),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    match parts {
                        Some(parts) => dims.extend(parts.into_iter().map(Some)),
                        None => dims.push(self.dim(d, values)),
                    }
                }
                let elem = self.resolve(elem, values);
                match dims.into_iter().collect::<Option<Vec<_>>>() {
                    Some(dims) => Spec::Array {
                        dims,
                        elem: Box::new(elem),
                    },
                    None => Spec::Plain(Ty::Error),
                }
            }
            ast::TypeKind::Optional(inner) => Spec::Opt(Box::new(self.resolve(inner, values)), None),
            ast::TypeKind::Tuple(items) if items.is_empty() => Spec::Plain(Ty::Unit),
            ast::TypeKind::Tuple(items) => {
                let specs: Vec<Spec> = items.iter().map(|t| self.resolve(t, values)).collect();
                Spec::Fields(Ty::Tuple(specs.iter().map(Spec::ty).collect()), specs)
            }
            ast::TypeKind::Record(fields) => {
                let specs: Vec<Spec> = fields
                    .iter()
                    .map(|(_, t)| self.resolve(t, values))
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&specs)
                        .map(|((name, _), spec)| (name.text.clone(), spec.ty()))
                        .collect(),
                );
                Spec::Fields(ty, specs)
            }
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !values {
                    return Spec::Plain(Ty::Int);
                }
                let lo = lo.as_deref().map(|lo| Box::new(self.expect(lo, &Ty::Int)));
                let hi = hi.as_deref().map(|hi| {
                    let mut hi = self.expect(hi, &Ty::Int);
                    if !inclusive {
                        hi = self.int_add(hi, -1);
                    }
                    Box::new(hi)
                });
                Spec::Int {
                    lo,
                    hi,
                    text: self.source(ty.span),
                }
            }
            ast::TypeKind::OwnDomain { name, elem } => {
                let dom_id = self.domains.len();
                self.domains.push(name.text.clone());
                self.own_domains.insert(dom_id);
                if let Some(scope) = self.cx.local_syms.last_mut() {
                    scope.insert(name.text.clone(), Sym::Domain(dom_id));
                } else {
                    self.mod_syms[self.cur_mod].insert(name.text.clone(), Sym::Domain(dom_id));
                }
                let elem = self.resolve(elem, values);
                Spec::Array {
                    dims: vec![DimSpec::Pos(dom_id)],
                    elem: Box::new(elem),
                }
            }
            ast::TypeKind::Fn { params, ret } => {
                let ps: Vec<Ty> = params.iter().map(|p| self.resolve_ty(p)).collect();
                let r = self.resolve_ty(ret);
                Spec::Plain(Ty::Fn(ps, Box::new(r)))
            }
            ast::TypeKind::Value(_) => Spec::Plain(Ty::Int),
            ast::TypeKind::NewDomain | ast::TypeKind::DomainBinder(_) => Spec::Plain(Ty::DomainVal),
            ast::TypeKind::Yields(inner) => Spec::Plain(Ty::list(self.resolve_ty(inner))),
            ast::TypeKind::Error => Spec::Plain(Ty::Error),
        }
    }

    fn named(&mut self, name: &ast::Name, args: &[ast::Type], values: bool) -> Spec {
        let arg = |lw: &mut Self, i: usize| match args.get(i) {
            Some(ty) => lw.resolve_ty(ty),
            None => {
                lw.error(
                    codes::UNKNOWN_TYPE,
                    name.span,
                    format!(
                        "`{}` needs its element type: `{}<Int>`",
                        name.text, name.text
                    ),
                );
                Ty::Error
            }
        };
        // Check `Self` and `gen_env` first.
        if name.text == "Self" {
            if let Some(sid) = self.self_struct {
                return if values {
                    self.struct_spec(sid)
                } else {
                    Spec::Plain(Ty::Struct(sid))
                };
            }
        }
        if args.is_empty() {
            if let Some(ga) = self.gen_env.get(&name.text).cloned() {
                return Spec::Plain(match ga {
                    GenArg::Ty(t) => t,
                    GenArg::Domain(d) => Ty::Pos(d),
                    GenArg::Int(_) | GenArg::RuntimeInt(_) => Ty::Int,
                    GenArg::Op(_) => Ty::Unit,
                });
            }
            if let Some(ty) = self.cx.local_type_aliases.get(&name.text).cloned() {
                return Spec::Plain(ty);
            }
            for ocx in self.outer_cxs.iter().rev() {
                if let Some(ty) = ocx.local_type_aliases.get(&name.text).cloned() {
                    return Spec::Plain(ty);
                }
            }
        }
        // Check qualified module type `mod_alias.Type` or `D.bounds` or `Rooted.Pre`.
        if let Some((prefix, rest)) = name.text.split_once('.') {
            if rest == "bounds" {
                if let Some(GenArg::Domain(d)) = self.gen_env.get(prefix).cloned() {
                    return Spec::Plain(Ty::Bound(d));
                }
                if let Some(Sym::Domain(d)) = self.get_sym(prefix) {
                    return Spec::Plain(Ty::Bound(d));
                }
            }
            if let Some(Sym::Mod(target_mod)) = self.get_sym(prefix) {
                let saved_mod = std::mem::replace(&mut self.cur_mod, target_mod);
                let inner_name = ast::Name {
                    text: rest.to_string(),
                    span: name.span,
                };
                let res = self.named(&inner_name, args, values);
                self.cur_mod = saved_mod;
                return res;
            }
            if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                let gargs = self.resolve_gen_args(args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
            if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                    return Spec::Plain(Ty::Pos(dom_id));
                }
            }
            // `xs.D`: the domain a value such as `compress(all)` brought.
            if let Some(dom_id) = self.domain_of_value(prefix, rest) {
                return Spec::Plain(Ty::Pos(dom_id));
            }
        }

        let ty = match name.text.as_str() {
            // The solution an `@interactor` talks to holds nothing.
            "Interactor" if self.judge => Ty::Unit,
            "Int" | "I64" => Ty::Int,
            "F64" => Ty::Float,
            "Bool" => Ty::Bool,
            "Char" => Ty::Char,
            "Str" => Ty::Str,
            "List" | "Iter" => Ty::list(arg(self, 0)),
            "Opt" => Ty::opt(arg(self, 0)),
            "Set" => Ty::Set(Box::new(arg(self, 0))),
            "HashSet" => Ty::HashSet(Box::new(arg(self, 0))),
            "Map" => {
                let key = arg(self, 0);
                Ty::Map(Box::new(key), Box::new(arg(self, 1)))
            }
            "HashMap" => {
                let key = arg(self, 0);
                Ty::HashMap(Box::new(key), Box::new(arg(self, 1)))
            }
            "Range" => Ty::Range(Box::new(arg(self, 0))),
            "Subset" => match arg(self, 0) {
                Ty::Pos(base) => Ty::Pos(self.subset_domain(base)),
                Ty::Error => Ty::Error,
                other => {
                    let message =
                        format!("`Subset` takes a domain, and `{}` is not one", self.show(&other));
                    self.error(codes::TYPE_MISMATCH, name.span, message);
                    Ty::Error
                }
            },
            "Bounds" => match arg(self, 0) {
                Ty::Pos(d) => Ty::Bound(d),
                other => other,
            },
            "I128" => Ty::I128,
            "U64" => Ty::U64,
            _ => match self.get_sym(&name.text) {
                Some(Sym::Struct(id)) if values => return self.struct_spec(id),
                Some(Sym::Struct(id)) => Ty::Struct(id),
                Some(Sym::GenStruct(gid)) => {
                    let gargs = self.resolve_gen_args(args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if values {
                        return self.struct_spec(sid);
                    }
                    Ty::Struct(sid)
                }
                Some(Sym::Enum(id)) => Ty::Enum(id),
                Some(Sym::GenEnum(gid)) => {
                    let gargs = self.resolve_gen_args(args);
                    Ty::Enum(self.instantiate_enum(gid, gargs))
                }
                Some(Sym::Domain(id)) => Ty::Pos(id),
                Some(Sym::Alias(id)) => {
                    let (m, ty, offset) = self.aliases[id];
                    let saved_mod = std::mem::replace(&mut self.cur_mod, m);
                    let saved_offset = std::mem::replace(&mut self.offset, offset);
                    let spec = self.resolve(ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    return spec;
                }
                Some(Sym::GenAlias(gaid)) => {
                    let gargs = self.resolve_gen_args(args);
                    let tpl_mod = self.gen_aliases[gaid].mod_id;
                    let tpl_offset = self.gen_aliases[gaid].offset;
                    let generics = self.gen_aliases[gaid].generics;
                    let target_ty = self.gen_aliases[gaid].ty;
                    let mut env = HashMap::new();
                    for (p, a) in generics.iter().zip(gargs) {
                        env.insert(p.text.clone(), a);
                    }
                    let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
                    let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
                    let saved_env = std::mem::replace(&mut self.gen_env, env);
                    let spec = self.resolve(target_ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    self.gen_env = saved_env;
                    return spec;
                }
                _ => {
                    self.error(
                        codes::UNKNOWN_TYPE,
                        name.span,
                        format!("`{}` is not a type `cpl run` knows", name.text),
                    );
                    Ty::Error
                }
            },
        };
        Spec::Plain(ty)
    }

    /// The domain of the subsets of `base`, made on first use.
    fn subset_domain(&mut self, base: DomainId) -> DomainId {
        if let Some((subset, _)) = self.subsets.iter().find(|(_, b)| *b == base) {
            return *subset;
        }
        let subset = self.domains.len();
        self.domains.push(format!("Subset<{}>", self.domains[base]));
        self.subsets.push((subset, base));
        subset
    }

    /// The domain whose subsets `subset` holds, when it is a `Subset<D>`.
    fn subset_base(&self, subset: DomainId) -> Option<DomainId> {
        self.subsets.iter().find(|(s, _)| *s == subset).map(|(_, base)| *base)
    }

    /// `v in mask`, both already lowered: bit `v` of the mask.
    fn subset_has(&mut self, mask: Expr, item: Expr, span: Span) -> Expr {
        let bits = self.cast(Cast::Ord, mask, Ty::Int);
        let at = self.cast(Cast::Ord, item, Ty::Int);
        let shifted = self.mk(ExprKind::Binary(BinOp::Shr, Box::new(bits), Box::new(at)), Ty::Int, span);
        let one = self.int_lit(1, span);
        let bit = self.mk(
            ExprKind::Binary(BinOp::BitAnd, Box::new(shifted), Box::new(one.clone())),
            Ty::Int,
            span,
        );
        self.mk(ExprKind::Cmp(CmpOp::Eq, Box::new(bit), Box::new(one)), Ty::Bool, span)
    }

    /// A list built by walking the ordinals of `domain`: `first` goes in
    /// before the walk, and `each` gives what an ordinal adds, if anything.
    fn listed(
        &mut self,
        domain: DomainId,
        elem: Ty,
        first: Option<Expr>,
        span: Span,
        each: impl FnOnce(&mut Self, Expr) -> (Option<Expr>, Expr),
    ) -> Expr {
        let list_ty = Ty::list(elem);
        let acc = self.temp(list_ty.clone());
        let at = self.temp(Ty::Int);
        let id = self.next_loop;
        self.next_loop += 1;
        let push = |lw: &mut Self, value: Expr| {
            Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(acc),
                        ty: list_ty.clone(),
                        span,
                    }),
                    vec![value],
                ),
                Ty::Unit,
                span,
            ))
        };
        let empty = self.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
        let mut stmts = vec![Stmt::Let(acc, Some(empty))];
        if let Some(first) = first {
            stmts.push(push(self, first));
        }
        let ord = self.local(at, span);
        let (keep, value) = each(self, ord);
        let add = push(self, value);
        let body = match keep {
            Some(keep) => Stmt::If(keep, Block { stmts: vec![add] }, Block::default()),
            None => add,
        };
        let lo = self.int_lit(0, span);
        let hi = self.mk(ExprKind::DomainLen(domain), Ty::Int, span);
        stmts.push(Stmt::For(
            id,
            at,
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            },
            Block { stmts: vec![body] },
        ));
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), list_ty, span)
    }

    /// The domain `rest` that the variable `prefix` holds: the `D` of
    /// `let xs = compress(all)`.
    fn domain_of_value(&mut self, prefix: &str, rest: &str) -> Option<DomainId> {
        let owner = ast::Expr {
            kind: ast::ExprKind::Name(prefix.to_string()),
            span: Span::point(0),
        };
        let Some(Ty::Struct(sid)) = self.probe_ty_silent(&owner) else {
            return None;
        };
        match self.structs[sid].env.get(rest) {
            Some(GenArg::Domain(id)) | Some(GenArg::Ty(Ty::Pos(id))) => Some(*id),
            _ => self.structs[sid].owned_domains.get(rest).copied(),
        }
    }

    /// A struct's fields as they are read from input: with their ranges.
    fn struct_spec(&mut self, id: StructId) -> Spec {
        if let Some(&fn_id) = self.structs[id].methods.get("read") {
            if self.sigs[fn_id].params.is_empty() {
                let call = self.method_call(fn_id, None, &[], Span::default());
                return Spec::Call(Box::new(call));
            }
        }
        let Some(decl) = self.structs[id].decl else {
            return Spec::Fields(Ty::Struct(id), Vec::new());
        };
        let (m, offset, env) = (
            self.structs[id].mod_id,
            self.structs[id].offset,
            self.structs[id].env.clone(),
        );
        let saved_mod = std::mem::replace(&mut self.cur_mod, m);
        let saved_offset = std::mem::replace(&mut self.offset, offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let fields = decl
            .fields
            .iter()
            .map(|f| self.resolve(&f.ty, true))
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        Spec::Fields(Ty::Struct(id), fields)
    }

    /// One dimension of an array type: a domain, its bounds, a custom `Index` struct, or a count.
    fn dim(&mut self, ty: &ast::Type, values: bool) -> Option<DimSpec> {
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            if args.is_empty() {
                if let Some(GenArg::Domain(id)) = self.gen_env.get(&name.text).cloned() {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(&name.text).cloned() {
                    return Some(DimSpec::Bound(id));
                }
                if let Some(Sym::Domain(id)) = self.get_sym(&name.text) {
                    return Some(DimSpec::Pos(id));
                }
                if let Some(Sym::Struct(sid)) = self.get_sym(&name.text) {
                    if let Some(dom_id) = self.structs[sid].index_domain {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(Sym::Enum(eid)) = self.get_sym(&name.text) {
                    if self.enums[eid].variants.iter().all(|(_, f)| f.is_empty()) {
                        return Some(DimSpec::Enum(eid));
                    }
                }
                // `type Val = xs.D`, then `[Val]Int`.
                if matches!(self.get_sym(&name.text), Some(Sym::Alias(_)))
                    || self.cx.local_type_aliases.contains_key(&name.text)
                {
                    match self.resolve_ty(ty) {
                        Ty::Pos(id) => return Some(DimSpec::Pos(id)),
                        Ty::Bound(id) => return Some(DimSpec::Bound(id)),
                        _ => {}
                    }
                }
                // `[xs.D]`: the domain a value such as `compress(all)` brought.
                if let Some((owner, member)) = name.text.split_once('.') {
                    if let Some(id) = self.domain_of_value(owner, member) {
                        return Some(DimSpec::Pos(id));
                    }
                }
                if let Some(domain) = name.text.strip_suffix(".bounds") {
                    if let Some(GenArg::Domain(id)) = self.gen_env.get(domain).cloned() {
                        return Some(DimSpec::Bound(id));
                    }
                    if let Some(Sym::Domain(id)) = self.get_sym(domain) {
                        return Some(DimSpec::Bound(id));
                    }
                }
            } else if name.text == "Subset" && args.len() == 1 {
                // `[Subset<Row>]Int`: one cell for each subset.
                if let Ty::Pos(id) = self.resolve_ty(ty) {
                    return Some(DimSpec::Pos(id));
                }
            } else if name.text == "Bounds" && args.len() == 1 {
                if let Ty::Pos(id) = self.resolve_ty(&args[0]) {
                    return Some(DimSpec::Bound(id));
                }
            } else if name.text == "Sym" && args.len() == 1 {
                if let Ty::Pos(id) = self.resolve_ty(&args[0]) {
                    return Some(DimSpec::Sym(id));
                }
            } else if let Some(Sym::GenStruct(gid)) = self.get_sym(&name.text) {
                let gargs = self.resolve_gen_args(args);
                let sid = self.instantiate_struct(gid, gargs);
                if let Some(dom_id) = self.structs[sid].index_domain {
                    return Some(DimSpec::Pos(dom_id));
                }
            } else if let Some((prefix, rest)) = name.text.split_once('.') {
                if let Some(Sym::GenStruct(gid)) = self.get_sym(prefix) {
                    let gargs = self.resolve_gen_args(args);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                    if let Some(&dom_id) = self.structs[sid].owned_domains.get(rest) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                // `[xs.D]`: the domain a value such as `compress(all)` brought.
                if let Some(dom_id) = self.domain_of_value(prefix, rest) {
                    return Some(DimSpec::Pos(dom_id));
                }
            }
        }
        // Also handle `[xs.D]T` parsed as `TypeKind::Value(Field { base: Name(prefix), name: rest })`.
        if let ast::TypeKind::Value(e) = &ty.kind {
            if let ast::ExprKind::Field { base, name } = &e.kind {
                if let ast::ExprKind::Name(prefix) = &base.kind {
                    if let Some(dom_id) = self.domain_of_value(prefix, &name.text) {
                        return Some(DimSpec::Pos(dom_id));
                    }
                    if let Some(Sym::Struct(sid)) = self.get_sym(prefix) {
                        if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                            return Some(DimSpec::Pos(dom_id));
                        }
                    }
                }
            }
        }
        // Anything else is a length: `[n]Int`, `[2 * n]Int`, `[0..n]Int`, `[V.bounds]Int`.
        let text = self.source(ty.span);
        if let Some(domain) = text.strip_suffix(".bounds") {
            let dname = domain.trim();
            if let Some(GenArg::Domain(id)) = self.gen_env.get(dname).cloned() {
                return Some(DimSpec::Bound(id));
            }
            if let Some(Sym::Domain(id)) = self.get_sym(dname) {
                return Some(DimSpec::Bound(id));
            }
        }
        match &ty.kind {
            ast::TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => {
                let first = match &lo.kind {
                    ast::ExprKind::Int(first) => i64::try_from(*first).ok(),
                    ast::ExprKind::Unary { op: ast::UnOp::Neg, expr } => match &expr.kind {
                        ast::ExprKind::Int(first) => i64::try_from(*first).ok().map(|first| -first),
                        _ => None,
                    },
                    _ => None,
                };
                let Some(first) = first else {
                    self.unsupported(lo.span, "an array dimension that starts at anything but a number");
                    self.help("name the range: `domain D = lo..=hi`, then `[D]T`");
                    return None;
                };
                if first != 0 {
                    if !values {
                        return Some(DimSpec::IntFrom(first, Box::new(self.int_lit(0, ty.span))));
                    }
                    let mut len = self.expect(hi, &Ty::Int);
                    len = self.int_add(len, if *inclusive { 1 - first } else { -first });
                    return Some(DimSpec::IntFrom(first, Box::new(len)));
                }
                if !*inclusive {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let probed = self.expr(hi, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let ExprKind::DomainVal(dom_id) = probed.kind {
                        return Some(DimSpec::Pos(dom_id));
                    }
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let mut len = self.expect(hi, &Ty::Int);
                if *inclusive {
                    len = self.int_add(len, 1);
                }
                Some(DimSpec::Int(Box::new(len)))
            }
            // `[Bool]T` has two cells, `false` then `true`.
            ast::TypeKind::Named { name, args }
                if args.is_empty() && name.text == "Bool" && self.get_sym(&name.text).is_none() =>
            {
                Some(DimSpec::Int(Box::new(self.int_lit(2, ty.span))))
            }
            // `[Sign]T` for a fieldless enum `Sign`: one cell per variant.
            ast::TypeKind::Named { name, args }
                if args.is_empty()
                    && matches!(self.get_sym(&name.text), Some(Sym::Enum(eid)) if !self.enums[eid].variants.is_empty() && self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty())) =>
            {
                let Some(Sym::Enum(eid)) = self.get_sym(&name.text) else {
                    unreachable!()
                };
                let n = self.enums[eid].variants.len() as i64;
                Some(DimSpec::Int(Box::new(self.int_lit(n, ty.span))))
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(Sym::Alias(aid)) = self.get_sym(&name.text) {
                    let (m, aliased_ty, offset) = self.aliases[aid];
                    let saved_mod = std::mem::replace(&mut self.cur_mod, m);
                    let saved_offset = std::mem::replace(&mut self.offset, offset);
                    let d = self.dim(aliased_ty, values);
                    self.cur_mod = saved_mod;
                    self.offset = saved_offset;
                    return d;
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let expr = ast::Expr {
                    kind: ast::ExprKind::Name(name.text.clone()),
                    span: name.span,
                };
                Some(DimSpec::Int(Box::new(self.expect(&expr, &Ty::Int))))
            }
            ast::TypeKind::Value(expr) => {
                if !values {
                    let saved_diags = self.diags.len();
                    let saved_cx = self.cx.clone();
                    let saved_outer = self.outer_cxs.clone();
                    let v = self.expr(expr, None);
                    self.diags.truncate(saved_diags);
                    self.cx = saved_cx;
                    self.outer_cxs = saved_outer;
                    if let ExprKind::DomainVal(dom_id) = v.kind {
                        return Some(DimSpec::Pos(dom_id));
                    }
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let v = self.expr(expr, None);
                if let ExprKind::DomainVal(dom_id) = v.kind {
                    return Some(DimSpec::Pos(dom_id));
                }
                Some(DimSpec::Int(Box::new(self.coerce(v, &Ty::Int, expr.span))))
            }
            _ => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    ty.span,
                    "an array is indexed by a domain or a length: `[V]Int`, `[n]Int`",
                );
                None
            }
        }
    }

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
            .map(|m| {
                let mut steps = cpl_syntax::order::Steps::default();
                for (offset, stmt) in self.mod_items(m) {
                    steps.read(std::slice::from_ref(stmt), offset, &mut |f, x| mutators.writes(f, x));
                }
                steps
            })
            .collect()
    }

    fn lookup(&self, name: &str) -> Option<Slot> {
        self.cx
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
    }

    /// Declares `name` with the value `init`: a global at the top level of
    /// the file, a local anywhere else.
    fn declare(&mut self, name: &str, init: Expr, mutable: bool, out: &mut Vec<Stmt>) {
        if self.cx.is_main && self.cx.scopes.len() == 1 {
            let id = self.globals.len();
            self.globals.push(Global {
                name: name.to_string(),
                ty: init.ty.clone(),
                span: init.span,
            });
            self.mod_syms[self.cur_mod].insert(name.to_string(), Sym::Global(id));
            let place = Place {
                kind: PlaceKind::Global(id),
                ty: init.ty.clone(),
                span: init.span,
            };
            out.push(Stmt::Assign(place, AssignOp::Set, init));
        } else {
            let slot = self.new_slot(name, init.ty.clone());
            if mutable {
                self.cx.mut_slots.insert(slot);
            }
            self.cx
                .scopes
                .last_mut()
                .expect("a scope")
                .insert(name.to_string(), slot);
            out.push(Stmt::Let(slot, Some(init)));
        }
    }

    // ----- fns -----

    fn main(&mut self) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        for (idx, &(offset, stmt)) in items.iter().enumerate() {
            self.offset = offset;
            // Skip `@test` functions in normal `cpl run`/`cpl build`.
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                continue;
            }
            for decorator in &stmt.decorators {
                if decorator.name.text == "multitest" {
                    self.divide(decorator, &mut out);
                }
            }
            if matches!(stmt.kind, ast::StmtKind::Let { ty: None, .. }) {
                let rest: Vec<ast::Stmt> = items[idx + 1..]
                    .iter()
                    .filter(|(_, s)| !s.decorators.iter().any(|d| d.name.text == "test"))
                    .map(|(_, s)| (*s).clone())
                    .collect();
                self.stmt_with_rest(stmt, &rest, &mut out);
            } else {
                self.stmt(stmt, &mut out);
            }
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// `@!multitest`: what was lowered so far runs once, and the rest of
    /// the file once per test case (decision 20.1). A bare `@multitest` is
    /// the older spelling of the same divider.
    fn divide(&mut self, decorator: &ast::Decorator, out: &mut Vec<Stmt>) {
        if self.multitest {
            self.error(
                codes::UNEXPECTED_TOKEN,
                decorator.span,
                "a second `@!multitest`".to_string(),
            );
            return;
        }
        self.multitest = true;
        for arg in &decorator.args {
            match &arg.value.kind {
                ast::ExprKind::Name(name) if arg.name.is_none() && name == "until_eof" => {
                    self.until_eof = true;
                }
                _ => self.error(
                    codes::UNEXPECTED_TOKEN,
                    arg.value.span,
                    "`@!multitest` takes nothing, or `until_eof`".to_string(),
                ),
            }
        }
        self.setup_body = Block {
            stmts: std::mem::take(out),
        };
    }

    fn main_for_test(&mut self, test_name: &str) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        for (offset, stmt) in items {
            self.offset = offset;
            // Under `@!multitest` the judge runs once per test case, as
            // the solution does, and tells it how many there are.
            if self.judge {
                for decorator in &stmt.decorators {
                    if decorator.name.text == "multitest" {
                        self.divide(decorator, &mut out);
                        self.announce_tests = !self.until_eof;
                    }
                }
            }
            // Run top-level constants/lets/domains/impl Index initializers before the test.
            match &stmt.kind {
                ast::StmtKind::Const { generics, .. } if generics.is_empty() => {
                    self.stmt(stmt, &mut out);
                }
                // The solution's own lines are not the judge's to run.
                ast::StmtKind::Let { .. } | ast::StmtKind::Domain { .. } if self.judge => {}
                ast::StmtKind::Let { .. }
                | ast::StmtKind::Domain { .. }
                | ast::StmtKind::Impl { .. } => {
                    self.stmt(stmt, &mut out);
                }
                _ => {}
            }
        }
        if let Some(Sym::Fn(fn_id)) = self.mod_syms[0].get(test_name).copied() {
            self.queue_fn(fn_id);
            let ret = self.ret_of(fn_id, Span::default());
            // An `@interactor` is handed the solution: `sol`.
            let mutable: Vec<bool> = self.sigs[fn_id].params.iter().map(|p| p.mutable).collect();
            let args = mutable
                .into_iter()
                .map(|mutable| {
                    if !mutable {
                        return Arg::Value(self.mk(ExprKind::Unit, Ty::Unit, Span::default()));
                    }
                    let slot = self.new_slot("", Ty::Unit);
                    out.push(Stmt::Let(slot, None));
                    Arg::Mut(Place {
                        kind: PlaceKind::Local(slot),
                        ty: Ty::Unit,
                        span: Span::default(),
                    })
                })
                .collect();
            let call = Expr {
                kind: ExprKind::Call(fn_id, args),
                ty: ret,
                span: Span::default(),
            };
            out.push(Stmt::Expr(call));
        } else {
            self.error(
                codes::UNRESOLVED_NAME,
                Span::default(),
                format!("no `@test` fn named `{test_name}`"),
            );
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// Whether `text` calls `read()`.
    fn reads_input(text: &str) -> bool {
        text.match_indices("read(").any(|(at, _)| {
            !text[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
        })
    }

    /// A `@brute(main)` or `@checker` fn as the program: the input header
    /// of the file, then a call of the fn. The header is every top-level
    /// line down to the last one that calls `read()`, so the fn sees the
    /// input by the names the solution gave it, in single-test and
    /// multitest files alike. Below the header only constants are kept:
    /// the rest is the solution's own work.
    fn main_for_aside(&mut self, name: &str) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        let is_item = |stmt: &ast::Stmt| {
            matches!(
                stmt.kind,
                ast::StmtKind::Fn(_) | ast::StmtKind::Struct(_) | ast::StmtKind::Impl { .. }
            )
        };
        let own_reads = items.iter().any(|(offset, stmt)| {
            self.offset = *offset;
            matches!(&stmt.kind, ast::StmtKind::Fn(decl) if decl.name.text == name)
                && Self::reads_input(&self.source(stmt.span))
        });
        let size = match self.aside {
            Some(Aside::Gen(size)) => Some(size),
            _ => None,
        };
        let header = if let Some(Aside::Read(line)) = self.aside {
            let text = self.mods[0].text.clone();
            let line_of = |at: u32| {
                text[..(at as usize).min(text.len())].matches('\n').count() as u32 + 1
            };
            items
                .iter()
                .filter(|(offset, stmt)| line_of(stmt.span.shift(*offset).start) < line)
                .count()
        } else if own_reads || size.is_some() {
            0
        } else {
            items
                .iter()
                .rposition(|(offset, stmt)| {
                    self.offset = *offset;
                    !is_item(stmt) && Self::reads_input(&self.source(stmt.span))
                })
                .map_or(0, |last| last + 1)
        };
        for (idx, &(offset, stmt)) in items.iter().enumerate() {
            self.offset = offset;
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                continue;
            }
            for decorator in &stmt.decorators {
                if decorator.name.text == "multitest" && !matches!(self.aside, Some(Aside::Gen(_))) {
                    self.divide(decorator, &mut out);
                }
            }
            let constant = matches!(&stmt.kind, ast::StmtKind::Const { generics, .. } if generics.is_empty());
            if idx < header || constant || is_item(stmt) {
                self.stmt(stmt, &mut out);
                continue;
            }
            match &stmt.kind {
                ast::StmtKind::Let { pat, .. } => Self::pattern_names(pat, &mut self.tail_names),
                ast::StmtKind::Domain { name, .. } => {
                    self.tail_names.insert(name.text.clone());
                }
                _ => {}
            }
        }
        if let Some(Sym::Fn(fn_id)) = self.mod_syms[0].get(name).copied() {
            self.queue_fn(fn_id);
            let at = self.sigs[fn_id].decl.name.span;
            let params: Vec<Ty> = self.sigs[fn_id].params.iter().map(|p| p.ty.clone()).collect();
            let args = match (size, params.as_slice()) {
                (_, []) => Vec::new(),
                (Some(size), [Ty::Int]) => vec![Arg::Value(Expr {
                    kind: ExprKind::Int(size),
                    ty: Ty::Int,
                    span: Span::default(),
                })],
                (Some(_), _) => {
                    let message = "a `@gen` fn takes nothing, or one `size: Int`: how large a case to print";
                    self.error(codes::WRONG_ARG_COUNT, at, message.to_string());
                    Vec::new()
                }
                (None, _) => {
                    let kind = if self.aside == Some(Aside::Brute) { "@brute(main)" } else { "@checker" };
                    self.error(
                        codes::WRONG_ARG_COUNT,
                        at,
                        format!("a `{kind}` fn takes no parameters: it reads the input the file reads"),
                    );
                    Vec::new()
                }
            };
            let ret = self.ret_of(fn_id, Span::default());
            let call = Expr {
                kind: ExprKind::Call(fn_id, args),
                ty: ret,
                span: Span::default(),
            };
            if let Some(Aside::Read(_)) = self.aside {
                let printed = self.fmt_arg(call, crate::FLOAT_DIGITS, Span::default());
                let print = self.mk(
                    ExprKind::Print(printed.into_iter().collect(), None),
                    Ty::Unit,
                    Span::default(),
                );
                out.push(Stmt::Expr(print));
            } else {
                out.push(Stmt::Expr(call));
            }
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    fn pattern_names(pat: &ast::Pattern, out: &mut HashSet<String>) {
        match &pat.kind {
            ast::PatternKind::Name(name) => {
                out.insert(name.clone());
            }
            ast::PatternKind::Domain(name) => {
                out.insert(name.text.clone());
            }
            ast::PatternKind::Tuple(pats) => pats.iter().for_each(|p| Self::pattern_names(p, out)),
            _ => {}
        }
    }

    pub(super) fn queue_fn(&mut self, id: FnId) {
        if self.queued_set.insert(id) {
            self.sigs[id].asked_at = self.asked_at;
            self.queued_fns.push(id);
        }
    }

    /// Lowers the body of fn `id`, once.
    fn function(&mut self, id: FnId) {
        if self.sigs[id].state != State::Pending {
            return;
        }
        // Skip uncalled `@test` fns in non-test runs unless explicitly referenced.
        self.sigs[id].state = State::InProgress;
        let decl = self.sigs[id].decl;
        let is_gen = self.sigs[id].is_generator;
        let new_doms = self.sigs[id].new_domain_slots.clone();
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);
        let saved_asked = std::mem::replace(&mut self.asked_at, self.sigs[id].asked_at);

        let mut cx = FnCx {
            ret: self.sigs[id].ret.clone(),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut params = Vec::new();
        if let (Some(mutable), Some(owner)) = (self.sigs[id].receiver, self.sigs[id].owner) {
            cx.locals.push(Local {
                name: "self".into(),
                ty: Ty::Struct(owner),
            });
            cx.scopes[0].insert("self".into(), 0);
            if mutable {
                cx.mut_slots.insert(0);
            }
            cx.receiver = Some((0, owner));
            params.push(ParamDef { mutable });
        }
        for param in &self.sigs[id].params {
            let slot = cx.locals.len();
            cx.scopes[0].insert(param.name.clone(), slot);
            if param.mutable {
                cx.mut_slots.insert(slot);
            }
            cx.locals.push(Local {
                name: param.name.clone(),
                ty: param.ty.clone(),
            });
            params.push(ParamDef {
                mutable: param.mutable,
            });
        }
        for ast_p in &decl.params {
            if let Some(ast::Type {
                kind: ast::TypeKind::Named { name, args },
                ..
            }) = &ast_p.ty
            {
                if args.is_empty() {
                    if let Some(GenArg::Op(op)) = self.gen_env.get(&name.text) {
                        cx.op_locals.insert(ast_p.name.text.clone(), *op);
                    }
                }
            }
        }
        let saved_cx = std::mem::replace(&mut self.cx, cx);
        for ast_p in &decl.params {
            if let Some(ty_ast) = &ast_p.ty {
                if let Some(&slot) = self.cx.scopes[0].get(&ast_p.name.text) {
                    if let Some(rinfo) = self.range_info_from_ast(ty_ast) {
                        self.cx.local_ranges.insert(slot, rinfo);
                    }
                }
            }
        }
        let mut out = Vec::new();

        // If this is a constructor/static method of a struct with owned domains (e.g. `Rooted<V>.Pre`),
        // initialize those owned domains at the start of the method.
        if let Some(owner) = self.sigs[id].owner {
            if self.sigs[id].receiver.is_none() {
                let inits = self.structs[owner].owned_domain_inits.clone();
                for init in inits {
                    let sm = std::mem::replace(&mut self.cur_mod, init.mod_id);
                    let so = std::mem::replace(&mut self.offset, init.offset);
                    self.init_domain_id(init.id, init.def, &mut out);
                    self.cur_mod = sm;
                    self.offset = so;
                }
            }
        }

        // If this fn is a generator (`-> yields T`), allocate the output `List<T>` accumulator.
        if is_gen {
            let list_ty = self.cx.ret.clone().unwrap_or_else(|| Ty::list(Ty::Error));
            let elem_ty = match &list_ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let acc = self.new_slot("__yields", list_ty.clone());
            self.cx.mut_slots.insert(acc);
            out.push(Stmt::Let(
                acc,
                Some(self.mk(ExprKind::Tuple(Vec::new()), list_ty, decl.name.span)),
            ));
            self.cx.yield_slot = Some((acc, elem_ty));
        }

        // If this fn returns `(domain C, ...)`, pre-bind each fresh domain as a local domain symbol.
        for (dname, dom_id) in &new_doms {
            self.cx.local_syms[0].insert(dname.clone(), Sym::Domain(*dom_id));
            self.cx.new_domain_slots.push(*dom_id);
        }

        match &decl.body {
            ast::FnBody::Block(block) => {
                let tail = if is_gen { Tail::Discard } else { Tail::Return };
                let body = self.block(block, tail);
                out.extend(body.stmts);
            }
            ast::FnBody::Expr(expr) => {
                let tail = if is_gen { Tail::Discard } else { Tail::Return };
                self.tail(expr, tail, &mut out);
            }
            // The judge's grader supplies the body: the signature is all
            // there is.
            ast::FnBody::None => {}
        }

        if let Some((acc, _)) = self.cx.yield_slot {
            let res = self.local(acc, decl.name.span);
            out.push(Stmt::Return(Some(res)));
        }

        let ret = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret != Ty::Unit && !matches!(out.last(), Some(Stmt::Return(_))) {
            out.push(Stmt::Trap(
                format!("fn `{}` ended without a value", decl.name.text),
                self.span(decl.name.span),
            ));
        }
        if self.sigs[id].assumed_unit && !matches!(ret, Ty::Unit | Ty::Error) {
            // Its own calls were read as giving nothing, but its last
            // line gives a value.
            let name = decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                decl.name.span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            let shown = self.show(&ret);
            self.help(format!("add `-> {shown}` to `fn {name}`"));
        }
        // `@memo`: the body moves to a fn of its own, and this one looks
        // the arguments up before calling it.
        let plain = params.iter().all(|p| !p.mutable) && self.sigs[id].receiver.is_none();
        if self.memo_fns.contains(&id) && plain && !is_gen && !matches!(ret, Ty::Unit | Ty::Error) {
            let wrapper = FnCx {
                ret: Some(ret.clone()),
                scopes: vec![HashMap::new()],
                local_syms: vec![HashMap::new()],
                ..FnCx::default()
            };
            let body_cx = std::mem::replace(&mut self.cx, wrapper);
            let body_id = self.sigs.len();
            self.sigs.push(FnSig {
                name: format!("{}_body", self.sigs[id].name),
                params: Vec::new(),
                ret: Some(ret.clone()),
                assumed_unit: false,
                owner: None,
                receiver: None,
                mod_id: self.sigs[id].mod_id,
                env: self.sigs[id].env.clone(),
                decl,
                offset: self.sigs[id].offset,
                is_generator: false,
                new_domain_slots: Vec::new(),
                state: State::Done,
                asked_at: self.asked_at,
            });
            self.bodies.push(Some(Function {
                name: format!("{}_body", self.sigs[id].name),
                params: params.clone(),
                ret: ret.clone(),
                locals: body_cx.locals,
                body: Block {
                    stmts: std::mem::take(&mut out),
                },
            }));
            let mut slots = Vec::new();
            for (param, written) in self.sigs[id].params.iter().zip(&decl.params) {
                let slot = self.cx.locals.len();
                self.cx.scopes[0].insert(param.name.clone(), slot);
                self.cx.locals.push(Local {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                });
                slots.push((slot, written.ty.as_ref()));
            }
            let args = slots
                .iter()
                .map(|(slot, _)| Arg::Value(self.local(*slot, decl.name.span)))
                .collect();
            let call = self.mk(ExprKind::Call(body_id, args), ret.clone(), decl.name.span);
            out = match self.memo_into.get(&id).cloned() {
                Some(table) => self.memo_into_wrapper(&table, decl, call, &ret),
                None => self.memo_wrapper(
                    memo::MemoHome::Global,
                    &slots,
                    &ret,
                    call,
                    decl.name.span,
                    &mut Vec::new(),
                ),
            };
            // A flat table is read by a short fn the C++ compiler puts
            // inline at each call; filling a cell is a fn of its own.
            if let Some(fast) = self.memo_fast.take() {
                let fresh = FnCx {
                    ret: Some(ret.clone()),
                    scopes: vec![HashMap::new()],
                    local_syms: vec![HashMap::new()],
                    ..FnCx::default()
                };
                let slow_cx = std::mem::replace(&mut self.cx, fresh);
                let slow_id = self.sigs.len();
                self.sigs.push(FnSig {
                    name: format!("{}_fill", self.sigs[id].name),
                    params: Vec::new(),
                    ret: Some(ret.clone()),
                    assumed_unit: false,
                    owner: None,
                    receiver: None,
                    mod_id: self.sigs[id].mod_id,
                    env: self.sigs[id].env.clone(),
                    decl,
                    offset: self.sigs[id].offset,
                    is_generator: false,
                    new_domain_slots: Vec::new(),
                    state: State::Done,
                    asked_at: self.asked_at,
                });
                self.bodies.push(Some(Function {
                    name: format!("{}_fill", self.sigs[id].name),
                    params: params.clone(),
                    ret: ret.clone(),
                    locals: slow_cx.locals,
                    body: Block {
                        stmts: std::mem::take(&mut out),
                    },
                }));
                for param in &self.sigs[id].params {
                    self.cx.locals.push(Local {
                        name: param.name.clone(),
                        ty: param.ty.clone(),
                    });
                }
                let args = (0..params.len())
                    .map(|slot| Arg::Value(self.local(slot, decl.name.span)))
                    .collect();
                let fill = self.mk(ExprKind::Call(slow_id, args), ret.clone(), decl.name.span);
                out = self.memo_lookup(fast, fill, &ret, decl.name.span);
            }
        }
        let cx = std::mem::replace(&mut self.cx, saved_cx);
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        self.asked_at = saved_asked;
        self.sigs[id].ret = Some(ret.clone());
        self.sigs[id].state = State::Done;
        self.bodies[id] = Some(Function {
            name: self.sigs[id].name.clone(),
            params,
            ret,
            locals: cx.locals,
            body: Block { stmts: out },
        });
    }

    /// The return type of fn `id`, reading its body now if it has no `->`.
    fn ret_of(&mut self, id: FnId, span: Span) -> Ty {
        if let Some(ty) = &self.sigs[id].ret {
            return ty.clone();
        }
        if self.sigs[id].state == State::InProgress {
            // A fn that calls itself and never writes `return value` is
            // a procedure: its calls give nothing.
            let decl = self.sigs[id].decl;
            if matches!(&decl.body, ast::FnBody::Block(block) if !returns_a_value(block)) {
                self.sigs[id].ret = Some(Ty::Unit);
                self.sigs[id].assumed_unit = true;
                return Ty::Unit;
            }
            let name = self.sigs[id].decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            self.help(format!(
                "add `-> Int`, or the type it returns, to `fn {name}`"
            ));
            return Ty::Error;
        }
        self.function(id);
        self.sigs[id].ret.clone().unwrap_or(Ty::Error)
    }

    // ----- small builders -----

    fn int_lit(&self, value: i64, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Int(value),
            ty: Ty::Int,
            span: self.span(span),
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

    fn mk(&self, kind: ExprKind, ty: Ty, span: Span) -> Expr {
        Expr {
            kind,
            ty,
            span: self.span(span),
        }
    }

    /// `e` as a `want`, converting where the language does it silently.
    fn coerce(&mut self, e: Expr, want: &Ty, span: Span) -> Expr {
        match self.try_coerce(e, want) {
            Ok(e) => e,
            Err(e) => {
                self.mismatch(span, want, &e.ty);
                self.err_expr(span)
            }
        }
    }

    /// Whether a struct is a number: it has a `zero`, a `one` and a
    /// `new(x: Int)`, so an `Int` is made into it where it is wanted.
    /// `Mod<P>` is one, and so is any struct written the same way.
    pub(super) fn is_number(&self, sid: StructId) -> bool {
        let info = &self.structs[sid];
        let made_from_int = info.methods.get("new").is_some_and(|&new_fn| {
            matches!(self.sigs[new_fn].params.as_slice(), [only] if only.ty == Ty::Int)
        });
        made_from_int && ["zero", "one"].iter().all(|c| info.consts.contains_key(*c))
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
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                match resolved {
                    Ty::Pos(id) => Some(id),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(super) fn fresh_loop(&mut self) -> LoopId {
        let id = self.next_loop;
        self.next_loop += 1;
        id
    }

    pub(super) fn push_synth_struct(&mut self, name: String, fields: Vec<(String, Ty)>) -> StructId {
        let id = self.structs.len();
        self.structs.push(StructInfo {
            def: StructDef {
                name,
                fields,
                fixed: Default::default(),
                narrow_ints: Default::default(),
                show_fn: None,
                eq_fn: None,
                lt_fn: None,
            },
            decl: None,
            mod_id: 0,
            offset: 0,
            origin_gen: None,
            env: HashMap::new(),
            methods: HashMap::new(),
            op_methods: Vec::new(),
            gen_methods: HashMap::new(),
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

    /// Whether `d` stands for the plain range that indexes an array, and
    /// this code is not the generic fn that was given it: here its
    /// positions are the plain `Int`s the array is indexed with.
    pub(super) fn is_range_domain(&self, d: DomainId) -> bool {
        self.range_domains.contains_key(&d) && !self.gen_env.values().any(|ga| *ga == GenArg::Domain(d))
    }

    /// The type as the caller of a generic fn writes it: a position or
    /// boundary of an array's plain range is an `Int`.
    pub(super) fn plain_ty(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Pos(d) | Ty::Bound(d) if self.is_range_domain(*d) => Ty::Int,
            Ty::Opt(inner) => Ty::Opt(Box::new(self.plain_ty(inner))),
            Ty::Range(inner) => Ty::Range(Box::new(self.plain_ty(inner))),
            Ty::Array { dims, elem }
                if matches!(dims.as_slice(), [Dim::Pos(d)] if self.is_range_domain(*d) && self.list_domains.contains(d)) =>
            {
                Ty::List(elem.clone())
            }
            Ty::Array { dims, elem } => Ty::Array {
                dims: dims
                    .iter()
                    .map(|dim| match dim {
                        Dim::Pos(d) if self.is_range_domain(*d) => match self.range_domains[d] {
                            0 => Dim::Int,
                            lo => Dim::IntFrom(lo),
                        },
                        other => *other,
                    })
                    .collect(),
                elem: elem.clone(),
            },
            other => other.clone(),
        }
    }

    /// The `List<T>` that was given when `want` is a `[D]T` whose `D`
    /// stands for a list's positions.
    pub(super) fn list_given(&self, want: &Ty) -> Option<Ty> {
        match want {
            Ty::Array { dims, elem } if matches!(dims.as_slice(), [Dim::Pos(d)] if self.list_domains.contains(d)) => {
                Some(Ty::List(elem.clone()))
            }
            _ => None,
        }
    }

    /// The domain and first index when an array indexed by a plain range
    /// is given where its `[D]T` is asked for.
    pub(super) fn range_domain_of(&self, want: &Ty, have: &Ty) -> Option<(DomainId, i64)> {
        let Ty::Array { dims: want_dims, elem: want_elem } = want else {
            return None;
        };
        let (lo, have_elem) = match have {
            Ty::Array { dims, elem } => match dims.as_slice() {
                [Dim::Int] => (0, elem),
                [Dim::IntFrom(lo)] => (*lo, elem),
                _ => return None,
            },
            // A list lent `mut` stays a list: see `list_given`.
            _ => return None,
        };
        match want_dims.as_slice() {
            [Dim::Pos(d)] if want_elem == have_elem && self.range_domains.get(d) == Some(&lo) => Some((*d, lo)),
            _ => None,
        }
    }

    /// What an array given for `D` does to it: the first sets its size,
    /// each one after must have that size.
    pub(super) fn size_domain(&mut self, d: DomainId, lo: i64, len: Expr, span: Span) -> Stmt {
        let first = self.call_sized.as_mut().is_none_or(|sized| sized.insert(d));
        if first {
            return Stmt::DefDomain {
                id: d,
                lo: self.int_lit(lo, span),
                len,
            };
        }
        let sized = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        let differ = self.mk(
            ExprKind::Cmp(CmpOp::Ne, Box::new(len), Box::new(sized)),
            Ty::Bool,
            span,
        );
        let name = self.domain_said(d);
        let message = format!("the arrays given for one `{name}` must have the same length");
        let trap = Stmt::Trap(message, self.span(span));
        Stmt::If(differ, Block { stmts: vec![trap] }, Block::default())
    }

    /// Between an array's plain range and the domain a generic fn knows
    /// it by: `[0..n]T` to `[D]T` (which sets the size of `D`), an `Int`
    /// to a position or boundary (checked), and each of those back.
    fn range_domain_coerce(&mut self, e: Expr, want: &Ty) -> Result<Expr, Expr> {
        // `e.span` is already where the file's text is; `mk` shifts again.
        let span = match self.cur_mod {
            0 => Span::new(e.span.start.saturating_sub(self.offset), e.span.end.saturating_sub(self.offset)),
            _ => e.span,
        };
        // A list is an array over `0..len`, and that array a list.
        let e = match (want, &e.ty) {
            (Ty::Array { dims, elem: want_elem }, Ty::List(elem) | Ty::ListView(elem)) if want_elem == elem => match dims.as_slice() {
                [Dim::Pos(d)] if self.list_domains.contains(d) => {
                    let d = *d;
                    // Passing the list sets the size of its domain.
                    return Ok(self.do_expr(Some(want), span, |lw, out| {
                        let held = lw.hold(e, out);
                        let len = lw.mk(
                            ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                            Ty::Int,
                            span,
                        );
                        let first = lw.range_domains.get(&d).copied().unwrap_or(0);
                        let size = lw.size_domain(d, first, len, span);
                        out.push(size);
                        let array = ExprKind::Builtin(Builtin::ListToArray, vec![held]);
                        out.push(Stmt::Give(lw.mk(array, want.clone(), span)));
                    }));
                }
                _ => e,
            },
            (Ty::List(want_elem), Ty::Array { dims, elem })
                if want_elem == elem && matches!(dims.as_slice(), [Dim::Pos(d)] if self.list_domains.contains(d)) =>
            {
                let list = ExprKind::Builtin(Builtin::ArrayToList, vec![e]);
                return Ok(self.mk(list, want.clone(), span));
            }
            _ => e,
        };
        let have = e.ty.clone();
        match (want, &have) {
            (Ty::Pos(d), Ty::Int) if self.range_domains.contains_key(d) => Ok(self.cast(Cast::FromLabel(*d), e, want.clone())),
            (Ty::Int, Ty::Pos(d)) if self.range_domains.contains_key(d) => Ok(self.cast(Cast::Label(*d), e, Ty::Int)),
            (Ty::Int, Ty::Bound(d)) if self.range_domains.contains_key(d) => {
                let ord = self.cast(Cast::Ord, e, Ty::Int);
                let lo = self.mk(ExprKind::DomainLo(*d), Ty::Int, span);
                Ok(self.mk(ExprKind::Binary(BinOp::Add, Box::new(ord), Box::new(lo)), Ty::Int, span))
            }
            (Ty::Bound(d), Ty::Int) if self.range_domains.contains_key(d) => {
                let d = *d;
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let lo = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                    let ord = lw.mk(ExprKind::Binary(BinOp::Sub, Box::new(e), Box::new(lo)), Ty::Int, span);
                    let slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(slot, Some(ord)));
                    let below = lw.mk(
                        ExprKind::Cmp(CmpOp::Lt, Box::new(lw.local(slot, span)), Box::new(lw.int_lit(0, span))),
                        Ty::Bool,
                        span,
                    );
                    let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let above = lw.mk(
                        ExprKind::Cmp(CmpOp::Lt, Box::new(len), Box::new(lw.local(slot, span))),
                        Ty::Bool,
                        span,
                    );
                    let outside = lw.mk(ExprKind::Binary(BinOp::Or, Box::new(below), Box::new(above)), Ty::Bool, span);
                    let trap = Stmt::Trap("a boundary outside the array's range".to_string(), lw.span(span));
                    out.push(Stmt::If(outside, Block { stmts: vec![trap] }, Block::default()));
                    let bound = lw.cast(Cast::Trust, lw.local(slot, span), Ty::Bound(d));
                    out.push(Stmt::Give(bound));
                }))
            }
            (Ty::Range(want_of), Ty::Range(have_of)) => {
                let (d, into) = match (&**want_of, &**have_of) {
                    (Ty::Pos(d), Ty::Int) if self.range_domains.contains_key(d) => (*d, true),
                    (Ty::Int, Ty::Pos(d)) if self.range_domains.contains_key(d) => (*d, false),
                    _ => return Err(e),
                };
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let held = lw.hold(e, out);
                    let mut end = |lw: &mut Self, which: Builtin| {
                        let at = lw.mk(ExprKind::Builtin(which, vec![held.clone()]), Ty::Int, span);
                        let first = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                        let op = if into { BinOp::Sub } else { BinOp::Add };
                        let moved = lw.mk(ExprKind::Binary(op, Box::new(at), Box::new(first)), Ty::Int, span);
                        let slot = lw.temp(Ty::Int);
                        out.push(Stmt::Let(slot, Some(moved)));
                        lw.local(slot, span)
                    };
                    let (lo, hi) = (end(lw, Builtin::RangeLo), end(lw, Builtin::RangeHi));
                    if into {
                        // An empty range reads nothing, wherever it is.
                        let cmp = |lw: &Self, a: Expr, b: Expr| {
                            lw.mk(ExprKind::Cmp(CmpOp::Lt, Box::new(a), Box::new(b)), Ty::Bool, span)
                        };
                        let or = |lw: &Self, op: BinOp, a: Expr, b: Expr| {
                            lw.mk(ExprKind::Binary(op, Box::new(a), Box::new(b)), Ty::Bool, span)
                        };
                        let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let outside = or(
                            lw,
                            BinOp::Or,
                            cmp(lw, lo.clone(), lw.int_lit(0, span)),
                            cmp(lw, len, hi.clone()),
                        );
                        let wrong = or(lw, BinOp::And, cmp(lw, lo.clone(), hi.clone()), outside);
                        let trap = Stmt::Trap("a range outside the array's range".to_string(), lw.span(span));
                        out.push(Stmt::If(wrong, Block { stmts: vec![trap] }, Block::default()));
                    }
                    let range = ExprKind::Range {
                        lo: Box::new(lo),
                        hi: Box::new(hi),
                    };
                    out.push(Stmt::Give(lw.mk(range, want.clone(), span)));
                }))
            }
            (Ty::Opt(want_inner), Ty::Opt(have_inner))
                if want_inner != have_inner && self.plain_ty(want_inner) == self.plain_ty(have_inner) =>
            {
                let (want_inner, have) = ((**want_inner).clone(), have.clone());
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let slot = lw.temp(have);
                    out.push(Stmt::Let(slot, Some(e)));
                    let held = lw.local(slot, span);
                    let inner_ty = match &held.ty {
                        Ty::Opt(inner) => (**inner).clone(),
                        _ => Ty::Error,
                    };
                    let inner = lw.mk(ExprKind::Unwrap(Box::new(held.clone()), false), inner_ty, span);
                    let inner = lw.coerce(inner, &want_inner, span);
                    let some = lw.mk(ExprKind::Some(Box::new(inner)), want.clone(), span);
                    let none = lw.mk(ExprKind::None, want.clone(), span);
                    let is_some = lw.mk(ExprKind::IsSome(Box::new(held)), Ty::Bool, span);
                    out.push(Stmt::Give(lw.mk(
                        ExprKind::If(Box::new(is_some), Box::new(some), Box::new(none)),
                        want.clone(),
                        span,
                    )));
                }))
            }
            (Ty::Array { dims: want_dims, elem: want_elem }, Ty::Array { dims: have_dims, elem: have_elem })
                if want_elem == have_elem && want_dims != have_dims && want_dims.len() == have_dims.len() =>
            {
                let first = |dim: &Dim| match dim {
                    Dim::Int => Some(0),
                    Dim::IntFrom(lo) => Some(*lo),
                    _ => None,
                };
                // To the domain sets its size; only a 1-D array has one.
                let mut sized = None;
                for (w, h) in want_dims.iter().zip(have_dims) {
                    match (w, h) {
                        _ if w == h => {}
                        (Dim::Pos(d), h)
                            if want_dims.len() == 1
                                && self.range_domains.get(d).is_some_and(|lo| first(h) == Some(*lo)) =>
                        {
                            sized = Some((*d, self.range_domains[d]));
                        }
                        (w, Dim::Pos(d))
                            if self.range_domains.get(d).is_some_and(|lo| first(w) == Some(*lo)) => {}
                        _ => return Err(e),
                    }
                }
                let Some((d, lo)) = sized else {
                    return Ok(Expr {
                        ty: want.clone(),
                        ..e
                    });
                };
                Ok(self.do_expr(Some(want), span, |lw, out| {
                    let held = lw.hold(e, out);
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                        Ty::Int,
                        span,
                    );
                    let size = lw.size_domain(d, lo, len, span);
                    out.push(size);
                    out.push(Stmt::Give(Expr {
                        ty: want.clone(),
                        ..held
                    }));
                }))
            }
            _ => Err(e),
        }
    }

    pub(super) fn try_coerce(&mut self, e: Expr, want: &Ty) -> Result<Expr, Expr> {
        if e.ty == *want || e.ty == Ty::Error || *want == Ty::Error {
            return Ok(e);
        }
        let span = e.span;
        let e = match self.range_domain_coerce(e, want) {
            Ok(done) => return Ok(done),
            Err(e) => e,
        };
        match (want, &e.ty) {
            (Ty::Str, Ty::StrView) => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ToStr, vec![e]),
                ty: Ty::Str,
                span,
            }),
            (Ty::List(want_elem), Ty::ListView(got_elem)) if want_elem == got_elem => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::Pos(d), Ty::Tuple(ts)) if self.product_domains.get(d).is_some_and(|fs| fs.len() == ts.len()) => {
                let d = *d;
                let ts = ts.clone();
                let mut stmts = Vec::new();
                let held = self.hold(e, &mut stmts);
                let parts: Vec<Expr> = ts
                    .into_iter()
                    .enumerate()
                    .map(|(i, fty)| self.mk(ExprKind::Field(Box::new(held.clone()), i), fty, span))
                    .collect();
                let encoded = self.encode_product(d, parts, span);
                if stmts.is_empty() {
                    Ok(encoded)
                } else {
                    stmts.push(Stmt::Give(encoded));
                    Ok(self.mk(ExprKind::Do(Block { stmts }), Ty::Pos(d), span))
                }
            }
            (Ty::Pos(d), Ty::Record(rfs)) if self.product_domains.get(d).is_some_and(|fs| fs.len() == rfs.len()) => {
                let d = *d;
                let rfs = rfs.clone();
                let mut stmts = Vec::new();
                let held = self.hold(e, &mut stmts);
                let parts: Vec<Expr> = rfs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, fty))| self.mk(ExprKind::Field(Box::new(held.clone()), i), fty, span))
                    .collect();
                let encoded = self.encode_product(d, parts, span);
                if stmts.is_empty() {
                    Ok(encoded)
                } else {
                    stmts.push(Stmt::Give(encoded));
                    Ok(self.mk(ExprKind::Do(Block { stmts }), Ty::Pos(d), span))
                }
            }
            (Ty::Tuple(ts), Ty::Pos(d))
                if self.anon_product_domain_set.contains(d)
                    && self.product_domains.get(d).is_some_and(|fs| fs.len() == ts.len()) =>
            {
                let d = *d;
                let mut stmts = Vec::new();
                let parts = self.decode_product(d, e, span, &mut stmts);
                let tup = self.mk(ExprKind::Tuple(parts), want.clone(), span);
                if stmts.is_empty() {
                    Ok(tup)
                } else {
                    stmts.push(Stmt::Give(tup));
                    Ok(self.mk(ExprKind::Do(Block { stmts }), want.clone(), span))
                }
            }
            (Ty::Record(rfs), Ty::Pos(d))
                if self.anon_product_domain_set.contains(d)
                    && self.product_domains.get(d).is_some_and(|fs| fs.len() == rfs.len()) =>
            {
                let d = *d;
                let mut stmts = Vec::new();
                let parts = self.decode_product(d, e, span, &mut stmts);
                let rec = self.mk(ExprKind::Tuple(parts), want.clone(), span);
                if stmts.is_empty() {
                    Ok(rec)
                } else {
                    stmts.push(Stmt::Give(rec));
                    Ok(self.mk(ExprKind::Do(Block { stmts }), want.clone(), span))
                }
            }
            (Ty::Pos(d), Ty::DomainVal) => Ok(Expr {
                kind: ExprKind::Cast(
                    Cast::Trust,
                    Box::new(Expr {
                        kind: ExprKind::Int(0),
                        ty: Ty::Int,
                        span,
                    }),
                ),
                ty: Ty::Pos(*d),
                span,
            }),
            // An `Int` where a number struct is wanted is made into it
            // (decision 3.10): `var ans: Z = 1` is `Z(1)`.
            (Ty::Struct(sid), Ty::Int) if self.is_number(*sid) => {
                match self.structs[*sid].methods.get("new").copied() {
                    Some(new_fn) => {
                        let ret = self.ret_of(new_fn, span);
                        Ok(Expr {
                            kind: ExprKind::Call(new_fn, vec![Arg::Value(e)]),
                            ty: ret,
                            span,
                        })
                    }
                    None => Err(e),
                }
            }
            (Ty::I128, Ty::Int) => Ok(self.cast(Cast::IntToI128, e, Ty::I128)),
            (Ty::U64, Ty::Int) => Ok(self.cast(Cast::IntToU64, e, Ty::U64)),
            (Ty::Float, Ty::Int) => Ok(match e.kind {
                ExprKind::Int(v) => Expr {
                    kind: ExprKind::Float(v as f64),
                    ty: Ty::Float,
                    span,
                },
                _ => Expr {
                    kind: ExprKind::Cast(Cast::IntToFloat, Box::new(e)),
                    ty: Ty::Float,
                    span,
                },
            }),
            (Ty::Opt(inner), _) if !matches!(e.kind, ExprKind::None) => {
                let inner = self.try_coerce(e, inner)?;
                Ok(Expr {
                    kind: ExprKind::Some(Box::new(inner)),
                    ty: want.clone(),
                    span,
                })
            }
            (Ty::List(elem), Ty::Str) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::StrToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::Str, Ty::List(elem)) if **elem == Ty::Char => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ListToStr, vec![e]),
                ty: Ty::Str,
                span,
            }),
            // `place(D.all)` where a `List<D>` is wanted: a range written
            // out is the list of what it goes over.
            (Ty::List(elem), Ty::Range(of)) if elem == of && matches!(e.kind, ExprKind::Range { .. }) => {
                let ExprKind::Range { lo, hi } = e.kind else { unreachable!() };
                Ok(self.range_list(*lo, *hi, (**elem).clone(), span))
            }
            // `V.all` (or any `Range<V>`) where a `Bitset<V>` is wanted.
            (Ty::Struct(sid), Ty::Range(of))
                if matches!(of.as_ref(), Ty::Pos(dom) if self.is_bitset_of(*sid, *dom)) =>
            {
                let Ty::Pos(dom) = of.as_ref() else { unreachable!() };
                let dom = *dom;
                let sid = *sid;
                if let ExprKind::Range { lo, hi } = &e.kind {
                    if matches!(lo.kind, ExprKind::Int(0))
                        && matches!(hi.kind, ExprKind::DomainLen(d) if d == dom)
                    {
                        return Ok(self.domain_bitset_full(dom, span));
                    }
                }
                let Some(&new_fn) = self.structs[sid].methods.get("new") else {
                    return Err(e);
                };
                let Some(&ins_fn) = self.structs[sid].methods.get("insert") else {
                    return Err(e);
                };
                self.queue_fn(new_fn);
                self.queue_fn(ins_fn);
                let bs_ty = Ty::Struct(sid);
                Ok(self.do_expr(Some(&bs_ty), span, |lw, out| {
                    let bs = lw.temp(bs_ty.clone());
                    let init = lw.mk(ExprKind::Call(new_fn, Vec::new()), bs_ty.clone(), span);
                    out.push(Stmt::Let(bs, Some(init)));
                    let v_slot = lw.temp(Ty::Pos(dom));
                    let iter = match e.kind {
                        ExprKind::Range { lo, hi } => Iter::Range {
                            lo: *lo,
                            hi: *hi,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        _ => {
                            let r = lw.hold(e, out);
                            let lo = lw.mk(ExprKind::Field(Box::new(r.clone()), 0), Ty::Pos(dom), span);
                            let hi = lw.mk(ExprKind::Field(Box::new(r), 1), Ty::Pos(dom), span);
                            Iter::Range {
                                lo,
                                hi,
                                step: None,
                                rev: false,
                                closed: false,
                            }
                        }
                    };
                    let place = Place {
                        kind: PlaceKind::Local(bs),
                        ty: bs_ty.clone(),
                        span: lw.span(span),
                    };
                    let call = lw.mk(
                        ExprKind::Call(ins_fn, vec![Arg::Mut(place), Arg::Value(lw.local(v_slot, span))]),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.fresh_loop();
                    out.push(Stmt::For(lid, v_slot, iter, Block { stmts: vec![Stmt::Expr(call)] }));
                    out.push(Stmt::Give(lw.local(bs, span)));
                }))
            }
            (Ty::List(elem), Ty::Array { elem: aelem, .. }) if *elem == *aelem => Ok(Expr {
                kind: ExprKind::Builtin(Builtin::ArrayToList, vec![e]),
                ty: want.clone(),
                span,
            }),
            (Ty::HashMap(..), Ty::Map(..)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::HashSet(_), Ty::Set(_)) if matches!(&e.kind, ExprKind::Tuple(v) if v.is_empty()) => {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::List(w_elem), Ty::List(e_elem)) if w_elem != e_elem => {
                // Element-wise list coercion (e.g. `List<Int>` -> `List<?Int>`, `List<(Char, Int)>` -> `List<(val: Char, len: Int)>`).
                let can_elem = matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Opt(inner), other) if **inner == *other
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Record(wf), Ty::Tuple(et)) if wf.len() == et.len() && wf.iter().zip(et).all(|((_, a), b)| a == b)
                ) || matches!(
                    (&**w_elem, &**e_elem),
                    (Ty::Tuple(wt), Ty::Record(ef)) if wt.len() == ef.len() && wt.iter().zip(ef).all(|(a, (_, b))| a == b)
                );
                if !can_elem {
                    return Err(e);
                }
                let dst_ty = want.clone();
                let w_elem = (**w_elem).clone();
                let e_elem = (**e_elem).clone();
                Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                    let src_ref = lw.hold(e, out);
                    let acc_slot = lw.temp(dst_ty.clone());
                    out.push(Stmt::Let(
                        acc_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), dst_ty.clone(), span)),
                    ));
                    let item_slot = lw.temp(e_elem.clone());
                    let item_expr = lw.local(item_slot, span);
                    let coerced = lw.try_coerce(item_expr, &w_elem).unwrap_or_else(|x| x);
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc_slot),
                                ty: dst_ty.clone(),
                                span,
                            }),
                            vec![coerced],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        item_slot,
                        Iter::Seq(src_ref, false),
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                }))
            }
            (
                Ty::Array {
                    dims: w_dims,
                    elem: w_elem,
                },
                Ty::Array {
                    dims: e_dims,
                    elem: e_elem,
                },
            ) if w_dims == e_dims && w_dims.len() == 1 => {
                // Structural record projection for 1D arrays, e.g. `[E](u: V, v: V, w: Int)` -> `[E](u: V, v: V)`.
                if let (Ty::Record(w_fields), Ty::Record(e_fields)) = (&**w_elem, &**e_elem) {
                    let indices: Option<Vec<usize>> = w_fields
                        .iter()
                        .map(|(wn, wt)| {
                            e_fields
                                .iter()
                                .position(|(en, et)| en == wn && et == wt)
                        })
                        .collect();
                    if let Some(indices) = indices {
                        let dim = w_dims[0];
                        let dst_ty = want.clone();
                        let w_rec = (**w_elem).clone();
                        let e_rec = (**e_elem).clone();
                        return Ok(self.do_expr(Some(&dst_ty), span, |lw, out| {
                            let src = lw.hold(e, out);
                            let dim_spec = match dim {
                                Dim::Pos(d) => DimSpec::Pos(d),
                                Dim::Bound(d) => DimSpec::Bound(d),
                                Dim::Sym(d) => DimSpec::Sym(d),
                                Dim::Enum(e) => DimSpec::Enum(e),
                                Dim::Int | Dim::IntFrom(_) => {
                                    let sref = src.clone();
                                    DimSpec::Int(Box::new(lw.len_of(sref, span)))
                                }
                            };
                            let arr_slot = lw.temp(dst_ty.clone());
                            out.push(Stmt::Let(
                                arr_slot,
                                Some(lw.mk(ExprKind::Fill(vec![dim_spec], None), dst_ty.clone(), span)),
                            ));
                            let idx_ty = match dim {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Enum(e) => Ty::Enum(e),
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                            };
                            let idx_slot = lw.temp(idx_ty.clone());
                            let idx_expr = lw.local(idx_slot, span);
                            let sref = src.clone();
                            let elem_expr = lw.mk(
                                ExprKind::Index(Box::new(sref.clone()), vec![idx_expr.clone()]),
                                e_rec.clone(),
                                span,
                            );
                            let elem_slot = lw.temp(e_rec);
                            let el_ref = lw.local(elem_slot, span);
                            let proj_fields: Vec<Expr> = indices
                                .iter()
                                .zip(w_fields)
                                .map(|(&fi, (_, ft))| {
                                    lw.mk(ExprKind::Field(Box::new(el_ref.clone()), fi), ft.clone(), span)
                                })
                                .collect();
                            let proj_val = lw.mk(ExprKind::Tuple(proj_fields), w_rec.clone(), span);
                            let set_stmt = Stmt::Assign(
                                Place {
                                    kind: PlaceKind::Index(
                                        Box::new(Place {
                                            kind: PlaceKind::Local(arr_slot),
                                            ty: dst_ty.clone(),
                                            span,
                                        }),
                                        vec![idx_expr],
                                    ),
                                    ty: w_rec,
                                    span,
                                },
                                AssignOp::Set,
                                proj_val,
                            );
                            let iter = match dim {
                                Dim::Pos(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Bound(d) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.int_add(lw.mk(ExprKind::DomainLen(d), Ty::Int, span), 1),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Enum(e) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.int_lit(lw.enums[e].variants.len() as i64, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Iter::Range {
                                    lo: lw.int_lit(0, span),
                                    hi: lw.len_of(sref, span),
                                    step: None,
                                    rev: false,
                                    closed: false,
                                },
                            };
                            let lid = lw.next_loop;
                            lw.next_loop += 1;
                            out.push(Stmt::For(
                                lid,
                                idx_slot,
                                iter,
                                Block {
                                    stmts: vec![Stmt::Let(elem_slot, Some(elem_expr)), set_stmt],
                                },
                            ));
                            out.push(Stmt::Give(lw.local(arr_slot, span)));
                        }));
                    }
                }
                Err(e)
            }
            (Ty::Record(fields), Ty::Tuple(items))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Tuple(items), Ty::Record(fields))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            (Ty::Record(f1), Ty::Record(f2))
                if f1.len() == f2.len()
                    && f1.iter().zip(f2).all(|((_, a), (_, b))| a == b) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            // Structural struct subtyping (e.g. `Group<T>` -> `Monoid<T>` -> `Semigroup<T>`).
            (Ty::Struct(want_id), Ty::Struct(have_id)) => {
                if let Some(projected) = self.coerce_struct_projection(e.clone(), *have_id, *want_id) {
                    Ok(projected)
                } else {
                    Err(e)
                }
            }
            _ => Err(e),
        }
    }
}

fn collect_block_calls(block: &Block, out: &mut Vec<FnId>) {
    for stmt in &block.stmts {
        collect_stmt_calls(stmt, out);
    }
}

fn collect_stmt_calls(stmt: &Stmt, out: &mut Vec<FnId>) {
    match stmt {
        Stmt::Let(_, init) => {
            if let Some(e) = init {
                collect_expr_calls(e, out);
            }
        }
        Stmt::Assign(place, _, value) => {
            collect_place_calls(place, out);
            collect_expr_calls(value, out);
        }
        Stmt::Expr(e) | Stmt::Give(e) => collect_expr_calls(e, out),
        Stmt::Return(value) => {
            if let Some(e) = value {
                collect_expr_calls(e, out);
            }
        }
        Stmt::If(cond, then, otherwise) => {
            collect_expr_calls(cond, out);
            collect_block_calls(then, out);
            collect_block_calls(otherwise, out);
        }
        Stmt::While(_, cond, body) => {
            collect_expr_calls(cond, out);
            collect_block_calls(body, out);
        }
        Stmt::For(_, _, iter, body) => {
            match iter {
                Iter::Range { lo, hi, step, .. } => {
                    collect_expr_calls(lo, out);
                    collect_expr_calls(hi, out);
                    if let Some(s) = step {
                        collect_expr_calls(s, out);
                    }
                }
                Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => {
                    collect_expr_calls(e, out)
                }
            }
            collect_block_calls(body, out);
        }
        Stmt::Assert { cond, message, .. } => {
            collect_expr_calls(cond, out);
            if let Some(m) = message {
                collect_expr_calls(m, out);
            }
        }
        Stmt::DefDomain { lo, len, .. } => {
            collect_expr_calls(lo, out);
            collect_expr_calls(len, out);
        }
        Stmt::DefDomainFromVal { val, .. } => collect_expr_calls(val, out),
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
    }
}

fn collect_place_calls(place: &Place, out: &mut Vec<FnId>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => collect_place_calls(base, out),
        PlaceKind::Index(base, indices) => {
            collect_place_calls(base, out);
            for idx in indices {
                collect_expr_calls(idx, out);
            }
        }
    }
}

fn collect_spec_calls(spec: &Spec, out: &mut Vec<FnId>) {
    match spec {
        Spec::Plain(_) => {}
        Spec::Call(call) => collect_expr_calls(call, out),
        Spec::Int { lo, hi, .. } => {
            if let Some(e) = lo {
                collect_expr_calls(e, out);
            }
            if let Some(e) = hi {
                collect_expr_calls(e, out);
            }
        }
        Spec::Fields(_, fields) => {
            for f in fields {
                collect_spec_calls(f, out);
            }
        }
        Spec::Array { dims, elem } => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            collect_spec_calls(elem, out);
        }
        Spec::Opt(inner, absent) => {
            collect_spec_calls(inner, out);
            if let Some(absent) = absent {
                collect_expr_calls(absent, out);
            }
        }
    }
}

fn collect_expr_calls(e: &Expr, out: &mut Vec<FnId>) {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::Str(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_)
        | ExprKind::DomainLen(_)
        | ExprKind::DomainLo(_)
        | ExprKind::DomainVal(_)
        | ExprKind::Closure(_)
        | ExprKind::Error => {}
        ExprKind::Do(block) => collect_block_calls(block, out),
        ExprKind::Some(a)
        | ExprKind::Unary(_, a)
        | ExprKind::Cast(_, a)
        | ExprKind::Field(a, _)
        | ExprKind::IsVariant(a, _)
        | ExprKind::Payload(a, _, _)
        | ExprKind::IsSome(a)
        | ExprKind::Unwrap(a, _) => collect_expr_calls(a, out),
        ExprKind::Binary(_, a, b)
        | ExprKind::Cmp(_, a, b)
        | ExprKind::Range { lo: a, hi: b } => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
        }
        ExprKind::RangeCheck { value, lo, hi, .. } => {
            collect_expr_calls(value, out);
            if let Some(lo) = lo {
                collect_expr_calls(lo, out);
            }
            if let Some(hi) = hi {
                collect_expr_calls(hi, out);
            }
        }
        ExprKind::If(a, b, c) => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
            collect_expr_calls(c, out);
        }
        ExprKind::Index(base, indices) => {
            collect_expr_calls(base, out);
            for i in indices {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
            for i in items {
                collect_expr_calls(i, out);
            }
        }
        ExprKind::Call(fid, args) => {
            out.push(*fid);
            for a in args {
                match a {
                    Arg::Value(v) => collect_expr_calls(v, out),
                    Arg::Mut(p) => collect_place_calls(p, out),
                }
            }
        }
        ExprKind::CallExpr(callee, args) => {
            collect_expr_calls(callee, out);
            for a in args {
                match a {
                    Arg::Value(v) => collect_expr_calls(v, out),
                    Arg::Mut(p) => collect_place_calls(p, out),
                }
            }
        }
        ExprKind::Mutate(_, place, args) => {
            collect_place_calls(place, out);
            for a in args {
                collect_expr_calls(a, out);
            }
        }
        ExprKind::Sort(place, sort_by, desc) => {
            collect_place_calls(place, out);
            if let Some(desc) = desc {
                collect_expr_calls(desc, out);
            }
            match sort_by {
                SortBy::Natural => {}
                SortBy::Key(_, key) => collect_expr_calls(key, out),
                SortBy::Fn { func, .. } => collect_expr_calls(func, out),
            }
        }
        ExprKind::Read(spec) => collect_spec_calls(spec, out),
        ExprKind::Print(args, end) => {
            for a in args {
                collect_expr_calls(&a.value, out);
            }
            if let Some(end) = end {
                collect_expr_calls(end, out);
            }
        }
        ExprKind::Debug(_, args) => {
            for a in args {
                collect_expr_calls(&a.value, out);
            }
        }
        ExprKind::Format(pieces) => {
            for p in pieces {
                match p {
                    FmtPiece::Value(a) => collect_expr_calls(&a.value, out),
                    FmtPiece::Number(value, _) => collect_expr_calls(value, out),
                    FmtPiece::Text(_) => {}
                }
            }
        }
        ExprKind::Fill(dims, value) => {
            for d in dims {
                if let DimSpec::Int(len) = d {
                    collect_expr_calls(len, out);
                }
            }
            if let Some(v) = value {
                collect_expr_calls(v, out);
            }
        }
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
