//! `cpl lint`: code that is legal but probably wrong, slow or unidiomatic
//! (`plan/45-dev-tools.md`).
//!
//! A lint never blocks a build. Each has a stable code, a default level, a
//! message that says what to write instead, and a fix where one is safe:
//!
//! ```text
//! warning[L0412]: `Map<V, Int>` is keyed by the domain `V`
//!   --> d.cp:7:11
//!   = help: `[V]?Int` uses one slot per `V` and has no lookup cost
//! ```
//!
//! [`lint`] is the whole interface: source and a [`Config`] in, diagnostics
//! out. Levels come from `[lints]` in `cp.toml` and from `@allow(name)` on
//! an item.

mod config;
mod rules;
mod walk;

pub mod cli;

pub use config::{Config, ConfigError};

use cpl_diag::{Code, Diagnostic, Severity, Span};
use cpl_syntax::ast::{Block, ExprKind, Stmt, StmtKind};
use cpl_syntax::ParsedItem;
use walk::Visitor;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// What happens when a lint fires.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    /// Nothing is reported.
    Allow,
    /// A warning.
    Warn,
    /// An error: `cpl lint` exits 1.
    Deny,
}

impl Level {
    pub fn parse(text: &str) -> Option<Level> {
        match text {
            "allow" => Some(Level::Allow),
            "warn" => Some(Level::Warn),
            "deny" => Some(Level::Deny),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Level::Allow => "allow",
            Level::Warn => "warn",
            Level::Deny => "deny",
        }
    }
}

/// One lint: its stable code, the name `@allow` and `cp.toml` use, and
/// what it looks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lint {
    pub code: Code,
    pub name: &'static str,
    pub default: Level,
    pub title: &'static str,
    pub summary: &'static str,
    pub explanation: &'static str,
    pub trigger: &'static str,
    pub fix: &'static str,
    pub near_miss: &'static str,
}

/// A lint whose explanation and examples are its record, the files in
/// `spec/lints/<name>/`.
macro_rules! lint {
    ($code:literal, $name:ident, $level:ident, $title:literal, $summary:literal $(,)?) => {
        Lint {
            code: Code($code),
            name: stringify!($name),
            default: Level::$level,
            title: $title,
            summary: $summary,
            explanation: include_str!(concat!(
                "../../../spec/lints/",
                stringify!($name),
                "/explain.md"
            )),
            trigger: include_str!(concat!(
                "../../../spec/lints/",
                stringify!($name),
                "/before.cp"
            )),
            fix: include_str!(concat!(
                "../../../spec/lints/",
                stringify!($name),
                "/after.cp"
            )),
            near_miss: include_str!(concat!(
                "../../../spec/lints/",
                stringify!($name),
                "/near_miss.cp"
            )),
        }
    };
}

