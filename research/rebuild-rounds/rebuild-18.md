# Rebuild round 18: holes found by a cold reader

Base: `main` at `6259cc162` (later than `79b4db75d`). Before this file was
written the reader read only `spec/` (core 00-07 and 10, `lib/graphs-trees.md`
parts, `research/design-history/stdlib-surveys/seq.md` §3.5) and root `std/` (`README.md`, `GAPS.md`,
signatures and docs in `fenwick.cp`, `graph.cp`, `tree.cp`, `heap.cp`,
`segtree.cp`, `dsu.cp`, `strings.cp`, `mod.cp`). Statements came from the Luogu
mirror (Codeforces returned a block page).

| # | File | Problem | Rating | Topic | Std used |
|---|---|---|---|---|---|
| 01 | `01-1936B.cp` | 1936B Pinball | 2000 | prefix sums, two pointers | `scan` |
| 02 | `02-1941G.cp` | 1941G Rudolf and Subway | 2000 | shortest paths (0-1 BFS) | `compress`, `bfs01` |
| 03 | `03-1942D.cp` | 1942D Learning to Paint | 2100 | DP + k-way merge | `Heap` |
| 04 | `04-1943B.cp` | 1943B Non-Palindromic Substring | 2000 | strings (palindromes) | `manacher` |
| 05 | `05-1945F.cp` | 1945F Kirill and Mushrooms | 1900 | Fenwick order statistics | `Fenwick.kth` |
| 06 | `06-1946D.cp` | 1946D Birthday Gift | 1900 | bitmasks, prefix XOR | `scan(^)` |
| 07 | `07-1946E.cp` | 1946E Girl Permutation | 2200 | combinatorics mod p | `Mod`, `Comb` |
| 08 | `08-1948E.cp` | 1948E Clique Partition | 2100 | constructive | none |
| 09 | `09-1949C.cp` | 1949C Annual Ants' Gathering | 1900 | tree DP / greedy | `Graph`, `Rooted.centroid`, `size` |
| 10 | `10-1949F.cp` | 1949F Dating | 2200 | sorting, laminar sets | `[U]List<A>` read |

## Summary

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 11 | 10 |
| contradiction | 3 | 3 |
| missing-stdlib | 3 | 2 |
| found-late | 2 (H18, H19, added after running) | 2 |
| **total** | **19** | **17** |

"Inside" means the feature family is marked supported in `spec/SUBSET.md`
(sequences, strings, graphs, trees, Fenwick, modular math, I/O, search).

## Top five risks

1. **H05 / H06: `Rooted`, `Graph`, `bfs01` are generic in std but `std/GAPS.md`
   says `cpl run` resolves them to Int-typed built-ins.** 02 and 09 depend on
   domain-typed `Graph`/`Rooted`; if the built-in only takes Int vertices, both
   fail to check even though the spec examples look exactly like them.
2. **H01: no disjoint union of domains.** 02 needs one vertex space of
   stations plus line colours; the reader fell back to an Int domain `N` and
   two hand-written offset maps, which is exactly the index-space mixing that
   domains exist to stop.
3. **H02: `manacher` answers in Int offsets, not the input's keys.** 04 works
   on a plain `Str`, so it is harmless here, but on a slice it breaks keys-in /
   keys-out (R07.4.1).
4. **H09: multi-line `if` blocks as an expression value (`let t = if c: ...`
   with nested block `if`s).** 01 relies on it; the grammar shows only the
   inline form as an expression.
5. **H13: `sorted(by:)` on a generator (`t.children(v)`).** 09 sorts the
   children of each vertex; ch07 lists `sorted` without saying whether a bare
   `Seq` (not `Indexed`) has it.

## Holes

