# Rebuild round 16: holes

Cold rebuild at commit `d6a144417` (`spec: add a reject case for loop labels`). I read only
`spec/` (README, SUBSET, RATIONALE, GLOSSARY, diagnostics, `core/`, `lib/`, `tools/`) and the
stubs in `std/`. I did not open `spec/conformance/`, `plan/`, `corpus/` or `decisions.md`, and
I did not run the compiler before this section was written. Codeforces was behind a Cloudflare
check, so I read the statements and samples on the Luogu mirror (CF1985F and the others).
Every program stays inside the Milestone 1 subset (`spec/SUBSET.md`). No problem forced me
outside it.

## Summary

| Class | Count | Inside the subset |
|---|---|---|
| silent | 6 | 6 |
| contradiction | 7 | 7 |
| missing-stdlib | 1 | 1 |
| found-late | 6 | 6 |
| **total** | **20** | **20** |

## Top five risks

1. **H-PREFIX-2D / H-PREFIX-CTOR**: the 2-D prefix sum has no spec text. `Prefix(sign)` on a
   `[R, C]Int` rests on one line of `std/README.md`, and the spec gives `Prefix` two constructor
   spellings. If either guess is wrong, 06 does not compile.
2. **H-GRID-INT-READ**: `read<[R, C]Int>()` is only described for `Char`. 04 and 06 assume
   `R * C` tokens in row-major order. Any other reading gives wrong answers with no error.
3. **H-LIST-METHODS**: `List<T>` has no stub at all (`push`, `len`, `is_empty`, `.rev`), though
   every chapter uses them. Five of the ten programs depend on them.
4. **H-VARIANT-TUPLE**: it is unclear whether bare variant names resolve inside a tuple through
   a `(Int, Covered)` annotation and `match` arms. If not, 10 needs `Covered.` on every variant.
5. **H-TEMP-VIEW**: 02 walks the `runs` views of a temporary `a.sorted(...)`. The ownership
   chapter does not say how long a temporary lives when a lent view of it drives a `for`.

## Holes by class

### Silent

- **H-GRID-INT-READ**: does `read<[R, C]T>()` for a non-`Char` `T` read `R.count * C.count`
  tokens in row-major order?
  - Spec: R10.2.3 describes only `[R, C]Char` (one line per row). R04.2.9 says a product domain
    walks row-major. Searched `spec/` for "row-major", "grid", "[R, C]".
  - Readings: (a) row-major tokens, whitespace-free of lines; (b) one line per row, checked.
    Chose (a).
  - Cost if wrong: silent wrong answers in 04 and 06 (b would still parse the same samples).
  - Subset: inside (R10.2).
- **H-VARIANT-TUPLE**: does the expected type reach bare variant names inside a tuple and the
  arms of a `match` expression (`let (cost, next): (Int, Covered) = match cov: ... => (1, Left)`)?
  - Spec: R02.7.1 says "where the expected type is the enum". R02.10.1 says siblings unify, but
    neither says the annotation flows through the tuple and the arms.
  - Readings: (a) it flows; (b) only direct positions, so write `Covered.Left`. Chose (a).
  - Cost if wrong: a compile error in 10, fixed in one edit.
  - Subset: inside (R02.7, R02.10).
- **H-TEMP-VIEW**: can a `for` walk views lent from a temporary, as in
  `for run in a.sorted(by: ...).runs(by: ...)`?
  - Spec: R07.5.10 says a lent value lives until its last use in the block that made it. Chapter
    5 has no word on unnamed temporaries. Searched for "temporar", "full expression", "unnamed".
  - Readings: (a) the temporary lives for the whole statement, loop included; (b) bind
    `let s = a.sorted(...)` first. Chose (a).
  - Cost if wrong: a borrow error in 02, one added line.
  - Subset: inside (R05.7, R07.5).
- **H-JOINT-BOUND**: how do I state a joint constraint such as `n * m <= 2e5` so that
  `domain Val = 1..=n * m` gets 4-byte positions?
  - Spec: R03.1.1 takes widths from the static bound of the count, and the static bound of
    `n * m` with `n, m: 1..=2e5` is `4e10`. No form for a joint bound or an `assert`-derived
    bound is described.
  - Readings: (a) accept 8-byte positions; (b) `assert n * m <= 2e5` narrows it. Chose (a).
  - Cost if wrong: memory only (2x on `at` in 04), never correctness.
  - Subset: inside (R03.1).
