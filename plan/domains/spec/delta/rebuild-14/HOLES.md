# Rebuild round 14: holes

Written against commit `387d89061` (spec: a compound assignment evaluates its place once). For this round I read only `spec/` (README, SUBSET, RATIONALE scope, `core/`, `lib/`, `tools/`, diagnostics, glossary) and the stdlib stubs in `std/`. I did not open `spec/conformance/`, `corpus/`, `decisions.md`, or anything under `plan/` other than this folder. The compiler was not built or run until this section was written. Problem statements and samples come from codeforces.com (read in a browser).

| class | count |
|---|---|
| silent | 5 |
| contradiction | 0 |
| missing-stdlib | 2 |
| found-late | 14 |
| **total** | **21** |
| inside the Milestone 1 subset | 17 |

## Top five risks

1. **H-PRINT-STR** (02): if `print(s)` on a `Str` prints its characters joined by `io.sep` (R10.3.1 treats a sequence that way) instead of printing one word, every case of 02 gets WA.
2. **H-GRID-KEYS-GAP** (01): `read<[R, C]Char>()` over a product domain is idiomatic (R10.2.3, R04.2.9), but SUBSET.md lists `product-domain-grid-reads-and-indexes` and `grid-keys-are-cells` as compiler gaps. 01 may not compile or run.
3. **H-DESTRUCT-PLACES** (07): `(p[l], p[r]) = (p[r], p[l])` may fall under the aliasing rule that rejects `swap(mut a[i], mut a[j])` (05-ownership.md:336), so 07 might not compile.
4. **H-MINMAX2** (10): two-argument `min` is described as a builtin (R11.3.3), but `std/` has no stub for it, only the sequence fold. If it does not resolve, 10 does not compile.
5. **H-COMP-MULTI-CLAUSE** (01): a comprehension with two `for` clauses (`[Int(r) for r in R, c in C if ...]`) is my extrapolation from the multi-clause loop header. A wrong guess here is a compile error in 01.

## Holes by class

### silent

- **H-PRINT-STR**: does `print(s)` with `s: Str` print the word, or the characters separated by `io.sep`?
  Readings: (a) Str is `Show` and prints as a word; (b) Str is a sequence (R02.3.1), and R10.3.1 says `print(xs)` prints a sequence's elements joined by `io.sep`. I searched `spec/` (10-io, 02 §2.3, 07, lib/strings.md) and `std/` (no `impl Show for Str`). Nothing says.
  Chose (a), `print(s)`. Cost if wrong: WA on every case of 02 (fix: `print(s, sep: "")`). Inside the subset (R10.3).
- **H-RANGE-ELEM-INDEX**: can an element of `[m]1..=n` (a range-typed value, not a domain position) take part in `i - 1` and then index a `Str`?
  Readings: (a) range types are integers with a contract (R02.10.2), so arithmetic gives `Int`, which indexes `0..len`; (b) a range-typed value is a position-like key that needs `Int(i)`. R04.3.3 only covers positions.
  Chose (a), `s[i - 1]`. Cost if wrong: a compile error in 02. Inside the subset.
- **H-DESTRUCT-PLACES**: does the destructuring assignment `(a, b) = (b, a)` (01-lexical-grammar.md:235) accept two element places of the same array, `(p[l], p[r]) = (p[r], p[l])`?
  Readings: (a) yes, the right side is evaluated first (R01 line 204, 05-ownership R11), so there is no alias; (b) it is rejected like `swap(mut a[i], mut a[j])` (05-ownership.md:336) unless `l != r` is proven.
  Chose (a). Cost if wrong: a compile error in 07. Inside the subset.
- **H-COMP-MULTI-CLAUSE**: may a comprehension take several `for` clauses, as a loop header does (`for i in V, j in V.after(i) if ok`, 01 syntax tour)?
  Readings: (a) yes, a comprehension's clauses are the loop header's; (b) only one `for` per comprehension, so the code needs `g.keys`. No example in `spec/` has two.
  Chose (a), to avoid `g.keys` (see H-GRID-KEYS-GAP). Cost if wrong: a compile error in 01. Inside the subset.
- **H-LIST-OF-ARRAYS**: can a list literal hold three arrays (`[read<[n]..>(), ...]`, which gives `List<[n]T>`) with chained indexing `v[p][i]`?
  Readings: (a) yes, arrays are ordinary heap values (R02) and move into the list; (b) arrays of a fresh counted domain cannot be element types. No `List<[` appears anywhere in `spec/`.
  Chose (a). The element order is fixed by left-to-right evaluation (01 line 204). Cost if wrong: a compile error in 04 (fix: three named arrays and a `match`). Inside the subset.

### contradiction

None found.

### missing-stdlib

- **H-MINMAX2**: two-argument `min(a, b)`. R11.3.3 (11-modules-prelude.md:80) says "`min` and `max` stay the builtins over two or more values", and std/seq.cp:56 calls it "the core binary `min`", but no stub exists in `std/`: only the fold `min(self: S, by:)` at std/seq.cp:64.
  Readings: a core builtin outside `std/`, or the fold reached through some overload. Chose the builtin. Cost if wrong: a compile error in 10. Inside the subset.
