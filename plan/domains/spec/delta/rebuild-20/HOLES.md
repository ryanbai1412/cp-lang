# Rebuild round 20: holes found by a cold reader

Base: `main` at `9069a65ea`. Before this file was written I read only `spec/`
(core, lib, tools, `SUBSET.md`, and greps over `spec/conformance` and
`spec/lints`, which are inside `spec/`) and root `std/`. Prior rebuild rounds
were checked by `ls` only.

## Problems

Ten Codeforces problems rated 1700-2300 from contests 1900-1919. None of
their IDs appears under `rebuild-*/`.

| # | File | Problem | Rating | Areas covered |
|---|---|---|---|---|
| 01 | `01-1903E.cp` | Geo Game | 2000 | interactive, geometry (squared Euclidean distance parity) |
| 02 | `02-1918E.cp` | ace5 and Task Order | 2200 | interactive, randomized quicksort, `Rng` |
| 03 | `03-1916D.cp` | Mathematical Problem | 1700 | any-answer output (`@check`), geometry/math, `BigInt` in the checker |
| 04 | `04-1906M.cp` | Triangle Construction | 1700 | geometry (regular polygon), 64-bit sums up to 4e14 |
| 05 | `05-1902E.cp` | Collapsing Strings | 1900 | strings: double-modulus rolling hash, hash map |
| 06 | `06-1907F.cp` | Shift and Reverse | 1800 | strings: KMP `find_all` over `Int` sequences |
| 07 | `07-1912K.cp` | Kim's Quest | 1800 | bitmask DP over a two-bit parity state, `Mod<998244353>` |
| 08 | `08-1900D.cp` | Small GCD | 2000 | 64-bit overflow risk (answer ~8.53e18, just under 2^63), `Sieve`, phi |
| 09 | `09-1916E.cp` | Happy Life in University | 2300 | segment tree with range add + range max over `t.Pre`, Euler tour |
| 10 | `10-1902D.cp` | Robot Queries | 1900 | prefix positions, `group`, `partition_point` |

Coverage notes:
- Geometry/floating point: no problem in 1900-1919 rated 1700-2300 prints a
  float. No statement among the 41 fetched says "relative error", so the
  geometry slots are integer geometry (01, 03, 04). `F64` output stays untested.
- Strings: 05 is a real hashing problem. 06 is a sorting problem that I solved
  with KMP on `Int` sequences, to exercise `find_all` over a non-`Str` sequence.

## Summary

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 11 | 10 inside, 1 outside |
| contradiction | 3 | 3 inside |
| missing-stdlib | 2 | 1 inside, 1 outside |
| found-late | 0 | (filled in after running) |
| **total** | **16** | |

"Inside" means the hole's section is listed as `supported` in `spec/SUBSET.md`.
"Known gap" means `SUBSET.md` already lists a conformance case for it under
*Known gaps*.

## Top five risks

1. **H1, the `@interactor` protocol.** `lib/random-stress.md` gives it two
   lines and `spec/conformance` has no `@interactor` case. Both interactive
   solutions depend on my guess about how the judge gets hidden data, talks to
   the program and gives a verdict.
2. **H8, `gen` is a known gap.** Every `core-06/gen-*` case is listed as a
   known gap, and both interactors plus 03's `@gen` draw from `gen(...)`.
3. **H3, the `@check` signature.** It names types `In` and `Reader` that no
   chapter defines. 03's checker is guesswork.
4. **H12, `sorted(by: desc)`.** `core-07/sorted-has-no-desc-flag` is a known
   gap, and 06 relies on it.
5. **H6, RollingHash bases.** Each constructor draws random bases, so hashes
   from two constructors never compare unless `bases:` is passed. 05 passes
   fixed bases. This contradicts the "salts fixed per build profile" rule.

## Holes