pub const UNKNOWN_LINT: Lint = lint!(
    "L0001",
    unknown_lint,
    Warn,
    "Unknown lint name",
    "`@allow` or `cp.toml` names a lint that does not exist",
);
pub const POSITION_ARITHMETIC: Lint = lint!(
    "L0101",
    position_arithmetic,
    Warn,
    "Position arithmetic via `.ord`",
    "`V(v.ord + 1)` builds a position from a number",
);
pub const VAR_NEVER_REASSIGNED: Lint = lint!(
    "L0201",
    var_never_reassigned,
    Warn,
    "Mutable binding never reassigned",
    "a `var` that is never changed",
);
pub const UNREAD_FIELD: Lint = lint!(
    "L0202",
    unread_field,
    Warn,
    "Struct field never read",
    "a struct field that is never read",
);
pub const SHADOWS_TOP_LEVEL: Lint = lint!(
    "L0203",
    shadows_top_level,
    Warn,
    "Local shadows top-level binding",
    "a local name that hides a top-level `let`",
);
pub const TURBOFISH_TYPE_ARGS: Lint = lint!(
    "L0204",
    turbofish_type_args,
    Warn,
    "Turbofish `::<...>` on generic call",
    "`f::<T>(...)`: canonical generic call spelling is `f<T>(...)`",
);
pub const POW_METHOD: Lint = lint!(
    "L0205",
    pow_method,
    Warn,
    "Power written with `.pow(e)`",
    "`a.pow(e)`: power is written `a ** e`",
);
pub const GETTER_PARENS: Lint = lint!(
    "L0206",
    getter_parens,
    Warn,
    "Built-in getter called with `()`",
    "`.len()`, `.is_empty()`, `.first()`, `.last()`, `.rev()`, `.items()` are read without `()`",
);
pub const IS_IN_COND_HEADER: Lint = lint!(
    "L0207",
    is_in_cond_header,
    Warn,
    "Pattern test `x is p` in `if`/`elif`/`while` header",
    "`if`/`elif`/`while` headers bind with `let p = x`, not `x is p`",
);
pub const REDUNDANT_PRELUDE_IMPORT: Lint = lint!(
    "L0208",
    redundant_prelude_import,
    Warn,
    "`use` of a name the prelude already provides",
    "`use std.tree.{Rooted}` imports what the prelude gives every program",
);
pub const REDUNDANT_ARG_NAME: Lint = lint!(
    "L0209",
    redundant_arg_name,
    Warn,
    "Named argument or record field repeating its own name",
    "`f(moves: moves)` or `(val: val)` can be punned as `f(moves:)` / `(val:)`",
);
pub const FLOAT_EQUALITY: Lint = lint!(
    "L0301",
    float_equality,
    Warn,
    "Exact float equality comparison",
    "`==` or `!=` on floats",
);
pub const DOMAIN_KEYED_MAP: Lint = lint!(
    "L0412",
    domain_keyed_map,
    Warn,
    "Map or Set keyed by a domain",
    "a `Map` or `Set` keyed by a domain",
);
pub const MEMO_KEY_SPACE: Lint = lint!(
    "L0413",
    memo_key_space,
    Warn,
    "`@memo` table exceeds memory budget",
    "a `@memo` whose table does not fit in memory at the input bounds",
);
pub const UNPROVEN_LIST_INDEX: Lint = lint!(
    "L0414",
    unproven_list_index,
    Warn,
    "Unproven `List` index in nested loop",
    "a `List` index inside nested loops that is checked on every access",
);
pub const LOOP_INVARIANT_MOD_DIV: Lint = lint!(
    "L0415",
    loop_invariant_mod_div,
    Warn,
    "Loop-invariant `Mod` divisor inside loop",
    "a `Mod<P>` division by a loop-invariant value computes an O(log P) inverse every iteration",
);
pub const SEARCH_LOOP: Lint = lint!(
    "L0501",
    search_loop,
    Warn,
    "Flag-and-break search loop",
    "a loop that sets a flag or `?T` and breaks: `first`, `any` or `all`",
);
pub const DISCARDED_RESULT: Lint = lint!(
    "L0502",
    discarded_result,
    Warn,
    "Discarded function return value",
    "`let _ =` throws away what a fn of this file returns",
);
pub const SYMMETRIC_WRITE: Lint = lint!(
    "L0503",
    symmetric_write,
    Warn,
    "Duplicate symmetric write",
    "the same fact written for `(u, v)` and then `(v, u)`",
);
pub const LOOP_REDUCTION: Lint = lint!(
    "L0504",
    loop_reduction,
    Warn,
    "Manual accumulation loop",
    "a loop that only accumulates: `sum`, `count`, `max` or `min`",
);
pub const INDEX_ONLY_LOOP: Lint = lint!(
    "L0505",
    index_only_loop,
    Warn,
    "Index loop that only reads `xs[i]`",
    "`for i in 0..xs.len` that only reads `xs[i]`",
);
pub const REPEATED_LITERAL: Lint = lint!(
    "L0506",
    repeated_literal,
    Warn,
    "Repeated large literal",
    "a big literal (a bound or modulus) written in several places",
);
pub const PUSH_LOOP: Lint = lint!(
    "L0507",
    push_loop,
    Warn,
    "Loop that only pushes",
    "a list built by a loop that only pushes: a comprehension",
);
pub const NONE_CHECK_UNWRAP: Lint = lint!(
    "L0508",
    none_check_unwrap,
    Warn,
    "Check for `none`, then unwrap",
    "`x != none and .. x!`: the check and the unwrap say one thing twice",
);
pub const FREE_FOLD: Lint = lint!(
    "L0509",
    free_fold,
    Warn,
    "Fold or sort of a sequence written as a free function",
    "`sum(xs)`, `count(xs, p)` or `sort(mut xs)`: the method reads on the sequence",
);
pub const BY_TWIN: Lint = lint!(
    "L0510",
    by_twin,
    Warn,
    "`_by` twin instead of a `by:` option",
    "`min_by`, `max_by`, `argmax_by`, `runs_by`, `group_by`, `first_ge_by`: pass `by:` to the base operation",
);
pub const DOMAIN_SPELLING: Lint = lint!(
    "L0512",
    domain_spelling,
    Warn,
    "Non-canonical domain/sequence method spelling",
    "`D.sorted(by:)` -> `D.sort(by:)`; `xs.first(where: p)` -> `xs.first(p)`",
);
pub const PREFIX_TWIN: Lint = lint!(
    "L0513",
    prefix_twin,
    Warn,
    "Dimension-suffixed prefix or diff helper instead of `prefix`/`Prefix`/`Diff`",
    "`prefix1`/`prefix2`/`prefix_n`, `Prefix1`/`Prefix2`/`PrefixN`, `Diff1`/`Diff2`: write `prefix`, `Prefix`, or `Diff`",
);

