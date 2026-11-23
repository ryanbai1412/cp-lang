# Rebuild round 29: holes

Cold read of `spec/` and root `std/` only (plus `ls` of `plan/domains/spec/delta/` names).
Ten Codeforces problems from contests 1950-1999, rated 1600-2200, none used in an earlier
round. Statements and samples from the Luogu mirror (Codeforces served a challenge page).

| # | Problem | Rating | Category |
|---|---|---|---|
| 01 | 1985H1 Maximize the Largest Component (Easy) | 1700 | DSU (on a grid) |
| 02 | 1998D Determine Winning Islands in Race | 2100 | topological order / DAG DP |
| 03 | 1994C Hungry Games | 1600 | sliding window (two pointers, `maximal(Window(+))`) |
| 04 | 1998E1 Eliminating Balls With Merging (Easy) | 2200 | sparse table (argmax) + divide and conquer |
| 05 | 1990D Grid Puzzle | 1800 | grid DP over rows x column-pair states |
| 06 | 1970C2 Game on Tree (Medium) | 1700 | tree DP (`Rooted.up`, subtree fold) |
| 07 | 1980F1 Field Division (easy) | 1900 | sort by custom tuple key `(c, desc(r))` |
| 08 | 1968F Equal XOR Segments | 1800 | xor prefix trick |
| 09 | 1954D Colored Balls | 1800 | counting mod 998244353 (knapsack) |
| 10 | 1979D Fixing a Binary String | 1800 | slices and reversed views (`s[p..]`, `s[..p].rev`, `t.rev`) |

Coverage note: no 1600-2200 problem in contests 1950-1999 needs a *monotone* deque. 03 uses
the std sliding window (`maximal` over `Window(+)`), whose strategy for `+` is a running
total, not a deque. The deque strategy (`Window(min)`) is not exercised this round.

## Summary (before running)

| Class | Count |
|---|---|
| silent | 15 |
| contradiction | 2 |
| missing-stdlib | 1 |
| found-late | 0 |
| total | 18 |

Inside the Milestone 1 subset (`spec/SUBSET.md`): 16 of 18. H13 (sparse table) is in
`std-sparse`, not among the listed contest staples (documented, not promised). H18 is a
user-library gap (missing-stdlib), outside by definition.

## Top five risks

1. **H01 `pre[k]` on a counted array.** `scan` results are indexed by cuts (`pre[c]`, left of
   `c`) and by positions (`pre[v]`, through `v`); on `[n]Int` both are `Int`. 04 and 08 rely
   on `pre[k]` = fold of the first `k`. If it is "through k", both are off by one.
2. **H02 `common_prefix_len(t.rev)`.** Spec 07 says reversed views are not directly indexable,
   std's `common_prefix_len` asks for `Indexed`. 10 needs it.
3. **H03 `@check` under `@!multitest`.** Nothing says whether a `@check` fn runs per test case
   (reading one case) or once over the whole input. Guessed per case.
4. **H04 `Window(+)` result type.** `Window<T, M>.all` is `?T`; with `T` a refined range
   (`1..=1e9`), sums overflow the element type. Guessed: read the array as `Int`.
5. **H05 binding a `List` out of a `Map` index** (`let xs = at[key]`): move, lend or copy?
   Spec 05 only says element reads copy. Guessed lend (no copy, map left intact).

## Holes

