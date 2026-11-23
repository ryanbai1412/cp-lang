# Rebuild round 15: holes

Written at commit `dbe8e9d85`. I read only `spec/` (README, SUBSET, RATIONALE, GLOSSARY, diagnostics, `core/`, `lib/`, `tools/`; `spec/lib/strings.md` and one `spec/lints/*/explain.md` turned up in greps of `spec/`) and the stdlib stubs in `std/`. I did not open `spec/conformance/`, `plan/`, `corpus/` or `decisions.md`, and I did not run the compiler before this section above `## After running` was written. Statements came from codeforces.com through the r.jina.ai cache (Codeforces itself returned a Cloudflare challenge).

## Summary

| class | count | inside the subset |
|---|---|---|
| silent | 3 | 3 |
| contradiction | 2 | 2 |
| missing-stdlib | 1 | 1 |
| found-late | 8 | 8 |
| **total** | **14** | **14** |

## Top five risks

1. **H-STEP-RANGE**: `(1..=n by 2).rev` (03) leans on a stepped range being `Bidir`; the grammar names `by` but no stub defines the stepped range type, so 03 may not compile or may print in the wrong order.
2. **H-LETTER-TO-CHAR**: nothing says how a position of `domain Letter = 'a'..='z'` turns back into a `Char`; 05 guesses `Char(l)`.
3. **H-TALLY-TYPE**: the spec says `tally` returns `Counter<K>`, the stub says `HashMap<K, Int>`; the natural 05 (`s.tally()`) would hit whichever is wrong, so I avoided it.
4. **H-COUNTED-KEY-RANGE**: `pre.fold(l..=r)` and `good[j + 1]` with plain `Int` ranges on a `[n]T` table (02, 09) rely on counted arrays taking `Int` keys through `Prefix` as well as `[]`.
5. **H-CHAR-DOMAIN**: `Letter(c)` and `[Letter]Int` (05) are only shown in `lib/strings.md` examples and a lint explanation, not in the domain chapter.

## Silent

### H-LETTER-TO-CHAR
- Question: given `l` in `domain Letter = 'a'..='z'`, how do I get the `Char` back to push onto a `Str`?
- Readings: `Char(l)` (by analogy with `Int(v)`, R04.2.15); `l.value`; `Char(Int('a') + Int(l))` (if `Int(l)` is the ordinal). Searched `spec/` for `Char(` and `.value`: only `Char(i)` on integers (02 table, line 50).
- Chose: `Char(l)`.
- Cost if wrong: 05 does not compile; one-token fix once known.
- Subset: inside (strings, domains).

### H-WHILE-TRUE
- Question: is `while true:` with an inner `break` the idiom for a loop whose exit is in the middle (02), or is there a `loop`?
- Readings: `while true:`; a `loop` keyword. Searched 01 for `loop` and `while true`: the reserved-word list (01 line 105) has no `loop`, nothing shows `while true`.
- Chose: `while true:`.
- Cost if wrong: none for correctness; a lint at most.
- Subset: inside.

### H-AND-SHORT-CIRCUIT
- Question: does `and` short-circuit, so `r >= 0 and s[r] != 'R'` (02) never indexes `s[-1]`, and `j < t.len and (... t[j])` (01) never reads past `t`?
- Readings: short-circuit (Python/C++); both sides evaluated. Searched `spec/core/` for "short-circuit": no hit; the operator table in 01 lists `and`/`or` without evaluation order.
- Chose: short-circuit.
- Cost if wrong: 01 and 02 trap on every test in checked builds.
- Subset: inside.

## Contradiction

### H-TALLY-TYPE
- Question: what does `s.tally()` return?
- Readings: `spec/core/07-sequences.md` R07.11.4 says `Counter<K>`; `std/seq.cp` line 1808 declares `tally(...) -> HashMap<K, Int>`.
- Chose: neither; 05 counts into `[Letter]Int`, which the lint explanation recommends anyway.
- Cost if wrong: a `cnt[c]` on a missing key may trap (HashMap) or return 0 (Counter); a silent behaviour split.
- Subset: inside (sequences).

### H-SORT-DESC
- Question: do `sort` / `sorted` take `desc:`?
- Readings: `spec/core/07-sequences.md` R07.11 says they take no `desc:` (decision 366); `std/seq.cp` lines 225 and 273 declare `desc: Bool = false`.
- Chose: did not use `desc:`; 08 and 10 sort ascending with no arguments.
- Cost if wrong: a contestant writing `desc: true` gets a compile error or not depending on which side wins.
- Subset: inside.