pub const TREE_ORDER: Lint = lint!(
    "L0511",
    tree_order,
    Warn,
    "`preorder` or `postorder` of a rooted tree",
    "`t.preorder` / `t.postorder`: iterate the preorder array `t.at` (`t.at.rev` for children first)",
);
pub const HAND_ROLLED_RANGE_TREE: Lint = lint!(
    "L0514",
    hand_rolled_range_tree,
    Warn,
    "Hand-rolled Fenwick or segment tree",
    "manual BIT (`p += p & -p`) or segment-tree (`2 * p` / `2 * p + 1`) loop: use `std.fenwick` / `std.segtree` / `std.lazy`",
);
pub const SORTED_RANGE_METHOD: Lint = lint!(
    "L0515",
    sorted_range_method,
    Warn,
    "Removed bound-query method on `SortedBag` or `SortedSet`",
    "`first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range` on `SortedBag`/`SortedSet`: use `.range(r)`",
);
pub const HAND_ROLLED_STRING_ALGO: Lint = lint!(
    "L0516",
    hand_rolled_string_algo,
    Warn,
    "Hand-rolled string algorithm",
    "manual prefix-function, Z-function, polynomial hash, or trie loop: use `std.strings`",
);
pub const HAND_ROLLED_DSU: Lint = lint!(
    "L0517",
    hand_rolled_dsu,
    Warn,
    "Hand-rolled union-find (DSU)",
    "manual parent-pointer path compression: use `std.dsu`",
);
pub const HAND_ROLLED_SIEVE: Lint = lint!(
    "L0518",
    hand_rolled_sieve,
    Warn,
    "Hand-rolled prime or SPF sieve",
    "manual Eratosthenes or linear sieve loop: use `std.math.Sieve`",
);
pub const HAND_ROLLED_BINARY_LIFTING: Lint = lint!(
    "L0519",
    hand_rolled_binary_lifting,
    Warn,
    "Hand-rolled binary lifting table",
    "manual `up[k][v] = up[k - 1][up[k - 1][v]]` table: use `std.tree`",
);

pub const HAND_ROLLED_ANSWER_SEARCH: Lint = lint!(
    "L0520",
    hand_rolled_answer_search,
    Warn,
    "Hand-rolled binary search",
    "manual `while lo < hi` midpoint loop over a predicate: use `bsearch_first`/`bsearch_last`",
);
pub const HAND_ROLLED_PREFIX_SUM: Lint = lint!(
    "L0521",
    hand_rolled_prefix_sum,
    Warn,
    "Hand-rolled prefix sum",
    "manual `p[i + 1] = p[i] + x` or `p[i.r] = p[i.l] + x` loop: use `prefix(...)`",
);
pub const COPY_SORT_WRITEBACK: Lint = lint!(
    "L0522",
    copy_sort_writeback,
    Warn,
    "Copy-mutate-writeback on a slice",
    "copying `xs[l..r]` to sort/reverse/rotate and writing it back: mutate `xs[l..r]` in place",
);

