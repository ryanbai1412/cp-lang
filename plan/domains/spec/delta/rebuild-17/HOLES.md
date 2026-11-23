# Rebuild round 17: holes

Cold read of `spec/` and root `std/` only, at `7715976ac`. Ten Codeforces
problems from contests 1950-1965, ratings 1200-1900, none already present under
`plan/domains/spec/delta/rebuild-*/`. This file was written before anything was
built or run; `## After running` was added afterwards.

## Problems

| # | CF | Rating | Topic | Approach |
|---|---|---|---|---|
| 01 | 1950F | 1700 | trees, greedy | fill levels breadth-first, two-child vertices first |
| 02 | 1950G | 1900 | bitmask DP, graphs, strings | reachable `(mask, last)` over a compatibility relation |
| 03 | 1951B | 1200 | greedy, simulation | no swap, swap with front, swap with first stronger cow |
| 04 | 1951C | 1400 | greedy + sorting | buy on cheapest days, price shift = tickets already bought |
| 05 | 1954C | 1200 | strings, greedy | first differing digit goes to x, the rest to y |
| 06 | 1954D | 1800 | DP (knapsack), combinatorics | sort, subset-sum counts, value `max(a_max, ceil(S/2))` |
| 07 | 1955C | 1300 | greedy, prefix sums | front takes `ceil(k/2)` hits, back `floor(k/2)` |
| 08 | 1955G | 1900 | grids, number theory | divisors of `gcd(corners)`, monotone-path reachability |
| 09 | 1957C | 1600 | DP, combinatorics, modular | `f(i) = f(i-1) + 2(i-1) f(i-2)` over free rows |
| 10 | 1957D | 1900 | bits, prefix xor | count prefix-xor pairs with `a_y`'s top bit clear |

## Summary

| Class | Count | Inside Milestone 1 |
|---|---|---|
| silent | 4 | 4 |
| contradiction | 4 | 4 |
| missing-stdlib | 2 | 2 |
| found-late | 2 | 2 |
| **total** | **12** | **12** |

"Inside Milestone 1" means the hole falls in a row or library family that
`spec/SUBSET.md` lists as supported (sequences, strings, grids, I/O, math,
search, DP); none of the holes is in a gap row, except H02 which SUBSET itself
already lists as a known stdlib gap (`core-07/sorted-has-no-desc-flag.cp`).

## Top five risks

1. **H05 `popcount` is used but never defined.** `std/bitset.cp` calls
   `popcount(w[k])` as if it were a builtin, but no `std/` file defines it and
   no spec chapter names it. Bitmask DP is a staple; a reader cannot know
   whether `popcount(x)`, `x.popcount` or a hand loop is the spelling.
2. **H09 reading an `Int` grid.** `spec/core/10-io.md` shows `[R, C]Char`
   reading lines and `[n]T` reading tokens, but never says what
   `read<[n, m]1..=1e6>()` reads. Grid problems with numbers are common.
3. **H06 prelude omits `divisors` and `bit_len`.** Both exist in
   `std/math.cp` with docs, but `std/prelude.cp` does not export them, so a
   reader guessing from `gcd`/`bit` (which are exported) writes a name that
   does not resolve.
4. **H01/H02 stdlib README vs spec on search and sort.** The README still
   advertises `bsearch_first`/`bsearch_last` and `std/seq.cp` still has
   `desc: Bool` on `sort`/`sorted`, while `spec/core/04` and `spec/core/07`
   say both are gone. A reader trusting `std/` writes code the spec rejects.
5. **H08 `Char` ordering.** `spec/core/02` lists `Int(c)`, `Char(i)`,
   `c.is_digit` for `Char` but never says `Char` is `Ord`; `max(x[i], y[i])`
   on digits is the natural spelling.

## Holes

