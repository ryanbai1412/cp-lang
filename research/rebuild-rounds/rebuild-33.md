# Rebuild round 33: holes

Cold reading: `spec/` and root `std/` only. Main at `d409eaa1a`. Problems were checked
against `plan/domains/spec/delta/` and `research/rebuild-rounds/` file names and
`git grep -l <cfid> -- research/rebuild-rounds plan/domains/spec/delta` (no hits).

## Problems

| # | CF | Rating | Slot | File |
|---|---|---|---|---|
| 01 | 1555E Boring Segments | 2100 | lazy segment tree (range add, range min) + two pointers | `01-1555E.cp` |
| 02 | 1523D Love-Hate | 2400 | bitmask DP over `Subset<B>` (superset sums) | `02-1523D.cp`, `02-1523D-rev.cp` |
| 03 | 1729G Cut Substrings | 2100 | Z-function string, multitest | `03-1729G.cp` |
| 04 | 1451D Circle Game | 1700 | float with `sqrt` | `04-1451D.cp` |
| 05 | 1486C2 Guessing the Greatest (hard) | 1900 | interactive, `@interactor` | `05-1486C2.cp` |
| 06 | 1613E Crazy Robot | 2000 | grid BFS over `[R, C]`, `g.rows` | `06-1613E.cp` |
| 07 | 1473E Minimum Path | 2400 | `dijkstra((v, k) => ...)` over `(V, K)` | `07-1473E.cp`, `07-1473E-rev.cp` |
| 08 | 1513D GCD and MST | 2000 | DSU (Kruskal-style) | `08-1513D.cp` |
| 09 | 1462E2 Close Tuples (hard) | 1700 | multitest, shared `Comb<Z>`, per-test arrays | `09-1462E2.cp` |
| 10 | 1548B Integers Have Friends | 1800 | two pointers, `maximal` over `Window(gcd)`, `max(e for ...)` | `10-1548B.cp` |

Slot note (unfilled parts): the grid BFS slot is only partly filled. 1613E uses `[R, C]`, `adj4` and
`g.rows`, but no column view or `max(e for ...)!` fold arises in it; the generator
fold appears in 10 (`max(r.len for r in runs)`, with `?? 0` instead of `!` because
`n = 1` gives no differences).

## Counts

| Class | Count | In M1 subset |
|---|---|---|
| silent | 14 | 12 |
| contradiction | 4 | 4 |
| missing-stdlib | 2 | 2 |
| found-late | 7 | 5 |
| total | 27 | 23 |

Silent guesses: 11 held, 2 partly (S2, S12), 1 untested (S3).

## Holes

M1 = the area is a supported row or contest staple in `spec/SUBSET.md`.
"Held?" was filled in after the tool runs; the cold version is the first commit of this round.

### Silent

