# Rebuild round 27: holes

Cold read of `spec/` and root `std/` only (base `0c2e1f99`, which contains `0edd62851`).
Ten Codeforces problems from contests 1950-1999, rated 1600-2200. This first part
was written before any `cpl` command was run.

## Summary

| Class | Count | Inside Milestone 1 |
|---|---|---|
| silent | 13 | 12 |
| contradiction | 2 | 2 |
| missing-stdlib | 2 | 2 |
| found-late | 0 | 0 |
| **total** | **17** | **16** |

"Inside Milestone 1" means the rule section or library family is *supported* in
`spec/SUBSET.md`. The one outside is H07 (`std-subset` is *documented* only).

## Top five risks

1. **H03 + H04: `components` over a grid lambda.** 01 needs `components` over the
   cells `(R, C)` with no `from:` to fix the vertex type, and needs a tuple of two
   domains to count as a dense `Domain`. If either fails, 01 falls back to a manual BFS.
2. **H11: `dijkstra` over a two-parameter lambda on tuple vertices.** 08 depends on the
   only example being a spec-only fragment.
3. **H09 + H10: `@check` under `@!multitest` and typed `read_output`.** 10 depends on
   where the checker goes and how `read_output` takes a type.
4. **H07: order of `S.subsets`.** 07 is correct only if subsets come in increasing
   mask order, or at least with every subset before its supersets.
5. **H14: printing a range.** 10 prints `1..=n` as a permutation line. If a range is not
   `Show` as its items, every operation line is wrong.

## Holes

| ID | Class | Location | What is unclear | What I guessed | M1 |
|---|---|---|---|---|---|
| H01 | silent | `spec/core/02-values-types.md:169` (`Rooted(g, root: r)`), `std/tree.cp:99` (`new(next, root)`) | spec passes `root:` by name; std's parameter is positional | a named argument may name a positional parameter | yes (std-tree) |
| H02 | contradiction | `spec/lib/graphs-trees.md` Trees table (`t.children(v)`: "`Indexed` view"), `std/tree.cp:286` (`-> yields V`) | is `children` indexable (`.len`, `[i]`) or only a generator? | used only in `for` (not needed in the end) | yes (std-tree) |
| H03 | silent | `spec/lib/graphs-trees.md` "Customizing" ("The vertex type comes from `from:`"), `std/graph.cp:685` `components(g: Next<V>)` | `components` has no `from:`; how is a lambda's vertex type found? | annotate the parameter: `(p: (R, C)) => ...` | yes (std-graph) |
| H04 | silent | `spec/lib/graphs-trees.md` "Storage follows the vertex type"; `spec/core/04-indices.md` product domains | is a tuple of domains `(R, C)` itself a `Domain` (dense `[V]` storage, `for s in V`)? | yes, it is the product domain | yes (R04.2) |
| H05 | silent | `spec/core/04-indices.md` product domains, `spec/lib/graphs-trees.md` `p[v]` | for a result keyed by a tuple vertex, is it `comp[r, c]` or `comp[(r, c)]`; `p[(i, s)]` for `Paths` | `comp[r, c]` for arrays, `p[(i, s)]` for `Paths` | yes (R04.2) |
| H06 | contradiction | `spec/core/04-indices.md` (`V.count` member, O(1)), `spec/core/07-sequences.md` (`.count(pred)` fold) | on a domain, `C.count` is the size, so `C.count(c => ...)` is ambiguous | avoided: counted with arrays | yes (R07.9) |
| H07 | silent | `spec/lib/search-dp.md:126-138` (`B.subsets`) | iteration order of `for s in S.subsets` | increasing mask order, so subsets come before supersets | no (std-subset documented) |
| H08 | silent | `std/graph.cp:117` (`Tree<V> = [0..V.len - 1]...`) vs `V.count` in the spec | does `V.len` exist on a domain? How many lines does `read<Tree<V>>()` read? | avoided: `read<[n - 1](u: V, v: V)>()` | yes (std-graph) |
| H09 | silent | `spec/lib/random-stress.md:61-67` | under `@!multitest`, does `@check fn` go above or below the divider? | above, as shared code; its `read()` sees one test | yes (Rstress) |
| H10 | silent | `spec/lib/random-stress.md:63` | `read_output()` is named but has no typed form | it mirrors `read`: `read_output<T>()` and `let x: T = read_output()` | yes (Rstress) |
| H11 | silent | `spec/lib/graphs-trees.md:145` (spec-only fragment) | `dijkstra((i, s) => ..., from: [((a, b), 0)])`: does a two-parameter lambda take a tuple vertex? Must arcs be `(to: (R, S), w:)` records? | yes, with a generator `fn` that yields `(to:, w:)` records | yes (std-graph) |
| H12 | missing-stdlib | `std/strings.cp:391` `z_function`, `:541` `prefix_matches` | no "LCP of each text suffix with a pattern" (Z matching); `prefix_matches` is the KMP end state | build `T#s` with a comprehension, then run `z_function` | yes (std-strings) |
| H13 | missing-stdlib | `spec/core/07-sequences.md` (only lazy `chain`) | no eager concatenation of two `Indexed` sequences to feed a matcher | a comprehension over `0..2n+1` with branches | yes (R07.8) |
| H14 | silent | `spec/core/10-io.md` printing ("print(xs) one line") | is a `Range` `Show` as its items separated by spaces? | yes: `print(1, i, 1..=n)` prints the permutation | yes (R10.3) |
| H15 | silent | `spec/core/07-sequences.md` folds (`min(by:)`) | on a domain, does `D.min(by: f)` return the argmin element or the minimum value? | the element, `?D` | yes (R07.9) |
| H16 | silent | `spec/core/06-control-functions.md` nested fns; `spec/core/10-io.md` `@!multitest` | can a `fn` declared below the divider capture that test's `let`s (`a`, `n`, `s`)? | yes, like any nested fn | yes (R06.2) |
| H18 | silent | `spec/core/06-control-functions.md` (`if let`) | is `if let Some(k) = e: x else: y` an expression? | yes | yes (R01.8) |