/// Every lint, in code order.
pub const LINTS: &[Lint] = &[
    UNKNOWN_LINT,
    POSITION_ARITHMETIC,
    VAR_NEVER_REASSIGNED,
    UNREAD_FIELD,
    SHADOWS_TOP_LEVEL,
    TURBOFISH_TYPE_ARGS,
    POW_METHOD,
    GETTER_PARENS,
    IS_IN_COND_HEADER,
    REDUNDANT_PRELUDE_IMPORT,
    REDUNDANT_ARG_NAME,
    FLOAT_EQUALITY,
    DOMAIN_KEYED_MAP,
    MEMO_KEY_SPACE,
    UNPROVEN_LIST_INDEX,
    LOOP_INVARIANT_MOD_DIV,
    SEARCH_LOOP,
    DISCARDED_RESULT,
    SYMMETRIC_WRITE,
    LOOP_REDUCTION,
    INDEX_ONLY_LOOP,
    REPEATED_LITERAL,
    PUSH_LOOP,
    NONE_CHECK_UNWRAP,
    FREE_FOLD,
    BY_TWIN,
    TREE_ORDER,
    DOMAIN_SPELLING,
    PREFIX_TWIN,
    HAND_ROLLED_RANGE_TREE,
    SORTED_RANGE_METHOD,
    HAND_ROLLED_STRING_ALGO,
    HAND_ROLLED_DSU,
    HAND_ROLLED_SIEVE,
    HAND_ROLLED_BINARY_LIFTING,
    HAND_ROLLED_ANSWER_SEARCH,
    HAND_ROLLED_PREFIX_SUM,
    COPY_SORT_WRITEBACK,
];

/// The taste lints (`plan/PORTING.md`): code that works but reads like
/// translated C++. `cpl lint --taste` counts them per file.
pub fn is_taste(lint: &Lint) -> bool {
    lint.code.0.starts_with("L05")
}

/// The lint called `name`.
pub fn lint_named(name: &str) -> Option<&'static Lint> {
    LINTS.iter().find(|lint| lint.name == name)
}

/// What a rule found, before levels are applied.
pub(crate) struct Finding {
    pub lint: &'static Lint,
    pub diag: Diagnostic,
}

/// A parsed file as the rules see it.
pub(crate) struct File<'a> {
    pub src: &'a str,
    /// Each top-level item with the offset its spans are relative to.
    pub items: &'a [(u32, &'a ParsedItem)],
}

fn with_parsed<T: Default>(src: &str, f: impl FnOnce(&File) -> T) -> T {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return T::default();
    }
    let items: Vec<(u32, &ParsedItem)> = parsed.items.iter().map(|(o, p)| (*o, p)).collect();
    f(&File { src, items: &items })
}

impl File<'_> {
    /// Walks every item, telling `visit` which offset its spans start at.
    pub(crate) fn walk<V: Visitor>(&self, visitor: &mut V, mut set_base: impl FnMut(&mut V, u32)) {
        for (base, item) in self.items {
            set_base(visitor, *base);
            for stmt in &item.stmts {
                walk::walk_stmt(visitor, stmt);
            }
        }
    }

    /// Every top-level statement with its offset.
    pub(crate) fn top_level(&self) -> impl Iterator<Item = (u32, &Stmt)> {
        self.items
            .iter()
            .flat_map(|(base, item)| item.stmts.iter().map(move |stmt| (*base, stmt)))
    }
}

/// An arithmetic expression whose value the declared input bounds prove:
/// `n * n` after `let n: 1..=100 = read()` is in `1..=10000`. Offsets are
/// byte offsets in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenExpr {
    /// The operator.
    pub op: (u32, u32),
    /// The whole expression.
    pub expr: (u32, u32),
    pub lo: i128,
    pub hi: i128,
    /// When the value can leave `Int`: inputs in their declared ranges for
    /// which it does, found and checked by the prover (`n = 55110`).
    pub overflow: Option<String>,
}

/// A `for` loop whose iteration count the declared bounds prove.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenLoop {
    /// From `for` (or its label) up to the loop's pattern.
    pub head: (u32, u32),
    /// The most times its body runs each time the loop is reached.
    pub times: u128,
    /// The most times its body runs in all, counting the `for` loops around
    /// it; `None` when one of them, or a `while` around it, has no bound.
    pub total: Option<u128>,
    /// `times` in the source's terms: `q` for `0..q`, `n` for a domain
    /// `1..=n`, else the number.
    pub count: String,
    /// `total` the same way, as a product: `n * q`.
    pub total_count: Option<String>,
}

