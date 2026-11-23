# Rebuild round 21: holes

Cold reader: before this file only `spec/` and root `std/` were read (plus the directory
names under `plan/domains/spec/delta/`). Statements came from codeforces.com.

## Problems

| NN | CF | Div | Rating | Category | Main facility |
|---|---|---|---|---|---|
| 01 | 1974E Money Buys Happiness | 3 | 1800 | multi-dim DP (month cut x happiness) | `[0..=m, 0..=H]?Int`, `min=` |
| 02 | 1969C Minimizing the Sum | Edu | 1700 | multi-dim DP (prefix cut x ops) | `[0..=n, 0..=k]?Int`, `?Int + Int` lifting |
| 03 | 1968G1 Division + LCP (easy) | 3 | 1900 | binary search on the answer | `last_true`, `z_function` |
| 04 | 1954E Chain Reaction | Edu | 2200 | DSU | `Dsu<V>`, `v.prev`/`v.next`, `by` ranges |
| 05 | 1976D Invertible Bracket Sequences | Edu | 2000 | sparse table | `SparseTable(max, ..)`, `scan(+)`, `last_true` |
| 06 | 1955G GCD on a grid | 3 | 1900 | grid | `[R, C]` arrays, `(r, c).up/.left`, `divisors`, `gcd` |
| 07 | 1978E Computing Machine | 2 | 2000 | mixed 0-based (strings) and 1-based (queries) | `Str`, `scan(+)` prefix over cuts |
| 08 | 1957C How Does the Rook Move? | 2 | 1600 | modular inverse | `Mod<P>`, `Comb.inv_fact` |
| 09 | 1730B Meeting on the Line | 2 | 1600 | float answer, 1e-6 | `F64`, `print` |
| 10 | 1543C Need for Pink Slips | 2 | 1900 | float answer, 1e-6 | `read<F64>`, `?F64` recursion, `precision:` |

Selection deviations (all forced, checked by scanning statements through the CF API):
- No rated 1600-2200 problem in contests 1950-1980 prints a float with an error tolerance
  (only the unrated April Fools 1952G/1953A do). A scan of 1900-2060 found none either, so
  the two float problems come from 1730 and 1543.
- No in-range problem uses a modulus other than 998244353 / 1e9+7; 1957C satisfies the
  category through its modular inverse (`inv_fact`).
- 1971G (the only other DSU-tagged one) is Div. 4, so DSU is 1954E.

## Summary

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 13 | 11 |
| contradiction | 1 | 1 |
| missing-stdlib | 2 | 2 |
| found-late | 2 | 2 |
| total | 18 | 16 |

## Top five risks

1. H1: product arrays over plain `Int` ranges (`[0..=n, 0..=k]?Int`); the spec only shows
   products of declared domains. Four solutions and all three revisions depend on it.
2. H3: `last_true` on a true-then-false predicate. The spec names it but never says which
   monotone shape it assumes. 03, 05 and 03-rev depend on it.
3. H11: what `print(F64)` prints with `io.precision = None` (shortest round trip? fixed 6?).
   It decides whether 09 passes a checker and whether `2.5` prints as `2.5`.
4. H4: `Str.map(..).scan(+)` is a scan of a view of an `Int`-keyed sequence; whether the
   result is indexed by plain `Int` cuts is stated for `List` only. 05 and 07.
5. H7: no way to index a `Str` by a 1-based domain, so the mixed-base problem subtracts 1 by
   hand (the off-by-one the language is meant to prevent).

## Holes

