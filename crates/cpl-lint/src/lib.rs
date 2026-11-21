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
use cpl_syntax::ast::{ExprKind, Stmt};
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
}

pub const UNKNOWN_LINT: Lint = Lint {
    code: Code("L0001"),
    name: "unknown_lint",
    default: Level::Warn,
    title: "Unknown lint name",
    summary: "`@allow` or `cp.toml` names a lint that does not exist",
    explanation: "An `@allow(...)` attribute or `[lints]` table entry names a lint that does not exist. Check the spelling against the lint catalog.",
    trigger: "@allow(not_a_real_lint)\nlet x = 1\nprint(x)\n",
    fix: "@allow(var_never_reassigned)\nvar x = 1\nprint(x)\n",
};
pub const POSITION_ARITHMETIC: Lint = Lint {
    code: Code("L0101"),
    name: "position_arithmetic",
    default: Level::Warn,
    title: "Position arithmetic via `.ord`",
    summary: "`V(v.ord + 1)` builds a position from a number",
    explanation: "Converting a domain index to `.ord`, adding or subtracting an offset, and wrapping it back in `V(...)` bypasses checked neighbor operations. Use `v.next`, `v.prev`, or `v + k` (which return `?V`).",
    trigger: "domain V = 1..=3\nfor v in V:\n    let _ = V(v.ord + 1)\n",
    fix: "domain V = 1..=3\nfor v in V:\n    if let nxt = v.next:\n        print(nxt)\n",
};
pub const VAR_NEVER_REASSIGNED: Lint = Lint {
    code: Code("L0201"),
    name: "var_never_reassigned",
    default: Level::Warn,
    title: "Mutable binding never reassigned",
    summary: "a `var` that is never changed",
    explanation: "A `var` binding is never mutated after initialization. Use `let` to signal immutability to readers and the compiler.",
    trigger: "var x = 10\nprint(x)\n",
    fix: "let x = 10\nprint(x)\n",
};
pub const UNREAD_FIELD: Lint = Lint {
    code: Code("L0202"),
    name: "unread_field",
    default: Level::Warn,
    title: "Struct field never read",
    summary: "a struct field that is never read",
    explanation: "A field declared on a `struct` is never read anywhere in the file. Remove unused fields to keep structs compact and clear.",
    trigger: "struct Pt:\n    x: Int\n    y: Int\nlet p = Pt(x: 1, y: 2)\nprint(p.x)\n",
    fix: "struct Pt:\n    x: Int\nlet p = Pt(x: 1)\nprint(p.x)\n",
};
pub const SHADOWS_TOP_LEVEL: Lint = Lint {
    code: Code("L0203"),
    name: "shadows_top_level",
    default: Level::Warn,
    title: "Local shadows top-level binding",
    summary: "a local name that hides a top-level `let`",
    explanation: "A parameter or local variable inside a function has the same name as a top-level `let` binding, which can hide an unintended reference to global input.",
    trigger: "let n = 5\nfn f() -> Int:\n    let n = 10\n    n\nprint(f())\n",
    fix: "let n = 5\nfn f() -> Int:\n    let k = 10\n    k + n\nprint(f())\n",
};
pub const TURBOFISH_TYPE_ARGS: Lint = Lint {
    code: Code("L0204"),
    name: "turbofish_type_args",
    default: Level::Warn,
    title: "Turbofish `::<...>` on generic call",
    summary: "`f::<T>(...)`: canonical generic call spelling is `f<T>(...)`",
    explanation: "Generic calls write type arguments directly as `f<T>(...)` without `::`. `cpl fix` (or `cpl lint --fix`) removes the legacy `::`.",
    trigger: "let x = read::<Int>()\nprint(x)\n",
    fix: "let x = read<Int>()\nprint(x)\n",
};
pub const FLOAT_EQUALITY: Lint = Lint {
    code: Code("L0301"),
    name: "float_equality",
    default: Level::Warn,
    title: "Exact float equality comparison",
    summary: "`==` or `!=` on floats",
    explanation: "Comparing `F64` values with `==` or `!=` is fragile under IEEE-754 rounding. Compare `abs(a - b) < eps` instead.",
    trigger: "let a: F64 = 0.1 + 0.2\nprint(a == 0.3)\n",
    fix: "let a: F64 = 0.1 + 0.2\nprint(abs(a - 0.3) < 1e-9)\n",
};
pub const DOMAIN_KEYED_MAP: Lint = Lint {
    code: Code("L0412"),
    name: "domain_keyed_map",
    default: Level::Warn,
    title: "Map or Set keyed by a domain",
    summary: "a `Map` or `Set` keyed by a domain",
    explanation: "A `Map<V, T>` or `Set<V>` keyed by a dense domain `V` pays hashing or tree overhead on every lookup. Use a flat `[V]?T` or `[V]Bool` array instead.",
    trigger: "domain V = 1..=5\nvar m: Map<V, Int> = Map()\nm[V(1)] = 42\nprint(m[V(1)] ?? 0)\n",
    fix: "domain V = 1..=5\nvar m: [V]?Int = none\nm[V(1)] = 42\nprint(m[V(1)] ?? 0)\n",
};
pub const MEMO_KEY_SPACE: Lint = Lint {
    code: Code("L0413"),
    name: "memo_key_space",
    default: Level::Warn,
    title: "`@memo` table exceeds memory budget",
    summary: "a `@memo` whose table does not fit in memory at the input bounds",
    explanation: "The product of the parameter domain sizes of a `@memo` function exceeds the flat-table memory budget, which can trigger TLE or MLE on contest judges. Narrow the state space or switch to iterative DP.",
    trigger: "domain A = 0..100_000\ndomain B = 0..100_000\n@memo\nfn dp(a: A, b: B) -> Int: a.to_idx + b.to_idx\nprint(dp(A.first, B.first))\n",
    fix: "domain A = 0..100\ndomain B = 0..100\n@memo\nfn dp(a: A, b: B) -> Int: a.to_idx + b.to_idx\nprint(dp(A.first, B.first))\n",
};
pub const UNPROVEN_LIST_INDEX: Lint = Lint {
    code: Code("L0414"),
    name: "unproven_list_index",
    default: Level::Warn,
    title: "Unproven `List` index in nested loop",
    summary: "a `List` index inside nested loops that is checked on every access",
    explanation: "Indexing a `List<T>` with an arbitrary `Int` inside nested loops emits a runtime bounds check on every iteration. Declare a `domain` and `[D]T` array so bounds are verified once.",
    trigger: "let a: List<Int> = [1, 2, 3]\nvar s = 0\nfor i in 0..3:\n    for j in 0..3:\n        s += a[j]\nprint(s)\n",
    fix: "domain D = 0..3\nlet a: [D]Int = [1, 2, 3]\nvar s = 0\nfor _ in D:\n    for j in D:\n        s += a[j]\nprint(s)\n",
};
pub const SEARCH_LOOP: Lint = Lint {
    code: Code("L0501"),
    name: "search_loop",
    default: Level::Warn,
    title: "Flag-and-break search loop",
    summary: "a loop that sets a flag or `?T` and breaks: `first`, `any` or `all`",
    explanation: "A loop that initializes a boolean flag or optional, updates it on a condition, and breaks can be written directly with `any`, `all`, or `first`.",
    trigger: "let xs: List<Int> = [1, 4, 7]\nvar found = false\nfor x in xs:\n    if x > 3:\n        found = true\n        break\nprint(found)\n",
    fix: "let xs: List<Int> = [1, 4, 7]\nlet found = any(x > 3 for x in xs)\nprint(found)\n",
};
pub const DISCARDED_RESULT: Lint = Lint {
    code: Code("L0502"),
    name: "discarded_result",
    default: Level::Warn,
    title: "Discarded function return value",
    summary: "`let _ =` throws away what a fn of this file returns",
    explanation: "Binding the result of a local function to `let _ =` hides intent. Either use the returned value or change the function to return nothing when it is only called for side effects.",
    trigger: "fn work(u: Int) -> Int:\n    print(u)\n    u\nlet _ = work(2)\n",
    fix: "fn work(u: Int):\n    print(u)\nwork(2)\n",
};
pub const SYMMETRIC_WRITE: Lint = Lint {
    code: Code("L0503"),
    name: "symmetric_write",
    default: Level::Warn,
    title: "Duplicate symmetric write",
    summary: "the same fact written for `(u, v)` and then `(v, u)`",
    explanation: "Writing the same value to both `(u, v)` and `(v, u)` duplicates one fact. Use `Graph.undirected(...)`, `Sym<V>`, or a two-element loop so the fact lives in one place.",
    trigger: "domain V = 1..=3\nvar adj: [V]List<V> = []\nlet (u, v) = (V(1), V(2))\nadj[u].push(v)\nadj[v].push(u)\nprint(adj[u].len)\n",
    fix: "domain V = 1..=3\nvar adj: [V]List<V> = []\nlet (u, v) = (V(1), V(2))\nfor (a, b) in [(u, v), (v, u)]:\n    adj[a].push(b)\nprint(adj[u].len)\n",
};
pub const LOOP_REDUCTION: Lint = Lint {
    code: Code("L0504"),
    name: "loop_reduction",
    default: Level::Warn,
    title: "Manual accumulation loop",
    summary: "a loop that only accumulates: `sum`, `count`, `max` or `min`",
    explanation: "A loop that only sums, counts, or takes a running `min`/`max` over an iterable reads more clearly as `sum(...)`, `count(...)`, `min(...)`, or `max(...)`.",
    trigger: "let xs: List<Int> = [1, 2, 3]\nvar total = 0\nfor x in xs:\n    total += x\nprint(total)\n",
    fix: "let xs: List<Int> = [1, 2, 3]\nlet total = sum(x for x in xs)\nprint(total)\n",
};
pub const INDEX_ONLY_LOOP: Lint = Lint {
    code: Code("L0505"),
    name: "index_only_loop",
    default: Level::Warn,
    title: "Index loop that only reads `xs[i]`",
    summary: "`for i in 0..xs.len` that only reads `xs[i]`",
    explanation: "Iterating `for i in 0..xs.len` solely to read `xs[i]` introduces an unnecessary index variable. Iterate directly with `for x in xs:`.",
    trigger: "let xs: List<Int> = [1, 2, 3]\nfor i in 0..xs.len:\n    print(xs[i])\n",
    fix: "let xs: List<Int> = [1, 2, 3]\nfor x in xs:\n    print(x)\n",
};
pub const REPEATED_LITERAL: Lint = Lint {
    code: Code("L0506"),
    name: "repeated_literal",
    default: Level::Warn,
    title: "Repeated large literal",
    summary: "a big literal (a bound or modulus) written in several places",
    explanation: "A big literal (a bound, a modulus, an infinity) is written on several lines: facts that must agree but are kept apart. Equal bounds in the types of different `read()` inputs are separate facts and do not count. Name it once (`const P = 998244353`, a domain, a type alias) and use the name.",
    trigger: "let a = [3, 4]\nprint(sum(a) % 998244353)\nprint(a[0] * a[1] % 998244353)\n",
    fix: "const P = 998244353\nlet a = [3, 4]\nprint(sum(a) % P)\nprint(a[0] * a[1] % P)\n",
};
pub const PUSH_LOOP: Lint = Lint {
    code: Code("L0507"),
    name: "push_loop",
    default: Level::Warn,
    title: "Loop that only pushes",
    summary: "a list built by a loop that only pushes: a comprehension",
    explanation: "A `var xs = [..]` followed by a loop whose only statement is `xs.push(e)`: a comprehension written as a loop. Write the list: `[e for x in xs]` (with `if c` for a filtered loop, `head + [..]` after a non-empty start). `cpl lint --fix` rewrites it.",
    trigger: "let raw = [3, 1, 2]\nvar doubled: List<Int> = []\nfor x in raw: doubled.push(2 * x)\nprint(doubled)\n",
    fix: "let raw = [3, 1, 2]\nlet doubled = [2 * x for x in raw]\nprint(doubled)\n",
};
pub const NONE_CHECK_UNWRAP: Lint = Lint {
    code: Code("L0508"),
    name: "none_check_unwrap",
    default: Level::Warn,
    title: "Check for `none`, then unwrap",
    summary: "`x != none and .. x!`: the check and the unwrap say one thing twice",
    explanation: "`x != none and .. x! ..` tests the option and then unwraps it: the same fact twice, and the `!` stays if the test is edited away. Bind the value once with `if let v = x:`, or compare the option itself (`x == some(e)`).",
    trigger: "fn hit(p: ?Int, v: Int) -> Bool = p != none and v == p!\nprint(hit(none, 1))\n",
    fix: "fn hit(p: ?Int, v: Int) -> Bool = p == some(v)\nprint(hit(none, 1))\n",
};