(H17 was dropped while drafting: `p.adj4` bounds come from the domains, as documented.)

## Per-problem mapping

| File | CF | Rating | Required class | Std used | Holes |
|---|---|---|---|---|---|
| 01-1985H1 | 1985H1 Maximize the Largest Component (Easy) | 1700 | range update (difference arrays over rows and columns) | `components`, grid `adj4`, `[R, C]Char` read | H03 H04 H05 H06 |
| 02-1994C | 1994C Hungry Games | 1600 | two pointers (+ suffix DP) | `[n]T` read | H18 |
| 03-1987D | 1987D World is Mine | 1800 | game theory (DP over value groups) | `[A, B]?Int` table, `min=`, `last_true` | - |
| 04-1992G | 1992G Ultra-Meow | 2000 | factorials mod a prime | `Mod`, `Comb<Z>.choose` | - |
| 05-1988D | 1988D The Omnipotent Monster Killer | 2000 | tree DP | `Rooted`, `t.order.rev`, `t.parent`, `min(by:)` | H01 H08 H15 |
| 06-1975D | 1975D Paint the Tree | 1700 | LCA / binary lifting (`t.dist`, `t.jump`) | `Rooted`, `bfs` | H01 H08 |
| 07-1950G | 1950G Shuffling Songs | 1900 | bitmask DP | `S.subsets`, `Subset.single/with/items/len` | H07 |
| 08-1990D | 1990D Grid Puzzle | 1800 | shortest path over (row, carried-half) states | `dijkstra` on a lambda | H04 H05 H11 H16 |
| 09-1979D | 1979D Fixing a Binary String | 1800 | string matching (Z function) | `z_function`, `common_prefix_len` | H12 H13 H16 |
| 10-1956C | 1956C Nene's Magical Matrix | 1600 | constructive, any answer, `@check` | `@check`, `read_output` | H09 H10 H14 |

No in-range problem tagged "shortest paths" needs states, so 08 models a row DP as
Dijkstra over `(Row, Carried)`. 09 also accepts any answer; its stress uses a checker.

## Revision test