- **H-MINUS-ONE-UNION**: how do I type a token that is `-1` or `1..=1e8` (1981C)?
  - Spec: R02.2.2 has literal unions of literals only, and R01.10 `type` has
    `literal ('|' literal)*` or `range_type`, not a mix.
  - Readings: (a) `-1..=1e8` and test `> 0`; (b) `-1 | 1..=1e8`; (c) read `Int`. Chose (a).
  - Cost if wrong: none for correctness. With (a), a `0` input passes validation.
  - Subset: inside (R02.2).
- **H-GUARD-EXHAUSTIVE**: does `match` exhaustiveness ignore guarded arms (`_ if x == 0 =>`)
  and count only the unguarded `Nothing`, `Left`, `Right`?
  - Spec: R01.7.3 allows guards and requires exhaustiveness, and says nothing about how the two
    combine.
  - Readings: (a) guarded arms never count, as in Rust; (b) a guarded `_` counts. Chose (a),
    with all three variants listed unguarded.
  - Cost if wrong: none either way for 10.
  - Subset: inside (R01.7).

### Contradiction

- **H-SORT-DESC**: does `sort`/`sorted` take `desc:`?
  - `spec/core/07-sequences.md` R07.11 ("Sorts are stable ... `sort` and `sorted` take no
    `desc:` argument (decision 366)") versus `std/seq.cp:225` and `:273`
    (`desc: Bool = false`, with the example `sorted(desc: true)`).
  - Readings: the flag exists, or it doesn't. Chose the spec (no flag; I didn't need descending).
  - Cost if wrong: none here. A reader who copies the std example gets a rejected flag.
  - Subset: inside (R07.11).
- **H-TALLY-TYPE**: what do `tally` and `group` return?
  - R07.11.3 and R07.11.4 say `Map<K, List<T>>` and `Counter<K>`. `std/seq.cp:1808` and `:1823`
    say `HashMap<K, Int>` and `HashMap<K, List<U>>`.
  - I avoided both in 09 (a counting array instead), because the iteration order of a
    `HashMap` would matter there.
  - Cost if wrong: order-dependent code is wrong.
  - Subset: inside (R07.11).
- **H-FOLD-OP**: what is the fold of an operation value?
  - R07.9.2 says `s.fold(op)` is `T` when `op` has an identity, and gives `s.fold(f, init: a)`.
  - `std/seq.cp:1728` has only `fold(self, f, init)` with `init` required.
    `std/algebra.cp:151` has the free `fold(m, xs)`, operation first.
  - Readings: `windows.fold(gcd)`, `fold(gcd, windows)`, or a loop. Chose the loop
    `g = gcd(g, ...)` in 06.
  - Cost if wrong: a compile error on either fold spelling.
  - Subset: inside (R07.9).
- **H-PREFIX-CTOR**: how is a prefix sum built?
  - `spec/core/03-numbers-layout.md` (the "Ops decide what a structure stores" table) writes
    `Prefix(+, a)`. `spec/lib/ops-ranges.md:401` and `spec/core/07-sequences.md` ("Scans are
    prefix sums over cuts") write `Prefix(a)` (`+` only). `std/prefix.cp:112` has `new(a)`.
  - Chose `Prefix(a)`.
  - Cost if wrong: a compile error in 06.
  - Subset: inside (std-prefix, R03.1).
- **H-INF-CONST**: is `inf` a legal name for a constant?
  - `std/math.cp:1923` declares `pub const inf: Int`. R01.3.2 (decision 363): "a lowercase
    `const` is the same error". `spec/lib/search-dp.md` also uses `inf` as the `Int` table fill.
  - I considered `inf` as the upper end of the search in 01 and used the statement's bound
    (`4e10`) instead.
  - Cost if wrong: none here.
  - Subset: inside (R01.3).
- **H-README-BSEARCH**: what is the binary search API?
  - `std/README.md` §4.2 documents `bsearch_first`/`bsearch_last` returning `none`. R04.8.3
    says `first_true`/`last_true` replaced them, and the glossary bans `none`.
  - Chose `first_true` (R04.4.19). `std/seq.cp:1063` agrees.
  - Cost if wrong: none. A reader of the README writes dead names.
  - Subset: inside (R04.8).
- **H-CLONE-COPY**: how is a heap value copied?
  - R02.1.2 (decision 128) says heap values are copied only by `.clone()`. `std/seq.cp:1227`
    (`inversions`) copies a `List` with `cur.copy()`, which no stub declares.
  - Chose `.clone()` for my own code (none needed it in the end).
  - Cost if wrong: none for 07, which calls `inversions` as a black box.
  - Subset: inside (R02.1).

### Missing-stdlib

- **H-LIST-METHODS**: where are `List<T>`'s own members (`List<T>()`, `push`, `pop`, `len`,
  `is_empty`, `.rev`, `+`) declared?
  - `std/` has no `struct List` stub. `std/README.md` §3 does not list them as intrinsics.
    `is_empty` exists only on `Slice` (`std/seq.cp:1408`) and library structs, and `rev` only as
    a generic `Bidir` getter (`std/seq.cp:169`).
  - Used in 02 (`len`), 05 (`push`, `is_empty`, `len`, `.rev`) and 09 (`len`), and assumed with
    their spec meaning.
  - Cost if wrong: a compile error in each, or a silently O(n) `len`.
  - Subset: inside (R02.8, R07.2).

