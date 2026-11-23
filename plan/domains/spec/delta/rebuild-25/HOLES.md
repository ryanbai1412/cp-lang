# Rebuild round 25: holes

Cold reader: before this file I read only `spec/` (core 00-04, 06, 07 headings, 10; lib
ops-ranges, graphs-trees, random-stress headings and sections; tools cli, stress; SUBSET,
conformance examples) and `std/` signatures (segtree, graph, tree, mod, heap, strings, seq,
prelude). Ten Codeforces problems from contests 2081-2120, rated 1600-2200.

## Summary

| Class | Count | In Milestone 1 subset |
|---|---|---|
| silent | 9 | 8 |
| contradiction | 3 | 3 |
| missing-stdlib | 3 | 2 |
| found-late | 2 | 2 |
| **total** | **17** | **15** |

Coverage note: no 1600-2200 problem in 2081-2120 is solved by a Z-function, prefix function or
hashing (the window's string problems are greedy, counting or subsequence matching), so the
string slot is 2104E (subsequence automaton). All other required kinds are covered (table below).

## Top five risks

1. **H02** `dijkstra(..., step: max)` is the spec's widest-path spelling (lib/ops-ranges.md §3)
   but `std/graph.cp` `dijkstra` has no `step:`; 10-2117G hand-rolls the heap loop instead.
2. **H01** a string with both a length and an alphabet: `Str(len: n..=n) of "01"` is a guess;
   01 and 05 depend on it.
3. **H06/H11** reading a word into `[I]Char` and indexing `[I.cuts, Letter]?I` (a cut domain as an
   array axis of a 2-D table): 07 depends on both.
4. **H09** per-test state on top-level arrays (stamps instead of reallocating 400k cells per test):
   04 mutates a top-level `var` and arrays from below `@!multitest`; no rule says it may.
5. **H04/H05** no inline `@sample` syntax and no `d.tests/` file naming, so samples go through
   `cpl test --add`.

## Holes

| ID | Class | Where | What was unclear | What I guessed | Subset |
|---|---|---|---|---|---|
| H01 | silent | core/02 §2.3 (R02.3.2) | `Str of "01"` and `Str(len: ...)` are shown apart; no combined spelling | `Str(len: n..=n) of "01"` | R02.3 supported |
| H02 | contradiction | lib/ops-ranges.md §3 "Dijkstra: a monoid and an order" vs std/graph.cp `dijkstra` | spec: `dijkstra(g, from:, step: max)` for bottleneck paths; std signature has `g, from, until, hashed` only | followed std (signatures win per spec/README), hand-rolled a `MinHeap` loop with `min=` | std-graph supported |
| H03 | contradiction | lib/graphs-trees.md Trees table vs std/tree.cp `children` | spec: `t.children(v)` is an `Indexed` view; std: `-> yields V` | avoided indexing it: `t.size(v) == 1` for leaves, `List(t.children(mid))` for two kids | std-tree supported |
| H04 | silent | tools/cli.md `cpl test` | names "inline `@sample`s" but no chapter gives the syntax | no inline samples; `cpl test --add` | Rcli supported |
| H05 | silent | tools/cli.md | file naming inside `d.tests/` is not given (only `stress-<seed>.in`) | `cpl test --add FILE --expect OUT` picks it | Rcli supported |
| H06 | silent | core/10 R10.2.3 | `[R, C]Char` and `Char` reads are specified, a 1-D `[I]Char` from one word is not | `read<[I]Char>()` reads n characters, skipping whitespace like `Char` | R10.2 supported |
| H07 | silent | core/02 R02.10.2 | "an unannotated `var` widens to `Int`": for range types only, or also positions and cuts? | `var cur = I.cuts.first!` stays a cut | R02.10 supported |
| H08 | silent | lib/ops-ranges §1, prelude | arity of free `max`/`min`: two arguments only, or variadic? | nested `max(a, max(b, c))` | R11.3 supported |
| H09 | silent | core/06 §6.5, core/10 R10.1.1 | top-level `var`s are shared by every test case; may per-test code mutate them (a stamp array to avoid an O(size) reset per case)? | yes: `case_no += 1` and stamped arrays in 04 | R06.5 supported |
| H10 | found-late | core/04 R04.8.10 vs R04.4.27 | `var out: [n + 1]List<Int> = []`: a list literal for `[D]T` is one value per cell (length mismatch trap), but a scalar `[]` of type `List<Int>` is a fill; which wins? | rewrote as a table `[n + 1] _ => List<Int>()` | R04.8 supported |
| H11 | silent | core/04 R04.2.25, R04.2.5 | may a cut domain `I.cuts` be one axis of a 2-D array `[I.cuts, Letter]T`? Plain ranges and domains are shown, cuts only in 1-D | yes | R04.2 supported |
| H12 | silent | core/10 R10.2.3 | dependent tuple bound with arithmetic, `(2..=2e5, 1..=n - 1)`: shown only as `1..=n` | allowed | R10.2 supported |
| H13 | silent | lib/random-stress.md "Checkers" | a `@check` can't see a wrong "impossible" (stated); restating the feasibility fact from the input is the only way. Fine, but nothing says whether a `@check` may `return` early | early `return` after checking `-1`/`NO` | R06.4 supported |
| H14 | missing-stdlib | std/graph.cp | no widest-path search (consequence of H02); ~10 lines hand-written | hand-rolled | std-graph supported |
| H15 | missing-stdlib | std/strings.cp, std/seq.cp | no subsequence automaton / next-occurrence table (only `is_subsequence`); 2104E builds the 26n table by hand | hand-rolled `nxt` table | std-strings supported |
| H16 | missing-stdlib | std (none) | no timestamped/resettable array for "precompute once, reset only touched cells per test" | stamp arrays by hand (H09) | outside: no family |
| H17 | found-late | core/04 R04.2.20, std/seq.cp | `"LIT".min(by: ...)` / `"LIT".find(...)` on a `Str` literal: `Str` is a `Seq<Char>` (R02.3.1), so assumed every `Seq` fold works on a literal | used them | R07.9 supported |

## Problems

| File | Problem | Kind (rule 2) | Holes touched |
|---|---|---|---|
| 01-2081A | Math Division | expected value mod 1e9+7 | H01 |
| 02-2085D | Serval and Kaitenzushi Buffet | lazy segment tree (range add, range min); greedy with sorting | H12 |
| 03-2086D | Even String | combinatorics with factorials / inverse factorials | - |
| 04-2090C | Dining Hall | greedy with sorting; unusual output (coordinates) | H09, H16 |
| 05-2092D | Mishkin Energizer | constructive with `@check`; output indices 1-based into a growing string | H01, H13, H17 |
| 06-2103C | Median Splits | two pointers (shortest good prefix vs suffix) plus prefix-min scan | H12 |
| 07-2104E | Unpleasant Strings | string (subsequence automaton; see coverage note) | H06, H07, H11, H15 |
| 08-2112D | Reachability and Tree | tree constructive with `@check`; YES + edge list output | H03, H10, H13 |
| 09-2117F | Wildflower | tree with LCA; mod 1e9+7 | H03 |
| 10-2117G | Omg Graph | shortest path (widest-path Dijkstra) | H02, H08, H14 |

## Revision test

| File | Requirement change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 02-2085D-rev | also print the minutes he takes plates at, increasing | +3 (list, push, print) | 1 (the greedy loop) | none: positions print as their integers (R10.3.2) | none | none |
| 09-2117F-rev | print the answer for every root r, not only root 1 | 13 of 14 body lines | 1 (whole answer block becomes `fn count(r)`) | yes: what "leaf" means under another root, and the cost of filtering the leaf list per root (a star is O(n^2) unless the degree count short-circuits first) | the "three leaves" rule is now written as a count check before the list is built | yes: the leaf test moved from `t.size(v) == 1` (root-1 tree) to `g.deg(v) == 1`, and `t.lca` + depths became `t.meet` + `t.dist`, both forced by the root no longer being fixed |

Everything in the 2117F revision except the two branch formulas was rewritten: the per-root
change touched the leaf set, the branch vertex and the lengths, three facts the original tied to
the fixed root. `t.meet(a, b, r)` (std only, not in the spec chapter's table) made the per-root
form O(log n).