| ID | Class | Location | Question | What the reader guessed | M1? | Problems |
|---|---|---|---|---|---|---|
| H01 | missing-stdlib | `spec/core/04-indices.md` (domains), `lib/graphs-trees.md` | No sum/disjoint-union domain (`domain N = V + C`) for graphs whose vertices are two kinds of thing (stations and lines). | `domain N = 0..n + C.count` with `st(v) = N(v.ord)`, `line(c) = N(n + c.ord)`. | yes (graphs) | 02 |
| H02 | contradiction | `std/strings.cp` `Palindromes` / `manacher` vs R07.4.1 | `manacher(s)` returns `Palindromes` over `Int` offsets (`is_palindrome(r)` takes an Int range), while every per-element string result is keyed like its input. | Pass a 0-based `l..r + 1` over a whole `Str`. | yes (strings) | 04 |
| H03 | silent | `spec/core/07-sequences.md` R07.11.5, `research/design-history/stdlib-surveys/seq.md` §3.5 | `compress(s)`: does it take any `Seq` or only a `List`? Core gives only `(domain C, comp)`; the methods (`at`, `try_at`) live only in the rationale. | `compress([r.c for r in es])` then `comp.at(x)`. | yes (sequences) | 02 |
| H04 | silent | `spec/core/04-indices.md`, `spec/core/10-io.md` R10.1.1 | May a `domain` declared below `@!multitest` take its size from another fresh domain's `.count` (`0..n + C.count`)? | Yes. | yes | 02 |
| H05 | contradiction | `std/GAPS.md` (graph, tree lines) vs `spec/lib/graphs-trees.md`, `spec/core/00-overview.md` example | GAPS says `Graph`, `bfs01`, `Rooted` resolve to Int-typed compiler built-ins in `cpl run`; the spec examples use them over declared domains. | Followed the spec: `bfs01(adj, from: [st(b)])` over `[N]List<(to: N, w: Int)>`, `Rooted(g, root: ...)` over `V`. | yes (graphs, trees) | 02, 09 |
| H06 | contradiction | `std/README.md` §4.2 surface index vs `spec/core/04-indices.md`, ch07 | The surface index still lists `bsearch_first`/`bsearch_last`, `scan(xs, f) -> [D]T`, `runs(a) -> List<(val, len)>`, `Dsu` vs the user prompt's `DSU`; core uses `first_true`/`last_true`, `scan(op) -> Prefix`, `runs(by:)`. | Trusted core spec and the `.cp` stubs; used `scan(+)`, `scan(^)` as `Prefix` indexed by cuts. | yes (search, sequences) | 01, 06 |
| H07 | missing-stdlib | `spec/core/10-io.md` R10.2.3 | Upper triangle with diagonal (row `i` holds `a[i][i..=n]`) has no reader; `[Pair<V>]T` reads a lower triangle without the diagonal. | Nested loop with `read<-1e6..=1e6>()` per cell into `[V, V]Int`. | yes (I/O) | 03 |
| H08 | silent | `spec/core/04-indices.md` (ranges) | Range from a position to the end of its domain: is `i..` legal outside a slice, or must it be `i..=V.last!`? | `i..=V.last!`. | yes | 03 |
| H09 | silent | `spec/core/01-lexical-grammar.md` (if expression, lines 255-265, 361) | Is a block `if`/`else` (with nested block `if`s) an expression whose last line is the value, usable as `let t = if ...:` ? | Yes, each arm a one-line inline `if ... else`. | yes | 01 |
| H10 | silent | `std/heap.cp` `Heap.new(by:)` | With `var h: Heap<R> = Heap(by: x => desc(x.v))`, does the declared element type type the lambda parameter? | Yes. | yes (containers) | 03 |
| H11 | silent | `spec/core/07-sequences.md` R07.9.6 | `(2..len).step(2).sum()` on an empty range: 0, and O(1) closed form? | 0, closed form. | yes | 04 |
| H12 | silent | `spec/core/07-sequences.md` scans | `a.scan(^)`: is bare `^` an operation value with identity 0 for `scan` (SegTree's doc lists it, scan's does not)? | Yes; `pre[k]` folds the first `k` on a counted array. | yes | 06 |
| H13 | silent | `spec/core/07-sequences.md` "Ordering and grouping" | Does `sorted(by:)` exist on a walk-only `Seq` (generator `t.children(v)`)? | Yes, it materializes. | yes | 09 |
| H14 | missing-stdlib | `std/tree.cp` `Rooted.centroid` | Finding a centroid needs a first `Rooted` just to call `.centroid()`, then a second `Rooted` at it; no free `centroid(g)`. | Built `Rooted` twice. | no (convenience, not in SUBSET) | 09 |
| H15 | silent | `spec/core/02-values-types.md` R02.4.x (optionals) | `?T` is not `Ord`; is it `Eq` (`last[a] != x` with both `?U`)? | Yes, `None == None`. | yes | 10 |
| H16 | silent | `std/mod.cp` `Comb.new(limit:)`, R10.3 | Is `Comb<Z>(2e5)` a capacity that also grows, and does `print` show a `Mod<P>` as its value? | Grows on demand; prints the residue. | yes (modular math) | 07 |
| H17 | silent | `spec/core/03-numbers-layout.md`, R04.4.30 | `read<1..=n>() - 1`: result type of arithmetic on a range-typed value used as a plain index into a counted array `[n + 1]Int`. | `Int`. | yes | 04, 07 |

Not holes (checked, documented): `let b, e: V = read()` (R01 line 222),
`let x = e else: break` (R01 line 227), `desc` is in the prelude, bit operator
precedence (`|` binds tighter than `==`), `Fenwick.kth(k)` is 1-based and
returns `?D`, top-level `return` ends the case (R10.1.2), `[V]List<A>` reads
count-prefixed rows (R10.2.3).

## Per-problem mapping

| Problem | Holes |
|---|---|
| 01-1936B | H06, H09 |
| 02-1941G | H01, H03, H04, H05 |
| 03-1942D | H07, H08, H10 |
| 04-1943B | H02, H11, H17 |
| 05-1945F | none |
| 06-1946D | H06, H12 |
| 07-1946E | H16, H17 |
| 08-1948E | none |
| 09-1949C | H05, H13, H14 |
| 10-1949F | H15 |