| id | Problem | Hole | M1 | Guess | Held? |
|---|---|---|---|---|---|
| S1 | 01 | `SegTree` with `act:` and only `fill:` (no `a:`); the domain comes from type args `SegTree<Gap, Int>(min, fill: 0, act: add)`. Std shows `SegTree(+, a, act: assign)` and `fill:` separately, never together. | yes (segment trees) | accepted | held: interp and C++ both accept it |
| S2 | 01 | `minimal` with a `track` whose state is a lazy segment tree: the required monotonicity of `ok` for `minimal` is not stated (`maximal` says "monotone under shrinking"); `ok` reads `t.state.all`. | yes (sequences) | ok must be monotone under growing; works | held in the interpreter only; the C++ fails to compile (F6), so 01 dropped `minimal` |
| S3 | 01 | `Range<Int>` from `minimal` over a `List`: `p.first!`/`p.last!` give the end positions (spec shows `w.first!` for pieces, not for the `Range` result). | yes | `?Int` ends | untested: the code went with the 01 rewrite |
| S4 | 02 | `F(x)` for `domain F = 1..=n`: whether the argument is the value or the ordinal. | yes | value | held |
| S5 | 02 | A `domain B = 0..mine.len` declared inside a loop body, re-made each iteration, and `[B.subsets]Int` arrays over it. | yes (DP) | allowed | held |
| S6 | 02 | `b not in s` for `Subset`; spec documents `b in s` only. | no (std-subset documented) | `not in` parses | held |
| S7 | 02-rev | Tuple `<`/`>` on `(Int, Int)` (spec says structs get lexicographic `Ord`; tuples not said). | yes | lexicographic | held (02-rev 200/200) |
| S8 | 03 | `Str` keys are `Int` so `z_function(t + "#" + s)[l + 1 + i]` indexes by plain `Int`. | yes (strings) | Int keys | held |
| S9 | 04 | `sqrt(F64)` is used inside std (`isqrt`, `Vec.len`) but declared nowhere in `std/` or `spec/core`; `floor(x)` returning `Int` is in 02 but `sqrt` is not. | no (geometry documented) | builtin free fn | held |
| S10 | 05 | How many times `first_true`/`last_true` call a side-effecting predicate (query budget 20); cost is stated as a binary search but the call count is not. | yes (search) | ceil(log2) calls | held: n = 1e5 runs stay inside the interactor's 20-query assert |
| S11 | 05 | `read<[n]Int>()` inside an `@interactor` keys by `0..n`; `return` inside the interactor's loop ends the run. | yes (I/O) | 0-based, ok | held |
| S12 | 06 | `Cell.find(...)` on a product domain, `(r, c).adj4` on a tuple of domain values, and `for row in g.rows: print(row)` printing a `[C]Char` row view as one string. | yes (grids) | works | partly: `Cell.find`, `adj4` and `print(g[r])` held; `g.rows` failed (F3) |
| S13 | 07 | Storage for `dijkstra` over `(V, K)` tuples: "a domain gives `[V]` arrays", but an unnamed tuple of domains is not declared a domain. | yes (graphs) | arrays | held: the known gap did not bite here |
| S14 | 09 | `let comb = Comb<Z>(2e5)` shared above `@!multitest`: `choose(self)` grows its tables through a `let` binding; whether that is allowed and does not count as cross-test state. | yes (modular math) | allowed | held |

### Contradiction

| id | Problem | Hole | M1 |
|---|---|---|---|
| C1 | 03 | `std/README.md` index: `fn z_function<S: Indexed>(s: S) -> List<Int>`; `spec/core/04b-keys.md` and `std/strings.cp`: `-> [K]Int` keyed by `s.keys`. | yes |
| C2 | 01 | `std/README.md` lists `struct LazySegTree<D, T, F>(act: Action<T, F>)` with `fold(mut self)`; `std/` has no `LazySegTree`, only `SegTree<D, T, F = Int>` with `act:`. | yes |
| C3 | 10 | `Window` has `pop(mut self)` and also implements `Track.pop(mut self, x)`: one name, two arities on one type. | yes |
| C4 | 05 | `sorted(by:, desc: true)` is in `std/seq.cp` and the README, but `spec/SUBSET.md` lists `core-07/sorted-has-no-desc-flag.cp` as a known gap. It ran fine in 05's interactor. | yes |

### Missing stdlib

| id | Problem | Hole | M1 |
|---|---|---|---|
| M1 | 02 | `a.fold_supersets(op)` / `fold_subsets` are in `spec/lib/search-dp.md` but nowhere in `std/`; 02 hand-rolls the superset-sum loop. | yes (DP) |
| M2 | 06 | No BFS that lets the visit rule depend on per-vertex counters (`bfs(..., enter:)` exists in `std/graph.cp` but is not in the spec tables); 06 hand-rolls the queue. | yes (graphs) |

### Found-late

Each needed a workaround (F7 only misleads). Repros run with `cpl run` / `cpl build` + `g++`.

