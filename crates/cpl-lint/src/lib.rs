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

pub const UNKNOWN_LINT: Lint = Lint {
    code: Code("L0001"),
    name: "unknown_lint",
    default: Level::Warn,
    title: "Unknown lint name",
    summary: "`@allow` or `cp.toml` names a lint that does not exist",
    explanation: include_str!("../../../spec/lints/unknown_lint/explain.md"),
    trigger: include_str!("../../../spec/lints/unknown_lint/before.cp"),
    fix: include_str!("../../../spec/lints/unknown_lint/after.cp"),
    near_miss: include_str!("../../../spec/lints/unknown_lint/near_miss.cp"),
};
pub const POSITION_ARITHMETIC: Lint = Lint {
    code: Code("L0101"),
    name: "position_arithmetic",
    default: Level::Warn,
    title: "Position arithmetic via `.ord`",
    summary: "`V(v.ord + 1)` builds a position from a number",
    explanation: include_str!("../../../spec/lints/position_arithmetic/explain.md"),
    trigger: include_str!("../../../spec/lints/position_arithmetic/before.cp"),
    fix: include_str!("../../../spec/lints/position_arithmetic/after.cp"),
    near_miss: include_str!("../../../spec/lints/position_arithmetic/near_miss.cp"),
};
pub const VAR_NEVER_REASSIGNED: Lint = Lint {
    code: Code("L0201"),
    name: "var_never_reassigned",
    default: Level::Warn,
    title: "Mutable binding never reassigned",
    summary: "a `var` that is never changed",
    explanation: include_str!("../../../spec/lints/var_never_reassigned/explain.md"),
    trigger: include_str!("../../../spec/lints/var_never_reassigned/before.cp"),
    fix: include_str!("../../../spec/lints/var_never_reassigned/after.cp"),
    near_miss: include_str!("../../../spec/lints/var_never_reassigned/near_miss.cp"),
};
pub const UNREAD_FIELD: Lint = Lint {
    code: Code("L0202"),
    name: "unread_field",
    default: Level::Warn,
    title: "Struct field never read",
    summary: "a struct field that is never read",
    explanation: include_str!("../../../spec/lints/unread_field/explain.md"),
    trigger: include_str!("../../../spec/lints/unread_field/before.cp"),
    fix: include_str!("../../../spec/lints/unread_field/after.cp"),
    near_miss: include_str!("../../../spec/lints/unread_field/near_miss.cp"),
};
pub const SHADOWS_TOP_LEVEL: Lint = Lint {
    code: Code("L0203"),
    name: "shadows_top_level",
    default: Level::Warn,
    title: "Local shadows top-level binding",
    summary: "a local name that hides a top-level `let`",
    explanation: include_str!("../../../spec/lints/shadows_top_level/explain.md"),
    trigger: include_str!("../../../spec/lints/shadows_top_level/before.cp"),
    fix: include_str!("../../../spec/lints/shadows_top_level/after.cp"),
    near_miss: include_str!("../../../spec/lints/shadows_top_level/near_miss.cp"),
};
pub const TURBOFISH_TYPE_ARGS: Lint = Lint {
    code: Code("L0204"),
    name: "turbofish_type_args",
    default: Level::Warn,
    title: "Turbofish `::<...>` on generic call",
    summary: "`f::<T>(...)`: canonical generic call spelling is `f<T>(...)`",
    explanation: include_str!("../../../spec/lints/turbofish_type_args/explain.md"),
    trigger: include_str!("../../../spec/lints/turbofish_type_args/before.cp"),
    fix: include_str!("../../../spec/lints/turbofish_type_args/after.cp"),
    near_miss: include_str!("../../../spec/lints/turbofish_type_args/near_miss.cp"),
};
pub const POW_METHOD: Lint = Lint {
    code: Code("L0205"),
    name: "pow_method",
    default: Level::Warn,
    title: "Power written with `.pow(e)`",
    summary: "`a.pow(e)`: power is written `a ** e`",
    explanation: include_str!("../../../spec/lints/pow_method/explain.md"),
    trigger: include_str!("../../../spec/lints/pow_method/before.cp"),
    fix: include_str!("../../../spec/lints/pow_method/after.cp"),
    near_miss: include_str!("../../../spec/lints/pow_method/near_miss.cp"),
};
pub const GETTER_PARENS: Lint = Lint {
    code: Code("L0206"),
    name: "getter_parens",
    default: Level::Warn,
    title: "Built-in getter called with `()`",
    summary:
        "`.len()`, `.is_empty()`, `.first()`, `.last()`, `.rev()`, `.items()` are read without `()`",
    explanation: include_str!("../../../spec/lints/getter_parens/explain.md"),
    trigger: include_str!("../../../spec/lints/getter_parens/before.cp"),
    fix: include_str!("../../../spec/lints/getter_parens/after.cp"),
    near_miss: include_str!("../../../spec/lints/getter_parens/near_miss.cp"),
};
pub const IS_IN_COND_HEADER: Lint = Lint {
    code: Code("L0207"),
    name: "is_in_cond_header",
    default: Level::Warn,
    title: "Pattern test `x is p` in `if`/`elif`/`while` header",
    summary: "`if`/`elif`/`while` headers bind with `let p = x`, not `x is p`",
    explanation: include_str!("../../../spec/lints/is_in_cond_header/explain.md"),
    trigger: include_str!("../../../spec/lints/is_in_cond_header/before.cp"),
    fix: include_str!("../../../spec/lints/is_in_cond_header/after.cp"),
    near_miss: include_str!("../../../spec/lints/is_in_cond_header/near_miss.cp"),
};
pub const REDUNDANT_PRELUDE_IMPORT: Lint = Lint {
    code: Code("L0208"),
    name: "redundant_prelude_import",
    default: Level::Warn,
    title: "`use` of a name the prelude already provides",
    summary: "`use std.tree.{Rooted}` imports what the prelude gives every program",
    explanation: include_str!("../../../spec/lints/redundant_prelude_import/explain.md"),
    trigger: include_str!("../../../spec/lints/redundant_prelude_import/before.cp"),
    fix: include_str!("../../../spec/lints/redundant_prelude_import/after.cp"),
    near_miss: include_str!("../../../spec/lints/redundant_prelude_import/near_miss.cp"),
};
pub const REDUNDANT_ARG_NAME: Lint = Lint {
    code: Code("L0209"),
    name: "redundant_arg_name",
    default: Level::Warn,
    title: "Named argument or record field repeating its own name",
    summary: "`f(moves: moves)` or `(val: val)` can be punned as `f(moves:)` / `(val:)`",
    explanation: include_str!("../../../spec/lints/redundant_arg_name/explain.md"),
    trigger: include_str!("../../../spec/lints/redundant_arg_name/before.cp"),
    fix: include_str!("../../../spec/lints/redundant_arg_name/after.cp"),
    near_miss: include_str!("../../../spec/lints/redundant_arg_name/near_miss.cp"),
};
pub const FLOAT_EQUALITY: Lint = Lint {
    code: Code("L0301"),
    name: "float_equality",
    default: Level::Warn,
    title: "Exact float equality comparison",
    summary: "`==` or `!=` on floats",
    explanation: include_str!("../../../spec/lints/float_equality/explain.md"),
    trigger: include_str!("../../../spec/lints/float_equality/before.cp"),
    fix: include_str!("../../../spec/lints/float_equality/after.cp"),
    near_miss: include_str!("../../../spec/lints/float_equality/near_miss.cp"),
};
pub const DOMAIN_KEYED_MAP: Lint = Lint {
    code: Code("L0412"),
    name: "domain_keyed_map",
    default: Level::Warn,
    title: "Map or Set keyed by a domain",
    summary: "a `Map` or `Set` keyed by a domain",
    explanation: include_str!("../../../spec/lints/domain_keyed_map/explain.md"),
    trigger: include_str!("../../../spec/lints/domain_keyed_map/before.cp"),
    fix: include_str!("../../../spec/lints/domain_keyed_map/after.cp"),
    near_miss: include_str!("../../../spec/lints/domain_keyed_map/near_miss.cp"),
};
pub const MEMO_KEY_SPACE: Lint = Lint {
    code: Code("L0413"),
    name: "memo_key_space",
    default: Level::Warn,
    title: "`@memo` table exceeds memory budget",
    summary: "a `@memo` whose table does not fit in memory at the input bounds",
    explanation: include_str!("../../../spec/lints/memo_key_space/explain.md"),
    trigger: include_str!("../../../spec/lints/memo_key_space/before.cp"),
    fix: include_str!("../../../spec/lints/memo_key_space/after.cp"),
    near_miss: include_str!("../../../spec/lints/memo_key_space/near_miss.cp"),
};
pub const UNPROVEN_LIST_INDEX: Lint = Lint {
    code: Code("L0414"),
    name: "unproven_list_index",
    default: Level::Warn,
    title: "Unproven `List` index in nested loop",
    summary: "a `List` index inside nested loops that is checked on every access",
    explanation: include_str!("../../../spec/lints/unproven_list_index/explain.md"),
    trigger: include_str!("../../../spec/lints/unproven_list_index/before.cp"),
    fix: include_str!("../../../spec/lints/unproven_list_index/after.cp"),
    near_miss: include_str!("../../../spec/lints/unproven_list_index/near_miss.cp"),
};
pub const LOOP_INVARIANT_MOD_DIV: Lint = Lint {
    code: Code("L0415"),
    name: "loop_invariant_mod_div",
    default: Level::Warn,
    title: "Loop-invariant `Mod` divisor inside loop",
    summary:
        "a `Mod<P>` division by a loop-invariant value computes an O(log P) inverse every iteration",
    explanation: include_str!("../../../spec/lints/loop_invariant_mod_div/explain.md"),
    trigger: include_str!("../../../spec/lints/loop_invariant_mod_div/before.cp"),
    fix: include_str!("../../../spec/lints/loop_invariant_mod_div/after.cp"),
    near_miss: include_str!("../../../spec/lints/loop_invariant_mod_div/near_miss.cp"),
};
pub const SEARCH_LOOP: Lint = Lint {
    code: Code("L0501"),
    name: "search_loop",
    default: Level::Warn,
    title: "Flag-and-break search loop",
    summary: "a loop that sets a flag or `?T` and breaks: `first`, `any` or `all`",
    explanation: include_str!("../../../spec/lints/search_loop/explain.md"),
    trigger: include_str!("../../../spec/lints/search_loop/before.cp"),
    fix: include_str!("../../../spec/lints/search_loop/after.cp"),
    near_miss: include_str!("../../../spec/lints/search_loop/near_miss.cp"),
};
pub const DISCARDED_RESULT: Lint = Lint {
    code: Code("L0502"),
    name: "discarded_result",
    default: Level::Warn,
    title: "Discarded function return value",
    summary: "`let _ =` throws away what a fn of this file returns",
    explanation: include_str!("../../../spec/lints/discarded_result/explain.md"),
    trigger: include_str!("../../../spec/lints/discarded_result/before.cp"),
    fix: include_str!("../../../spec/lints/discarded_result/after.cp"),
    near_miss: include_str!("../../../spec/lints/discarded_result/near_miss.cp"),
};
pub const SYMMETRIC_WRITE: Lint = Lint {
    code: Code("L0503"),
    name: "symmetric_write",
    default: Level::Warn,
    title: "Duplicate symmetric write",
    summary: "the same fact written for `(u, v)` and then `(v, u)`",
    explanation: include_str!("../../../spec/lints/symmetric_write/explain.md"),
    trigger: include_str!("../../../spec/lints/symmetric_write/before.cp"),
    fix: include_str!("../../../spec/lints/symmetric_write/after.cp"),
    near_miss: include_str!("../../../spec/lints/symmetric_write/near_miss.cp"),
};
pub const LOOP_REDUCTION: Lint = Lint {
    code: Code("L0504"),
    name: "loop_reduction",
    default: Level::Warn,
    title: "Manual accumulation loop",
    summary: "a loop that only accumulates: `sum`, `count`, `max` or `min`",
    explanation: include_str!("../../../spec/lints/loop_reduction/explain.md"),
    trigger: include_str!("../../../spec/lints/loop_reduction/before.cp"),
    fix: include_str!("../../../spec/lints/loop_reduction/after.cp"),
    near_miss: include_str!("../../../spec/lints/loop_reduction/near_miss.cp"),
};
pub const INDEX_ONLY_LOOP: Lint = Lint {
    code: Code("L0505"),
    name: "index_only_loop",
    default: Level::Warn,
    title: "Index loop that only reads `xs[i]`",
    summary: "`for i in 0..xs.len` that only reads `xs[i]`",
    explanation: include_str!("../../../spec/lints/index_only_loop/explain.md"),
    trigger: include_str!("../../../spec/lints/index_only_loop/before.cp"),
    fix: include_str!("../../../spec/lints/index_only_loop/after.cp"),
    near_miss: include_str!("../../../spec/lints/index_only_loop/near_miss.cp"),
};
pub const REPEATED_LITERAL: Lint = Lint {
    code: Code("L0506"),
    name: "repeated_literal",
    default: Level::Warn,
    title: "Repeated large literal",
    summary: "a big literal (a bound or modulus) written in several places",
    explanation: include_str!("../../../spec/lints/repeated_literal/explain.md"),
    trigger: include_str!("../../../spec/lints/repeated_literal/before.cp"),
    fix: include_str!("../../../spec/lints/repeated_literal/after.cp"),
    near_miss: include_str!("../../../spec/lints/repeated_literal/near_miss.cp"),
};
pub const PUSH_LOOP: Lint = Lint {
    code: Code("L0507"),
    name: "push_loop",
    default: Level::Warn,
    title: "Loop that only pushes",
    summary: "a list built by a loop that only pushes: a comprehension",
    explanation: include_str!("../../../spec/lints/push_loop/explain.md"),
    trigger: include_str!("../../../spec/lints/push_loop/before.cp"),
    fix: include_str!("../../../spec/lints/push_loop/after.cp"),
    near_miss: include_str!("../../../spec/lints/push_loop/near_miss.cp"),
};
pub const NONE_CHECK_UNWRAP: Lint = Lint {
    code: Code("L0508"),
    name: "none_check_unwrap",
    default: Level::Warn,
    title: "Check for `none`, then unwrap",
    summary: "`x != none and .. x!`: the check and the unwrap say one thing twice",
    explanation: include_str!("../../../spec/lints/none_check_unwrap/explain.md"),
    trigger: include_str!("../../../spec/lints/none_check_unwrap/before.cp"),
    fix: include_str!("../../../spec/lints/none_check_unwrap/after.cp"),
    near_miss: include_str!("../../../spec/lints/none_check_unwrap/near_miss.cp"),
};
pub const FREE_FOLD: Lint = Lint {
    code: Code("L0509"),
    name: "free_fold",
    default: Level::Warn,
    title: "Fold or sort of a sequence written as a free function",
    summary: "`sum(xs)`, `count(xs, p)` or `sort(mut xs)`: the method reads on the sequence",
    explanation: include_str!("../../../spec/lints/free_fold/explain.md"),
    trigger: include_str!("../../../spec/lints/free_fold/before.cp"),
    fix: include_str!("../../../spec/lints/free_fold/after.cp"),
    near_miss: include_str!("../../../spec/lints/free_fold/near_miss.cp"),
};
pub const BY_TWIN: Lint = Lint {
    code: Code("L0510"),
    name: "by_twin",
    default: Level::Warn,
    title: "`_by` twin instead of a `by:` option",
    summary: "`min_by`, `max_by`, `argmax_by`, `runs_by`, `group_by`, `first_ge_by`: pass `by:` to the base operation",
    explanation: include_str!("../../../spec/lints/by_twin/explain.md"),
    trigger: include_str!("../../../spec/lints/by_twin/before.cp"),
    fix: include_str!("../../../spec/lints/by_twin/after.cp"),
    near_miss: include_str!("../../../spec/lints/by_twin/near_miss.cp"),
};
pub const DOMAIN_SPELLING: Lint = Lint {
    code: Code("L0512"),
    name: "domain_spelling",
    default: Level::Warn,
    title: "Non-canonical domain/sequence method spelling",
    summary: "`D.sorted(by:)` -> `D.sort(by:)`; `xs.first(where: p)` -> `xs.first(p)`",
    explanation: include_str!("../../../spec/lints/domain_spelling/explain.md"),
    trigger: include_str!("../../../spec/lints/domain_spelling/before.cp"),
    fix: include_str!("../../../spec/lints/domain_spelling/after.cp"),
    near_miss: include_str!("../../../spec/lints/domain_spelling/near_miss.cp"),
};
pub const PREFIX_TWIN: Lint = Lint {
    code: Code("L0513"),
    name: "prefix_twin",
    default: Level::Warn,
    title: "Dimension-suffixed prefix or diff helper instead of `prefix`/`Prefix`/`Diff`",
    summary: "`prefix1`/`prefix2`/`prefix_n`, `Prefix1`/`Prefix2`/`PrefixN`, `Diff1`/`Diff2`: write `prefix`, `Prefix`, or `Diff`",
    explanation: include_str!("../../../spec/lints/prefix_twin/explain.md"),
    trigger: include_str!("../../../spec/lints/prefix_twin/before.cp"),
    fix: include_str!("../../../spec/lints/prefix_twin/after.cp"),
    near_miss: include_str!("../../../spec/lints/prefix_twin/near_miss.cp"),
};