- **H-SWAP**: swapping two array elements. `a.swap_at(i, j)` is named in 05-ownership.md:762 and spec/lib/rationale/seq.md:176, and `swap(mut a[i], mut a[j])` in 05-ownership.md:336, but `std/` has neither for arrays or lists (only private `swap`/`swap_at` in std/heap.cp).
  Readings: use `swap_at` anyway, or use destructuring. Chose destructuring (see H-DESTRUCT-PLACES). Cost if wrong: none for my code, but a contestant who copies `swap_at` from the spec gets an unresolved name. Inside the subset.

### found-late

- **H-LAZY-TWICE**: may a `map` view be walked twice (once for `min`, once for `max`, or once for `sum` and once for `print`)? I looked in the R07.7 adapters table and found the answer in 07-sequences.md, the paragraph after R07.8.1 ("Lazy chains redo their work on every consumption … Lint flags a lazy chain walked twice").
  Chose comprehensions (`[l / x for x in k]`) in 01 and 05. Cost if wrong: only a lint warning. Inside the subset.
- **H-STR-LEN-GAP**: may I read `Str(len: n..=n)` with a runtime bound? R02.3.2 says yes, but SUBSET.md lists `core-02/str-len-runtime-bound` as a compiler gap.
  Chose plain `read<Str>()`. Cost: I lose the length check. Not inside the subset as a feature; the chosen code is inside.
- **H-LET-ELSE-GAP**: may 06 use a top-level `let i = opt else: ... return` under `@!multitest`? I looked at the let-else rule in 01-lexical-grammar (R01 §7); SUBSET.md lists `core-01/let-else-some-pattern-top-level` and `let-else-falls-through` as gaps.
  Chose `if let … else:`. Not inside the subset as a feature; the chosen code is inside.
- **H-WIDEN-READ**: can I write `var x: Int = read<1..2 ** 30>()`, so the value is validated on read and then becomes a plain `Int` that may leave the range (`x += 1` can reach 2^30)? I looked at R02.10.2; SUBSET.md lists `core-03/annotation-widens-fresh-read` as a gap, and a range-typed `var` would trap when it leaves the range (R03).
  Chose `var x = read<Int>()`. Cost: I lose the input check. Not inside the subset as a feature.
- **H-GRID-KEYS-GAP**: does `g.keys` of `[R, C]Char` give `(r, c)` cells, and does a product-domain grid read work? I looked at R04.2.10 (yes, keys are cells, and `g[p]` is `g[r, c]`) and R10.2.3 (reads one line per row). SUBSET.md lists `core-04/grid-keys-are-cells`, `product-domain-grid-reads-and-indexes` and `inline-product-domain-is-seq` as compiler gaps.
  Chose to keep `read<[R, C]Char>()` (there is no other idiomatic grid read) and to walk `R` and `C` directly instead of `g.keys`. **Outside the subset**: 01 depends on a listed gap.
- **H-STR-INDEXMUT**: is `s[i] = c` allowed on a `Str`? I looked in 02 §2.3 Strings, which mentions only `push` and `+`. The answer is in 07-sequences.md:89 (R07.3.6): `Str` is `IndexedMut`.
  Inside the subset.