### Found-late

- **H-ABS**: is there a free `abs`?
  - I looked in `spec/core/02-values-types.md` §2.2, `std/prelude.cp` and `std/math.cp`
    (whose `gcd` calls `abs` but never declares it). Chapters 1 and 6 use `abs(...)` in examples.
  - Found in `std/README.md` §3 ("Core features the stdlib relies on"), in the intrinsics list.
  - Used in 08.
  - Subset: inside.
- **H-MIN-BINARY**: is `min(a, b)` of two values a core builtin, given that `min` in
  `std/seq.cp` is a fold?
  - I looked in `std/seq.cp:64` (whose doc says "the core binary `min`") and
    `spec/core/11-modules-prelude.md` R11.3.3 ("stay the builtins").
  - Found as an intrinsic in `std/README.md` §3. No core chapter declares it.
  - Used in 01 and 08.
  - Subset: inside.
- **H-MOD-POW**: does `**` work on `Mod<P>`?
  - I looked in `spec/core/01-lexical-grammar.md` R01.5.8, which describes `**` on `Int` only,
    and in `spec/lib/math.md`.
  - Found in `std/mod.cp:142` (`impl Pow for Mod<P>`).
  - Used in 03.
  - Subset: inside.
- **H-PREFIX-2D**: how do I get rectangle sums?
  - I looked in `spec/lib/ops-ranges.md` (the static `Prefix(a)` row, 1-D `fold(r)`), in
    `spec/core/07-sequences.md` ("Scans are prefix sums over cuts") and in `std/prefix.cp`.
  - Found in `std/README.md:1025` ("prelude shape dispatch over `[D]T`, `[R, C]T`, and
    `List<T>`") and `std/prefix.cp:301` (`_Prefix2.new(a: [R, C]T) -> Prefix<R, C, T>`,
    `fold(rows, cols)`). No spec chapter says that one name covers both arities, and R06.1.6
    forbids overloading.
  - Used in 06.
  - Subset: inside (std-prefix).
- **H-IF-LET-CHAIN**: how do I bind two optionals in one `if let`?
  - I looked in R01.7.1, whose example is `if let v = m.get(k), v > 0` (a binder, then a
    condition).
  - Found in the grammar of §1.10 (`cond := 'let' pattern '=' expr (',' cond)*`): each binder
    repeats `let`.
  - Used in 06.
  - Subset: inside.
- **H-LAMBDA-INFER**: does a `let`-bound lambda infer its parameter type from later calls
  (`let step = j => ...`, then `step(i)`)?
  - I looked in `spec/core/06-control-functions.md` §6.2.
  - Found in R02.10.1: "Inference never looks at later statements", so I annotated
    `(j: Int) =>`.
  - Used in 02.
  - Subset: inside.

## Per problem

| # | Problem | Holes | Subset |
|---|---|---|---|
| 01 | 1985F Final Boss | H-MIN-BINARY, H-README-BSEARCH, H-INF-CONST | inside |
| 02 | 1986E Beautiful Array | H-SORT-DESC, H-TEMP-VIEW, H-LAMBDA-INFER, H-LIST-METHODS | inside |
| 03 | 1985G D-Function | H-MOD-POW | inside |
| 04 | 1980E Permutation of Rows and Columns | H-GRID-INT-READ, H-JOINT-BOUND | inside |
| 05 | 1981C Turtle and an Incomplete Sequence | H-MINUS-ONE-UNION, H-LIST-METHODS | inside |
| 06 | 1982D Beauty of the Mountains | H-GRID-INT-READ, H-PREFIX-2D, H-PREFIX-CTOR, H-FOLD-OP, H-IF-LET-CHAIN | inside |
| 07 | 1983D Swap Dilemma | H-SORT-DESC, H-CLONE-COPY | inside |
| 08 | 1984C1 Magnitude (Easy) | H-ABS, H-MIN-BINARY | inside |
| 09 | 1987D World is Mine | H-TALLY-TYPE, H-LIST-METHODS | inside |
| 10 | 1990D Grid Puzzle | H-VARIANT-TUPLE, H-GUARD-EXHAUSTIVE | inside |

