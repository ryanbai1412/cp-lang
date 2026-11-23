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
| found-late | 9 | 9 (3 already listed as known gaps in `spec/SUBSET.md`) |

M1 = the feature's family is `supported` in `spec/SUBSET.md` (every family used
here is: seq, window, tree, graph, grid, fenwick, mod, heap, io, search).

## Silent holes (spec/std did not say; I guessed)

"Held?" from the tool runs: 12 of 16 held; S2, S6, S8 and S14 failed (each is a found-late item below).

| id | Where | Guess | M1 | Held? |
|---|---|---|---|---|
| S1 | 01, 05 | a top-level `fn` captures top-level `let`s (`days`, `a`, `k`) like a nested function | yes | held |
| S2 | 01 | an unannotated `fn` parameter (`pre`) is generic over the prefix type returned by `scan(+)` | yes | no: E0204 "parameter pre needs a type" (F2) |
| S3 | 01 | `RingSlice` has `start`/`end` positions with `.l`/`.r` cuts, and positions compare with `<=` | yes | held |
| S4 | 01 | a `scan(+)` prefix is indexed by cuts, total at `M.last!.r` | yes | held |
| S5 | 03 | `return` in the body below `@!multitest` ends the current test | yes | held (random NO cases pass) |
| S6 | 03 | `order.enumerate()` exists and yields `(i, v)` | yes | no: spelled `.items` (F6) |
| S7 | 03 | a list comprehension takes a trailing `if` filter | yes | held |
| S8 | 04 | `a.zip(a.rev).take(n / 2)` pairs `a[i]` with `a[n + 1 - i]` (no direct index on `.rev`) | yes | no: `zip` unknown to `cpl run` (F7) |
| S9 | 04, 07 | `X(int)` / `I(int)` converts a value to a position of an explicit-ends domain, checked | yes | held |
| S10 | 05 | a range bound may name an earlier variable: `k: 2..=n` | yes | held |
| S11 | 07 | `Fenwick<I, Int>()` builds an empty tree (std shows `new(a: ?[D]T = None)`) | yes | held |
| S12 | 07 | `(I.first!..=i).last_true(p)` searches a range of domain positions | yes | held |
| S13 | 08 | `Comb<Z>(n)` sizes the tables; `choose(n, k)` is 0 when `k > n` or `n < 0` | yes | held (brute agrees on `k > n / a1` cases) |
| S14 | 09 | `grid.keys` of a `[R, C]Char` yields `(R, C)` cells that have `.adj4` and index the grid | yes | no: keys are rows (F8) |
| S15 | 09 | `bfs` result compares with `None` (`reach[p] != None`) | yes | held |
| S16 | 10, 02 | `Heap.len` exists; free `max(x, 0)` exists | yes | held |

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
| 01 | S1 S2 S3 S4 C1 M1 F1 F2 F3 F4 |
| 02 | S16 |
| 03 | S5 S6 S7 F5 F6 F9 |
| 04 | S8 S9 F7 |
| 05 | S1 S10 |
| 06 | S16 (`max`) |
| 07 | S9 S11 S12 M2 M3 |
| 08 | S13 |
| 09 | S14 S15 F8 |
| 10 | S16 |

## Revision test

| File | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 02-1324F-rev (DP) | weights in -1e9..=1e9 replace colours | 2 code (1 deleted, 1 replaced); after `cpl test`, the read moved up 2 lines | 1 cold, 2 final | none while writing, but the cold edit replaced the derived `own` line in place, after the edge read, so input order was wrong (R0102 "-4 is not in V" on the first test). Input order is positional, so a read cannot stand where its derived value stood. | none | none |
| 10-1526C2-rev (greedy) | start with health h; also print the final health | 3 code (read, init, print) | 3 | whether the same greedy maximises the final health among max-count choices (kept it; 200 random tests against a subset brute agree) | none | none |

## Found-late

Every tool failure that needed a workaround. All are inside M1 (their families
are `supported` in `spec/SUBSET.md`).

