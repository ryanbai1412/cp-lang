# Rebuild round 24: holes found by a cold reader

Base: `main` at 8fea5406b (descendant of 78655ba8f). Before this file was written only
`spec/` and root `std/` were read (plus the `rebuild-*` directory names under
`plan/domains/spec/delta/`, to skip problems already used). Statements came from
codeforces.com.

## Problems

| # | CF | Rating | Category it covers | Std facilities used |
|---|---|---|---|---|
| 01 | 2075D Equalization | 2000 | DP: 0/1 knapsack (2D) | `?Int` table, `min=` |
| 02 | 2053C Bewitching Stargazer | 1500 | DP on intervals (one state per segment length) | named tuple return |
| 03 | 2060E Graph Composition | 1500 | graph, DSU | `Dsu<V>` |
| 04 | 2070C Limited Repainting | 1500 | binary search on the answer | `Range.first_true` |
| 05 | 2046B Move Back at a Cost | 1600 | monotonic stack | `List` as stack, `sort` |
| 06 | 2052J Judicious Watching | 2000 | many queries answered offline (sorted by time) | `sorted(by:)` |
| 07 | 2050F Maximum modulo equality | 1700 | number theory: gcd | `SparseTable(gcd, ...)`, `try_fold` |
| 08 | 2065E Skibidus and Rizz | 1600 | string construction / implementation | `Str.repeat`, `+` |
| 09 | 2077B Finding OR Sum | 1900 | interactive (+ `@interactor`) | `Wire`, `Interactor` |
| 10 | 2075C Two Colors | 1500 | multitest, t = 1e4 with sum n <= 2e5: an O(max n) reset per test is 2e9 | per-test `[1..=n]Int` |

## Summary

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 10 | 7 |
| contradiction | 2 | 1 |
| missing-stdlib | 1 | 1 |
| found-late | 1 (F01, see After running) | 1 |
| **total** | **14** | **10** |

"Inside" means the rule or library family has a *supported* row in `spec/SUBSET.md`.

## Top five risks

1. **H05/H06/H07: interactive multitest is under-specified.** Who sends `t` to the program
   when the `@interactor` "runs once per test", whether `Wire()` must be `var`, and how an
   interactor reads a token that is either `!` or a number. R10.4 has no row in `SUBSET.md`.
2. **H03: the monotone-stack helpers (`prev_less`, `next_less`, ...) are documented
   (`std/README.md` 4.2, `std/seq.cp` module header) but not defined in any `std/*.cp`.**
   A reader who trusts the README writes a call that cannot resolve; 05 was hand-rolled.
3. **H04: `List` stack verbs (`push`, `pop`, `last`, `is_empty`, `min`) have no stub
   signature.** Return types (`pop() -> ?T`?) are guessed from examples (`xs.pop()!` in
   05-ownership, `while let x = q.pop()` in 01).
4. **H09/H10: arrays built over a literal range (`[0..n - 1] i => e`) and folding them with
   an `Int` range (`try_fold(l - 1..r - 1)`).** Whether the keys stay plain `Int` and whether
   a `Range<Int>` fits `Range<D>` is not stated.
5. **H01: three spellings of `MonoDeque` construction** in one README; a reader picks one at
   random.

## Holes

