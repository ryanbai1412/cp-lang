# Rebuild round 35: holes

Cold reader: `spec/` and root `std/` only (plus file names under `research/` and
`plan/domains/spec/delta/` for exclusions). Statements and samples from the Luogu
mirror (Codeforces returned a block page). `cpl` built from `main` at `abd8d27bf`.

Classes: **silent** (spec/std say nothing I could find; I guessed), **contradiction**
(two normative places disagree), **missing-stdlib** (no std facility for a contest
staple), **found-late** (a tool failure that needed a workaround; filled in after the
cold commit). M1 = inside the Milestone 1 subset (`spec/SUBSET.md` row status
*supported*).

## Counts

| Class | Total | In M1 |
|---|---|---|
| silent | 23 | 23 |
| contradiction | 3 | 2 |
| missing-stdlib | 1 | 1 |
| found-late | 3 | 1 |

## Problems

| # | CF | Rating | Slot | Main tools |
|---|---|---|---|---|
| 01 | 1198A MP3 | 1600 | two pointers with `maximal` | `sort`, `maximal(Counter)`, `max(by:)` |
| 02 | 1286A Garland | 1800 | DP over `[A, B]T` | `[0..=e, 0..2]?Int`, `min=`, row view + `present().min()` |
| 03 | 1207E XOR Guessing | 1900 | interactive, budget 2 | `Wire()`, fn with a counted budget |
| 04 | 1176E Cover it! | 1700 | multitest, per-test reset | `@!multitest`, per-test `domain V`, `bfs`, `Graph.undirected` |
| 05 | 1114C Trailing Loves | 1700 | 64-bit care (n up to 1e18) | divide-down Legendre, `?Int` with `min=` |
| 06 | 1200E Compress Words | 2000 | string, `prefix_function` | `Str + Str + slice`, `Str.push` |
| 07 | 1263E Editor | 2100 | segtree, custom monoid | `struct Bal`, `SegTree<Pos, Bal>(Monoid(id:, op:))` |
| 08 | 1063B Labyrinth | 1800 | 0-1 BFS on product state `(R, C)` | `bfs01` over a generator fn, `.up/.down/.left/.right` |
| 09 | 1299C Water Balance | 2100 | floating output | `List` stack, `F64`, `print(v, precision: 9)` |
| 10 | 1245D Shichikuji | 1900 | MST | `kruskal` on `[E]` records, virtual vertex `V(0)` |

## Silent holes (guess, and whether it held)

"Held" was filled in after the tool runs. 22 of 23 held; S10 held for `bfs`, `kruskal`,
`prefix_function` and `Graph` but not for `bfs01` (F1).

| id | Problem | Guess | M1 section | Held |
|---|---|---|---|---|
| S1 | 01 | `Counter<Int>(0)` is the `Track` for `maximal`; `c.len` is the distinct count (spec writes `Counter<C>(0)`, std `new(init:)`) | std-hashmap / std-window | yes |
| S2 | 01,02,05,09,10 | `read<[n]T>()` gives an array keyed `0..n` by `Int`: `p[0]`, `p[1..]`, `pt[i].x` | R10.2, R04.2 | yes |
| S3 | 02 | `[0..=e, 0..2]?Int` is indexed `dp[u, q]` with plain `Int`s and `dp[e]` is a row view with `.present()` | R04.2 | yes |
| S4 | 02,05 | `x min= y` with `x: ?Int` and `y: Int` (rhs not optional) | R02.4 | yes |
| S5 | 03 | a nested fn may mutate a top-level `var` it captures (`used += 1`) | R06.2, R05.10 | yes |
| S6 | 03 | `print("?", xs)` prints a `List<Int>` space-joined after the string | R10.3 | yes |
| S7 | 03 | `read()` as the tail of a fn takes its type from the declared return type | R10.2, R02.10 | yes |
| S8 | 04 | `read<[m](u: V, v: V)>()` has a count domain good enough for `Graph.undirected`'s `E: Domain` | R10.2, std-graph | yes |
| S9 | 04,10 | printing `V` positions (and a list of them) prints their values, 1-based | R10.3 | yes |
| S10 | 04,06,08,10 | `bfs`, `bfs01`, `kruskal`, `prefix_function`, `Graph` need no `use` | R10.1 | no: `bfs01` needs `use std.graph.{bfs01}` (F1) |
| S11 | 06 | `w + "#" + res[a..]` builds a new `Str` (slice on the right); `res.push(c)` appends a `Char` | R02.3 | yes |
| S12 | 06 | `prefix_function(key)[key.len - 1]` indexes by `Int` | std-strings | yes |
| S13 | 07 | `SegTree<Pos, Bal>(Monoid(id: x, op: f))` with no `a`/`fill` holds the identity; `t[i] = v`; `t.all` | std-segtree | yes |
| S14 | 07 | inline `if a: x elif b: y else: z` is an expression | R01.7 | yes |
| S15 | 07 | `cur.prev ?? cur` and `cur.next!` on a domain position | R04.3 | yes |
| S16 | 08 | a `-> yields (to: (R, C), w: Int)` fn is a `WeightedNext` for `bfs01`; the vertex type comes from `from:` | std-graph, R06.3 | yes |
| S17 | 08 | `.up/.down/.left/.right` on an `(R, C)` tuple give `?(R, C)`; `lefts[(r, c)]` reads `Paths` | std-grid | yes |
| S18 | 08 | `let r0: R = read()` reads a position by its value | R10.2 | yes |
| S19 | 08 | `for (q, w) in [(p.up, 0), ...]` destructures a list of `(?(R, C), Int)` | R01.7 | yes |
| S20 | 09 | `F64(Int)` converts; `print(v, precision: 9)` prints 9 decimals; `repeat k:` loops | R10.3, R02.11 | yes |
| S21 | 09 | `st.pop()` returns `?T`; `st.last!.sum` | R02.8 | yes |
| S22 | 10 | `abs` is a free fn on `Int` (not found in root std by name) | R02.2 | yes |
| S23 | 10 | `[E] e => es[e.ord]` turns a `List` into `[E]` (copied from a conformance case under `spec/`); `mst.take[e]` | R02.8, std-graph | yes |