## Missing-stdlib

### H-STEP-RANGE
- Question: what is `a..=b by k`, and does it have `.rev` (needs `Bidir`, `std/seq.cp` line 169) so that `(1..=n by 2).rev` is n, n-2, ..., 1?
- Readings: a stepped range that is `Indexed` like a range; a walk-only generator (no `.rev`); `.rev` starting from `b` rather than the last stepped value. The grammar has `by` (01 line 131, R01.8.4) but no stub in `std/` defines its type or methods.
- Chose: Bidir stepped range, `.rev` starting from the last value reached (n is odd, so the same here).
- Cost if wrong: 03 fails to compile; fallback is `[n - 2 * i for i in 0..(n + 1) / 2]`.
- Subset: inside (core ranges).

## Found-late

### H-PRINT-GENERATOR
- Question: can `print` take `chain(...)` (a generator) and print it space-separated (03)?
- First looked: R10.3.* printing rules. Found: `spec/core/10-io.md` §10.5 (line 160): any `Seq`, generators included, prints joined by `sep`.
- Chose: `print(chain(...))`. Cost if wrong: compile error. Subset: inside.

### H-MULTITEST-RETURN
- Question: how does a case end early under `@!multitest` (03, 06, 10)?
- First looked: 01 control flow and 06. Found: `spec/core/10-io.md` R10.1.2: top-level `return` ends the current case.
- Chose: `return`. Cost if wrong: the program would end after the first early case. Subset: inside.

### H-SHADOW-PRELUDE
- Question: may a local be called `count` (07) when the prelude exports `count`?
- First looked: 01 naming (R01.3). Found: `spec/core/11-modules-prelude.md` R11.2.3: a local shadows a prelude name. Still renamed `sum` to `total`/`score` elsewhere to avoid lint `shadows_top_level` (diagnostics L0203).
- Cost if wrong: compile error in 07. Subset: inside.

### H-VAR-WIDEN
- Question: does `var big = a.filter(...).max()!` (10) keep the element range `1..=1e9` and trap on `big += e`?
- First looked: 03 numbers and layout. Found: `spec/core/02-values-types.md` R02.10.2: an unannotated `var` widens to `Int`; `let` keeps the range.
- Cost if wrong: checked builds trap on the first sum past 1e9. Subset: inside.

### H-COUNTED-KEY-RANGE
- Question: does `[n]T` take `Int` keys and `Int` ranges, including through `Prefix.fold(l..=r)` and `(i..n).first_true(...)` (02, 09)?
- First looked: `std/prefix.cp` `fold(self, r: Range<D>)` (line 88), which only says `D`. Found: `spec/core/10-io.md` R10.2.2 and R04.4.30 (counted arrays take plain `Int` keys; `[n]T` is `[0..n]T`, 02 R02.8.1).
- Cost if wrong: type errors in 02 and 09. Subset: inside.

### H-CHAR-DOMAIN
- Question: is `domain Letter = 'a'..='z'` a domain, with `Letter(c)` to enter it and `for l in Letter` to walk it (05)?
- First looked: `spec/core/04-indices.md` R04.2 (domains are integer ranges there). Found: `spec/lib/strings.md` lines 120, 500 and `spec/lints/char_offset_index/explain.md`.
- Cost if wrong: 05 needs `Int(c) - Int('a')` offsets. Subset: inside.

### H-PREFIX-SUM-WIDTH
- Question: does `a.scan(+)` over `[n]1..=1e5` store `Int` sums, or values of `1..=1e5` that trap (02, 09)?
- First looked: `std/prefix.cp` scan (line 31: `[D.cuts]T`, so `T` is the element type). Found: `spec/core/03-numbers-layout.md` R03.1.7: a `Prefix` over range-typed elements accumulates `Int`.
- Cost if wrong: trap on the first prefix past the element bound. Subset: inside.

### H-TUPLE-RANGE-READ
- Question: may a tuple annotation carry range types, `let (n, x): (1..=1e6, 1..=1e6) = read()` (04, 06, 07, 08, 09), and is it validated per field?
- First looked: R10.2.1 examples (single values and `[n](u: V, ...)` records). Found: `spec/core/10-io.md` R10.2.3: tuples read fields in order, range types included. Also found there that `Range<T>` reads `l r` as `l..=r`, which 06 could have used; I kept two tuples because the intersection reads plainer with four ints.
- Cost if wrong: type errors in five files. Subset: inside.