| ID | Class | In M1? | Location | What is unclear | What I guessed |
|---|---|---|---|---|---|
| H01 | contradiction | no (std-window row has no MonoDeque) | `std/README.md` 4.9 (`MonoDeque.new(by:)`), 4.10 (`new(pick:)`, `by(better:)`), rename note near 4.16 (`MonoDeque<T>(pick: min)`) | constructor name and label | not used; would write `MonoDeque<Int>(pick: min)` |
| H02 | silent | yes (std-deque) | `std/README.md` 4.9 `Deque`: `@get fn last(self, k: Int` | truncated signature: is `last` a getter `?T` or `last(k)`? | not used |
| H03 | missing-stdlib | yes (std-seq) | `std/README.md` 4.2 "Monotone stack" block; `std/seq.cp` line 10 header | `prev_less` etc. documented, no definition in `std/*.cp` | hand-rolled the stack in 05 |
| H04 | silent | yes (R02.8) | `spec/core/02-values-types.md` 2.8; `std/seq.cp` (`last` only) | `List.push/pop/is_empty/min` signatures | `push(x)`, `pop() -> ?T`, `.last -> ?T`, `.is_empty` getter, `.min -> ?T` |
| H05 | silent | no (R10.4 absent) | `spec/lib/random-stress.md` Interactors; `spec/core/10-io.md` 10.4 | with `@!multitest`, who sends the test count to the program? | the harness forwards `t`; the interactor only does one test |
| H06 | silent | no | `spec/core/10-io.md` 10.4 | is `wire.ask` a `mut` method (is `let wire = Wire()` enough)? | `let` |
| H07 | silent | no | `spec/lib/random-stress.md` | interactor reading a token that is `!` or an Int | `sol.recv()` as `Str`, then `parse_int()!` |
| H08 | contradiction | yes (std-strings) | `spec/core/01-lexical-grammar.md` (`s.parse<Mod<P>>()`) vs `std/strings.cp` `parse<T>` = `todo` | is generic `parse` usable? | used `parse_int` instead |
| H09 | silent | yes (R02.8, R04.2) | `spec/core/02-values-types.md`, `04-indices.md` | table over a literal range `[0..n - 1] i => e`: allowed, and keys plain `Int`? | yes |
| H10 | silent | yes (std-sparse) | `std/sparse.cp` `fold/try_fold(r: Range<D>)` | does an `Int` range fit `Range<D>` for an array built over `0..n-1`? | yes |
| H11 | silent | yes (R04.2) | `spec/core/04-indices.md` | product array over literal ranges `[0..=K, 0..=K]?Int = None`, indexed with `Int` arithmetic (`best[a - k, b]`) | allowed; out-of-range traps |
| H12 | silent | yes (R01.8) | `spec/core/01-lexical-grammar.md` 1.5 | inline `if c: a else: b` inside a call argument / before `>>` — needs parentheses? | parenthesized `(if ...: a else: b) >> i & 3` |
| H13 | silent | yes (R01.4/R03) | `spec/core/01-lexical-grammar.md` 1.4 | value-position literal `1e15` vs typed bound; used `?Int` + `??` instead of a sentinel | n/a |

## Per-problem mapping

| # | Holes touched |
|---|---|
| 01 | H11 |
| 02 | — |
| 03 | — |
| 04 | — |
| 05 | H03, H04 |
| 06 | H04, H13 |
| 07 | H09, H10 |
| 08 | — |
| 09 | H05, H06, H07, H08, H12 |
| 10 | — |

## Revision test

| File | Change | Lines/chunks changed | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|
| 01-2075D-rev.cp | DP gains a dimension: make three numbers x, y, z equal (t <= 1000) | 4 chunks, 10 lines: table type + seed (2), loop header + one new update line (2), `read` (1), answer loop header + body (2), header comment (3) | the answer filter: chaining `if` clauses in one `for` (`a, b if ..., c if ...`) vs nesting; chose clauses, which hit B1 after running, so it became one trailing `if ... and ...` | the dimension list `0..=K` is repeated per axis (3x in the type, 3x in loops); no single "state shape" to edit | none |
| 06-2052J-rev.cp | return the end time of the last watched episode instead of the count | 1 line (`ans[k] = ...`) + header comment | whether to make `ans` `[q]?Int` and print with `none: 0`, or keep `Int` with an inline `if`; kept `Int` | none | none |

## After running

`cargo build -p cpl-cli` (debug) and `--release` both built; the runs below use the release `cpl`.
Each sample is the statement's; `NN.tests/1.in` (+ `1.out` where the answer is unique).