## Contradictions

| id | Where | What | M1 |
|---|---|---|---|
| C1 | `spec/lib/ops-ranges.md` §12 vs `spec/lib/graphs-trees.md` | §12 lists `dijkstra(step:)`; graphs-trees and `std/graph.cp` take `dijkstra(g, from:, until:)`, no `step:` | yes (std-graph) |
| C2 | `spec/core/12-powers.md` W9 vs `spec/lib/ops-ranges.md` §1 | W9: "`impl Add for T` is no longer written by hand (decision 133)", but the `fn (a) + (b)` spelling "does not parse yet"; ops-ranges says a record "implements `Add`" and `std/matrix.cp` still writes `impl Add for Mat<T>`. A cold reader cannot tell which spelling to write for a custom monoid, so 07 used `Monoid(id:, op:)` | no (12-powers has no M1 row) |
| C3 | `std/segtree.cp` `new` doc vs `spec/lib/ops-ranges.md` §2 | the doc says `op` may be "a `Monoid(id, op)`" (positional), §2 writes `Monoid(id:, op:)` (named) | yes (std-segtree) |

## Missing stdlib

| id | Problem | What | M1 |
|---|---|---|---|
| M1 | 03 | `Wire.ask` takes a `Str` only. A 100-number query needs a list-to-`Str` join I could not find for `List<Int>` (`join` is listed for `Str`), so 03 prints with `print("?", xs)` and reads by hand; the budget counter is user code with an `assert`. An `ask` taking `Show` parts like `print` would keep the one-call form | yes (std-io, R10.4) |

## Found-late

| id | Problem | Tool | What, with a minimal repro | Workaround | M1 | Known gap in SUBSET.md |
|---|---|---|---|---|---|---|
| F1 | 08 | `cpl check` | `bfs01` is not in the prelude while its siblings `bfs`, `dijkstra`, `kruskal` are (`std/prelude.cp` `use std.graph.{...}`). Repro: `domain V = 0..1` / `let p = bfs01((v: V) => [(to: v, w: 0)], from: [V(0)])` gives E0201 "no `bfs01` in scope". The diagnostic names the fix | added `use std.graph.{bfs01}` | yes (R11.3 prelude, std-graph) | no |
| E1 | all | `cpl explain` | phantom entries at `1:1`: `explicit copy -> std::vector<Char>, O(n), owned` and `reverse view -> cpl::ListView<Char>` on 8 of 12 files (all but 02, 02-rev, 03, 05). Repro: `var xs: List<Int> = []` / `xs.push(1)` / `print(xs)`. A reader hunting copies in loops cannot tell whether it is theirs | none needed (ignored) | no (no `explain` row in SUBSET.md) | no |
| E2 | 02 | `cpl explain` | a `let` whose type is inferred from a fold shows as `_  on the heap, as large as it grows`, and a table sized by it says "no bound on the count". Repro: `let a = read<[3]Int>()` / `let k = 1 - a.count(x => x > 0)` / `print(k)`: `k  _  on the heap` | none needed | no (no `explain` row) | no |