| ID | Class | Location | Hole | What I guessed | M1 subset |
|---|---|---|---|---|---|
| H1 | silent | `lib/random-stress.md` §Interactors; `tools/stress.md`; `tools/cli.md` (`cpl run`, `cpl judge`) | `@interactor fn judge(w: Wire)` says only that it is "the other end of the pipes" run "with generated hidden data". It does not say where hidden data comes from (it has no `input` parameter), which streams its `read`/`print` use, how it rejects (return value? trap?), whether it runs once or per test under `@!multitest`, what `cpl judge d input` feeds it, or how `d.tests/` stores interactive samples. | The interactor draws hidden data with `gen(...)`. Its `print` goes to the program's stdin and its `read` reads the program's stdout. `assert` failure means reject; normal return means accept. It runs once and prints `t` itself. Interactive problems get no static `.in`/`.out` samples. | inside (Rstress supported, 2 pass); no conformance case |
| H2 | contradiction | `tools/cli.md` row `cpl judge` vs `core/06-control-functions.md` R06.4.3 | The CLI table says `cpl judge` runs the file's "`@checker` on an output", but R06.4.3 makes `@checker` error E0503 and names `@check`. | `@check` is the name; the CLI row is stale. | inside (R06.4) |
| H3 | silent | `lib/random-stress.md` §Checkers | `@check fn ok(input: In, out: Reader) -> Bool` uses `In` and `Reader`, which no chapter or `std/` module defines. How a checker reads tokens is never shown. | Both parameters are `Reader`, with `r.read<T>()` / `let x: T = r.read()` working like `read`. | inside (R06.4, 9 known gaps) |
| H4 | silent | `core/10-io.md` §10.4; `std/io.cp` | No exit/abort primitive is documented. Interactive statements say "terminate immediately on -1". | Read the judge's reply as a domain type (`read<P>()`), so a `-1` traps and the program stops. 02 does not check for `-1`. | inside (R10.1) |
| H5 | contradiction | `lib/graphs-trees.md` Trees table vs `std/tree.cp:278` | The spec says `t.children(v)` is an "`Indexed` view of `v`'s children". `std` declares `pub fn children(self, v: V) -> yields V`, a generator. | Use it only in `for`. | inside (std-tree supported, 3 known gaps) |
| H6 | contradiction | `std/strings.cp` `RollingHash.new` vs `lib/random-stress.md` §Randomness | `RollingHash.new` draws bases with an undocumented `rand(lo, hi)` per constructor. random-stress says "Hashing salts (`HashMap`, string hashing) are fixed per build profile, not drawn from an `Rng`". So whether two runs, or interpreter and C++, give the same hashes is unclear. Two constructors' hashes are never comparable without `bases:`. | Pass `bases: (91_138_233, 97_266_353)` to every constructor. | inside (std-strings supported) |
| H7 | missing-stdlib | `std/math.cp` BigInt | `BigInt` has `+ - * /`, `Ord`, `from_str` and `divmod`, but no `isqrt`/`is_square`. A checker for "output squares" needs one. `std-math/bigint-matches-int-arithmetic` is also a known gap. | Binary search on `BigInt` with `*` and `<=`. | inside (std-math supported); bigint is a known gap |
| H8 | silent | `lib/random-stress.md` §Typed generators; `SUBSET.md` known gaps `core-06/gen-*` | `gen(lo..=hi)` with runtime bounds inside a loop (`gen(0..=free.len - 1)`) is never shown. Whether `gen` may be called in an `@interactor` (not just `@gen`) is unstated, and every `gen` form is a known gap. | `gen(a..=b)` returns an `Int` uniformly in range, anywhere in a tool role. | inside, known gap |
| H9 | silent | `core/10-io.md` §10.1, §10.4 | Whether `Wire()` may be built above `@!multitest`, and whether the hidden read of `t` flushes after it, are not stated. | `let wire = Wire()` above `@!multitest`; every later read, including `t`, flushes. | inside (R10.1) |
| H10 | silent | `core/06-control-functions.md` §6.5 | "Top-level bindings above `@!multitest` are shared". It does not say whether a top-level `var` (the `Rng` in 02) can be mutated from a per-case nested `fn`. | Allowed; the stream continues across cases. | inside (R06.5) |
| H11 | silent | `core/10-io.md` R10.3.1; `core/04-indices.md` R04.2.15 | How a domain position prints (`print(j)` for `j: P`, `"? {i}"` for `i: I`) is not stated. `Int(v)` is the conversion, but `Show` for positions is not listed. | A position prints as `Int(v)` (1-based for `1..=n`). | inside (R10.3) |
| H12 | silent | `core/06-control-functions.md` R06.1.8 (`sorted(by: desc)`); `SUBSET.md` known gap `core-07/sorted-has-no-desc-flag` | `sorted(by: desc)` is the documented spelling of a descending sort, but its behaviour is a known gap. | It sorts non-increasing. | inside, known gap |
| H13 | silent | `core/01-lexical-grammar.md` layout; `SUBSET.md` known gap `core-01/continuation-leading-or` | A boolean `or` chain broken over lines: is a leading `or` allowed inside parentheses? It is a known gap. | Wrapped the expression in parentheses with a leading `or` (10). | inside, known gap |
| H14 | silent | `core/01-lexical-grammar.md` | A multi-line `match` expression as a call argument (`pos.push(match ch:` newline arms `)`) is never shown. | Arms indented under the `match`; the closing paren goes after the last arm. | inside (R01.8) |
| H15 | silent | `lib/random-stress.md` §Checkers; `tools/cli.md` `cpl test` | How `cpl test` handles a sample whose `.out` is one of many valid answers when the file has `@check` is not stated. Does it run the checker instead of a diff? Is the expected file `.out` or `.ans`? | Stored 03's sample answer as `1.ans` so no exact diff runs; validate with the checker through stress. | outside (no row for `@check` in `cpl test`) |
| H16 | missing-stdlib | `lib/strings.md` / `std/strings.cp` | There is no `RollingHash` facility for hashing many strings under one shared basis (a "hash family" object). Every caller must invent and pass `bases:`. Combined with H6, this is a missing piece for multi-string hashing. | Pass a literal pair everywhere. | outside (design gap) |

## Per-problem mapping

| Problem | Holes |
|---|---|
| 01-1903E | H1, H4, H8, H9, H11 |
| 02-1918E | H1, H4, H8, H9, H10, H11 |
| 03-1916D | H2, H3, H7, H8, H15 |
| 04-1906M | none |
| 05-1902E | H6, H16 |
| 06-1907F | H12 |
| 07-1912K | none |
| 08-1900D | none |
| 09-1916E | H5 |
| 10-1902D | H13, H14 |