pub const TREE_ORDER: Lint = Lint {
    code: Code("L0511"),
    name: "tree_order",
    default: Level::Warn,
    title: "`preorder` or `postorder` of a rooted tree",
    summary: "`t.preorder` / `t.postorder`: iterate the preorder array `t.at` (`t.at.rev` for children first)",
    explanation: include_str!("../../../spec/lints/tree_order/explain.md"),
    trigger: include_str!("../../../spec/lints/tree_order/before.cp"),
    fix: include_str!("../../../spec/lints/tree_order/after.cp"),
    near_miss: include_str!("../../../spec/lints/tree_order/near_miss.cp"),
};
pub const HAND_ROLLED_RANGE_TREE: Lint = Lint {
    code: Code("L0514"),
    name: "hand_rolled_range_tree",
    default: Level::Warn,
    title: "Hand-rolled Fenwick or segment tree",
    summary: "manual BIT (`p += p & -p`) or segment-tree (`2 * p` / `2 * p + 1`) loop: use `std.fenwick` / `std.segtree` / `std.lazy`",
    explanation: include_str!("../../../spec/lints/hand_rolled_range_tree/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_range_tree/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_range_tree/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_range_tree/near_miss.cp"),
};
pub const SORTED_RANGE_METHOD: Lint = Lint {
    code: Code("L0515"),
    name: "sorted_range_method",
    default: Level::Warn,
    title: "Removed bound-query method on `SortedBag` or `SortedSet`",
    summary: "`first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range` on `SortedBag`/`SortedSet`: use `.range(r)`",
    explanation: include_str!("../../../spec/lints/sorted_range_method/explain.md"),
    trigger: include_str!("../../../spec/lints/sorted_range_method/before.cp"),
    fix: include_str!("../../../spec/lints/sorted_range_method/after.cp"),
    near_miss: include_str!("../../../spec/lints/sorted_range_method/near_miss.cp"),
};
pub const HAND_ROLLED_STRING_ALGO: Lint = Lint {
    code: Code("L0516"),
    name: "hand_rolled_string_algo",
    default: Level::Warn,
    title: "Hand-rolled string algorithm",
    summary: "manual prefix-function, Z-function, polynomial hash, or trie loop: use `std.strings`",
    explanation: include_str!("../../../spec/lints/hand_rolled_string_algo/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_string_algo/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_string_algo/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_string_algo/near_miss.cp"),
};
pub const HAND_ROLLED_DSU: Lint = Lint {
    code: Code("L0517"),
    name: "hand_rolled_dsu",
    default: Level::Warn,
    title: "Hand-rolled union-find (DSU)",
    summary: "manual parent-pointer path compression: use `std.dsu`",
    explanation: include_str!("../../../spec/lints/hand_rolled_dsu/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_dsu/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_dsu/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_dsu/near_miss.cp"),
};
pub const HAND_ROLLED_SIEVE: Lint = Lint {
    code: Code("L0518"),
    name: "hand_rolled_sieve",
    default: Level::Warn,
    title: "Hand-rolled prime or SPF sieve",
    summary: "manual Eratosthenes or linear sieve loop: use `std.math.Sieve`",
    explanation: include_str!("../../../spec/lints/hand_rolled_sieve/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_sieve/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_sieve/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_sieve/near_miss.cp"),
};
pub const HAND_ROLLED_BINARY_LIFTING: Lint = Lint {
    code: Code("L0519"),
    name: "hand_rolled_binary_lifting",
    default: Level::Warn,
    title: "Hand-rolled binary lifting table",
    summary: "manual `up[k][v] = up[k - 1][up[k - 1][v]]` table: use `std.tree`",
    explanation: include_str!("../../../spec/lints/hand_rolled_binary_lifting/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_binary_lifting/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_binary_lifting/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_binary_lifting/near_miss.cp"),
};