| ID | Class | Where | What was unclear | Guess | M1 |
|---|---|---|---|---|---|
| H01 | contradiction | `spec/core/07-sequences.md` scan / Prefix; `std/prefix.cp` | `pre[x]` with `Int` keys: cut rule ("left of c") vs position rule ("through v") | `pre[k]` = fold of first `k`, `k in 0..=n` | yes (R07.9, std-prefix) |
| H02 | contradiction | `spec/core/07-sequences.md` (`.rev` not indexable) vs `std/strings.cp` `common_prefix_len<A: Indexed, B: Indexed>` | can a reversed view be passed where `Indexed` is required | yes, walked by `.nth` | yes (R07.4, std-strings) |
| H03 | silent | `spec/core/06-control-functions.md` 6.4, `spec/tools/stress.md` | `@check` with `@!multitest`: per case or whole input | per case, reading one case | yes (R06.4) |
| H04 | silent | `std/window.cp` `Window<T, M>`, `spec/lib/sequence-patterns.md` | element type of the fold when `T` is a refined range | the fold stays `T`; use `Int` elements | yes (std-window) |
| H05 | silent | `spec/core/05-ownership.md` R05.3.5 | `let xs = m[k]` for a `List` value in a `Map` | a lend, no copy | yes (R05.3) |
| H06 | silent | `spec/core/08-traits-generics.md` | generic fn over two domains: is the bound spelled `A: Domain`, and needed | `fn f<A: Domain, B: Domain>(g: [A, B]Char)` | yes (R08.2) |
| H07 | silent | `spec/core/04-indices.md` (decision 159), `std/dsu.cp` | `Dsu<(A, B)>` and `for p in (A, B)` with an inline product of generic domains | works as a named product domain | yes (R04.2, std-dsu) |
| H08 | silent | `spec/core/01-lexical-grammar.md` 1.6, `spec/core/02-values-types.md` 2.4 | `let Some(x) = e else: continue` (pattern let-else; spec shows `let x = e else: return`) | pattern form allowed, `continue` allowed as the diverging arm | yes (R01.6, R02.4) |
| H09 | silent | `spec/core/04-indices.md` `V.at(x)` | does `A.at` take the value or the ordinal; `Int(r)` for `0..n` | value; same for `0..n` | yes (R04.2) |
| H10 | silent | `spec/lib/sequence-patterns.md` `maximal` | end of a returned piece: `Int(piece[l].r)` as the first left-out index on a counted array | `.r` is the end cut, `Int(cut)` its index | yes (R04.3, std-seq) |
| H11 | silent | `spec/core/07-sequences.md` scan; ops | is `^` an op value `scan` accepts (xor group) | yes | yes (R07.9) |
| H12 | silent | `std/tree.cp` `Rooted.up`; `spec/lib/graphs-trees.md` | constructor spelling `Rooted(g, root:)` (std says "compiler built-in"), and no Bool `or` op value for `up` | `Rooted(g, root: v)`; count losing children with `+` over `Int` | yes (std-graph/tree) |
| H13 | silent | `std/sparse.cp` `SparseTable.new(op, a)` | `op` over indices capturing `a` (`(i, j) => if a[j] > a[i]: j else: i`) with `[n] i => i` table | allowed; idempotent holds | no (std-sparse documented) |
| H14 | silent | `spec/core/09-contexts-checks.md` 9.6, 6.2 | nested recursive fn in the per-test section capturing per-test lets, depth 2e5 | allowed; 1 GB judge stack | yes (R06.2, R09.6) |
| H15 | silent | `spec/core/02-values-types.md` 2.8, `spec/core/05-ownership.md` | `dp = nx` with a fresh `[4]?Int` per row: move or copy | move, but one allocation per row | yes (R05.3) |
| H16 | silent | `spec/core/03-numbers-layout.md` 5 | `Z * Int` and `max(x: 1..=5000, Int)` mixing | Int converts to `Mod` implicitly; `max` widens to `Int` | yes (R03.5) |
| H17 | silent | `spec/core/02-values-types.md` 2.3, `std/strings.cp` | `Str([chars])` from a list comprehension, `t + s` concat, `z_function` of a `Str` keyed `Int` | all as written | yes (R02.3, std-strings) |
| H18 | missing-stdlib | `std/` | no Bool monoid op value (`or`/`any`) for tree folds; `+` over `Int` used instead | - | no (user-library gap) |

## Per-problem mapping

| File | Holes leaned on |
|---|---|
| 01-1985H1 | H06, H07, H08, H09 |
| 02-1998D | (none beyond core; uses `Graph.directed`, `u.next`, `min=`) |
| 03-1994C | H04, H10 |
| 04-1998E1 | H01, H13, H14 |
| 05-1990D | H08, H15 |
| 06-1970C2 | H12, H18 |
| 07-1980F1 | (none; `sorted(by: (c, desc(r)))`) |
| 08-1968F | H01, H05, H11 |
| 09-1954D | H16 |
| 10-1979D | H02, H03, H17 |
| 08-1968F-rev | H01, H05, H11 |
| 09-1954D-rev | H16 (and a two-clause `for` with in-place updates) |

## Revision test