| ID | Class | Location | Hole | What I guessed | M1 |
|---|---|---|---|---|---|
| H01 | contradiction | `std/README.md` §4.2 vs `spec/core/04-indices.md` R04.8.3 | README lists `bsearch_first(lo..hi, ok)`/`bsearch_last` returning `none`; the spec says binary search is `r.first_true(f)`/`r.last_true(f)` and the `bsearch_*` names are replaced | spec wins; used `find` for the linear scan in 03 and no binary search elsewhere | inside (std-search) |
| H02 | contradiction | `std/seq.cp` `sort`/`sorted` (`desc: Bool = false`) vs `spec/core/07-sequences.md` R07.11 | std signature keeps a `desc:` flag that the spec says does not exist (key `desc(x)` instead) | used plain `.sorted()` ascending (04, 06) | inside, known gap |
| H03 | contradiction | `std/README.md` §4.2 `zip(a: List<A>, b: List<B>) -> List<(A, B)>` vs `spec/core/07-sequences.md` | README says `zip` materializes a `List`; the spec says `zip` is an O(1) indexed view keyed by the first operand | not used | inside (sequences) |
| H04 | contradiction | `std/README.md` §4.2 `runs(a) -> List<(val, len)>` vs `spec/core/07-sequences.md` R07.7.10 | README: list of `(val, len)` records; spec: `runs(by:)` yields views | not used | inside (sequences) |
| H05 | missing-stdlib | `std/bitset.cp` (calls `popcount`), no definition in `std/`, not in `spec/core/03` | population count has no documented spelling | `(0..n).count(i => mask.bit(i))` with prelude `bit` (02) | inside (DP, bitmasks) |
| H06 | missing-stdlib | `std/math.cp` `divisors`, `bit_len` vs `std/prelude.cp` `use std.math.{...}` | documented functions missing from the prelude; `spec/core/11` says the prelude table decides what is in scope | `use std.math.{divisors}` (08), `use std.math.{bit_len}` (10) | inside (std-math) |
| H07 | silent | `spec/core/02-values-types.md` (heap values), `spec/core/07-sequences.md` R07.12.9 | `Map`/`Set` are named everywhere but declared nowhere: hash or ordered, key requirements (`Str`? tuples?), iteration order | not used; compared `Str` fields directly in 02 | inside (containers) |
| H08 | found-late | `spec/core/02-values-types.md` §2 `Char` row | `Char` comparison/`Ord` not stated; found while writing 05 | `Char` is ordered by code, so `max`/`min` work | inside (strings) |
| H09 | silent | `spec/core/10-io.md` R10.2 | reading `[R, C]T` for a non-`Char` `T` is not described (only `[R, C]Char` lines and `[n]T` tokens) | `read<[n, m]1..=1e6>()` reads `n * m` tokens row-major (08) | inside (grids, I/O) |
| H10 | silent | `spec/core/10-io.md` R10.2.1 | records are shown with position fields (`(u: V, v: V)`); `Str` fields in a record read are not shown | `read<[n](g: Str, w: Str)>()` reads two tokens per row (02) | inside (I/O) |
| H11 | silent | `spec/core/07-sequences.md` R07.2.11, `spec/core/02` arrays | whether a `[n]T` array (counted domain) passes where `List<T>` is expected, or needs `List(...)` | wrapped explicitly: `List(read<[n]1..=1e9>())` (03) | inside (sequences) |
| H12 | found-late | `spec/core/06-control-functions.md` R06.1.7, `spec/core/10-io.md` R10.1 | a `fn` after `@!multitest` capturing per-case `let`s (`g`, `n`, `m`) and used as a predicate value (`ds.rev.find(passable)`) is implied by "functions hoist... captures that case's names" but never shown passed as a value | named nested fn passes as `T -> Bool` (08) | inside (control) |

Reader corrections made before running (not holes): `.copy()` -> `.clone()`
(`spec/core/05-ownership.md` R2), `.rev` is a property, not `.rev()`
(`spec/core/04-indices.md` R04.4.13).

## Per-problem mapping

| # | CF | Holes touched |
|---|---|---|
| 01 | 1950F | none |
| 02 | 1950G | H05, H07, H10 |
| 03 | 1951B | H01, H11 |
| 04 | 1951C | H02 |
| 05 | 1954C | H08 |
| 06 | 1954D | H02 |
| 07 | 1955C | none |
| 08 | 1955G | H06, H09, H12 |
| 09 | 1957C | none |
| 10 | 1957D | H06 |

H03 and H04 were found while reading and touch no solution.

## After running

`cargo build -p cpl-cli` at `7715976ac`. Per solution: `cpl check`, `cpl run`
on every sample, `cpl build` then `g++ -std=c++17 -O2` and the binary on every
sample. No sample crashed or differed, so no sanitizer rerun was needed.

| # | CF | check | run (samples) | C++ build + g++ | C++ (samples) |
|---|---|---|---|---|---|
| 01 | 1950F | no errors | 1/1 | ok | 1/1 |
| 02 | 1950G | no errors | 1/1 | ok | 1/1 |
| 03 | 1951B | no errors | 1/1 | ok | 1/1 |
| 04 | 1951C | no errors | 1/1 | ok | 1/1 |
| 05 | 1954C | no errors | 1/1 | ok | 1/1 |
| 06 | 1954D | no errors | 3/3 | ok | 3/3 |
| 07 | 1955C | no errors | 1/1 | ok | 1/1 |
| 08 | 1955G | no errors | 1/1 | ok | 1/1 |
| 09 | 1957C | no errors | 1/1 | ok | 1/1 |
| 10 | 1957D | no errors | 1/1 | ok | 1/1 |

Failures: none (0 reader errors, 0 spec holes, 0 implementation bugs).

### What the implementation does with each guess

Single-line probes run with `cpl run`, to settle the guesses the solutions
avoided or made:

| Hole | Probe | Result | Classification |
|---|---|---|---|
| H01 | `print(bsearch_first(0..10, i => i * i >= 20))` | E0201 "not in the stdlib (yet)", help suggests `let bsearch_first = ...` | spec hole (stale `std/README.md`); the help should point at `first_true` |
| H02 | `print([3, 1, 2].sorted(desc: true))` | prints `3 2 1` | spec hole: implementation follows `std/seq.cp`, not R07.11 (SUBSET already lists it) |
| H03 | `print(zip([1, 2], [3, 4]).len)` | prints `2` | consistent with both readings; contradiction stays a doc hole |
| H05 | `print(popcount(13))` | prints `3` | spec hole: `popcount` is a working builtin with no spec or `std/` entry |
| H06 | `print(divisors(12))` without `use` | E0201 with help `add use std.math.{divisors}` | spec hole confirmed; the diagnostic recovers it |
| H11 | `fn f(xs: List<Int>)` called with `read<[3]Int>()` | prints `3` | spec hole: a counted `[n]T` passes as `List<T>`; my `List(...)` wrap was unneeded |

H07-H10 and H12 were exercised directly by the solutions and behaved as
guessed (H08 `Char` `max`/`min` in 05, H09 `Int` grid read in 08, H10 `Str`
record fields in 02, H12 nested fn as a predicate in 08).