/// Every lint, in code order.
pub const LINTS: &[Lint] = &[
    UNKNOWN_LINT,
    POSITION_ARITHMETIC,
    VAR_NEVER_REASSIGNED,
    UNREAD_FIELD,
    SHADOWS_TOP_LEVEL,
    TURBOFISH_TYPE_ARGS,
    FLOAT_EQUALITY,
    DOMAIN_KEYED_MAP,
    MEMO_KEY_SPACE,
    UNPROVEN_LIST_INDEX,
    SEARCH_LOOP,
    DISCARDED_RESULT,
    SYMMETRIC_WRITE,
    LOOP_REDUCTION,
    INDEX_ONLY_LOOP,
    REPEATED_LITERAL,
    PUSH_LOOP,
    NONE_CHECK_UNWRAP,
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
    pub items: &'a [(u32, ParsedItem)],
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

/// What the declared input bounds prove about each top-level `let` written
/// without a type: `let m = n * n` after `let n: 1..=100 = read()` is in
/// `1..=10000`. Each is the name's byte offset and the inclusive range; a file
/// with a syntax error has none.
pub fn proven_ranges(src: &str) -> Vec<(u32, i128, i128)> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_ranges(&File {
        src,
        items: &parsed.items,
    })
}

/// Lints one file. Returns warnings (errors for `deny`), in source order.
///
/// A file with a syntax error gets no lints: what the parser could not read
/// may hold the assignment or the read a lint would say is missing.
pub fn lint(src: &str, config: &Config) -> Vec<Diagnostic> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let file = File {
        src,
        items: &parsed.items,
    };
    let allows = Allows::collect(&file);
    let mut findings = allows.unknown;
    findings.extend(rules::run(&file, config));

    let mut out: Vec<Diagnostic> = Vec::new();
    for Finding { lint, mut diag } in findings {
        let level = config.level(lint);
        let allowed = allows
            .regions
            .iter()
            .any(|(span, name)| *name == lint.name && span.contains(diag.span));
        if level == Level::Allow || allowed {
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
        collector.out
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