| ID | Class | M1 | Location | What is unclear | What I guessed |
|---|---|---|---|---|---|
| H1 | silent | yes (R04.2, R04a.7) | spec/core/04-indices.md product domains | Products are shown only over declared domains (`[V, K]T`); not over anonymous `Int` ranges | `[0..=m, 0..=H]?Int` works and indexes with `dp[i, s]` |
| H2 | silent | yes (R02.8) | spec/core/02-values-types.md arrays; 04-indices.md | Array comprehension `[D] v => e` shown over domains; over an `Int` range and with `_` binder unclear | `[0..=n] c => pre[c]`, `[1..=A] _ => List<V>()` |
| H3 | silent | yes (std-search) | spec/lib/search-dp.md, spec/core/04-indices.md | `last_true` monotonicity direction not stated | predicate true...true false...false, returns last true |
| H4 | silent | yes (R07.9, std-prefix) | spec/core/07-sequences.md "Scans are prefix sums over cuts" | Cuts of a `Str` or of a `.map` view over `0..n` | `Int` cuts, `pre[k]` folds the first `k` |
| H5 | silent | yes (R01.7) | spec/core/01-lexical-grammar.md 1.7 | header conditions: may a plain boolean come before `let` (`if s >= h, let p = ..:`)? | avoided: nested `if`s (see F1) |
| H6 | silent | yes (R02.4) | spec/core/02-values-types.md optionals | Is `opt != None` valid for `?Int` (needs `Eq` on `?T`)? | yes |
| H7 | missing-stdlib | yes (R02.3, R10.2) | spec/core/02-values-types.md 2.3, spec/core/10-io.md `io.base` | `Str` keys are always `0..len`; no 1-based view, `io(base: 1)` only covers counted domains | read `l` with `read<1..=n>() - 1` |
| H8 | silent | yes (R10.2) | spec/core/10-io.md | `read<[R, C]Int>()` order (row-major?) only stated for `[R, C]Char` | row-major, whitespace-separated |
| H9 | silent | yes (std-grid) | std/grid.cp `up`/`left`, 04-indices.md | indexing a `[R, C]` array with a tuple value `a[start]` and comparing `(r, c) == start` | both work |
| H10 | silent | yes (std-math, R06.2) | std/math.cp `divisors`; 06-control-functions.md | passing a nested `fn` as a predicate value to `.filter` | allowed |
| H11 | silent | yes (R10.3) | spec/core/10-io.md R10.3.4, std/io.cp `precision: ?Int = None` | what `None` precision prints | shortest exact form; 10 forces `precision: 12` |
| H12 | silent | yes (R10.2) | spec/core/10-io.md | `read<F64>()` on `0.4998` (decimal tokens) | works |
| H13 | silent | yes (R02.4) | spec/core/02-values-types.md optionals | `return x + y` in a `-> ?F64` fn wraps to `Some` | implicit wrap |
| H14 | contradiction | yes (std-mod) | std/mod.cp `Comb.new` vs spec constructor calls | `Comb<Z>(3)` in the doc example vs `pub fn new(limit:)`; spec says `T(...)` calls `new`, but the example name differs from the struct-literal form `Comb<M>([..], ..)` used inside | `Comb<Z>(300_000)` calls `new` |
| H15 | silent | yes (std-mod, R07.9) | spec/core/07-sequences.md folds | `.sum()` over `Mod<P>` values (zero from where?) | works via `Add` zero |
| H16 | missing-stdlib | yes (std-strings) | std/strings.cp | no "greedy count of disjoint occurrences" (`find_all` overlaps); wrote a loop over `z` | hand loop |
| H17 | silent | yes (R02.3) | spec/core/02-values-types.md 2.3 | combining `Str(len: n..=n)` with `of "01"` | used only `of "01"` |
| H18 | silent | yes (R04.4) | spec/core/04-indices.md ranges | `read<l + 1..=n>()` with a dependent arithmetic bound | works |
| F1 | found-late | yes (R01.7) | spec/core/01-lexical-grammar.md line ~261 | comma header conditions `if let v = m.get(k), v > 0:` exist | found after writing 01/01-rev; nesting kept |
| F2 | found-late | yes (R07.9) | spec/core/07-sequences.md "Position queries" | `a.keys.min(by:)` / `.keys.map` is the documented argmin; I used `x.keys.map(i => ..)` where `zip` might be more natural | kept `.keys.map` |

## Per problem