## After running

I built `cargo build -p cpl-cli` (debug `target/debug/cpl`) at `d6a144417` and ran each file on its
statement's sample. `cpl build` used the default `generic` judge, and the C++ was compiled with
`g++ -std=c++17 -O2`. The samples are exact matches except 05, where any valid answer is accepted.
I checked 05 with a small validator: fixed entries kept, the halving rule holds, values are in
`1..=1e9`, and the `-1` cases match.

| # | `cpl check` | `cpl run` sample | C++ (`cpl build` + g++) | Who was wrong |
|---|---|---|---|---|
| 01 | ok | match | match | none |
| 02 | ok | match | **wrong output** (garbage numbers; ASan: heap-use-after-free) | compiler |
| 03 | ok | match | match | none |
| 04 | **E0202** as first written; ok after a rename | match | match | spec vs compiler |
| 05 | ok | valid (differs from the sample text, which the statement allows) | valid, same as `cpl run` | none |
| 06 | **2 root errors** as first written; ok after one edit | match | match | spec (silent) |
| 07 | ok | match | match | none |
| 08 | ok | match | match | none |
| 09 | ok | match | match | none |
| 10 | ok | match | match | none |

### New findings

- **R16-TEMP-VIEW-UAF (02, compiler)**: this confirms H-TEMP-VIEW.
  - `for run in a.sorted(by: ...).runs(by: ...)` checks and runs correctly in the interpreter.
  - The emitted C++ frees the temporary `sorted` list before the loop walks the `runs` views of
    it. The output is garbage, and `-fsanitize=address` reports a heap-use-after-free.
  - Binding the temporary first (`let s = a.sorted(...)`, then `for run in s.runs(...)`) makes
    the C++ match.
  - The interpreter and the C++ disagree on a program the checker accepts. That breaks goal 5
    ("the interpreter and the C++ agree on every program") and gives UB with no `@trusted`.
  - Either the lowering must keep the temporary alive for the loop, or the checker must reject
    the program (E0601-style, "a view of a temporary outlives it"). The spec should then state
    the rule in chapter 5. I left 02 as written so the bug reproduces.
- **R16-SHADOW-UNWRAP (04, spec vs compiler)**:
  - `var at: [Val]?(R, C) = None` ... `let at = at!` is E0202 ("`at` is already defined in this
    file ... a top-level name has one definition"). The per-test region below `@!multitest`
    counts as top level.
  - R02.4.8 teaches exactly this shadowing ("This ends a fill phase: `let depth = depth!`"), and
    R01.9.1 says the per-test region is a function body.
  - Either the spec example needs "in a fn body only" plus a spelling for top level, or the
    checker should allow the shadow below the divider.
  - Fixed by renaming the `var` to `spot`.
- **R16-TABLE-TUPLE-PARAM (06, spec silent, compiler strict)**:
  - `[R, C] p => ...` (one parameter that takes the cell as a tuple) is E0303: "this table runs
    over 2 domains but names 1 positions".
  - R01.8.2 says `[(A, B)] (a, b) => e` and `[A, B]` are the same table. R04.2.10 and R04.2.11
    say a position of a product domain is a tuple and `[P] p => f(p)` builds it. R06.1.4 lets a
    lambda that wants the whole tuple take one parameter.
  - So the one-parameter form looked legal. The compiler wants `(r, c) =>`.
  - Fixed to `[R, C] (r, c) => ...`. Either the spec should say a 2-D table needs one name per
    factor, or the checker should accept a single cell parameter as R06.1.4 suggests.
- **H-PREFIX-2D resolved (06, found-late confirmed)**:
  - `Prefix(sign)` on `[R, C]Int` with `pre.fold(rows, cols)` works. The two `fold` errors in the
    first check were a cascade of the table error above.
  - The 2-D prefix exists, but only `std/README.md` says so (see H-PREFIX-2D).
- **Holes that did not bite**: H-GRID-INT-READ (row-major tokens, as guessed), H-VARIANT-TUPLE
  (bare variants through the tuple annotation), H-LIST-METHODS, H-MOD-POW, H-ABS,
  H-MIN-BINARY and H-IF-LET-CHAIN all compiled and ran as guessed.
- **Size**: every C++ file is under 32 KB, from 5.4 KB (08) to 16.4 KB (02).