pub const HAND_ROLLED_ANSWER_SEARCH: Lint = Lint {
    code: Code("L0520"),
    name: "hand_rolled_answer_search",
    default: Level::Warn,
    title: "Hand-rolled binary search",
    summary:
        "manual `while lo < hi` midpoint loop over a predicate: use `bsearch_first`/`bsearch_last`",
    explanation: include_str!("../../../spec/lints/hand_rolled_answer_search/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_answer_search/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_answer_search/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_answer_search/near_miss.cp"),
};
pub const HAND_ROLLED_PREFIX_SUM: Lint = Lint {
    code: Code("L0521"),
    name: "hand_rolled_prefix_sum",
    default: Level::Warn,
    title: "Hand-rolled prefix sum",
    summary: "manual `p[i + 1] = p[i] + x` or `p[i.r] = p[i.l] + x` loop: use `prefix(...)`",
    explanation: include_str!("../../../spec/lints/hand_rolled_prefix_sum/explain.md"),
    trigger: include_str!("../../../spec/lints/hand_rolled_prefix_sum/before.cp"),
    fix: include_str!("../../../spec/lints/hand_rolled_prefix_sum/after.cp"),
    near_miss: include_str!("../../../spec/lints/hand_rolled_prefix_sum/near_miss.cp"),
};
pub const COPY_SORT_WRITEBACK: Lint = Lint {
    code: Code("L0522"),
    name: "copy_sort_writeback",
    default: Level::Warn,
    title: "Copy-mutate-writeback on a slice",
    summary:
        "copying `xs[l..r]` to sort/reverse/rotate and writing it back: mutate `xs[l..r]` in place",
    explanation: include_str!("../../../spec/lints/copy_sort_writeback/explain.md"),
    trigger: include_str!("../../../spec/lints/copy_sort_writeback/before.cp"),
    fix: include_str!("../../../spec/lints/copy_sort_writeback/after.cp"),
    near_miss: include_str!("../../../spec/lints/copy_sort_writeback/near_miss.cp"),
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
    // A library module is where Fenwick and segment-tree loops belong.
    let library = file.top_level().any(|(_, stmt)| stmt.public);
    let mut findings = allows.unknown;
    findings.extend(rules::run(file, config));

    let mut out: Vec<Diagnostic> = Vec::new();
    for Finding { lint, mut diag } in findings {
        let level = config.level(lint);
        let allowed = allows
            .regions
            .iter()
            .any(|(span, name)| *name == lint.name && span.contains(diag.span));
        let library_only = library && lint.code == HAND_ROLLED_RANGE_TREE.code;
        if level == Level::Allow || allowed || library_only {
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

#[cfg(test)]
mod record_tests;

#[cfg(test)]
mod std_tests;
