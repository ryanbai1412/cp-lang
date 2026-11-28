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
//! out. Levels come from `[lints]` in `cp.toml` and from
//! `with lint(name: allow)`.

/// Where std lives, relative to the repository root: the one place
/// `cpl-lint` names it.
#[cfg(test)]
pub(crate) const STD_DIR: &str = "crates/cpl-check/frozen/std";

/// The source of one std module, `std_source!("prelude.cp")`; the path
/// repeats `STD_DIR` because `include_str!` needs a literal.
#[cfg(test)]
macro_rules! std_source {
    ($file:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../cpl-check/frozen/std/",
            $file
        ))
    };
}
#[cfg(test)]
pub(crate) use std_source;

/// `shards!(suite, check, shard_0 = 0, ...)`: tests `suite::shard_k` run
/// `check(k, n)` over every n-th file of a corpus-wide suite, so nextest
/// spreads the suite over cores and CI shards instead of one test holding a
/// core for minutes.
#[cfg(test)]
macro_rules! shards {
    ($suite:ident, $check:ident, $($shard:ident = $k:literal),+ $(,)?) => {
        mod $suite {
            const N: usize = [$($k),+].len();
            $(
                #[test]
                fn $shard() {
                    super::$check($k, N);
                }
            )+
            #[test]
            fn the_shards_are_every_index_once() {
                let ks: [usize; N] = [$($k),+];
                assert_eq!(ks, std::array::from_fn(|i| i));
            }
        }
    };
}
#[cfg(test)]
pub(crate) use shards;

mod checker;
mod config;
pub mod facts;
mod fix;
mod library;
mod lint_run;
mod recheck;
mod rules;
/// The names std declares `pub fn`, generated from `std/*.cp`: lint rules
/// name std's API through these, never a string literal.
pub mod std_fn {
    include!(concat!(env!("OUT_DIR"), "/std_fn.rs"));
}
mod typed;
mod walk;

pub mod cli;

pub use checker::{at_levels, checked_analysis, Checked, CheckerLint};
#[cfg(test)]
pub(crate) use checker::{checked, checker_lints};
pub use config::{Config, ConfigError};
pub use facts::Facts;
pub use lint_run::LintRun;
pub use typed::{lint_analysis, refine_with_analysis};

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

/// A family of lints, so a project can set one level for all of them
/// (`cp.toml [lints] style = "allow"`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Group {
    /// Probably a bug: denied by default.
    Correctness,
    /// Slower or bigger than it looks.
    Perf,
    /// The language or the stdlib has a construct for it.
    Idiom,
    /// A spelling the language writes another way.
    Style,
    /// Opinions most code can ignore: allowed by default.
    Pedantic,
}

impl Group {
    pub const ALL: [Group; 5] = [
        Group::Correctness,
        Group::Perf,
        Group::Idiom,
        Group::Style,
        Group::Pedantic,
    ];

    pub fn parse(text: &str) -> Option<Group> {
        Group::ALL.into_iter().find(|g| g.as_str() == text)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Group::Correctness => "correctness",
            Group::Perf => "perf",
            Group::Idiom => "idiom",
            Group::Style => "style",
            Group::Pedantic => "pedantic",
        }
    }
}

/// One lint: its stable code, the name `with lint` and `cp.toml` use, and
/// what it looks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lint {
    pub code: Code,
    pub name: &'static str,
    /// Its record's directory, `spec/lints/<name>`; empty for a library's lint.
    pub dir: &'static str,
    pub default: Level,
    pub group: Group,
    pub title: &'static str,
    pub summary: &'static str,
    /// The checklist rule the lint enforces, `<topic>/R<n>`, defined in
    /// `handbook/guidelines/<topic>-checklist.md`; `None` for a checker lint.
    pub rule: Option<&'static str>,
    /// Why `cpl lint --fix` (and the editor's quick fix) has no rewrite for
    /// it; `None` when it has one.
    /// The std call it names replaces its span whole, so a finding inside
    /// it is about code that goes away.
    pub replaces_whole: bool,
    /// The std call it names is a structure, which a library module is
    /// where one is written, so it is quiet there.
    pub std_structure: bool,
    /// Its message names the call to write, so it is quiet inside a fn of
    /// that name.
    pub quiet_in_namesake: bool,
    /// The types it is about, from its record's `on`: a finding stands only
    /// where the checker types its first label as one (`Ty::is_one_of`).
    pub on: &'static [&'static str],
    pub no_fix: Option<&'static str>,
    /// The checker error `trigger` shows because it uses a removed spelling;
    /// `None` when it passes `cpl check`.
    pub removed: Option<&'static str>,
    /// Where `spec/idioms/lints.md` places it, `S.seq: building`.
    pub family: Option<&'static str>,
    /// The practices of `spec/idioms/principles.md` it serves, `#2 #7`.
    pub practice: Option<&'static str>,
    pub explanation: &'static str,
    pub trigger: &'static str,
    pub fix: &'static str,
    pub near_miss: &'static str,
}