| File | check | run (sample) | test | build | g++ -O2 | C++ on sample | fuzz 200 vs brute | max input (C++) |
|---|---|---|---|---|---|---|---|---|
| 01-2075D | ok | ok | ok | ok | ok | ok | 200/200 | 0.27 s (t = 1e5) |
| 01-2075D-rev | ok after B1 workaround | ok (brute-checked, 30 inputs) | ok | ok | ok | ok (= interpreter) | 30/30 (3-number brute) | 0.09 s (t = 1e3) |
| 02-2053C | ok | ok | ok | ok | ok | ok | 200/200 | 0.02 s |
| 03-2060E | ok | ok | ok | ok | ok | ok | — | 0.03 s |
| 04-2070C | ok | ok | ok | ok | ok | ok | 200/200 | 0.07 s |
| 05-2046B | ok after R1 + B3 workaround | ok | ok | ok | ok | ok | 200/200 | 0.01 s |
| 06-2052J | ok | ok | ok | ok | ok | ok | 200/200 | 0.07 s |
| 06-2052J-rev | ok | ok | ok | ok | ok | ok | — | 0.07 s |
| 07-2050F | ok after B2 workaround | ok | ok | ok | ok | ok | 200/200 | 0.06 s |
| 08-2065E | ok | ok (checker) | ok | ok | ok | ok (checker) | 200/200 (checker + exhaustive -1) | 0.004 s |
| 09-2077B | ok | `cpl run` with `@interactor`: accepted | ok | ok | ok | `cpl run --release` with interactor: accepted | — | 0.86 s for t = 1e4 through `cpl run --release`, C++ compile included |
| 10-2075C | ok | ok | ok | ok | ok | ok | 200/200 | 0.01 s |

Fuzz: `fuzz/fuzz.py` (generators + brutes for 01, 02, 04, 05, 06, 07, 08, 10): 1600 inputs,
interpreter, C++ and brute agree on all. No crash or mismatch, so no sanitizer rerun was
needed. No max-size run took over 1 s.

Guesses confirmed by the runs: H04 (`push`, `pop() -> ?T`, `.last`, `.is_empty`), H05 (the
harness forwards `t`; the interactor does one test), H06 (`let wire` is enough), H07, H09,
H10, H11, H12.

### Failures

| ID | Class | File | Smallest reproduction | Workaround in the file |
|---|---|---|---|---|
| B1 | implementation bug | 01-2075D-rev | `for a in 0..3 if a > 0, b in 0..3:` + body: E0201 "no `b` in scope" (with a filter on `b` too: E0103). The grammar (`spec/core/01-lexical-grammar.md`, `for_stmt`/`clause`) gives every clause its own `('if' expr)?`; only a filter on the last clause parses | one trailing `if ... and ...` |
| B2 | implementation bug | 07-2050F | `repeat 1:` / `    let (l, r): (1..=5, l..=5) = read()`: E0201 "no `l` in scope". The same line at top level works (`spec/core/10-io.md` 10.2: a component may name an earlier one) | two `let`s |
| B3 | implementation bug | 05-2046B | `var xs: List<Int> = []` / `let m: ?Int = xs.min()`: R0001 "`min` of an empty sequence"; `xs.min() ?? -1` is E0309 (left side is `Int`). `std/seq.cp` `min -> ?T` and the 07 folds table ("`?T`, `None` when empty") say optional | `var low: ?Int = None; for x in out: low min= x` |
| R1 | reader error | 05-2046B | `out.min` without `()`: lint L0608; 07 spells it `s.min()` | `out.min()` |
| F01 | found-late (silent) | all | `cpl build d.cp` writes `d.cpp` beside the source and prints "wrote d.cpp (... bytes ...)" on stdout; `spec/tools/cli.md` says only "the C++ for the file's judge", so `cpl build d > d.cpp` gives a file g++ rejects | read the file it names |

Counts: 3 implementation bugs (B1, B2, B3), 1 reader error (R1), 1 late-found spec hole (F01);
0 wrong answers once the files compile.