Found late (after running): H18 in 02, H19 in 01 and 06.

## After running

`cargo build -p cpl-cli` on `6259cc162`. Samples from the Luogu mirror;
08 (any construction) and 10 (any pair) are judged by a checker. Stress: 200
seeded small inputs per problem through `cpl run` and the `g++ -std=c++17 -O2`
build, compared with each other and with a Python brute force or checker
(`stress/brutes.py`, `stress/stress.py`). Programs that needed a workaround to
run are in `wa/`; the top-level files stay as first written.

| Problem | `cpl check` | `cpl run` samples | `cpl build` + g++ + samples | Stress (200) |
|---|---|---|---|---|
| 01-1936B | ok | **trap** R0102 (B1) | builds; **wrong output** (B1) | wa: 200/200 vs brute |
| 02-1941G | **E0201** `no bfs01 in scope` (H18); ok after `use std.graph.{bfs01}` | 2/2 ok | 2/2 ok | 200/200 vs brute |
| 03-1942D | **E0302** `expected Int, found Desc<Int>` (B2) | wa: 1/1 ok | wa: 1/1 ok | wa: 200/200 vs brute |
| 04-1943B | ok | 1/1 ok | 1/1 ok | 200/200 vs brute |
| 05-1945F | ok | 1/1 ok | 1/1 ok | 200/200 vs brute |
| 06-1946D | ok | **trap** R0102 (B1) | builds; **wrong output** (B1) | wa: 200/200 vs brute |
| 07-1946E | ok | 1/1 ok | 1/1 ok | 200/200 vs brute (all permutations, n <= 7) |
| 08-1948E | ok | 1/1 checker ok | 1/1 checker ok | 200/200 checker (incl. q = ceil(n/k)) |
| 09-1949C | ok | 3/3 ok | 3/3 ok | 200/200 vs brute (state search, n <= 9) |
| 10-1949F | ok | 2/2 ok | 2/2 ok | 200/200 checker |

Max-size timing of the C++ builds: 01 at n = 5e5 runs in 0.03 s, 10 at
n = 2e5 (10^6 activities) in 0.05 s.

The holes guessed before running that the compiler settled as guessed:
H01-H05 (02 checks and runs with the Int union domain, `compress`/`comp.at`, the
fresh-size domain and `bfs01` over `[N]List`), H07-H17. `Rooted`, `Graph` over a
declared domain and `Rooted.centroid` all work in both backends, so the GAPS.md
risk (H05) did not bite.

### Failures

| ID | Where | Class | Smallest reproduction | What happens |
|---|---|---|---|---|
| B1 | 01, 06 | **implementation bug** (plus spec hole H19) | `wa/repro-b1-scan-index.cp`: `let a = [1, 2, 4]; let p = a.scan(+); print(p[0], p[1], p[2])` | Prints `1 2 4` (the elements) in both backends; R07.1.6 says `p[k]` on Int keys is the cut, the fold of the first `k`: `0 1 3`. |
| B1b | 01, 06 | **implementation bug** | `wa/repro-b1-scan-last-cut.cp`: `let n = 3; let a = read<[n]Int>(); let p = a.scan(+); print(p[n])` on `1 2 3` | `cpl run` traps R0102 "3 is not in `D_0`, which is 0..=2"; the C++ build prints `-6` and under `-fsanitize=address,undefined` reports heap-buffer-overflow in `Prefix_..._get`. The two backends disagree on a rejected input and the C++ has no bounds check. |
| B2 | 03 | **implementation bug** | `wa/repro-b2-heap-desc.cp`: `var h: Heap<Int> = Heap(by: x => desc(x))` | E0302 "expected `Int`, found `Desc<Int>`"; `Heap(by: desc)` (the doc example) checks. Any key type other than `Int` from a lambda is refused, though `Heap.new<K>(by: ?(T -> K))` is generic in `K`. Workaround: `by: x => -x.v`. |
| H18 | 02 | **spec hole** (found-late) | `let p = bfs01(adj, from: [s])` with no `use` | E0201 "no `bfs01` in scope; add `use std.graph.{bfs01}`". The spec's graph examples (`00-overview.md`, `lib/graphs-trees.md`) call `dijkstra`, `bfs` and `bfs01` without a `use`, and nothing the reader read says which std names are in the prelude. |
| H19 | 01, 06 | **spec hole** (found-late, contradiction) | as B1 | `std/prefix.cp` documents `p.get(i)` as the element and `p.prefix(k)` as the fold of the first `k`, and never says what `p[k]` means; `spec/core/07-sequences.md` R07.1.6 says `p[k]` on Int keys is the fold of the first `k`. The implementation follows `get`. |
| R1 | stress harness for 10 | **reader error** | 1949F checker | The reader's first checker forgot that a good match must share an activity; 35/200 "failures" were the checker's. Fixed; the solution was right. |

Counts: 3 implementation bugs (B1, B1b, B2), 2 late spec holes (H18, H19),
1 reader error (in the harness, not a solution). No solution was wrong in its
algorithm.
