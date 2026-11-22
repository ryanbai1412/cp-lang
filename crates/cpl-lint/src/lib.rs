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
pub const POW_METHOD: Lint = Lint {
    code: Code("L0205"),
    name: "pow_method",
    default: Level::Warn,
    title: "Power written with `.pow(e)`",
    summary: "`a.pow(e)`: power is written `a ** e`",
    explanation: "CP Lang spells exponentiation with `**` on numbers, `Mod<P>`, `Mat<T>`, and any type implementing `Pow`. `cpl fix` rewrites `a.pow(e)` to `a ** e`.",
    trigger: "let x = 2\nprint(x.pow(10))\n",
    fix: "let x = 2\nprint(x ** 10)\n",
};
pub const GETTER_PARENS: Lint = Lint {
    code: Code("L0206"),
    name: "getter_parens",
    default: Level::Warn,
    title: "Built-in getter called with `()`",
    summary: "`.len()`, `.is_empty()`, `.first()`, `.last()`, `.rev()`, `.items()` are read without `()`",
    explanation: "Built-in getters (`len`, `is_empty`, `first`, `last`, `rev`, `items`) are read as properties without parentheses. `cpl fix` drops the empty `()`.",
    trigger: "let xs = [1, 2, 3]\nprint(xs.len())\n",
    fix: "let xs = [1, 2, 3]\nprint(xs.len)\n",
};
pub const IS_IN_COND_HEADER: Lint = Lint {
    code: Code("L0207"),
    name: "is_in_cond_header",
    default: Level::Warn,
    title: "Pattern test `x is p` in `if`/`elif`/`while` header",
    summary: "`if`/`elif`/`while` headers bind with `let p = x`, not `x is p`",
    explanation: "In `if`, `elif`, and `while` headers, bind with `if let p = x` / `while let p = x` and chain extra guards with `, cond`. Keep `x is pat` for value position where a `Bool` is wanted. `cpl fix` rewrites `if`/`elif`/`while` `is` headers to `let`.",
    trigger: "let x: ?Int = some(3)\nif x is v and v > 0:\n    print(v)\n",
    fix: "let x: ?Int = some(3)\nif let v = x, v > 0:\n    print(v)\n",
};
pub const REDUNDANT_PRELUDE_IMPORT: Lint = Lint {
    code: Code("L0208"),
    name: "redundant_prelude_import",
    default: Level::Warn,
    title: "`use` of a name the prelude already provides",
    summary: "`use std.tree.{Rooted}` imports what the prelude gives every program",
    explanation: "The prelude (`stdlib/prelude.cp`) brings in each of its names when a program uses it, so `use std.tree.{Rooted}` or `use std.mod.Mod` says nothing. `cpl fix` drops the name, and the `use` when nothing is left. An alias, a glob, a `pub use` and the imports of a library (a file with `pub` items) are left alone.",
    trigger: "use std.tree.{Rooted}\ndomain V = 1..=3\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n",
    fix: "domain V = 1..=3\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n",
};
pub const REDUNDANT_ARG_NAME: Lint = Lint {
    code: Code("L0209"),
    name: "redundant_arg_name",
    default: Level::Warn,
    title: "Named argument or record field repeating its own name",
    summary: "`f(moves: moves)` or `(val: val)` can be punned as `f(moves:)` / `(val:)`",
    explanation: "When a named call argument or record field passes a variable of the same name, write `name:` instead of `name: name`. `cpl fix` shortens `name: name` to `name:`.",
    trigger: "fn f(moves: Int, cost: Int) -> Int = moves + cost\nlet moves = 3\nlet cost = 4\nprint(f(moves: moves, cost: cost))\n",
    fix: "fn f(moves: Int, cost: Int) -> Int = moves + cost\nlet moves = 3\nlet cost = 4\nprint(f(moves:, cost:))\n",
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
pub const LOOP_INVARIANT_MOD_DIV: Lint = Lint {
    code: Code("L0415"),
    name: "loop_invariant_mod_div",
    default: Level::Warn,
    title: "Loop-invariant `Mod` divisor inside loop",
    summary: "a `Mod<P>` division by a loop-invariant value computes an O(log P) inverse every iteration",
    explanation: "Dividing a `Mod<P>` value computes an O(log P) modular inverse on every iteration. When the divisor does not change inside the loop, hoist `let inv_d = d.inv` before the loop and multiply by `inv_d` instead.",
    trigger: "type Z = Mod<998_244_353>\nlet d = Z(3)\nvar xs = [Z(1), Z(2), Z(3)]\nfor i in 0..xs.len:\n    xs[i] = xs[i] / d\nprint(xs)\n",
    fix: "type Z = Mod<998_244_353>\nlet d = Z(3)\nvar xs = [Z(1), Z(2), Z(3)]\nlet inv_d = d.inv\nfor i in 0..xs.len:\n    xs[i] = xs[i] * inv_d\nprint(xs)\n",
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
    trigger: "let a = [3, 4]\nprint(a.sum() % 998244353)\nprint(a[0] * a[1] % 998244353)\n",
    fix: "const P = 998244353\nlet a = [3, 4]\nprint(a.sum() % P)\nprint(a[0] * a[1] % P)\n",
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
pub const FREE_FOLD: Lint = Lint {
    code: Code("L0509"),
    name: "free_fold",
    default: Level::Warn,
    title: "Fold or sort of a sequence written as a free function",
    summary: "`sum(xs)`, `count(xs, p)` or `sort(mut xs)`: the method reads on the sequence",
    explanation: "A fold or sort of one sequence is a method on it: `xs.sum()`, `xs.count(p)`, `xs.all(p)`, `xs.first(p)`, `xs.max()`, `xs.sorted(by: k)`, `xs.sort()`. The free form stays where it reads as math: a generator (`sum(f(x) for x in xs)`) or two or more values (`min(a, b)`). `cpl lint --fix` rewrites it.",
    trigger: "let xs = [3, 1, 4]\nprint(sum(xs), count(xs, x => x > 2))\n",
    fix: "let xs = [3, 1, 4]\nprint(xs.sum(), xs.count(x => x > 2))\n",
};
pub const BY_TWIN: Lint = Lint {
    code: Code("L0510"),
    name: "by_twin",
    default: Level::Warn,
    title: "`_by` twin instead of a `by:` option",
    summary: "`min_by`, `max_by`, `argmax_by`, `runs_by`, `group_by`, `first_ge_by`: pass `by:` to the base operation",
    explanation: "Keyed sequence operations take a `by:` named option on the base function rather than a `_by` twin: `xs.min(by: k)`, `xs.max(by: k)`, `xs.argmax(by: k)`, `xs.group(by: k)`, `xs.is_sorted(by: k)`, `xs.runs(by: k)`, `xs.run_ranges(by: k)`, `xs.first_ge(x, by: k)`. `cpl lint --fix` rewrites both method and free `_by` calls.",
    trigger: "let xs = [\"bbb\", \"a\", \"cc\"]\nprint(min_by(xs, s => s.len))\n",
    fix: "let xs = [\"bbb\", \"a\", \"cc\"]\nprint(xs.min(by: s => s.len))\n",
};
pub const DOMAIN_SPELLING: Lint = Lint {
    code: Code("L0512"),
    name: "domain_spelling",
    default: Level::Warn,
    title: "Non-canonical domain/sequence method spelling",
    summary: "`D.sorted(by:)` -> `D.sort(by:)`; `xs.first(where: p)` -> `xs.first(p)`",
    explanation: "Domains follow the one-spelling sequence rules: sort a domain with `D.sort(by: key)` (matching `xs.sort(by: key)`), and pass the predicate to `.first(p)` positionally (without `where:`). `cpl fix` rewrites both.",
    trigger: "domain V = 1..=3\nlet a: [V]Int = [3, 1, 2]\nlet _ = V.sorted(by: a)\nlet _ = V.first(where: v => a[v] > 1)\n",
    fix: "domain V = 1..=3\nlet a: [V]Int = [3, 1, 2]\nlet _ = V.sort(by: a)\nlet _ = V.first(v => a[v] > 1)\n",
};
pub const PREFIX_TWIN: Lint = Lint {
    code: Code("L0513"),
    name: "prefix_twin",
    default: Level::Warn,
    title: "Dimension-suffixed prefix or diff helper instead of `prefix`/`Prefix`/`Diff`",
    summary: "`prefix1`/`prefix2`/`prefix_n`, `Prefix1`/`Prefix2`/`PrefixN`, `Diff1`/`Diff2`: write `prefix`, `Prefix`, or `Diff`",
    explanation: "`prefix(a)` dispatches by shape across `[D]T`, `[R, C]T`, and `List<T>` under one `Prefix` type family, and `Diff` covers both 1-D `Diff<D, T>` and 2-D `Diff<R, C, T>`. `cpl fix` rewrites the removed arity-suffixed names (`prefix1`, `prefix2`, `prefix_n`, `Prefix1`, `Prefix2`, `PrefixN`, `Diff1`, `Diff2`) to `prefix`, `Prefix`, and `Diff`.",
    trigger: "let xs = [3, 1, 4]\nlet p = prefix_n(xs)\nprint(p.sum(0..3))\n",
    fix: "let xs = [3, 1, 4]\nlet p = prefix(xs)\nprint(p.sum(0..3))\n",
};

pub const TREE_ORDER: Lint = Lint {
    code: Code("L0511"),
    name: "tree_order",
    default: Level::Warn,
    title: "`preorder` or `postorder` of a rooted tree",
    summary: "`t.preorder` / `t.postorder`: iterate the preorder array `t.at` (`t.at.rev` for children first)",
    explanation: "A `Rooted` tree has one traversal order, its preorder domain `t.Pre` (decision 63). `for v in t.at` visits each vertex before its descendants, `for v in t.at.rev` after them, and `for (p, v) in t.at.items` gives the position too. `cpl lint --fix` rewrites `t.preorder` to `t.at` and `t.postorder` to `t.at.rev`.",
    trigger: "use std.graph.{Graph}\nuse std.tree.{Rooted}\ndomain V = 0..2\nlet t = Rooted(Graph.undirected([(V(0), V(1))]), V(0))\nfor v in t.postorder: print(v)\n",
    fix: "use std.graph.{Graph}\nuse std.tree.{Rooted}\ndomain V = 0..2\nlet t = Rooted(Graph.undirected([(V(0), V(1))]), V(0))\nfor v in t.at.rev: print(v)\n",
};
pub const HAND_ROLLED_RANGE_TREE: Lint = Lint {
    code: Code("L0514"),
    name: "hand_rolled_range_tree",
    default: Level::Warn,
    title: "Hand-rolled Fenwick or segment tree",
    summary: "manual BIT (`p += p & -p`) or segment-tree (`2 * p` / `2 * p + 1`) loop: use `std.fenwick` / `std.segtree` / `std.lazy`",
    explanation: "Hand-rolling a binary indexed tree (`p += p & -p` / `p -= p & -p`) or a segment tree (`2 * p` and `2 * p + 1` with halving or midpoint recursion) duplicates tested, domain-safe stdlib range structures. Use `Fenwick` / `DualFenwick` / `RangeFenwick` or `SegTree` / `LazySegTree` / `DualSegTree`.",
    trigger: "var bit: [0..=8]Int = 0\nvar p = 3\nwhile p <= 8:\n    bit[p] += 1\n    p += p & -p\nprint(bit[4])\n",
    fix: "domain Pos = 1..=8\nvar bit = Fenwick<Pos, Int>()\nbit.add(Pos(3), 1)\nprint(bit.prefix(Pos(4).r))\n",
};
pub const SORTED_RANGE_METHOD: Lint = Lint {
    code: Code("L0515"),
    name: "sorted_range_method",
    default: Level::Warn,
    title: "Removed bound-query method on `SortedBag` or `SortedSet`",
    summary: "`first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range` on `SortedBag`/`SortedSet`: use `.range(r)`",
    explanation: "`SortedBag.range(r)` and `SortedSet.range(r)` return an O(1) view whose `.first`, `.last`, and `.len` cost O(log n) and whose iteration costs O(log n + k). `cpl fix` rewrites `first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range` to `.range(...)` calls.",
    trigger: "var s = SortedBag<Int>()\nprint(s.first_ge(3), s.last_lt(5), s.count_range(1, 5))\n",
    fix: "var s = SortedBag<Int>()\nprint(s.range(3..).first, s.range(..5).last, s.range(1..5).len)\n",
};
pub const HAND_ROLLED_STRING_ALGO: Lint = Lint {
    code: Code("L0516"),
    name: "hand_rolled_string_algo",
    default: Level::Warn,
    title: "Hand-rolled string algorithm",
    summary: "manual prefix-function, Z-function, polynomial hash, or trie loop: use `std.strings`",
    explanation: "Hand-rolling a KMP prefix function, Z-function, polynomial rolling hash, or trie insertion loop duplicates tested, linear-time `std.strings` primitives (`prefix_function`, `z_function`, `RollingHash`, `Trie`, `AhoCorasick`).",
    trigger: "let s = \"ababa\"\nvar pi = List.filled(s.len, 0)\nfor i in 1..s.len:\n    var j = pi[i - 1]\n    while j > 0 and s[i] != s[j]: j = pi[j - 1]\n    if s[i] == s[j]: j += 1\n    pi[i] = j\nprint(pi)\n",
    fix: "use std.strings.{prefix_function}\nlet s = \"ababa\"\nprint(prefix_function(s))\n",
};
pub const HAND_ROLLED_DSU: Lint = Lint {
    code: Code("L0517"),
    name: "hand_rolled_dsu",
    default: Level::Warn,
    title: "Hand-rolled union-find (DSU)",
    summary: "manual parent-pointer path compression: use `std.dsu`",
    explanation: "Hand-rolling union-find path compression (`while par[x] != x: par[x] = par[par[x]]` or recursive `par[x] = find(par[x])`) duplicates domain-safe, path-halving `Dsu`, `DsuOf`, `ParityDsu`, and `PotentialDsu` in `std.dsu`.",
    trigger: "domain V = 0..4\nvar par: [V]V = [V] v => v\nfn find(v: V) -> V:\n    var x = v\n    while par[x] != x:\n        par[x] = par[par[x]]\n        x = par[x]\n    x\nprint(find(V(0)))\n",
    fix: "use std.dsu.{Dsu}\ndomain V = 0..4\nvar d = Dsu<V>.new()\nprint(d.find(V(0)))\n",
};
pub const HAND_ROLLED_SIEVE: Lint = Lint {
    code: Code("L0518"),
    name: "hand_rolled_sieve",
    default: Level::Warn,
    title: "Hand-rolled prime or SPF sieve",
    summary: "manual Eratosthenes or linear sieve loop: use `std.math.Sieve`",
    explanation: "Hand-rolling a prime or smallest-prime-factor sieve (`j = i * i; while j <= n: is_prime[j] = false; j += i` or linear `spf[i * p] = p`) duplicates `Sieve` and `sieve_multiplicative` in `std.math`.",
    trigger: "let n = 20\nvar is_prime = List.filled(n + 1, true)\nfor i in 2..=n:\n    if is_prime[i]:\n        var j = i * i\n        while j <= n:\n            is_prime[j] = false\n            j += i\nprint(is_prime[7])\n",
    fix: "use std.math.{Sieve}\nlet sv = Sieve.new(20)\nprint(sv.is_prime(7))\n",
};
pub const HAND_ROLLED_BINARY_LIFTING: Lint = Lint {
    code: Code("L0519"),
    name: "hand_rolled_binary_lifting",
    default: Level::Warn,
    title: "Hand-rolled binary lifting table",
    summary: "manual `up[k][v] = up[k - 1][up[k - 1][v]]` table: use `std.tree`",
    explanation: "Hand-rolling a binary lifting ancestor table (`up[k, v] = up[k - 1, up[k - 1, v]]`) duplicates `Rooted.lca` / `Rooted.kth_ancestor` and `JumpTree` in `std.tree`.",
    trigger: "var up = [[0, 0, 1], [0, 0, 0]]\nfor k in 1..2:\n    for v in 0..3:\n        up[k][v] = up[k - 1][up[k - 1][v]]\nprint(up[1][2])\n",
    fix: "use std.tree.{JumpTree}\ndomain V = 0..3\nvar jt = JumpTree<V>.new()\njt.add(V(1), V(0))\njt.add(V(2), V(1))\nprint(jt.ancestor(V(2), 2))\n",
};

pub const HAND_ROLLED_ANSWER_SEARCH: Lint = Lint {
    code: Code("L0520"),
    name: "hand_rolled_answer_search",
    default: Level::Warn,
    title: "Hand-rolled binary search",
    summary: "manual `while lo < hi` midpoint loop over a predicate: use `bsearch_first`/`bsearch_last`",
    explanation: "A `while lo < hi` loop that halves with `let mid = (lo + hi) / 2` and sets `hi = mid` / `lo = mid + 1` (or `lo = mid` / `hi = mid - 1` with the rounded-up midpoint) on a predicate of `mid` is `bsearch_first` (or `bsearch_last`) in `std.seq`: O(log n) calls of the predicate, no off-by-one in the bounds, and the same midpoint that cannot overflow. `cpl fix` rewrites the loop and its `var (lo, hi)` when `hi` is not read after it. The predicate must be monotone along the range, which the loop already assumes.",
    trigger: "let a = [1, 3, 5, 7]\nvar (lo, hi) = (0, a.len)\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a[mid] >= 5: hi = mid else: lo = mid + 1\nprint(lo)\n",
    fix: "let a = [1, 3, 5, 7]\nlet lo = bsearch_first(0..a.len, mid => a[mid] >= 5) ?? a.len\nprint(lo)\n",
};
pub const HAND_ROLLED_PREFIX_SUM: Lint = Lint {
    code: Code("L0521"),
    name: "hand_rolled_prefix_sum",
    default: Level::Warn,
    title: "Hand-rolled prefix sum",
    summary: "manual `p[i + 1] = p[i] + x` or `p[i.r] = p[i.l] + x` loop: use `prefix(...)`",
    explanation: "A loop that fills a prefix-sum array (`p[i + 1] = p[i] + x`, `p[i] = p[i - 1] + x`, `p[i.r] = p[i.l] + x`, or the 2D inclusion-exclusion recurrence) duplicates `prefix(...)` in `std.prefix` (prelude). `Prefix` owns the extra boundary slot and gives O(1) `.sum(range)` and `.prefix(cut)` without off-by-one indexing. `cpl fix` rewrites a zero-initialized declaration, its build loop, and subsequent `p[k]` reads into `let p = prefix(...)` and `p.prefix(k)` when `p` is not mutated again.",
    trigger: "let a = [1, 2, 3, 4]\nvar p = List.filled(a.len + 1, 0)\nfor i in 0..a.len:\n    p[i + 1] = p[i] + a[i]\nprint(p[3] - p[1])\n",
    fix: "let a = [1, 2, 3, 4]\nlet p = prefix(a)\nprint(p.prefix(3) - p.prefix(1))\n",
};
pub const COPY_SORT_WRITEBACK: Lint = Lint {
    code: Code("L0522"),
    name: "copy_sort_writeback",
    default: Level::Warn,
    title: "Copy-mutate-writeback on a slice",
    summary: "copying `xs[l..r]` to sort/reverse/rotate and writing it back: mutate `xs[l..r]` in place",
    explanation: "Copying a contiguous slice `xs[l..r]` into a temporary `List`, sorting/reversing/rotating it, and writing the elements back into `xs[l..r]` allocates an unnecessary copy. Mutable slice views lend `xs[l..r]` in O(1) via `IndexedMut`, so `xs[l..r].sort(...)` / `.reverse()` / `.rotate_left(...)` runs in place with no copy. `cpl fix` rewrites the copy-mutate-writeback sequence when the temporary is not read afterward.",
    trigger: "var xs = [5, 4, 3, 2, 1]\nlet sub = xs[1..4].sorted()\nfor i in 1..4: xs[i] = sub[i - 1]\nprint(xs)\n",
    fix: "var xs = [5, 4, 3, 2, 1]\nxs[1..4].sort()\nprint(xs)\n",
};

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
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_fns(&File {
        src,
        items: &parsed.items,
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
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_sizes(&File {
        src,
        items: &parsed.items,
    })
}

/// Each list only pushed to in counted loops, with the most elements it
/// can hold: the number of pushes (decision 29.45).
pub fn list_sizes(src: &str) -> Vec<(String, u128)> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::list_sizes(&File {
        src,
        items: &parsed.items,
    })
}

/// The most positions each top-level domain can have, from the bounds of
/// its range: `domain V = 0..n` after `let n: 1..=2e5 = read()` has 200000.
pub fn domain_sizes(src: &str) -> Vec<(String, u128)> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::domain_sizes(&File {
        src,
        items: &parsed.items,
    })
}

/// Each index proven in bounds; a file with a syntax error has none.
pub fn proven_indexes(src: &str) -> Vec<ProvenIndex> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_indexes(&File {
        src,
        items: &parsed.items,
    })
}

/// Each `for` over a bounded range or domain, with how often it runs.
pub fn proven_loops(src: &str) -> Vec<ProvenLoop> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_loops(&File {
        src,
        items: &parsed.items,
    })
}

/// What the declared bounds prove about each `+`, `-`, `*`, `/`, `%` and `**` outside
/// lambdas, comprehensions and match arms; a file with a syntax error has
/// none.
pub fn proven_expr_ranges(src: &str) -> Vec<ProvenExpr> {
    let parsed = cpl_syntax::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    rules::proven_expr_ranges(&File {
        src,
        items: &parsed.items,
    })
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
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
pub fn fix_checked_removed_syntax(_path: Option<&std::path::Path>, src: &str) -> (String, usize) {
    (src.to_string(), 0)
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
