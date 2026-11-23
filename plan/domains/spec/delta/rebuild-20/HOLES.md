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
| found-late | 5 | 5 inside |
| **total** | **21** | |

The 16 silent, contradiction and missing-stdlib holes were written before any
build. The five found-late holes (H17-H21) are in *After running*.

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

## After running

Build: `cargo build -p cpl-cli` (debug) on `9069a65ea`. `cpl` is
`target/debug/cpl`. "C++" means `cpl build` and then `g++ -std=c++17 -O2`.
Times are the native binary's on one maximum-size input, on this VM.

### Per problem

| # | `cpl check` | `cpl test` / `cpl run` samples | C++ samples | Stress / random | Max-size time (C++) |
|---|---|---|---|---|---|
| 01-1903E | ok after IB4 | no static samples (interactive) | builds | `cpl run`/`stress`/`judge` against `@interactor`: FAIL (IB2). Python interactor: native 250 runs, interpreter 20 runs, all win | n = 1e5: 0.57 s CPU (0.15 user + 0.42 sys, pipe-bound); t = 2000 small: ok |
| 02-1918E | ok after H17, H18, IB4 | no static samples (interactive) | builds | `@interactor`: FAIL (IB2). Python interactor: native 200 runs, interpreter 20 runs, all correct. n = 2000, 8 seeds: 30.9n-35.2n queries (limit 40n) | n = 2000: 0.33 s CPU |
| 03-1916D | ok after IB4, H17, H19, R2 | `cpl test` 0/1: FAIL (IB1, the checker cannot read the output) | builds | Python checker: native and interpreter both valid on every odd n in 1..99, identical output; the sample's output is valid | t = 10, n = 99: 0.00 s |
| 04-1906M | ok | 3/3 | 3/3 | no brute (closed form) | n = 2e5, a = 2e9: 0.00 s, prints 133333333333333 |
| 05-1902E | ok after H17, IB3, R4 | 2/2 | 2/2 | 200 random: 0 mismatches | 1e6 total chars: 0.16 s (worst of 3 shapes) |
| 06-1907F | ok | 1/1 | 1/1 | 200 random (BFS brute): 0 mismatches | 2 x 1e5: 0.01 s |
| 07-1912K | ok | 5/5 | 5/5 | 200 random (subset brute): 0 mismatches | n = 2e5: 0.00 s |
| 08-1900D | ok | 1/1 | 1/1 | 200 random (triple loop): 0 mismatches | n = 8e4: 0.10 s (all 83160). All-1e5 prints 8533013336000000000, exactly C(80000,3) x 1e5, 92.5% of 2^63 |
| 09-1916E | ok after R1, H21 | 1/1 | 1/1 | 200 random (all pairs): 0 mismatches | n = 3e5 path: 0.22 s |
| 10-1902D | ok after IB3, R3 | 3/3 | 3/3 | 200 random (simulate): 0 mismatches | n = q = 2e5: 0.05 s |

No maximum-size input took over 1 s. The random comparison ran `cpl run`, the
C++ binary and a Python brute on 200 inputs each for six problems (05-10):
1200 inputs, 0 mismatches, 0 crashes. The one "crash" seen came from my
generator (09, a colour 2 with n = 1). The C++ correctly rejected it with
"a number in the input is outside its range", and the ASan/UBSan build gave the
same rejection. ASan/UBSan builds of 05, 08 and 09 also ran clean on the
maximum inputs, with identical output.

### Failures

**Implementation bugs**