## Per problem

| # | file | holes | subset |
|---|---|---|---|
| 1 | `01-slavics-exam.cp` | H-AND-SHORT-CIRCUIT | inside |
| 2 | `02-right-left-wrong.cp` | H-WHILE-TRUE, H-AND-SHORT-CIRCUIT, H-COUNTED-KEY-RANGE, H-PREFIX-SUM-WIDTH | inside |
| 3 | `03-generate-permutation.cp` | H-STEP-RANGE, H-PRINT-GENERATOR, H-MULTITEST-RETURN | inside |
| 4 | `04-black-circles.cp` | H-TUPLE-RANGE-READ | inside |
| 5 | `05-turtle-and-good-pairs.cp` | H-LETTER-TO-CHAR, H-CHAR-DOMAIN, H-TALLY-TYPE | inside |
| 6 | `06-game-with-doors.cp` | H-MULTITEST-RETURN, H-TUPLE-RANGE-READ | inside |
| 7 | `07-fun.cp` | H-SHADOW-PRELUDE, H-TUPLE-RANGE-READ | inside |
| 8 | `08-bouquet-easy.cp` | H-TUPLE-RANGE-READ, H-SORT-DESC | inside |
| 9 | `09-hungry-games.cp` | H-COUNTED-KEY-RANGE, H-PREFIX-SUM-WIDTH, H-TUPLE-RANGE-READ | inside |
| 10 | `10-parity-and-sum.cp` | H-VAR-WIDEN, H-MULTITEST-RETURN, H-SORT-DESC | inside |

## After running

`cargo build -p cpl-cli`, then `target/debug/cpl` `check`, `run` on the statement sample, and `build` (judge `generic`) compiled with `g++ -std=c++17 -O2` (GCC 11.4) and run on the same sample. Problems with several valid answers (01, 03, 05) were judged by validity: 01 by subsequence check, 05 by a brute-force good-pair count against the expected output. 1993B's sample is missing from the Codeforces cache, so it came from Luogu (CF1993B).

| # | check | `cpl run` on sample | C++ on sample |
|---|---|---|---|
| 01 | ok | ok (different valid answers: `xbxaa`, `ayyax`) | ok, same as `run` |
| 02 | ok | ok | ok |
| 03 | **E0501** | not run (same E0501) | not built |
| 04 | ok | ok | ok |
| 05 | **E0302** | not run (same E0302) | not built |
| 06 | ok | ok | ok |
| 07 | ok | ok | ok |
| 08 | ok | ok | ok |
| 09 | ok | ok | ok |
| 10 | ok | ok | ok |

8 of 10 pass as written; the committed files are the cold-read versions, unchanged.

### Failures

- **03, E0501, compiler.** `(1..=n by 2).rev` and `2..n by 2` as values: "`cpl run` and `cpl build` can't handle a stepped range kept as a value yet". The grammar allows them (01 R01.8.4), so the program is valid per the spec; this is an implementation gap, and also confirms H-STEP-RANGE (no stub for the stepped range). `cpl explain E0501` shows an unrelated `yield` example (diagnostics fault). Workaround `print(chain([n - 2 * i for i in 0..(n + 1) / 2], [2 * i for i in 1..=n / 2]))` passes `run` and C++.
- **05, E0302, spec.** "a `Letter` can't be turned into a `Char`" confirms H-LETTER-TO-CHAR: the spec gives no conversion. `l.char` and `l.value` are E0203. What works is `Char(Int(l))`, which passes `run` and C++ with optimal good-pair counts on every sample case.

### New findings

- **H-INT-OF-CHAR-POS** (silent, inside the subset): on `domain Letter = 'a'..='z'`, `Int(l)` is the character code (97 for `'a'`), not the ordinal; R04.2.15 only says "position to its integer" with an integer-domain example. The ordinal reading (`Char(Int('a') + Int(l))`) is wrong.
- **H-CHAR-OUT-OF-RANGE** (compiler): that wrong reading built `Char(194)` and `cpl run` printed raw bytes instead of trapping, though `Char` is ASCII `0..=127` (02 R02.2 table, line 50).
- **H-TALLY-TYPE** and **H-SORT-DESC** were not exercised; the other holes guessed right: every program that compiled agreed with the expected output in both `run` and C++.