/// A fn, and how many loop iterations one call makes at most, by the
/// declared bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenFn {
    pub name: (u32, u32),
    /// The whole fn, from `fn` to the end of its body.
    pub body: (u32, u32),
    /// The parts of its body that run other than once per call: each loop,
    /// lambda, table or comprehension, and each nested fn.
    pub runs: Vec<Run>,
    /// The sum over its `for` loops of how often each body runs, nested
    /// loops multiplied, plus each call of a fn in the file times that fn's
    /// count; `None` when a `while`, an unbounded `for`, recursion or a call
    /// in a lambda makes the count unknown.
    pub steps: Option<u128>,
    /// `steps` in the source's terms, a sum of loop counts as for
    /// `ProvenLoop::total_count` (`n * q + n`); `None` past six terms.
    pub count: Option<String>,
    /// The callee of each call `steps` leaves out, outside nested fns: the
    /// name of a call by name to no fn in the file, the member of a method
    /// call, or the whole callee expression otherwise.
    pub callees: Vec<(u32, u32)>,
    /// Where it builds a struct declared in the file, one step each: the
    /// fields only move in.
    pub builds: Vec<(u32, u32)>,
    /// Each call of a fn in the file, whose loops `steps` counts but whose
    /// other costs it leaves to the caller.
    pub helpers: Vec<Helper>,
}

/// A call by name of a fn in the file: where, the name of the fn called,
/// and how often the call runs per call of the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Helper {
    pub at: u32,
    pub callee: (u32, u32),
    pub times: Option<u128>,
    /// `times` in the source's terms; `None` when it runs once.
    pub count: Option<String>,
}

/// A part of a fn's body and how often it runs per call of the fn: `None`
/// when unknown, `Some(0)` for a nested fn, which runs only when called.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub span: (u32, u32),
    pub times: Option<u128>,
    /// `times` in the source's terms, as `ProvenLoop::total_count`.
    pub count: Option<String>,
}

/// Each fn with a body; a file with a syntax error has none.
pub fn proven_fns(src: &str) -> Vec<ProvenFn> {
    with_parsed(src, rules::proven_fns)
}

pub fn proven_fns_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenFn> {
    rules::proven_fns(&File { src, items })
}

/// An index `i` in `xs[i]` the bounds and relations prove is in
/// `0..xs.len`, for a sequence whose length never changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenIndex {
    /// `xs[i]`, `xs` and `i`.
    pub access: (u32, u32),
    pub seq: (u32, u32),
    pub index: (u32, u32),
    /// Why: `0 <= lo <= mid < hi <= xs.len`.
    pub proof: String,
}

/// A method call's receiver, an array whose declared type bounds how many
/// cells it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenSize {
    pub receiver: (u32, u32),
    pub max: u128,
    /// `max` in the source's terms, as `ProvenLoop::total_count`: `n` for
    /// `[0..n]Int`, `R * C` for `[R, C]Int`.
    pub count: String,
}

/// Each method call on an array of bounded size, and each such array
/// passed to `print` or `ask`; a file with a syntax error has none.
pub fn proven_sizes(src: &str) -> Vec<ProvenSize> {
    with_parsed(src, rules::proven_sizes)
}

pub fn proven_sizes_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenSize> {
    rules::proven_sizes(&File { src, items })
}

/// Each list only pushed to in counted loops, with the most elements it
/// can hold: the number of pushes (decision 29.45).
pub fn list_sizes(src: &str) -> Vec<(String, u128)> {
    with_parsed(src, rules::list_sizes)
}

/// The most positions each top-level domain can have, from the bounds of
/// its range: `domain V = 0..n` after `let n: 1..=2e5 = read()` has 200000.
pub fn domain_sizes(src: &str) -> Vec<(String, u128)> {
    with_parsed(src, rules::domain_sizes)
}

pub fn domain_sizes_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<(String, u128)> {
    rules::domain_sizes(&File { src, items })
}

/// Each index proven in bounds; a file with a syntax error has none.
pub fn proven_indexes(src: &str) -> Vec<ProvenIndex> {
    with_parsed(src, rules::proven_indexes)
}