| id | Where | One-line repro | Workaround | Known gap in `spec/SUBSET.md` |
|---|---|---|---|---|
| F1 | 01 | `let f = (w: RingSlice<M, Int>) => 0` without an import: E0204 "no type or domain named RingSlice", though `a.ring.minimal` returns it | `use std.seq.{RingSlice}` | no |
| F2 | 01 | `fn around(pre, w: Int) -> Int = w`: E0204 "parameter pre needs a type" (S2) | annotate `pre: Prefix<M, Int>` | no |
| F3 | 01 | `a.ring.minimal(Window(+), ok: t => (t.all ?? 0) >= x)`: E0303 inside `std/seq.cp` "pop takes 0 arguments, but 1 were given" (`Window.pop()` shadows `Track.pop(x)`) | `track(0, (mut s, v) => s += v, (mut s, v) => s -= v)` | yes: `std-seq/maximal-window-sum.cp` (same `Window` as a `maximal`/`minimal` state; also `std-window/window-pop-*`) |
| F4 | 01 | `let a: [4]Int = [1, 2, 3, 4]; print(a.minimal(Window(+), ok: w => (w.all ?? 0) >= 5).present().map(r => r.len).collect())`: E0203, `.present()` leaves `?Range<Int>` (it works on `[3]?Int`) | `for piece in shortest: let w = piece else: continue` | no |
| F5 | 03 | `let g = directed_of_pairs([(1, 2)])`: "no directed_of_pairs in scope", though `spec/lib` lists it with the graph builders | `use std.graph.{directed_of_pairs}` | no |
| F6 | 03 | `for (i, v) in [5, 6].enumerate(): print(i)`: E0203 "CP Lang spells index-element pairs .items" (S6; good diagnostic) | `.items` | no |
| F7 | 04 | `let a = [1, 2, 3]; print(List(a.zip(a.rev)))`: E0203 "`[Int]Int` has no method zip that cpl run knows" (S8) | index `a[I(n + 1 - Int(i))]` by hand | yes: `core-04/zip-keyed-by-first.cp` (and `core-07/rev-does-not-satisfy-indexed.cp` for the `.rev` side) |
| F8 | 09 | `domain R = 0..2; domain C = 0..3; let g = read<[R, C]Char>(); for p in g.keys: print(g[p] == '.')`: `g[p]` is a `List<Char>` row (S14) | `let cells = [(r, c) for r in R for c in C]` | yes: `core-04/grid-keys-are-cells.cp` |
| F9 | 03 | `cpl test` on a problem with many valid answers compares tokens exactly ("first difference at line 2, token 1: expected 3, got 1"); its hint is right | a `@check fn ok() -> Bool` that re-reads the per-test input and validates with `read_output()` and `topo` | no (documented, rule 06.4.3) |

## Explain findings (no workaround needed, not counted)

- Every file lists `1:1 explicit copy -> std::vector<Char>, O(n), owned` with no source at 1:1.
- 08: `[copy] c.choose(..)` "owns its value", but the C++ passes `Comb&` (no copy). Meanwhile `.map(..).sum()` fills a `std::vector` first (one O(n) allocation explain does not list).
- 01: `let w = piece else: continue` is a `[copy]` in the loop; it copies an `optional<RingSlice>` that holds no heap data, so no heap copy.
- 09: the `bfs` result copy is once per test, not in a loop.
- No heap copy inside a loop in any file.

## Verification

All 12 files (10 + 2 rev):

| Check | Result |
|---|---|
| `cpl check` | 12/12 no errors (after F1 to F8) |
| samples via `cpl test` | 12/12 files pass (03 through its `@check`; rev files use 2 brute-made cases each) |
| `cpl build` + `g++ -std=c++17 -O2` | 12/12 build |
| 200 random tests vs independent Python brute (03: Python validity checker) | 12 x 200/200; interpreter, `-O2` native and sanitizer build agree on every input |
| `g++ -std=c++17 -O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG` on samples and all random inputs | clean, exit 0, no diagnostics |
| `cpl explain` | no heap copies in loops (findings above) |

The generators, brutes and the runner are in `stress/` (`python3 stress/run.py [name ..]`, `N` sets the count).