// Every lint and the `LINTS` table, generated by `build.rs` from the
// records in `spec/lints/<name>/`: `lint.toml` gives the code, default level,
// title, summary and the handbook rule it enforces; `explain.md`, `before.cp`, `after.cp` and `near_miss.cp`
// the rest.
include!(concat!(env!("OUT_DIR"), "/lints.rs"));

/// `text` with its `\\( ... \\)` math in plain Unicode for the terminal and the editor:
/// `\\(O(n \\log n)\\)` reads `O(n log n)`. Records keep the LaTeX for the site.
pub fn plain_math(text: &str) -> String {
    cpl_diag::plain::text(text)
}

/// The taste lints (`plan/PORTING.md`): code that works but reads like
/// translated C++. `cpl lint --taste` counts them per file.
pub fn is_taste(lint: &Lint) -> bool {
    lint.code.0.starts_with("L05")
}

/// The lint called `name`.
pub fn lint_named(name: &str) -> Option<&'static Lint> {
    LINTS.iter().find(|lint| lint.name == name)
}

/// The name `name` as a lint of this crate or of the checker.
pub(crate) fn any_lint_named(name: &str) -> Option<&'static str> {
    lint_named(name)
        .map(|lint| lint.name)
        .or_else(|| checker::checker_lint_named(name).map(|lint| lint.name))
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
    /// What the checker found, for rules that read types; `None` where
    /// only the syntax is at hand.
    pub facts: Option<&'a Facts>,
    /// What the declared bounds prove, worked out once for every rule.
    pub proven: &'a std::sync::OnceLock<rules::Proven>,
}

/// Whether the call or getter at `at` (a file span) is std's `name`: the
/// checker resolved it to a fn the prelude or a std module declares, or
/// a compiler builtin member, so a program's own fn of that name is not it. `name` comes from [`std_fn`].
pub(crate) fn is_std(facts: Option<&Facts>, at: Span, name: &str) -> bool {
    facts.and_then(|f| f.callee(at)).is_some_and(|c| {
        c.name == name && (c.builtin || c.module == "prelude" || c.module.starts_with("std."))
    })
}

/// The checker's facts and the offset of the item a rule is in: asks
/// [`is_std`] of an item-relative expression.
/// `(start, end)` as one key that orders as the pair does.
pub(crate) fn span_key((start, end): (u32, u32)) -> u64 {
    (start as u64) << 32 | end as u64
}

/// The codes of `codes` in text order, once each, for [`code_rank`].
pub(crate) fn code_order(codes: impl Iterator<Item = cpl_diag::Code>) -> Vec<&'static str> {
    let mut all: Vec<&'static str> = codes.map(|c| c.0).collect();
    cpl_diag::sort_by_str_key(&mut all, |c| c);
    all.dedup();
    all
}

/// `code`'s place in `order`, a key that orders as the code's text does.
pub(crate) fn code_rank(order: &[&'static str], code: cpl_diag::Code) -> u64 {
    order.binary_search(&code.0).map_or(0, |i| i as u64)
}

/// `diags` by span, then code, through cpl_diag's one sort rather than a
/// sort instantiated for each element type.
pub(crate) fn by_place(diags: &mut [Diagnostic]) {
    let order = code_order(diags.iter().map(|d| d.code));
    cpl_diag::sort_by_key(diags, |d| code_rank(&order, d.code));
    cpl_diag::sort_by_key(diags, |d| span_key((d.span.start, d.span.end)));
}

#[derive(Clone, Copy)]
pub(crate) struct Calls<'a> {
    facts: Option<&'a Facts>,
    base: u32,
}