pub fn proven_indexes_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenIndex> {
    rules::proven_indexes(&File { src, items })
}

/// Each `for` over a bounded range or domain, with how often it runs.
pub fn proven_loops(src: &str) -> Vec<ProvenLoop> {
    with_parsed(src, rules::proven_loops)
}

pub fn proven_loops_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenLoop> {
    rules::proven_loops(&File { src, items })
}

/// What the declared bounds prove about each `+`, `-`, `*`, `/`, `%` and `**` outside
/// lambdas, comprehensions and match arms; a file with a syntax error has
/// none.
pub fn proven_expr_ranges(src: &str) -> Vec<ProvenExpr> {
    with_parsed(src, rules::proven_expr_ranges)
}

pub fn proven_expr_ranges_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenExpr> {
    rules::proven_expr_ranges(&File { src, items })
}

/// What the declared input bounds prove about each top-level `let` written
/// without a type: `let m = n * n` after `let n: 1..=100 = read()` is in
/// `1..=10000`. Each is the name's byte offset and the inclusive range; a file
/// with a syntax error has none.
pub fn proven_ranges(src: &str) -> Vec<(u32, i128, i128)> {
    with_parsed(src, rules::proven_ranges)
}

pub fn proven_ranges_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<(u32, i128, i128)> {
    rules::proven_ranges(&File { src, items })
}

/// Lints one file. Returns warnings (errors for `deny`), in source order.
///
/// A file with a syntax error gets no lints: what the parser could not read
/// may hold the assignment or the read a lint would say is missing.
pub fn lint(src: &str, config: &Config) -> Vec<Diagnostic> {
    with_parsed(src, |file| lint_file(file, config))
}

pub fn lint_parsed(src: &str, items: &[(u32, &ParsedItem)], config: &Config) -> Vec<Diagnostic> {
    lint_file(&File { src, items }, config)
}

fn lint_file(file: &File, config: &Config) -> Vec<Diagnostic> {
    let allows = Allows::collect(file);
    let library = file.top_level().any(|(_, stmt)| stmt.public);
    let mut fns = Fns::default();
    file.walk(&mut fns, |fns, base| fns.base = base);
    let mut findings = allows.unknown;
    findings.extend(rules::run(file, config));

    let mut out: Vec<Diagnostic> = Vec::new();
    for Finding { lint, mut diag } in findings {
        let level = config.level(lint);
        let allowed = allows
            .regions
            .iter()
            .any(|(span, name)| *name == lint.name && span.contains(diag.span));
        let library_only = library && LIBRARY_LOOPS.iter().any(|l| l.code == lint.code);
        if level == Level::Allow || allowed || library_only || fns.defines(lint, &diag) {
            continue;
        }
        diag.severity = if level == Level::Deny {
            Severity::Error
        } else {
            Severity::Warning
        };
        out.push(diag);
    }
    out.sort_by_key(|d| (d.span.start, d.span.end, d.code));
    out.dedup();
    out
}

/// Lints that point to a std structure: a library module is where the
/// loops that build one are written.
const LIBRARY_LOOPS: &[&Lint] = &[
    &HAND_ROLLED_RANGE_TREE,
    &HAND_ROLLED_SIEVE,
    &HAND_ROLLED_PREFIX_SUM,
];

/// Lints whose message names the call to write instead.
const NAME_THE_CALL: &[&Lint] = &[&FREE_FOLD, &HAND_ROLLED_ANSWER_SEARCH];

/// Each fn of a file, so a finding that says "this is `first`" is not
/// reported inside `fn first` itself.
#[derive(Default)]
struct Fns {
    base: u32,
    spans: Vec<(Span, String)>,
}

impl Visitor for Fns {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(decl) = &stmt.kind {
            let quoted = format!("`{}`", decl.name.text);
            self.spans.push((stmt.span.shift(self.base), quoted));
        }
    }
}

impl Fns {
    fn defines(&self, lint: &Lint, diag: &Diagnostic) -> bool {
        NAME_THE_CALL.iter().any(|l| l.code == lint.code)
            && self
                .spans
                .iter()
                .any(|(span, quoted)| span.contains(diag.span) && diag.message.contains(quoted))
    }
}