| ID | Smallest reproduction | Effect |
|---|---|---|
| IB1 | `let n: Int = read()` / `print(n)` / `@check` / `fn ok() -> Bool:` / `    let n: Int = read()` / `    n == 5`. `cpl judge f.cp in out ans` with in = `5`, out = `4` exits 0. A second `read()` in the checker traps with "input ended (input line 2)". | A `@check` sees only the input. It has no way to read the contestant's output, so it can never reject a wrong answer. Any-answer problems cannot be checked (03's `cpl test` fails). |
| IB2 | `let wire = Wire()` / `let n: Int = read()` / `print(n * 2)` / `@interactor` / `fn judge(w: Wire):` / `    let n: 1..=9 = read()` / `    let m: Int = w.ask("{n}")` / `    assert m == 2 * n`. `cpl run f.cp h` with h = `3` prints `6` (the program read the hidden file itself), then "the interactor `judge` rejected the run: input ended (input line 2)" at `w.ask`. `cpl stress` shows "got:" empty and the same interactor error. | The program and the interactor are never connected. The interactor's `w.ask` reads the hidden input, not the program's output; the program reads the hidden input directly. No interactive file can pass `cpl run`, `cpl judge` or `cpl stress`. |
| IB3 | `let s = read<Str of "UDLR">()` gives E0107 "expected the end of the statement". `read<[n](Str of "ab")>()` gives E0103. `let s: Str of "UDLR" = read()` works. | `Str of "..."` (R02.3.2) is not accepted as a type argument. |
| IB4 | `@gen` / `fn input(): let t = gen(1..=5)` gives E0201 "no `gen` in scope". | Typed generators do not exist, which confirms H8 (`core-06/gen-*` known gaps). The tool roles use `Rng` instead. |
| IB5 | `@check fn ok(a: Str, b: Str) -> Bool` gives E0303 "a `@checker` fn takes no parameters". | The diagnostic uses the removed `@checker` name (H2). E0303's `cpl explain` text describes a call-arity error instead. |

**Reader errors** (my code was wrong; the spec was clear enough)

| ID | Problem | Fix |
|---|---|---|
| R1 | 09: a `match` on `t.tour` events lacked an `Again` arm (E0314; R06.3.4 lists `Again`). | Add `_ => ()`. |
| R2 | 03: `BigInt` is not `Copy`. `var hi = one` moved `one`, and `seen.add(s)` moved `s` before its later use (E0608). | Build fresh values; move `add` after the last use. |
| R3 | 10: `pos.last` is `?T`, so it cannot be destructured into `(x, y)` (E0302). | `pos.last!`. |
| R4 | 05: `RollingHash(s.rev, ...)` stores a lent view (E0603). | `RollingHash(Str(s.rev), ...)`. |

**Found-late spec holes**

| ID | Class | Location | Hole | Hit by | M1 subset |
|---|---|---|---|---|---|
| H17 | found-late | `lib/random-stress.md` (`Rng(seed)`), `std/math.cp` (`BigInt`), `std/strings.cp` (`HashVal`) | The spec shows `Rng`, `BigInt` and `HashVal` bare, but each needs `use std.<mod>.{Name}` (E0201/E0204). `Sieve`, `RollingHash`, `SegTree` and `Rooted` work bare. Nothing I read lists which std names are in the prelude. | 02, 03, 05 | inside |
| H18 | found-late | `core/04-indices.md`, `core/07-sequences.md` | `List(I)` over a domain is a `List<domain I>`, not a `List<I>` of positions (E0302). The `domain T` type is not described where `List(D)` is. `[i for i in I]` gives `List<I>`. | 02 | inside |
| H19 | found-late (contradiction) | `lib/random-stress.md` §Checkers vs the compiler | The spec's `@check fn ok(input: In, out: Reader) -> Bool` is rejected: "takes no parameters: it reads the input the file reads". Neither form says how the output is read (IB1). Resolves H3 as "no such types". H15: `cpl test` does accept `1.ans`. | 03 | inside |
| H20 | found-late | `lib/random-stress.md` §Interactors; `tools/stress.md` | Observed rules the spec never states: (a) the interactor's `read()` reads the hidden test, which `stress` generates from the interactor's typed reads; (b) under `@!multitest` the interactor runs once per test, and the harness supplies `t`; (c) `stress` rejects an interactor whose `read()` sits in a loop ("read them into one first"); (d) no interactor-side "read the program's reply" exists, so the only spelling is `w.ask(...)`, and I used `w.ask("")` to read without asking. My H1 guess (`gen` hidden data, `read` = program output, interactor prints `t`) was wrong on every point. | 01, 02 | inside |
| H21 | found-late | `core/06-control-functions.md` §6.3 (match) | There is no empty statement (`pass`), and no example of a no-op `match` arm. `()` works. | 09 | inside |

### Risk notes

- 02 uses 30.9n-35.2n queries at n = 2000 against a 40n limit. It passes, but
  with about a 12% margin. The CP Lang code matches the usual C++ approach.
- No problem printed a float, so `F64` output formatting was not exercised
  (coverage note at the top).

## Support files

`support/` holds what the runs above used. `gb.py gen|brute NN seed` is the generator and brute for 05-10, and `diff.sh NN name 200` compares the three on 200 inputs (set `CPL`, `SRC`, `BIN`). `inter.py` is the Python interactor for 1903E and 1918E (`FULL=1` for maximum n). `chk1916D.py in out` checks 03.