| id | Problem | Repro (one line) | M1 | Known gap | Workaround |
|---|---|---|---|---|---|
| F1 | 01 | `[(w: 1)].minimal(state: track(0, (mut s, x) => s += x.w, (mut s, x) => s -= x.w), ok: t => t.state >= 1)` gives E0203 "an `Int` has no member `w`": `track` infers the element type from the state | yes (std-seq) | no | annotate both lambdas' parameters |
| F2 | 02 | `var rng = Rng(1)` gives E0201 "no `Rng` in scope"; spec/lib/random-stress.md uses `Rng` without a `use` | no (std-rand documented) | no | `use std.rand.{Rng}` |
| F3 | 06 | `for row in g.rows: print(row)` on `[R, C]Char` gives E0203 "has no method `rows`" | yes (std-grid) | yes: `core-04/grid-rows-is-a-getter`, `core-04/grid-rows-cols-views` | `for r in R: print(g[r])` |
| F4 | 10 | `[1, 2].maximal(Window(+), ok: w => (w.all ?? 0) <= 9)` (spec/lib/sequence-patterns.md:139) gives E0303 "`pop` takes 0 arguments, but 1 were given" (C3 bites); and `var w = Window(gcd); w.push(4)` gives "`cpl run` doesn't know a fn named `gcd`" | yes (std-window, std-seq) | no | `fn g(x: Int, y: Int) -> Int = gcd(x, y)` and `track(Window(g), push, pop)` with typed lambdas |
| F5 | 02 | `for b in s.items: ...` on `Subset<B>` gives E0302 "a `for` can't walk over a `Items<Subset<B>>`" though `items` is `-> yields B` | no (std-subset documented) | no | `for b in B if b in s` |
| F6 | 01 | `let f = (t: SegTree<G, Int>) => t.all` runs in the interpreter; its C++ fails: "binding reference of type `SegTree...&` to `const SegTree...` discards qualifiers" (the `all` getter takes a non-const ref; lambdas and the `minimal` `ok:` get const) | yes (std-segtree) | no (the known gap is `closure-op-picks-index`) | hand-written two-pointer loop instead of `minimal` over `track` |
| F7 | 01, 09 | `var t = SegTree<G, Int>(min, fill: 0, act: add)` then `for i in 0..3: t.apply(G(1)..=G(2), 1)`: `cpl explain` lists `[copy] 3:16 ... what it reads may change`, but the C++ passes `t` by reference; same for `comb.choose(...)` in 09's loop | yes (Rcli) | no | none (verified the C++ by hand) |

## Verification

Release `cpl` built from `d409eaa1a`. Every file after the found-late workarounds:

| Step | Result |
|---|---|
| `cpl check` | 12/12 clean |
| samples (`cpl run`, statement samples) | all pass; 05 has no sample, its 3 hidden tests go through the `@interactor` |
| `cpl test` | 12/12 files pass, 23 cases (each `NN.tests/`) |
| `cpl build` + `g++ -std=c++17 -O2` | 12/12 build |
| 200 random tests vs Python brute, interpreter and native agree | 12 x 200/200 (02 and 02-rev with a checker: any largest set; 05 via `cpl run` and `cpl run --release` against the interactor, n up to 1e5) |
| `-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG` | clean on all 20 sample cases, 11 x 200 random inputs and 05's 203 interactive runs (piped to `cpl judge`) |
| `cpl explain` heap copies in loops | none real: the two in-loop `[copy]` lines (01, 09) are F7; the rest are outside loops |
| max-size native timing | 01 0.24 s, 02 0.47 s, 07 0.13 s, others under 0.05 s |

The 02 brute is a checker (any largest set is accepted), not a token compare.

## Top five risks

Cold list (the guess before tools), then what happened:

1. S13 / known gap `dijkstra-from-unpacks-tuple-vertex`: 07 depends on tuple-vertex Dijkstra. Held.
2. S12 / known gaps `grid-rows-is-a-getter`, `grid-rows-cols-views`: 06 prints via `g.rows`. Hit (F3).
3. S1 + C2: lazy `SegTree` construction by type args plus `fill:` and `act:` together. Held.
4. S2: `minimal` over a `track` of a heap structure. Hit (F1, F6).
5. S5: a domain declared inside a loop, with `[B.subsets]` arrays over it. Held.

After tools, the top risk is F4: the spec's own `maximal(Window(+), ...)` example does not run.

## Per-problem mapping

| Problem | Holes |
|---|---|
| 01 | S1, S2, S3, C2, F1, F6, F7 |
| 02 | S4, S5, S6, M1, F2, F5 |
| 02-rev | S7 |
| 03 | S8, C1 |
| 04 | S9 |
| 05 | S10, S11, C4 |
| 06 | S12, M2, F3 |
| 07 | S13 |
| 08 | none |
| 09 | S14, F7 |
| 10 | C3, F4 |

## Revision test

| Problem | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 02 1523D (DP) | among largest sets prefer most friends; print that count | 4 (key var, compare, assign, extra print) + header comment | 3 | whether tuples compare (S7) | none | none |
| 07 1473E | graph may be disconnected; print -1 when unreachable | 1 (`!` to `?? -1`) + header comment | 1 | none | none | none |
