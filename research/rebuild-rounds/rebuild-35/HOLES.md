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
| found-late | TBD | TBD |

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

"Held" is filled in after the tool runs.

| id | Problem | Guess | M1 section | Held |
|---|---|---|---|---|
| S1 | 01 | `Counter<Int>(0)` is the `Track` for `maximal`; `c.len` is the distinct count (spec writes `Counter<C>(0)`, std `new(init:)`) | std-hashmap / std-window | TBD |
| S2 | 01,02,05,09,10 | `read<[n]T>()` gives an array keyed `0..n` by `Int`: `p[0]`, `p[1..]`, `pt[i].x` | R10.2, R04.2 | TBD |
| S3 | 02 | `[0..=e, 0..2]?Int` is indexed `dp[u, q]` with plain `Int`s and `dp[e]` is a row view with `.present()` | R04.2 | TBD |
| S4 | 02,05 | `x min= y` with `x: ?Int` and `y: Int` (rhs not optional) | R02.4 | TBD |
| S5 | 03 | a nested fn may mutate a top-level `var` it captures (`used += 1`) | R06.2, R05.10 | TBD |
| S6 | 03 | `print("?", xs)` prints a `List<Int>` space-joined after the string | R10.3 | TBD |
| S7 | 03 | `read()` as the tail of a fn takes its type from the declared return type | R10.2, R02.10 | TBD |
| S8 | 04 | `read<[m](u: V, v: V)>()` has a count domain good enough for `Graph.undirected`'s `E: Domain` | R10.2, std-graph | TBD |
| S9 | 04,10 | printing `V` positions (and a list of them) prints their values, 1-based | R10.3 | TBD |
| S10 | 04,06,08,10 | `bfs`, `bfs01`, `kruskal`, `prefix_function`, `Graph` need no `use` | R10.1 | TBD |
| S11 | 06 | `w + "#" + res[a..]` builds a new `Str` (slice on the right); `res.push(c)` appends a `Char` | R02.3 | TBD |
| S12 | 06 | `prefix_function(key)[key.len - 1]` indexes by `Int` | std-strings | TBD |
| S13 | 07 | `SegTree<Pos, Bal>(Monoid(id: x, op: f))` with no `a`/`fill` holds the identity; `t[i] = v`; `t.all` | std-segtree | TBD |
| S14 | 07 | inline `if a: x elif b: y else: z` is an expression | R01.7 | TBD |
| S15 | 07 | `cur.prev ?? cur` and `cur.next!` on a domain position | R04.3 | TBD |
| S16 | 08 | a `-> yields (to: (R, C), w: Int)` fn is a `WeightedNext` for `bfs01`; the vertex type comes from `from:` | std-graph, R06.3 | TBD |
| S17 | 08 | `.up/.down/.left/.right` on an `(R, C)` tuple give `?(R, C)`; `lefts[(r, c)]` reads `Paths` | std-grid | TBD |
| S18 | 08 | `let r0: R = read()` reads a position by its value | R10.2 | TBD |
| S19 | 08 | `for (q, w) in [(p.up, 0), ...]` destructures a list of `(?(R, C), Int)` | R01.7 | TBD |
| S20 | 09 | `F64(Int)` converts; `print(v, precision: 9)` prints 9 decimals; `repeat k:` loops | R10.3, R02.11 | TBD |
| S21 | 09 | `st.pop()` returns `?T`; `st.last!.sum` | R02.8 | TBD |
| S22 | 10 | `abs` is a free fn on `Int` (not found in root std by name) | R02.2 | TBD |
| S23 | 10 | `[E] e => es[e.ord]` turns a `List` into `[E]` (copied from a conformance case under `spec/`); `mst.take[e]` | R02.8, std-graph | TBD |

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

Filled in after the cold commit.

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