/// The `@allow(name)` decorators of a file.
struct Allows {
    /// The statement each one covers and the lint it turns off.
    regions: Vec<(Span, &'static str)>,
    unknown: Vec<Finding>,
}

impl Allows {
    fn collect(file: &File) -> Allows {
        struct Collector {
            base: u32,
            out: Allows,
        }
        impl Visitor for Collector {
            fn stmt(&mut self, stmt: &Stmt) {
                for decorator in stmt.decorators.iter().filter(|d| d.name.text == "allow") {
                    // A decorator sits above the statement it covers.
                    let covered = stmt.span.to(decorator.span).shift(self.base);
                    for arg in &decorator.args {
                        let ExprKind::Name(name) = &arg.value.kind else {
                            continue;
                        };
                        match lint_named(name) {
                            Some(lint) => self.out.regions.push((covered, lint.name)),
                            None => self
                                .out
                                .unknown
                                .push(unknown_lint(name, arg.value.span.shift(self.base))),
                        }
                    }
                }
            }
        }
        let mut collector = Collector {
            base: 0,
            out: Allows {
                regions: Vec::new(),
                unknown: Vec::new(),
            },
        };
        file.walk(&mut collector, |c, base| c.base = base);
        let mut contexts = Contexts {
            base: 0,
            out: &mut collector.out,
        };
        let mut leading = Vec::new();
        for (base, stmt) in file.top_level() {
            contexts.base = base;
            if matches!(stmt.kind, StmtKind::With { body: None, .. }) {
                leading.push((base, stmt));
            }
            cpl_syntax::visit::walk_stmt(stmt, &mut contexts);
        }
        // A bare line at the top of the file covers the rest of it.
        for (base, stmt) in leading {
            contexts.base = base;
            let start = stmt.span.shift(base).start;
            contexts.allow(stmt, Span::new(start, u32::MAX));
        }
        collector.out
    }
}

/// The `with lint(name: allow)` contexts of a file. (R09.4.4)
struct Contexts<'a> {
    base: u32,
    out: &'a mut Allows,
}

impl Contexts<'_> {
    fn allow(&mut self, stmt: &Stmt, covered: Span) {
        let StmtKind::With { ctx, args, .. } = &stmt.kind else {
            return;
        };
        if ctx.text != "lint" {
            return;
        }
        for arg in args {
            let Some(field) = &arg.name else { continue };
            match lint_named(&field.text) {
                Some(lint) if matches!(&arg.value.kind, ExprKind::Name(v) if v == "allow") => {
                    self.out.regions.push((covered, lint.name))
                }
                Some(_) => {}
                None => self
                    .out
                    .unknown
                    .push(unknown_lint(&field.text, field.span.shift(self.base))),
            }
        }
    }
}

impl cpl_syntax::visit::Visitor for Contexts<'_> {
    fn block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            if matches!(stmt.kind, StmtKind::With { body: None, .. }) {
                self.allow(stmt, stmt.span.to(block.span).shift(self.base));
            }
        }
    }
    fn stmt(&mut self, stmt: &Stmt) {
        if matches!(stmt.kind, StmtKind::With { body: Some(_), .. }) {
            self.allow(stmt, stmt.span.shift(self.base));
        }
    }
}

fn unknown_lint(name: &str, span: Span) -> Finding {
    let mut diag = Diagnostic::warning(
        UNKNOWN_LINT.code,
        span,
        format!("there is no lint called `{name}`"),
    );
    diag.help = Some(match closest_lint(name) {
        Some(close) => format!("did you mean `{close}`?"),
        None => "`cpl lint --list` prints every lint".to_string(),
    });
    Finding {
        lint: &UNKNOWN_LINT,
        diag,
    }
}

/// The lint name within two edits of `name`, if there is one.
pub(crate) fn closest_lint(name: &str) -> Option<&'static str> {
    LINTS
        .iter()
        .map(|lint| (edit_distance(name, lint.name), lint.name))
        .filter(|(distance, _)| *distance <= 2)
        .min()
        .map(|(_, name)| name)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut diagonal = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let substitute = diagonal + usize::from(a[i - 1] != b[j - 1]);
            diagonal = row[j];
            row[j] = substitute.min(row[j] + 1).min(row[j - 1] + 1);
        }
    }
    row[b.len()]
}