- **H-COUNTED-INT-KEYS**: does `a[i]` with a plain `Int` `i` work on `read<[n]T>()`? I looked in 04-indices (counted domains); the answer is in 10-io.md R10.2.2: "A counted array takes plain `Int` keys (`a[i - m]`, R04.4.30)". Used in 04, 06, 09 and 10. Inside the subset.
- **H-REV-GETTER**: is it `.rev` or `.rev()`? I looked in the R07.7 adapters; the answer is in std/seq.cp:169 (`@get pub fn rev`) and 07-sequences.md:13 (`.rev`). Chose `(0..60).rev`, iterated only (SUBSET.md also lists `rev-view-is-not-indexable` as a gap, which I avoid). Inside the subset.
- **H-PERM-NO-IMPORT**: `permutations` is not in std/prelude.cp, so does it need a `use`? I looked in the prelude; the answer is R11.3.4: "A method call needs no import". Chose `[0, 1, 2].permutations()`. Inside the subset (SUBSET.md lists the `core-06/gen-perm` gap, which is about generators, not this method).
- **H-CEIL-DIV-FORM**: should I write `ceil_div(a, b)` or `a.ceil_div(b)`? lib/math.md:387 writes the free form; std/math.cp:23 declares `self: Int`, which makes it a method (R11.3.4). Chose `v[0].sum().ceil_div(3)`. Cost if wrong: a compile error. Inside the subset.
- **H-SORTED-REBASE**: is `w.sorted()` on a window (a slice that keeps the source's keys, R07.8) keyed from 0, so that `[1]` is the median of three? I first looked in 07-sequences (views keep keys); the answer is the std/seq.cp:273 body, which builds a fresh `List`. Inside the subset.
- **H-BIT-METHOD**: how do I test bit `b` of an `Int`? I looked in lib/math.md (`bit_len`, no `bit`); the answer is in std/math.cp:468, `n.bit(b)`. Inside the subset.
- **H-LOOP-ELSE**: does `for` take an `else:` that runs when the loop did not `break`, including a loop over a library generator? I looked in 06-control-functions; the answer is R01.7.2 and R01.7.4 (01-lexical-grammar.md:278-285). Used in 04. Inside the subset.
- **H-IF-IN-COMPREHENSION**: is `[if c: x else: y for i in 0..k]` legal? I looked at R01.8.2 (comprehension forms); the answer is in spec/lib/rationale/dp.md:267 and spec/lib/rationale/graphs.md:470. Used in 09. Inside the subset.

## Per problem

| # | problem | holes | subset |
|---|---|---|---|
| 01 | 1985D Manhattan Circle | H-GRID-KEYS-GAP, H-COMP-MULTI-CLAUSE, H-LAZY-TWICE | outside (product-domain grid read is a listed gap) |
| 02 | 1986C Update Queries | H-PRINT-STR, H-RANGE-ELEM-INDEX, H-STR-INDEXMUT, H-STR-LEN-GAP | inside |
| 03 | 1988C Increasing Sequence with Fixed OR | H-REV-GETTER, H-BIT-METHOD | inside |
| 04 | 1983C Have Your Cake and Eat It Too | H-LIST-OF-ARRAYS, H-PERM-NO-IMPORT, H-CEIL-DIV-FORM, H-LOOP-ELSE, H-COUNTED-INT-KEYS | inside |
| 05 | 1979C Earning on Bets | H-LAZY-TWICE | inside |
| 06 | 1980D GCD-sequence | H-LET-ELSE-GAP, H-COUNTED-INT-KEYS | inside |
| 07 | 1978C Manhattan Permutations | H-DESTRUCT-PLACES, H-SWAP | inside |
| 08 | 1977B Binary Colouring | H-WIDEN-READ | inside |
| 09 | 1976C Job Interview | H-IF-IN-COMPREHENSION, H-COUNTED-INT-KEYS | inside |
| 10 | 1975C Chamo and Mocha's Array | H-MINMAX2, H-SORTED-REBASE, H-COUNTED-INT-KEYS | inside |

## After running

I built the compiler with `cargo build -p cpl-cli` and used `target/debug/cpl` at `387d89061`. For each file I ran `cpl check <file>` and `cpl run <file> < sample`. Where the problem accepts several answers (03, 04, 05, 07, 08), I checked the output with a token-based checker against the statement's conditions rather than comparing it to the sample text. The solution files above are unchanged from what I wrote before the run.

| # | check | sample | notes |
|---|---|---|---|
| 01 | no errors | matches | |
| 02 | no errors | matches | |
| 03 | no errors | valid (different sequence, same lengths) | `14 -> 6 10 12 14` is a valid answer |
| 04 | no errors | valid tokens; line layout differs | see A-PRINT-LIST-OF-TUPLES |
| 05 | **1 error** (E0203) | **fails** (does not run) | see A-FOLD-ON-ARRAY |
| 06 | no errors | matches | |
| 07 | no errors | valid (different permutations) | |
| 08 | no errors | valid (31 digits, padded with zeros) | |
| 09 | no errors | matches | |
| 10 | no errors | matches | |

New findings:

- **A-FOLD-ON-ARRAY** (05): `k.fold(lcm, init: 1)` on a `[n]2..=20` array fails with `error[E0203]: a [Int]Int has no method fold that cpl run knows`. std/seq.cp:1728 declares `pub fn fold<S: Seq, T, Acc>(self: S, f: (Acc, T) -> Acc, init: Acc) -> Acc`, so **the compiler is wrong**. Two rewrites, tried outside the committed files, pass the sample: `k.reduce(lcm)!`, and an explicit loop `for x in k: l = lcm(l, x)`.
- **A-PRINT-LIST-OF-TUPLES** (04): `print(slice)`, where `slice` is a `List<(Int, Int)>`, prints one pair per line. R10.3.5 says `sep:` joins the elements and its example `print(ps, sep: "\n")` implies the default `io.sep` keeps them on one line. Judges read tokens, so the answer is still accepted, but **the compiler disagrees with the spec**: R10.3.6, which puts one row per line for `List<List<T>>`, does not cover tuples. `print(slice, sep: " ")` prints the expected single line.
- The run settled these holes as the readings I chose: H-PRINT-STR (a `Str` prints as a word), H-RANGE-ELEM-INDEX, H-DESTRUCT-PLACES, H-COMP-MULTI-CLAUSE, H-LIST-OF-ARRAYS, H-MINMAX2 (binary `min` resolves although `std/` has no stub), H-CEIL-DIV-FORM, H-PERM-NO-IMPORT, H-LOOP-ELSE, H-IF-IN-COMPREHENSION, H-REV-GETTER and H-BIT-METHOD.
- H-GRID-KEYS-GAP: the product-domain grid read `read<[R, C]Char>()` worked on 01's sample despite the gaps listed in SUBSET.md. I did not test `g.keys`.

Who was wrong, per failure: 05, the compiler (the std stub exists). 04's layout, the compiler against R10.3.5. The spec and std were not wrong in any failure I hit.