| File | Change | Lines / chunks changed | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|
| 09-1954D-rev (DP) | count only colour sets with an even number of colours: the knapsack gains a parity dimension | 4 lines in 1 chunk: the table type `[5001]` to `[2, 5001]`, its seed, the read `ways[s]` to `ways[1, s]`, the update loop gains `, q in 0..2` | whether the in-place update with two clauses is safe: writes go to `s + x > s`, reads at `s`, descending `s`, so yes | none | none |
| 08-1968F-rev (other) | print the cut points of a valid split instead of YES/NO | 2 lines in 2 chunks: the early 2-part `print`, the final `print` | none | the output format sits at two sites because the 2-part case `continue`s early | none |

## After running

`cargo build -p cpl-cli` at `ed42acb51` (descends from `77e0021b5`); g++ 11.4, `-std=c++17 -O2`.
Fuzz: 200 random small inputs per file through `cpl run`, the native binary and an independent
Python brute force (exhaustive game search for 02/06, merge search for 04, BFS over grid
masks for 05, staircase enumeration for 07, all splits for 08, all subsets for 09); 08-rev and
10 are any-answer and are validated by checking the printed cuts / position.
10 has a `@check` (`cpl judge` accepts the native output, rejects a wrong `p` with exit 3).

| File | check | samples (interp) | `cpl test` | native samples | fuzz 200 | max input (native) | copies in loops |
|---|---|---|---|---|---|---|---|
| 01-1985H1 | ok | 1/1 | 1/1 | 1/1 | 200/200 | 0.16 s (1000x1000) | `Dir4.all()` builds a 4-element vector per `.adj4` call (allocation, not a copy) |
| 02-1998D | ok | 1/1 | 1/1 | 1/1 | 200/200 | 0.03 s (n = m = 2e5) | none |
| 03-1994C | failed twice (L1, L4), rewritten | 1/1 | 1/1 | 1/1 | 200/200 | 0.01 s (n = 2e5) | none |
| 04-1998E1 | ok | 1/1 | 1/1 | 1/1 | 200/200 | 0.04 s (n = 2e5 increasing, depth 2e5, default stack) | none |
| 05-1990D | ok | 1/1 | 1/1 | 1/1 | 200/200 | 0.01 s (n = 2e5) | none (`nx` is a `std::array`) |
| 06-1970C2 | failed (L2), fixed | 1/1 | 1/1 | 1/1 | 200/200 | 0.05 s (path, n = 2e5) | none |
| 07-1980F1 | ok | 1/1 | 1/1 | 1/1 | 200/200 | 0.04 s (k = 2e5) | none (one sorted key list per test) |
| 08-1968F | ok | 1/1 | 1/1 | 1/1 | 200/200 | **33.3 s, TLE** (n = q = 2e5, all ones) | **`std::vector<ll> xs = at.at(...)`, `ys` likewise: two list copies per query (L5)** |
| 08-1968F-rev | ok | 1/1 (valid cuts) | - | 1/1 | 200/200 | **38.1 s, TLE** | **same as 08 (L5)** |
| 09-1954D | ok | 3/3 | 3/3 | 3/3 | 200/200 | 0.04 s (n = 5000) | none |
| 09-1954D-rev | ok | 3/3 (brute) | - | 3/3 | 200/200 | 0.05 s | none |
| 10-1979D | failed (L3), fixed | 1/1 (`@check`) | 1/1 | 1/1 | 200/200 | 0.00 s (n = 1e5) | none (the target string is built once per `c`) |

No interpreter/native mismatch in any sample, fuzz or max run, so no sanitizer rerun was needed.
The rev files have no `.tests/` of their own; their samples are the base problem's inputs.

### Failures and smallest reproductions

- **L1 (03): `w.all <= x` rejected.** `a.maximal(Window(+), ok: w => w.all <= x)` gives E0308
  "an optional has no order". `spec/lib/sequence-patterns.md` writes exactly
  `ok: w => w.all <= K`. Fix in the solution: `(w.all ?? 0) <= x`.
- **L4 (03): `maximal` over `Window(+)` does not instantiate.** After the L1 fix:
  E0303 "`maximal` (`std.seq`) does not work on the types it was given here: `pop` takes 0
  arguments, but 1 were given", at `std/seq.cp:3175`. `Window.pop()` takes no argument,
  `maximal` calls `pop(x)`. The documented `a.maximal(Window(+), ok: ...)` cannot compile.
  Smallest: `use std.seq.{maximal}` / `let a = [1, 2]` / `print(a.maximal(Window(+), ok: w => (w.all ?? 0) <= 2))`.
  Fix in the solution: explicit two pointers with `Window` `push`/`pop`.