/// The removed spellings the parser offers a rewrite for (`where` as a
/// filter, `--- multitest ---`), applied to a fixed point. Parser recovery
/// guesses, such as an inserted `:`, are not among them.
pub fn fix_removed_syntax(src: &str) -> (String, usize) {
    let mut src = src.to_string();
    let mut total = 0;
    loop {
        let codemods: Vec<_> = cpl_syntax::parse_file(&src)
            .diagnostics
            .into_iter()
            .filter(|d| {
                (d.code == cpl_diag::codes::REMOVED_SYNTAX
                    || (d.code == cpl_diag::codes::UNKNOWN_CHAR
                        && d.message == "CP Lang concatenates with `+`"))
                    && !d.fixes.is_empty()
            })
            .collect();
        let (text, applied) = apply_fixes(&src, &codemods);
        if applied == 0 {
            return (src, total);
        }
        total += applied;
        src = text;
    }
}

/// Type-directed removed-syntax rewrites reported by the checker (such as
/// `Range.lo`/`.hi` -> `.start`/`.end`, leaving user struct fields named
/// `lo`/`hi` untouched), applied to a fixed point.
pub fn fix_checked_removed_syntax(path: Option<&std::path::Path>, src: &str) -> (String, usize) {
    let mut src = src.to_string();
    let mut total = 0;
    let key = path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<fix>.cp".to_string());
    let mut db = cpl_check::Database::new();
    loop {
        db.set_file(&key, src.as_str());
        let Some(analysis) = db.analyze(&key) else {
            return (src, total);
        };
        let codemods: Vec<_> = analysis
            .diagnostics
            .iter()
            .filter(|d| d.code == cpl_diag::codes::REMOVED_SYNTAX && !d.fixes.is_empty())
            .cloned()
            .collect();
        let (text, applied) = apply_fixes(&src, &codemods);
        if applied == 0 {
            return (src, total);
        }
        total += applied;
        src = text;
    }
}

/// What `cpl fix` makes of a source: removed-syntax rewrites (parser, then
/// type-directed checker rewrites), then every lint fix, each to a fixed point.
pub fn fix(src: &str, config: &Config) -> (String, usize) {
    fix_at(None, src, config)
}

/// Like [`fix`], resolving imports relative to `path` when given.
pub fn fix_at(path: Option<&std::path::Path>, src: &str, config: &Config) -> (String, usize) {
    let (mut src, mut total) = fix_removed_syntax(src);
    if !cpl_syntax::parse_file(&src).diagnostics.is_empty() {
        return (src, total);
    }
    let (checked, checked_applied) = fix_checked_removed_syntax(path, &src);
    if checked_applied > 0 {
        src = checked;
        total += checked_applied;
    }
    loop {
        let (text, applied) = apply_fixes(&src, &lint(&src, config));
        if applied == 0 {
            return (src, total);
        }
        total += applied;
        src = text;
    }
}

/// Applies the first fix of every diagnostic that has one. Fixes that
/// overlap an earlier one are left for the next run.
pub fn apply_fixes(src: &str, diagnostics: &[Diagnostic]) -> (String, usize) {
    let mut edits: Vec<(Span, &str)> = Vec::new();
    let mut applied = 0;
    let mut fixes: Vec<_> = diagnostics.iter().filter_map(|d| d.fixes.first()).collect();
    fixes.sort_by_key(|fix| fix.edits.iter().map(|(span, _)| span.start).min());
    for fix in fixes {
        let clashes = fix.edits.iter().any(|(span, _)| {
            edits
                .iter()
                .any(|(taken, _)| span.start < taken.end && taken.start < span.end)
        });
        if clashes || fix.edits.is_empty() {
            continue;
        }
        edits.extend(fix.edits.iter().map(|(span, text)| (*span, text.as_str())));
        applied += 1;
    }
    edits.sort_by_key(|(span, _)| span.start);
    let mut out = String::with_capacity(src.len());
    let mut at = 0usize;
    for (span, text) in edits {
        out.push_str(&src[at..span.start as usize]);
        out.push_str(text);
        at = span.end as usize;
    }
    out.push_str(&src[at..]);
    (out, applied)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod record_tests;

#[cfg(test)]
mod std_tests;

#[cfg(test)]
mod corpus_tests;