| File | Change | Lines / chunks changed | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|
| 02-1994C-rev (DP) | print the answer for every prefix `a[1..=i]`, not only the total | header comment, plus one chunk: the 4-line backward suffix DP becomes an 11-line forward chain count; the input and two-pointer chunk are untouched | yes. The suffix DP (`good[l]`) can't answer per prefix, so the recurrence is rethought (chains restart at `nxt + 1`). This is algorithmic, not the language | none | none |
| 09-1979D-rev (non-DP) | print the largest valid `p`, not any `p` | header comment, plus one chunk of 5 lines: an accumulator `best: ?Int` with `max=`, a reversed loop `(1..=min(agree, n)).rev`, `return best` | small: the early `return Some(p)` had to become an accumulator because both targets must be tried. `max=` on `?Int` made it one line | none | none |

## After running

Everything below was run after the part above was written, with `cargo build -p cpl-cli`
(debug) at the same base. Per file: `cpl check`; `cpl run` on the statement sample
(each sample is one multitest input holding all of the statement's cases); `cpl test`
on `NN-<cfid>.tests/1.in` / `1.out`; `cpl build` + `g++ -std=c++17 -O2` and the native
binary on the sample; 200 random small inputs (1-3 cases each) through `cpl run`, the
native binary and an independent Python brute force (a validity checker for 09 and 10,
plus an exhaustive search for whether any `p` exists in 09); one maximum-size input
timed on the native binary (wall clock, one run).

### Results per file (final versions)

| File | check | sample (run) | test | build + g++ | native = interp on sample | stress 200 | max input | native time |
|---|---|---|---|---|---|---|---|---|
| 01-1985H1 | ok after 3 fixes | ok | ok | ok | yes | 200/200 | 1000 x 1000 random grid | 0.057 s |
| 02-1994C | ok | ok | ok | ok | yes | 200/200 | n = 2e5, x = 1e9 | 0.009 s |
| 02-1994C-rev | ok | brute-checked | ok | ok | yes | 200/200 | n = 2e5 | 0.015 s |
| 03-1987D | ok | ok | ok | ok | yes | 200/200 | n = 5000, all distinct | 0.110 s |
| 04-1992G | ok | ok | ok | ok | yes | 200/200 | n = 5000 | 0.081 s |
| 05-1988D | ok | ok | ok | ok | yes | 200/200 | n = 3e5, a up to 1e12 | 0.094 s |
| 06-1975D | ok | ok | ok | ok | yes | 200/200 | n = 2e5 path, a = 1, b = n | 0.034 s |
| 07-1950G | ok after 1 fix | ok | ok | ok | yes | 200/200 | n = 16, 1e4-char strings | 0.031 s |
| 08-1990D | ok after 1 fix | ok | ok | ok | yes | 200/200 | n = 2e5 | 0.019 s |
| 09-1979D | ok | valid (other `p`) | ok after adding `@check` | ok | yes | 200/200 | two cases, n = 1e5 | 0.009 s |
| 09-1979D-rev | ok | brute-checked | ok | ok | yes | 200/200 | two cases, n = 1e5 | 0.019 s |
| 10-1956C | ok after 1 fix | valid (other construction) | ok (`@check`) | ok | yes | 200/200 | two cases, n = 500 | 0.031 s |

The `-rev` files have no statement output; their sample output was compared with the
Python brute force for the changed requirement. Slowest maximum run: 03-1987D, 0.110 s.
The interpreter and the native binary agreed on every sample and on all 2400 stress
inputs, so no sanitizer rerun was needed (none was run). For `cpl test` of 10, the
expected file holds the statement's construction and ours differs, so the pass is the
`@check`'s; a deliberately wrong `print(1)` in a copy of 09 was rejected by its `@check`.

### Failures, with the smallest reproduction

No solution failed a sample or a stress case once it compiled. Every failure was at
`cpl check` (or a wrong first sample run) and came from a guess in the table above,
except F7.

| # | File | Kind | Smallest reproduction | What happened | Fix in the file |
|---|---|---|---|---|---|
| F1 | 01 | spec hole (found-late, H19) | `domain R = 0..2` / `domain C = 0..2` / `let (domain K, comp) = components((p: (R, C)) => p.adj4)` | E0201 "no `components` in scope", help: `use std.graph.{components}`. The spec never says which std names are in the prelude; `dijkstra`, `bfs`, `Rooted`, `z_function` were | (superseded by F2) |
| F2 | 01 | spec hole (H03 + H04 wrong) | same, with the `use` | E0302 "`components` needs a domain for `V`, and `Int` is not one": a tuple of two domains is not a `Domain`, and the lambda's annotation does not supply one | hand-written flood fill with a stack over `[R, C]?Int` |
| F3 | 01 | spec hole (H05 wrong) | `comp[r, c]` where `comp` is keyed by a tuple vertex | E0305 "`comp` takes 1 index"; `comp[(r, c)]` checks | `comp[(r, c)]`, then superseded by F2 |
| F4 | 07 | compiler gap vs std (found-late, H20) | `domain S = 0..3` / `let s = Subset<S>.single(S(1))` / `for i in s.items: print(i)` | E0302 "a `for` can't walk over a `Items<Subset<S>>`", although `std/subset.cp` declares `items` as `-> yields` | `for i in S if i in s` |
| F5 | 08 | compiler gap vs spec example (H11 wrong) | `dijkstra((i, s) => mv(i, s), from: [((A.first!, B(0)), 0)])` | E0302: the two parameters are typed `Int`, the lambda does not destructure the tuple vertex as the spec fragment shows | `(v: (Row, Carried)) => moves(v.0, v.1)` |
| F6 | 10 | spec hole (H10 half wrong) | `@check fn ok() -> Bool:` / `    let p = read_output<[2]Int>()` / `    true` | E0501 "`read_output` is not in scope during lowering"; `let p: [2]Int = read_output()` works | annotated `let` |
| F7 | 10 | spec hole (H14 wrong) | `print(1..=3)` | prints `1..4`, not `1 2 3` | `(1..=n).collect()` |
| F8 | 09 | implementation bug (mine) | the statement sample, case `5 5` / `00000` | `cpl test` failed: we print `1` where the sample prints `5`. Both are valid; the problem accepts any `p`, and the first version had no `@check` | added a brute-force `@check` |

### Which guessed holes held

| Hole | Outcome |
|---|---|
| H01 | held: `Rooted(g, root: r)` checks and runs (05, 06) |
| H02 | avoided (only `for` over `children`); not tested |
| H03, H04 | failed together: became found-late (F2) |
| H05 | failed for arrays keyed by a tuple vertex: found-late (F3); `p[(i, s)]` on `Paths` held (08) |
| H06 | avoided; not tested |
| H07 | held: 07 agrees with an exhaustive permutation brute force on 200 inputs |
| H08 | avoided; not tested |
| H09 | held: `@check` above `@!multitest` reads one test (`cpl test` on 09 and 10) |
| H10 | half failed: `read_output<T>()` rejected, the annotated form works: found-late (F6) |
| H11 | failed: found-late (F5); arcs as `(to:, w:)` records held |
| H12, H13 | held: the comprehension + `z_function` workaround is correct on 400 stress inputs (09, 09-rev) |
| H14 | failed: found-late (F7) |
| H15 | held: `min(by:)` on a domain gives the element (05) |
| H16 | held: nested `fn` below the divider sees the test's `let`s (08, 09) |
| H18 | held: `if let ... else` as an expression (02) |

New holes found only by running:

| ID | Class | Location | What is unclear | M1 |
|---|---|---|---|---|
| H19 | found-late | `spec/lib/graphs-trees.md`, `spec/core/11-modules-prelude.md` | which std names are in the prelude; `components` is not, its neighbours are | yes (std-graph) |
| H20 | found-late | `std/subset.cp` `items` (`@get ... -> yields`), `spec/lib/search-dp.md` | `for` over a getter generator on `Subset` is rejected | no (std-subset documented) |

### Counts after running

Guesses that failed move to found-late.

| Class | Before running | After running | Inside Milestone 1 (after) |
|---|---|---|---|
| silent | 13 | 7 | 6 |
| contradiction | 2 | 2 | 2 |
| missing-stdlib | 2 | 2 | 2 |
| found-late | 0 | 8 (H03 H04 H05 H10 H11 H14 H19 H20) | 7 |
| **total** | **17** | **19** | **17** |