No other tool needed a workaround: every file checked, passed its samples under `cpl test`
(with the `verify/` checkers for 04, 09, 10 and `verify/inter03.py` as `--interactor` for 03; `verify/rt.py` is the random harness), built with
`cpl build` + `g++ -O2`, and passed the sanitizer build.

## Verification

| File | check | samples (`cpl test`, native, ASan) | random vs brute (interp, native, ASan; interp == native) | max-size native |
|---|---|---|---|---|
| 01-1198A | ok | 3/3 ×3 | 200/200 | n = 4e5: 0.04 s |
| 01-1198A-rev | ok | none (new requirement) | 200/200 | |
| 02-1286A | ok | 2/2 ×3 | 200/200 (permutation brute) | |
| 02-1286A-rev | ok | none (new requirement) | 200/200 (circular brute) | |
| 03-1207E | ok | 3/3 via `--interactor` | 200/200 via `cpl test --interactor` on the interpreter, `--fast` native and `--fast` ASan; the interactor rejects a 3rd query or a non-distinct list | |
| 04-1176E | ok | 1/1 ×3 (checker) | 200/200 (checker: dominating, at most n/2) | t = 1e5 tests: 0.04 s |
| 05-1114C | ok | 4/4 ×3 | 200/200 (exact factorial for n ≤ 60, Python big ints to 1e18) | 0.00 s |
| 06-1200E | ok | 2/2 ×3 | 200/200 | 1e5 words: 0.01 s |
| 07-1263E | ok | 2/2 ×3 | 200/200 (O(n²) simulation) | n = 1e6: 0.08 s |
| 08-1063B | ok after F1 | 2/2 ×3 | 200/200 (BFS over (cell, lefts, rights)) | 2000×2000: 0.44 s |
| 09-1299C | ok | 3/3 ×3 (1e-9 checker) | 200/200 (exact fractions) | n = 1e6: 0.35 s |
| 10-1245D | ok | 2/2 ×3 (checker) | 200/200 (Prim total + structural checker) | n = 2000 (2e6 edges): 0.57 s, 110 MB |

Sanitizers: `g++ -std=c++17 -O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG`, every
sample and every random input, no reports.

`cpl explain`, heap copies in loops: none. The loop copies it lists are plain values
(07 `t[cur]` a 24-byte `Bal`, 09 `st.pop()!` a 16-byte record); 02's `dp = next` is a move
(not listed); 06's `w + "#" + res[a..]` is listed as one `O(n + m)` concat per word, bounded
by twice the word length (the slice is a view). 01 and 08/10 copy a result once outside any
loop. E1/E2 are explain's own errors.

## Revision test

| File | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 01-1198A-rev | the file also stores a 32-bit header per distinct value: K values cost `n * ceil(log2 K) + 32 * K` bits | 2 code lines (+1 comment, +1 bound `4..=1e8` so K = 1 always fits) | 1 (the `ok` predicate) | small: whether `bit_width(k - 1)` is ceil(log2 k) at k = 1 (`bit_width(0) == 0` guessed) | none | none |
| 02-1286A-rev | the garland is a circle: last and first bulbs are neighbours | 13 of 20 code lines | 3 (table type ×2, transition, read-out) | yes: the 2-D row read-out `dp[e].present().min()!` has no 3-D analogue that also adds a per-cell cost, so it became a loop | yes: the table shape `[0..=free_even, 0..2, 0..2]?Int` is written twice (`dp` and `next`) | yes: the new `f` loop re-indents the 6-line transition body |

## Top five risks

1. **S3/S2 Int-keyed product tables.** The DP slot rests on `[0..=e, 0..2]?Int` taking plain
   `Int` indices and on `read<[n]T>()` keys; if either is a fresh domain, 02 needs
   conversions everywhere.
2. **S16 generator graphs for `bfs01`.** 08 builds its graph as a `yields` fn over a tuple
   state; if a generator fn is not a `WeightedNext`, the product-state slot needs a lambda.
3. **S5 capture mutation in 03.** The query budget lives in a captured `var`; if nested fns
   cannot write captures, the counter must move into a struct or a `mut` parameter.
4. **S13 + C2 custom monoid.** Two spellings for a user monoid in the spec; `Monoid(id:, op:)`
   on a struct must specialize in C++ to fit 1263E's 1e6 updates.
5. **Copies in loops.** 02 assigns `dp = next` per row and 06 slices `res[a..]` into a new
   `Str` per word; `cpl explain` must show a move and a bounded copy, not O(n) per step.
