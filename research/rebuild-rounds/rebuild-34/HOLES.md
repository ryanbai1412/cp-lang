# Rebuild round 34: holes

Cold reader: wrote these ten solutions from `spec/` and root `std/` only, with
`cpl` rebuilt from `main` and no tool run on the solutions before this commit.
Statements came from the Luogu mirror (1358D through a reader proxy, since
Luogu refused it). Skipped: 1326D2 (named in `research/rebuild-rounds/rebuild-32.md`).

| # | Problem | Rating | Slot |
|---|---|---|---|
| 01 | 1358D The Best Vacation | 1900 | sliding window: `ring.minimal(Window(+))` |
| 02 | 1324F Maximum White Subtree | 1800 | tree DP / rerooting: `Rooted.reroot` |
| 03 | 1385E Directing Edges | 2000 | output over 64 KiB (up to 2e5 lines), `print(yes(..))` + `print(u, v)` |
| 04 | 1343D Constant Palindrome Sum | 1700 | multitest with per-test domain `X = 2..=2k` |
| 05 | 1370D Odd-Even Subsequence | 2000 | binary search on the answer: `first_true` |
| 06 | 1430D String Deletion | 1700 | string: `runs()` |
| 07 | 1404C Fixed Point Removal | 2300 | Fenwick with offline queries |
| 08 | 1359E Modular Stability | 2000 | combinatorics: `Mod` + `Comb` |
| 09 | 1365D Solve The Maze | 1700 | grid graph: `[R, C]Char`, `adj4`, `bfs` |
| 10 | 1526C2 Potions (Hard) | 1600 | greedy with a `Heap` |

## Counts (cold)

| Class | Count | Inside M1 |
|---|---|---|
| silent | 16 | 16 |
| contradiction | 1 | 1 |
| missing-stdlib | 3 | 3 |
| found-late | (filled after the tool runs) | |

M1 = the feature's family is `supported` in `spec/SUBSET.md` (every family used
here is: seq, window, tree, graph, grid, fenwick, mod, heap, io, search).

## Silent holes (spec/std did not say; I guessed)

"Held?" is filled in after the tool runs.

| id | Where | Guess | M1 | Held? |
|---|---|---|---|---|
| S1 | 01, 05 | a top-level `fn` captures top-level `let`s (`days`, `a`, `k`) like a nested function | yes | |
| S2 | 01 | an unannotated `fn` parameter (`pre`) is generic over the prefix type returned by `scan(+)` | yes | |
| S3 | 01 | `RingSlice` has `start`/`end` positions with `.l`/`.r` cuts, and positions compare with `<=` | yes | |
| S4 | 01 | a `scan(+)` prefix is indexed by cuts, total at `M.last!.r` | yes | |
| S5 | 03 | `return` in the body below `@!multitest` ends the current test | yes | |
| S6 | 03 | `order.enumerate()` exists and yields `(i, v)` | yes | |
| S7 | 03 | a list comprehension takes a trailing `if` filter | yes | |
| S8 | 04 | `a.zip(a.rev).take(n / 2)` pairs `a[i]` with `a[n + 1 - i]` (no direct index on `.rev`) | yes | |
| S9 | 04, 07 | `X(int)` / `I(int)` converts a value to a position of an explicit-ends domain, checked | yes | |
| S10 | 05 | a range bound may name an earlier variable: `k: 2..=n` | yes | |
| S11 | 07 | `Fenwick<I, Int>()` builds an empty tree (std shows `new(a: ?[D]T = None)`) | yes | |
| S12 | 07 | `(I.first!..=i).last_true(p)` searches a range of domain positions | yes | |
| S13 | 08 | `Comb<Z>(n)` sizes the tables; `choose(n, k)` is 0 when `k > n` or `n < 0` | yes | |
| S14 | 09 | `grid.keys` of a `[R, C]Char` yields `(R, C)` cells that have `.adj4` and index the grid | yes | |
| S15 | 09 | `bfs` result compares with `None` (`reach[p] != None`) | yes | |
| S16 | 10, 02 | `Heap.len` exists; free `max(x, 0)` exists | yes | |

## Contradiction

| id | Where | What | M1 |
|---|---|---|---|
| C1 | 01 | `spec/lib/sequence-patterns.md` passes the state positionally (`a.maximal(Window(+), ok: ..)`); every `std/seq.cp` example names it (`maximal(state: Counter..)`). I used the positional form. | yes |

## Missing stdlib

| id | Where | What | M1 |
|---|---|---|---|
| M1 | 01 | no fold of a prefix over a wrapping `RingSlice`/`RingRange`: I wrote the wrap formula by hand (`around`). | yes |
| M2 | 07 | Fenwick has `first_ge`/`first_gt` on prefixes but no "last position whose suffix is still >= b"; I used `last_true` over positions, \(O(\log^2 n)\). | yes |
| M3 | 07 | no documented step from a cut to the position before it (`first_gt` returns a cut, the update needs a position). | yes |

## Top five risks (cold)

1. S14 grid keys as cells: `spec/SUBSET.md` lists `core-04/grid-keys-are-cells.cp` as a known compiler gap.
2. S8 `zip` with `.rev`: known gaps `core-04/zip-keyed-by-first.cp` and `core-07/rev-does-not-satisfy-indexed.cp`.
3. 01 `ring.minimal(Window(+))`: known gap `std-seq/maximal-window-sum.cp` and the `std-window/window-pop-*` gaps.
4. S2 generic unannotated `fn` parameter in 01.
5. S5 `return` from a multitest body in 03.

## Per-problem mapping

| # | Holes |
|---|---|
| 01 | S1 S2 S3 S4 C1 M1 |
| 02 | S16 |
| 03 | S5 S6 S7 |
| 04 | S8 S9 |
| 05 | S1 S10 |
| 06 | S16 (`max`) |
| 07 | S9 S11 S12 M2 M3 |
| 08 | S13 |
| 09 | S14 S15 |
| 10 | S16 |

## Revision test

| File | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 02-1324F-rev (DP) | weights in -1e9..=1e9 replace colours | 2 code (1 deleted, 1 replaced) | 1 | none | none | none |
| 10-1526C2-rev (greedy) | start with health h; also print the final health | 3 code (read, init, print) | 3 | whether the same greedy maximises the final health among max-count choices (kept it) | none | none |

## Found-late

(Filled after the tool runs.)
