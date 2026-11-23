# Rebuild round 12: holes

I read commit `4f6b4ab58` of `ryanbai1412/cp-lang`, and only `spec/` (README, SUBSET, RATIONALE, `core/`, `lib/`) and the root `std/` stubs. I did not open `spec/conformance/`, `plan/`, `corpus/` or `decisions.md`, and I did not run the compiler. The ten solutions are written from that text alone. Every guess I made is recorded below, including the ones where the intent seemed clear.

| Class | Count |
|---|---|
| silent | 11 |
| contradiction | 2 |
| missing-stdlib | 3 |
| **total** | **16** |
| inside the Milestone 1 subset | 16 |

## Top risks

1. H-SHORT-CIRCUIT: if `or` does not short-circuit, 03 reads `a[-1]` and `a[n]` and traps.
2. H-LOOP-ELSE-LABEL: if a `break 'label` inside an inner loop's `else` is invalid, or the `else` binds differently, 08 prints nothing or prints every k.
3. H-STR-SLICE-CONCAT: if `Str` slices cannot be joined with `+` into a printable `Str`, 06 fails.
4. H-ADJACENT-TUPLE-LAMBDA: if `(x, y) => e` is a two-argument lambda and not a pair pattern, 01 does not type-check.
5. H-ARRAY-FILL: if `var a: [n + 1]Int = 0` does not fill a runtime-sized array, four solutions break (02, 04, 07, 08).

## Holes

### contradiction

**H-TALLY-TYPE**
- Question: what does `s.tally()` return?
- Readings: (a) `Counter<K>`, per `spec/core/07-sequences.md` line 344 (R07.11.4); (b) `HashMap<K, Int>`, per `std/seq.cp:1807`.
- Chosen: neither. 09 and 10 build a `Counter` directly with the `std/hashmap.cp` API.
- Cost of a wrong guess: a type error when a missing key is read (`Counter` reads 0, a map read traps), or a missing-key trap.
- Subset: inside (R07.11 supported, std-hashmap supported).

**H-RANGE-ENDS**
- Question: what are the names of a range's ends and pieces?
- Readings: (a) `r.l`/`r.r`, because `spec/core/04-indices.md` line 362 (R04.8) says "Rejected: `.start`/`.end`/`.before`/`.after`"; (b) `.start`/`.end`, as `std/prefix.cp` lines 91-94 and `Diff.add` write `r.start`/`r.end`. In addition, `04-indices.md` line 41 (R04.2.19) itself lists `v.after`/`v.before`.
- Chosen: no solution names a range end. I wrote 08 by hand instead of using `Diff`, because its add-then-finish shape is not online.
- Cost of a wrong guess: a compile error in any code that touches range ends.
- Subset: inside (R04.8 supported, std-prefix supported).

### missing-stdlib

**H-ABS-STUB**
- Question: where is the free function `abs(x: Int)`?
- Readings: (a) a builtin, since spec examples and `std/math.cp:1464` call `abs(p)`; (b) a method `x.abs`. `std/` has no free `pub fn abs`; only `Ratio.abs` and `BigInt.abs` exist.
- Chosen: (a), `abs(x[i])` in 04.
- Cost of a wrong guess: an unknown-name compile error.
- Subset: inside (std-math supported).

**H-MAP-SET-STUB**
- Question: what are the constructors and API of `Map`/`Set`?
- Readings: `spec/lib/containers.md` (lines 63, 176, 535) calls `Map`, `Set`, `HashMap` and `HashSet` builtins, but `std/` has no stub for them, so the constructor (`Map()`, `{}`, `Set(xs)`) cannot be found.
- Chosen: I avoided them. 01 sorts and compares neighbours; 09 and 10 use the stubbed `Counter`.
- Cost of a wrong guess: a compile error. The detour also costs an O(n log n) sort in 01.
- Subset: inside (R07.12 supported).

**H-MINMAX-STUB**
- Question: are `min`/`max` over two values builtins?
- Readings: R11.3.3 (`11-modules-prelude.md` line 80) says they are builtins, but there is no stub.
- Chosen: builtin `min(a, b)`, used in 02 and 05.
- Cost of a wrong guess: an unknown-name compile error.
- Subset: inside (R11.3 supported).

### silent

**H-ADJACENT-TUPLE-LAMBDA**
- Question: `.adjacent` yields `(T, T)`. Does `(x, y) => x == y` destructure that pair, or is it a two-argument lambda?
- Readings: (a) it destructures; (b) it must be written `p => p.0 == p.1`.
- Chosen: (a), in 01.
- Cost of a wrong guess: a compile error.
- Subset: inside (R01.8, R07.7 supported).

**H-SHORT-CIRCUIT**
- Question: do `and`/`or` short-circuit?
- Readings: (a) they do, like Python, C++ and Rust; (b) both sides are evaluated.
- Chosen: (a). 03 guards `a[i - 1]` with `i == 0 or`, 06 guards `b[j - 1]` with `j > 0 and`, and 09 relies on it too.
- Cost of a wrong guess: an index-out-of-range trap.
- Subset: inside (R01.5 supported).