- **L2 (06): `Rooted.up` with `U != T`.** `tree.up(+, lift: (w, c, p) => if w: 0 else: 1,
  finish: (losing, v) => losing > 0)` (T = Bool, U = Int) gives E0302 "expected `Bool`, found
  `Int`" at the lift. The std signature has separate `T` and `U`. Fix: carry wins as `Int`.
- **L3 (10): `Str` slice has no `.chain`.** `s[p..].chain(s[..p].rev)` gives E0203 "a `Str`
  has no method `chain` that `cpl run` knows". Fix: `List(s[p..]) + List(s[..p].rev)`.
- **L5 (08, 08-rev): binding a map entry copies the list when the key is an index expression.**
  `let xs = at[p[r]]` generates `std::vector<ll> xs = at.at(Prefix_..._at(p, ...))`, a copy per
  query: 33 s on the max input. With the key in a local first (`let pr = p[r]` /
  `let xs = at[pr]`) it generates `const std::vector<ll>& xs = at.at(pr)` and runs in 0.07 s,
  same output. Writing the lookup inline (`at[p[r]][at[p[r]].partition_point(...)]`) is worse:
  302 s. Smallest:
  ```
  let n: 1..=10 = read()
  let a = read<[n]Int>()
  let p = a.scan(^)
  let at = (0..=n).group(by: k => p[k])
  var s = 0
  for r in 0..=n:
      let xs = at[p[r]]            // std::vector<ll> xs = ... (copy); `at[r % 2]` gives const&
      s += xs[xs.partition_point(k => k > 0)]
  print(s)
  ```
  The committed 08 files keep the natural spelling as the evidence.

### Guessed holes: which held

| ID | Held? | Evidence |
|---|---|---|
| H01 | held | `pre[k]` = fold of first `k`: 04, 08 pass fuzz |
| H02 | held | `common_prefix_len(t.rev)` checks and is right (10 fuzz) |
| H03 | held | `@check` reads one case under `@!multitest`; `cpl judge` rejects a wrong `p` |
| H04 | not tested | sidestepped by reading `Int`; L1 shows `all` is `?T` |
| H05 | **failed in part** | lend only when the key is a simple expression (L5) |
| H06-H11 | held | check ok, fuzz ok |
| H12 | **failed in part** | `Rooted(g, root:)` works; `up` with `U != T` does not (L2) |
| H13, H14 | held | 04: depth 2e5 on the default stack |
| H15 | held | `nx` is a `std::array`; assignment, no heap |
| H16, H17 | held | 09, 10 fuzz |
| H18 | held (still a gap) | wins carried as `Int` |

### New holes found by running (found-late)

| ID | Class | Where | What | M1 |
|---|---|---|---|---|
| L1 | found-late (contradiction) | `spec/lib/sequence-patterns.md` example vs checker | `w.all <= K` on `?T` is E0308 | yes (std-seq) |
| L2 | found-late | `std/tree.cp` `Rooted.up` | `lift`'s `U` is unified with `T` | yes (graphs and trees) |
| L3 | found-late | `spec/core/07-sequences.md` adapters | no `.chain` on a `Str` slice | yes (strings) |
| L4 | found-late (implementation) | `std/seq.cp:3175`, `std/window.cp` | `maximal(Window(op), ...)` calls `pop(x)`; `Window.pop()` takes none | yes (std-seq, std-window) |
| L5 | found-late (implementation, perf) | codegen of `let x = m[k]` | list copy per iteration when `k` is an index expression | yes (R05.3) |

### Updated counts

| Class | Before | After |
|---|---|---|
| silent | 15 | 15 |
| contradiction | 2 | 2 |
| missing-stdlib | 1 | 1 |
| found-late | 0 | 5 |
| total | 18 | 23 |

Of the 18 guessed: 15 held, 2 failed in part (H05, H12), 1 untested (H04). All 5 found-late
holes are inside the Milestone 1 subset; L4 and L5 are implementation bugs, L1 a spec example
the checker rejects.
