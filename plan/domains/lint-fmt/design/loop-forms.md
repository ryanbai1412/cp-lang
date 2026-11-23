# Loop forms (2026-11-22)

Draft for M3 workstream 5 ("recognizers"), renamed from "shapes" because
`cpl-check` already uses "shapes" for grid shapes (tech-debt review point 3).
spec/ does not describe how the linter recognizes loops; this note does.

## Today
The loop-shaped taste lints each match their own pattern on the syntax tree:

| Matcher | File | Lint |
|---|---|---|
| `Taste::reduction` (`var x = i` then `for ..: if c: x = v`) | `rules/taste/mod.rs` | L0504 loop_reduction |
| `Taste::search` (set, then `break`) | `rules/taste/mod.rs` | L0501 search_loop |
| `Taste::collect` (`var xs = []` then `push`) | `rules/taste/mod.rs` | L0507 push_loop |
| `Taste::index_only` | `rules/taste/mod.rs` | L0505 index_only_loop |
| `Taste::answer_search`, `search::halving_loop` | `rules/taste/mod.rs`, `search.rs` | L0519 hand_rolled_answer_search |
| `Taste::prefix_sum`, `stdlib_algo::prefix_sum_loop` | `rules/taste/mod.rs`, `stdlib_algo.rs` | L0520 hand_rolled_prefix_sum |
| `Taste::copy_mutate_writeback` | `rules/taste/mod.rs` | L0522 copy_sort_writeback |
| `is_sieve_loop`, `is_binary_lifting_loop`, DSU and segment-tree loops | `stdlib_algo.rs`, `range_tree.rs` | L0514-L0518 hand_rolled_* |

The first three each re-check the same thing: a `var` declared, then a `for`
loop over one iterable that only writes that `var`. A `while` loop that does
the same job matches none of them.

## Proposal
`rules/forms.rs` classifies each loop once per file, before the lints run:

```rust
enum LoopForm<'a> {
    Fold { acc: &'a str, op: FoldOp, init: &'a Expr },  // L0504
    Search { found: &'a str, init: &'a Expr },          // L0501
    Build { list: &'a str },                            // L0507
    Scan { table: &'a str },                            // L0520
    Bisect { lo: &'a str, hi: &'a str },                // L0519
    IndexSpace { seq: &'a str },                        // L0505
    CopyWriteback { src: &'a str },                     // L0522
}
```

The lints become functions of a loop's form and its iteration source. A
`while i < n: ..; i += 1` loop over `0..n` gets the same form as
`for i in 0..n`, so the two agree (design/overview.md). The `hand_rolled_*`
matchers stay specific to their algorithm, but they start from a form (a
sieve is a `Build` over a stepped range) instead of re-walking the loop.

Forms are lint-internal. A form that LSP hover or `cpl explain` shows
("this loop is a fold") is a fact, so it moves to the `cpl-check` fact
table, requested in the compiler worklog. `cpl-lsp` never reads forms from
`cpl-lint`.

## Order, each a commit with findings unchanged
1. Fold, Search and Build from `reduction`, `search` and `collect`. Proof:
   identical findings, compared by code, file and span, on corpus `spec.cp`,
   `port.cp`, `dream.cp` and `std` before and after. The record tests
   and `corpus_warnings_only_go_down` must also pass.
2. Scan and Bisect. Already one recognizer each, returning a structured
   match: `stdlib_algo::prefix_sum_loop` (`PrefixLoop`) and
   `search::halving_loop` (`Halving`), each read by one lint. Moving them
   into `forms.rs` would only move files, so it waits until a second lint
   needs either one.
3. IndexSpace and CopyWriteback: the same holds.
4. `while` normalization. This is the first step that adds findings: each
   new one needs a near-miss case or a corpus run with 0 false positives.
   Deferred because there is little evidence for it: a scan finds 35 counted
   `while v < ..: ..; v += 1` loops (31 corpus `port.cp`, 2 `spec.cp`, 2
   `dream.cp`), and many of them are searches with a compound condition
   (`while idx < n and a[idx] == v`), not folds.

Status: step 1 landed with findings unchanged (worklog). Steps 2-4 are
deferred for the reasons above.

## Risks
- A refactor that silently drops findings. Steps 1-3 compare findings
  exactly rather than counting them.
- Forms grow into a second analysis. Forms read only syntax and the checker
  facts (`Facts`); anything that needs ranges asks `cpl-check`.