**H-RANGE-REV**
- Question: is an integer `Range` `Bidir`, so that `(0..30).rev` works? `std/seq.cp:169` requires `Bidir`.
- Readings: (a) yes; (b) write a `while` loop that counts down.
- Chosen: (a), in 02 and 08.
- Cost of a wrong guess: a compile error.
- Subset: inside (R07.7 supported).

**H-ARRAY-FILL**
- Question: does a scalar initializer fill an array, including one with a runtime size: `var c: [n + 1]Int = 0`?
- Readings: (a) yes, as `std/prefix.cp` `Diff.new` does with `var d: [D.cuts]T = T.zero`; (b) a comprehension `[0 for _ in 0..n + 1]`. The spec text states neither.
- Chosen: (a), in 02, 04, 07 and 08.
- Cost of a wrong guess: a compile error, or a size expression that is not allowed.
- Subset: inside (R02.8 supported).

**H-STR-SLICE-CONCAT**
- Question: is `a[0..=i] + b[i..]` (two `Str` slices) a `Str` that `print` shows as text?
- Readings: (a) yes, as `spec/lib/strings.md` line 206 writes `s[r..] + s[..r]`; (b) the slices must be converted with `Str(...)` first.
- Chosen: (a), in 06.
- Cost of a wrong guess: a compile error, or output printed as a sequence of characters.
- Subset: inside (R02.3 supported).

**H-COUNTER-INDEX**
- Question: is `c[k]` a read that gives `init` for a missing key, and is `c[k] += 1` allowed?
- Readings: the stub examples show `c[5] == 2` reads and `c['a'] = 4` writes, but no compound assignment.
- Chosen: reads with `c[k]`, updates with `c.add(k, d)`, in 09 and 10.
- Cost of a wrong guess: low; this choice avoids the open question.
- Subset: inside (std-hashmap supported).

**H-NEW-SUGAR**
- Question: is `Counter(0)` sugar for `Counter.new(0)`, and are the type arguments inferred from the annotation?
- Readings: (a) yes, as the stub example `var c: Counter<Char, Int> = Counter(0)` shows; (b) the explicit form `Counter<Char, Int>.new(0)`.
- Chosen: (a), in 09 and 10.
- Cost of a wrong guess: a compile error.
- Subset: inside (R02.6 supported).

**H-LOOP-ELSE-LABEL**
- Question: can a labeled `break 'search` sit inside the `else` of an inner `for` to leave the outer loop? Does `else` run only when the loop ends without a `break`?
- Readings: (a) Python semantics plus labels, as in the R01.7 example; (b) `else` is limited to the outermost loop.
- Chosen: (a), in 08.
- Cost of a wrong guess: a wrong answer (no output, or every k printed).
- Subset: inside (R01.7 supported).

**H-BOOL-XOR**
- Question: is `!=` on `Bool` a valid exclusive or?
- Readings: (a) yes, by equality on any scalar; (b) `^` on `Bool`.
- Chosen: (a), in 08.
- Cost of a wrong guess: a compile error.
- Subset: inside (R01.5 supported).

**H-CONST-EXPR**
- Question: can `const` hold a shift expression such as `(1 << 31) - 1`, and do consts use lowercase names?
- Readings: (a) yes, and lowercase as for values; (b) only literals, or SCREAMING case.
- Chosen: (a), in 10.
- Cost of a wrong guess: a compile error.
- Subset: inside (R01.6 supported).

**H-RANGE-FIND**
- Question: what is the idiom for "the first index where a predicate that is not monotone holds"? `first_true` traps in debug builds when the predicate is not monotone.
- Readings: (a) `(0..m).find(p)`, treating the range as a `Seq`; (b) a loop with `break`.
- Chosen: (a), in 06.
- Cost of a wrong guess: a compile error.
- Subset: inside (std-seq supported).

## Per problem

| # | File | Holes hit | Left the subset? |
|---|---|---|---|
| 1 | 01-romantic-glasses.cp | H-ADJACENT-TUPLE-LAMBDA, H-MAP-SET-STUB | no |
| 2 | 02-game-with-multiset.cp | H-RANGE-REV, H-ARRAY-FILL, H-MINMAX-STUB | no |
| 3 | 03-closest-cities.cp | H-SHORT-CIRCUIT | no |
| 4 | 04-monsters-attack.cp | H-ABS-STUB, H-ARRAY-FILL | no |
| 5 | 05-turtle-vs-rabbit-race.cp | H-MINMAX-STUB | no |
| 6 | 06-binary-path.cp | H-STR-SLICE-CONCAT, H-RANGE-FIND, H-SHORT-CIRCUIT | no |
| 7 | 07-mex-game-1.cp | H-ARRAY-FILL | no |
| 8 | 08-long-inversions.cp | H-LOOP-ELSE-LABEL, H-BOOL-XOR, H-RANGE-REV, H-ARRAY-FILL, H-RANGE-ENDS | no |
| 9 | 09-ingenuity-2.cp | H-COUNTER-INDEX, H-NEW-SUGAR, H-TALLY-TYPE, H-SHORT-CIRCUIT | no |
| 10 | 10-vlad-and-division.cp | H-COUNTER-INDEX, H-NEW-SUGAR, H-TALLY-TYPE, H-CONST-EXPR | no |