| NN | Holes |
|---|---|
| 01 | H1, H5, H6, F1 |
| 02 | H1 |
| 03 | H3, H16, H18 |
| 04 | H2 |
| 05 | H2, H3, H4 |
| 06 | H8, H9, H10 |
| 07 | H4, H7, H17, H18 |
| 08 | H14, H15 |
| 09 | H11, F2 |
| 10 | H11, H12, H13 |

## Revision test

| Problem | Change | Lines changed | Chunks | Hesitation / duplicated fact / unrelated rewrite |
|---|---|---|---|---|
| 01-1974E | at most q purchases: dp gains a purchase-count dimension | 9 of 19 (read q, dp type, init, extra loop + re-indent of 5 body lines, answer) | 3 (read, table, answer) | re-indenting the loop body is the biggest part; the answer line needed `.any` over the new dimension: the "reachable" fact is restated per dimension. No unrelated rewrite. |
| 02-1969C | copies become directional: kl from the left, kr from the right | 11 of 17 | 3 (reads, table, transition) | The transition is a real rewrite (running min no longer valid: the kept element must be chosen), so not local; that is the problem, not the language. Init needs a second nested loop because there is no fill of a sub-array `dp[0, .., ..] = 0`. |
| 03-1968G1 | print f_l..f_r instead of f_l | 2 of 21 (drop `let k = l`, wrap the search in a comprehension) | 1 | none; the search expression moved unchanged into `[.. for k in l..=r]`. |

## After running

Toolchain: `cargo build -p cpl-cli` (debug). Per file: `cpl check`, `cpl run d` on every
sample in `d.tests/`, `cpl test d`, `cpl build d.cp -o d.cpp` + `g++ -std=c++17 -O2` + run on
the samples. Stress: `stress/stress.py` (Python generator + independent brute, 200 seeded
small inputs per file, interpreter vs C++ vs brute; floats compared at 1e-6 abs/rel).
Timing: `stress/maxgen.py` maximum-size input, C++ -O2 wall time. Every max input was also
run through the interpreter and an `-fsanitize=address,undefined` build: outputs identical
to the -O2 build, no sanitizer report, everywhere. Revisions have one stress-derived case in
`d.tests/`.

### Status

| NN | check | samples (`cpl run` / `cpl test`) | C++ samples | stress 200 | C++ max | interp max |
|---|---|---|---|---|---|---|
| 01 | ok after A1 | 1/1 | exact | 200/200 | 0.024 s | 0.3 s |
| 01-rev | ok after A1 | 1/1 | - | 200/200 | 0.026 s | 6.6 s |
| 02 | ok first try | 1/1 | exact | 200/200 | 0.041 s | 1.4 s |
| 02-rev | ok first try | 1/1 | - | 200/200 | 0.213 s | 152.6 s |
| 03 | ok after A2 | 1/1 | exact | 200/200 | 0.007 s | 0.8 s |
| 03-rev | ok after A2 | 1/1 | - | 200/200 | 0.051 s | 13.2 s |
| 04 | ok after A1, A4 | 3/3 | exact | 200/200 | 0.025 s | 0.8 s |
| 05 | ok after A3 | 1/1 after A6 | exact | 200/200 | 0.044 s | 4.3 s |
| 06 | ok first try | 1/1 | exact | 200/200 | 0.127 s | 95.2 s |
| 07 | ok after A3, A5 | 1/1 after A6 | exact | 200/200 | 4.103 s -> 0.019 s after A9 | 0.5 s |
| 08 | ok first try | 1/1 | exact | 200/200 | 0.008 s | 0.5 s |
| 09 | ok after A7 | 1/1 with `@check` (exact compare fails by design: `0.0000000000` vs `0`) | within 1e-6 | 200/200 | 0.010 s | 0.2 s |
| 10 | ok after A7 | 1/1 with `@check` | exact | 200/200 | 0.003 s | 0.1 s |

Over 1 s: only 07 before its fix (A9). Interpreter and C++ agreed on every sample, every
stress case and every max input. No crash, no sanitizer finding, no wrong answer left.