impl<'a> Calls<'a> {
    pub fn new(facts: Option<&'a Facts>, base: u32) -> Self {
        Calls { facts, base }
    }

    /// Whether `e`, a call or getter, is std's `name`.
    pub fn is(self, e: &cpl_syntax::ast::Expr, name: &str) -> bool {
        is_std(self.facts, e.span.shift(self.base), name)
    }

    /// The member `e`, a member access or a call of one, is written as when
    /// the checker could not resolve it: `first` of `xs.first(x => ..)`.
    pub fn unresolved(self, e: &cpl_syntax::ast::Expr) -> Option<&'a str> {
        let path = self.facts?.unresolved(e.span.shift(self.base))?;
        path.last().map(String::as_str)
    }

    /// The name of the std fn `e`, a call or getter, calls: `None` for a
    /// program's own fn or where the checker has no answer.
    pub fn name(self, e: &cpl_syntax::ast::Expr) -> Option<&'a str> {
        let callee = self.facts?.callee(e.span.shift(self.base))?;
        is_std(self.facts, e.span.shift(self.base), &callee.name).then_some(callee.name.as_str())
    }
}

fn with_parsed<T: Default>(src: &str, f: impl FnOnce(&File) -> T) -> T {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return T::default();
    }
    let items: Vec<(u32, &ParsedItem)> = parsed.items.iter().map(|(o, p)| (*o, p)).collect();
    f(&File {
        src,
        items: &items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

impl File<'_> {
    /// Walks every item, telling `visit` which offset its spans start at.
    /// The top-level statements, one item each, reach `stmts` as one sequence.
    pub(crate) fn walk<V: Visitor>(&self, visitor: &mut V, mut set_base: impl FnMut(&mut V, u32)) {
        set_base(visitor, 0);
        visitor.stmts(&self.top_level().collect::<Vec<_>>());
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
    rules::proven_fns(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
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
    rules::proven_sizes(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
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
    rules::domain_sizes(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// Each index proven in bounds; a file with a syntax error has none.
pub fn proven_indexes(src: &str) -> Vec<ProvenIndex> {
    with_parsed(src, rules::proven_indexes)
}

pub fn proven_indexes_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenIndex> {
    rules::proven_indexes(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// `proven_loops_parsed` and `proven_indexes_parsed` from one proof of the
/// file, for a caller that wants both: each alone proves the whole file.
pub fn proven_loops_and_indexes_parsed(
    src: &str,
    items: &[(u32, &ParsedItem)],
) -> (Vec<ProvenLoop>, Vec<ProvenIndex>) {
    rules::proven_loops_and_indexes(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// The part of `proven_loops_and_indexes_parsed` inside `items[item]`,
/// without proving the file's other fns: for an editor request in one fn.
pub fn proven_loops_and_indexes_in_item(
    src: &str,
    items: &[(u32, &ParsedItem)],
    item: usize,
) -> (Vec<ProvenLoop>, Vec<ProvenIndex>) {
    rules::proven_loops_and_indexes_in(
        &File {
            src,
            items,
            facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
            proven: &Default::default(),
        },
        item,
    )
}

/// Each `for` over a bounded range or domain, with how often it runs.
pub fn proven_loops(src: &str) -> Vec<ProvenLoop> {
    with_parsed(src, rules::proven_loops)
}

pub fn proven_loops_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenLoop> {
    rules::proven_loops(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// What the declared bounds prove about each `+`, `-`, `*`, `/`, `%` and `**` outside
/// lambdas, comprehensions and match arms; a file with a syntax error has
/// none.
pub fn proven_expr_ranges(src: &str) -> Vec<ProvenExpr> {
    with_parsed(src, rules::proven_expr_ranges)
}

pub fn proven_expr_ranges_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<ProvenExpr> {
    rules::proven_expr_ranges(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// What the declared input bounds prove about each top-level `let` written
/// without a type: `let m = n * n` after `let n: 1..=100 = read()` is in
/// `1..=10000`. Each is the name's byte offset and the inclusive range; a file
/// with a syntax error has none.
pub fn proven_ranges(src: &str) -> Vec<(u32, i128, i128)> {
    with_parsed(src, rules::proven_ranges)
}

pub fn proven_ranges_parsed(src: &str, items: &[(u32, &ParsedItem)]) -> Vec<(u32, i128, i128)> {
    rules::proven_ranges(&File {
        src,
        items,
        facts: analyze_alone(src).as_deref().map(|a| &**a.facts()),
        proven: &Default::default(),
    })
}

/// Lints one file. Returns warnings (errors for `deny`), in source order.
///
/// A file with a syntax error gets no lints: what the parser could not read
/// may hold the assignment or the read a lint would say is missing.
pub fn lint(src: &str, config: &Config) -> Vec<Diagnostic> {
    (analyze_alone(src).as_deref())
        .map(|analysis| lint_analysis(analysis, config))
        .unwrap_or_default()
}

/// `src` checked in a database of its own, for callers that hold none.
pub(crate) fn analyze_alone(src: &str) -> Option<std::sync::Arc<cpl_facade::today::Analysis>> {
    let mut db = cpl_facade::Session::default();
    db = db.with_file("<lint>.cp", src);
    db.analyze("<lint>.cp")
}

/// `diags` less the lints at the place of one of `errors`: the error
/// already names that root cause (L0509 beside the E0606 it explains). A lint
/// with a quick fix stays unless the error carries one too.
pub(crate) fn drop_repeats(diags: &mut Vec<Diagnostic>, errors: &[Diagnostic]) {
    diags.retain(|d| {
        !errors.iter().any(|e| {
            e.severity == Severity::Error
                && e.span == d.span
                && e.code != d.code
                && (d.fixes.is_empty() || !e.fixes.is_empty())
        })
    });
}

pub fn lint_parsed(
    src: &str,
    items: &[(u32, &ParsedItem)],
    facts: &Facts,
    config: &Config,
) -> Vec<Diagnostic> {
    lint_file(
        &File {
            src,
            items,
            facts: Some(facts),
            proven: &Default::default(),
        },
        config,
        &[],
    )
}

/// [`lint_parsed`] with the `@lint` declarations the file can see.
pub(crate) fn lint_parsed_library(
    src: &str,
    items: &[(u32, &ParsedItem)],
    facts: Option<&Facts>,
    config: &Config,
    library: &[library::Declared],
    proven: &std::sync::OnceLock<rules::Proven>,
) -> Vec<Diagnostic> {
    lint_file(
        &File {
            src,
            items,
            facts,
            proven,
        },
        config,
        library,
    )
}

fn lint_file(file: &File, config: &Config, declared: &[library::Declared]) -> Vec<Diagnostic> {
    let allows = Allows::collect(file, declared);
    let library = file.top_level().any(|(_, stmt)| stmt.public);
    let mut fns = Fns::default();
    file.walk(&mut fns, |fns, base| fns.base = base);
    let mut findings = allows.unknown;
    findings.extend(rules::run(file, config));
    findings.extend(library::find(file, declared, None));

    let mut out: Vec<Diagnostic> = Vec::new();
    for Finding { lint, mut diag } in findings {
        let level = config.level(lint);
        let allowed = allows
            .regions
            .iter()
            .any(|(span, name)| *name == lint.name && span.contains(diag.span));
        let library_only = library && lint.std_structure;
        if level == Level::Allow || allowed || library_only || fns.defines(lint, &diag, library) {
            continue;
        }
        diag.severity = if level == Level::Deny {
            Severity::Error
        } else {
            Severity::Warning
        };
        diag.match_line_endings(file.src);
        out.push(diag);
    }
    by_place(&mut out);
    out.dedup();
    let all = out.clone();
    out.retain(|d| {
        !all.iter().any(|wider| {
            (supersedes(file.src, wider, d)
                && !library::refires(file.src, wider, d.code, &all, declared))
                || writes_literal(file.src, wider, d)
        })
    });
    out
}

/// Whether `inner` is about a literal alone (L0506 on `998244353`) that
/// `wider` puts to use: `wider` spans a copy of it and its rewrite writes it
/// (`pow_mod(b, e, 998244353)`, `Mod<998244353>`). The literal is one fact,
/// so the finding whose rewrite uses it reports it, unless only `inner`
/// carries a fix.
fn writes_literal(src: &str, wider: &Diagnostic, inner: &Diagnostic) -> bool {
    let text = |s: Span| src.get(s.start as usize..s.end as usize).unwrap_or("");
    let lit = text(inner.span);
    let literal = lit.starts_with(|c: char| c.is_ascii_digit())
        && lit.bytes().all(|b| b.is_ascii_digit() || b == b'_');
    if wider.code == inner.code || !literal || (wider.fixes.is_empty() && !inner.fixes.is_empty()) {
        return false;
    }
    let value = lit.replace('_', "");
    let spans_copy = std::iter::once(wider.span)
        .chain(wider.labels.iter().map(|l| l.span))
        .any(|s| has_word(text(s), lit));
    let rewrite = std::iter::once(wider.message.as_str())
        .chain(wider.help.as_deref())
        .chain(
            wider
                .fixes
                .iter()
                .flat_map(|f| f.edits.iter().map(|(_, to)| to.as_str())),
        );
    spans_copy
        && rewrite
            .into_iter()
            .any(|t| has_word(t, lit) || has_word(t, &value))
}

/// Whether `t` holds `word` with no letter, digit or `_` on either side.
fn has_word(t: &str, word: &str) -> bool {
    let part = |c: char| c.is_ascii_alphanumeric() || c == '_';
    (t.match_indices(word))
        .any(|(at, _)| !t[..at].ends_with(part) && !t[at + word.len()..].starts_with(part))
}

/// Whether `wider`'s rewrite removes the code `inner` is about, so only
/// `wider` is shown. With a fix: `inner` lies inside one of its edits and
/// no edit carries `inner`'s code over (`s = s + x` in a loop that becomes
/// one `sum`). With none, when its std call replaces the code whole
/// (`replaces = "whole"`): `inner` or its evidence lies inside `wider`'s
/// span or evidence (the cache lookup a `@memo` removes; on one span, a
/// search loop that is `is_palindrome`). A rewrite that keeps the inner code
/// keeps both; `inner`'s evidence counts as its code (`var found = -1`, set
/// in a search loop that becomes `first(..) ?? -1`).
fn supersedes(src: &str, wider: &Diagnostic, inner: &Diagnostic) -> bool {
    let whole = |d: &Diagnostic| LINTS.iter().any(|l| l.code == d.code && l.replaces_whole);
    // Trailing whitespace a statement's span runs on into does not widen it.
    let len = |s: Span| {
        let text = src.get(s.start as usize..s.end as usize).unwrap_or("");
        text.trim_end().len() as u32
    };
    let shorter = len(inner.span) < len(wider.span)
        || (len(inner.span) == len(wider.span)
            && whole(wider)
            && wider.fixes.is_empty()
            && (!whole(inner) || !inner.fixes.is_empty()));
    if wider.code == inner.code || !shorter {
        return false;
    }
    let trim = |s: Span| Span::new(s.start, s.start + len(s));
    let about: Vec<Span> = std::iter::once(trim(inner.span))
        .chain(inner.labels.iter().map(|l| l.span))
        .collect();
    if let Some(fix) = wider.fixes.first() {
        let text = |s: Span| {
            src.get(s.start as usize..s.end as usize)
                .unwrap_or("")
                .trim()
        };
        return about.iter().any(|a| {
            fix.edits.iter().any(|(at, _)| at.contains(*a))
                && !fix.edits.iter().any(|(_, to)| to.contains(text(*a)))
        });
    }
    let evidence: Vec<Span> = std::iter::once(trim(wider.span))
        .chain(wider.labels.iter().map(|l| l.span))
        .collect();
    whole(wider)
        && about
            .iter()
            .any(|a| evidence.iter().any(|e| e.contains(*a)))
}

/// Each fn of a file, so a finding that says "this is `first`" is not
/// reported inside `fn first` itself. In a library module that holds for
/// every lint whose help names the call `first(..)`: the fn is where the
/// library writes the loop the lint teaches, and so is a private fn such a
/// fn calls: calling `first_true` there would recurse.
#[derive(Default)]
struct Fns {
    base: u32,
    fns: Vec<FnSpan>,
    /// Each call of a fn by name: where, and the name.
    calls: Vec<(u32, String)>,
}

struct FnSpan {
    span: Span,
    name: String,
    public: bool,
}

impl Visitor for Fns {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(decl) = &stmt.kind {
            self.fns.push(FnSpan {
                span: stmt.span.shift(self.base),
                name: decl.name.text.clone(),
                public: stmt.public,
            });
        }
    }

    fn expr(&mut self, expr: &cpl_syntax::ast::Expr) {
        if let ExprKind::Call { callee, .. } = &expr.kind {
            if let ExprKind::Name(name) = &callee.kind {
                self.calls
                    .push((expr.span.shift(self.base).start, name.clone()));
            }
        }
    }
}

impl Fns {
    /// The innermost fn around `span`.
    fn around(&self, span: Span) -> Option<&FnSpan> {
        (self.fns.iter())
            .filter(|f| f.span.contains(span))
            .min_by_key(|f| f.span.end - f.span.start)
    }

    fn defines(&self, lint: &Lint, diag: &Diagnostic, library: bool) -> bool {
        let named = |f: &FnSpan| {
            // `count(..)` or the method `xs.count(..)` (decision 53).
            let name = &f.name;
            let calls =
                |h: &str| h.contains(&format!("`{name}(")) || h.contains(&format!(".{name}("));
            (lint.quiet_in_namesake && diag.message.contains(&format!("`{name}`")))
                || (library && diag.help.as_deref().is_some_and(calls))
        };
        if self
            .fns
            .iter()
            .any(|f| f.span.contains(diag.span) && named(f))
        {
            return true;
        }
        let Some(helper) = self.around(diag.span).filter(|f| library && !f.public) else {
            return false;
        };
        let mut callers = (self.calls.iter())
            .filter(|(_, name)| *name == helper.name)
            .filter_map(|(at, _)| self.around(Span::new(*at, *at)))
            .filter(|f| !std::ptr::eq(*f, helper));
        callers.any(named)
    }
}

/// The `with lint(name: allow)` contexts of a file.
struct Allows {
    /// The span each one covers and the lint it turns off.
    regions: Vec<(Span, &'static str)>,
    unknown: Vec<Finding>,
}

impl Allows {
    fn collect(file: &File, library: &[library::Declared]) -> Allows {
        let mut out = Allows {
            regions: Vec::new(),
            unknown: Vec::new(),
        };
        let mut contexts = Contexts {
            base: 0,
            out: &mut out,
            library: library.iter().map(|d| d.lint.name).collect(),
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
        out
    }
}

/// The `with lint(name: allow)` contexts of a file. (R09.4.4)
struct Contexts<'a> {
    base: u32,
    out: &'a mut Allows,
    /// The names of the `@lint`s the file can see.
    library: Vec<&'static str>,
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
            match any_lint_named(&field.text)
                .or_else(|| self.library.iter().find(|n| **n == field.text).copied())
            {
                Some(name) if matches!(&arg.value.kind, ExprKind::Name(v) if v == "allow") => {
                    self.out.regions.push((covered, name))
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
    let mut diag = cpl_diag::msgs::lint_rules::UNKNOWN_LINT.warning(span, &[&name]);
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
    let names = LINTS.iter().map(|lint| lint.name);
    names
        .chain(CHECKER_LINTS.iter().map(|lint| lint.name))
        .map(|known| (edit_distance(name, known), known))
        .filter(|(distance, _)| *distance <= 2)
        .min()
        .map(|(_, name)| name)
}

use cpl_diag::edit_distance;

pub use fix::{
    apply_lint_fixes, apply_lint_fixes_where, fix_checked_removed_syntax_in, fix_in,
    fix_removed_syntax, fix_rounds,
};
#[cfg(test)]
pub(crate) use fix::{fix, fix_at};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod record_tests;

#[cfg(test)]
mod std_tests;

#[cfg(test)]
mod corpus_tests;

#[cfg(test)]
mod fuzz_tests;
#[cfg(test)]
mod library_tests;
#[cfg(test)]
mod plant_tests;
#[cfg(test)]
mod speed_tests;

#[cfg(test)]
mod behaviour_tests;
#[cfg(test)]
mod fix_tests;

#[cfg(test)]
mod checker_tests;

#[cfg(test)]
mod fact_tests;

#[cfg(test)]
mod idiom_tests;