### Failures

| ID | Files | Class | What happened | Smallest reproduction | Fix in the solution |
|---|---|---|---|---|---|
| A1 | 01, 01-rev, 04 | reader error | E0208 `let H` / `let A`: values are lowercase (I knew the rule, slipped) | `let H = 1` | renamed `hs`, `top` |
| A2 | 03, 03-rev | implementation bug (E0504 "not implemented yet") | `Str(len: ..)` is in spec 2.3 but unimplemented; resolves H17 | `let s = read<Str(len: 1..=5)>()` | `read<Str>()` |
| A3 | 05, 07 | implementation bug | `Str of "01"` (spec 2.3) is a parse error E0107 | `let s = read<Str of "01">()` | `read<Str>()` |
| A4 | 04 | implementation bug (E0501 "can't handle ... yet") | a stepped range used as a value | `print((0..10 by 3).map(t => t).sum())` | `(0..top).step(k).map(..)` |
| A5 | 07 | implementation bug; resolves H18 against my guess | `read<l + 1..=n>()` parses as `read < l + 1..=n > () - 1`, a `Bool`; the error lands later as E0309 at the uses of `r`, naming nothing about generics. `read<l..=n>()` (03) works | `let l = read<1..=5>()` / `let r = read<l + 1..=9>()` / `print(r + 1)` | `read<1..=n>()` (the `r > l` contract is lost) |
| A6 | 05, 07 | implementation bug (contradicts spec 07 "a List's cuts are Ints: `xs.scan(+)[k]` folds the first `k`"); resolves H4 | `Int` index into a `Prefix` is taken as a position (folds through it): `p[0]` is the first element, `p[len]` traps R0102 "`len` is not in `D_0`". Same for a `Str.map` view | `let p = [1, 1, -1, -1].scan(+)` / `print(p[0], p[4])` (prints 1, then traps; spec says 0 and 0) | explicit `[0..=n]Int` prefix arrays |
| A7 | 09, 10 | implementation bug | `read_output<F64>()` in a `@check` fn: E0501 "`read_output` is not in scope during lowering" | `@check fn ok() -> Bool:` / `    let x = read_output<Int>()` / `    x == 0` | `let got: F64 = read_output()` |
| A8 | 09, 10 | implementation bug | a `@check` that returns `false` is never honoured: `cpl judge f.cp in bad.out` exits 0 (help says 3 on reject) and `cpl test` passes. With and without `@!multitest` | `@check fn ok() -> Bool:` / `    let n = read<Int>()` / `    let got: Int = read_output()` / `    got == n` / `let n = read<Int>()` / `print(n + 1)`; `cpl judge ck.cp` on input `5`, output `6` -> exit 0 | none possible; float tolerance verified by the harness instead (`stress.same`) |
| A9 | 07 | reader error | `for i in l..=r if i < l + 2 or i > r - 2` walks the whole window per query: O(nq), 4.1 s | (07 max input) | two explicit end ranges, 0.019 s |
| A10 | 09, 10 | spec hole (silent) | under `@!multitest` the `@check` fn runs once per test case, after the count is read; spec/tools/stress.md does not say so. My first checker read the count itself and failed with "0 is outside `1..=1e5`" | (09 checker with `let tests = read<1..=1e3>()` first) | per-case checker |

Pre-run guesses that held: H1 (product arrays over plain ranges), H2, H3 (`last_true` on
true-then-false), H5/F1, H6, H8, H9, H10, H12, H13, H14 (`Comb<Z>(n)`), H15, H16 (hand
loop needed). H11 resolved: `print(F64)` with no precision prints 10 decimals, and the C++
build prints the same text. Wrong guesses: H4 (A6), H17 (A2), H18 (A5).

Counts: 10 failures across 13 files: 2 reader errors (A1, A9), 7 implementation bugs (A2-A8;
A2 and A4 are announced as "not implemented yet"), 1 spec hole (A10). 5 files checked on the
first try; all 13 end green on samples, C++ samples, 200-case stress and timing.
