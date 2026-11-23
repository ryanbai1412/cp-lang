# Rebuild round 11: holes

I read commit `32e03b5ac` (`spec: pin .items on a temporary`) of ryanbai1412/cp-lang and nothing else in the repo: only `spec/` (README, SUBSET, GLOSSARY, `core/`, `lib/`, `tools/` file list) and the stdlib stubs in `std/`. I did not open `spec/conformance/`, `plan/`, `corpus/` or `decisions.md`, and did not run the compiler. Statements came from the Luogu mirrors of the ten Codeforces problems. Every place I guessed is listed below, even when the intent seemed clear.

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 12 | 12 |
| contradiction | 5 | 4 |
| missing-stdlib | 1 | 1 |
| **total** | **18** | **17** |

## Top risks

1. H-WRAP-NEG: if `V.wrap` of a negative integer truncates instead of reducing Euclidean-style, 09 sends the ball to the wrong player and gives a silent wrong answer.
2. H-TALLY-TYPE: if `tally` returns `HashMap` (the stub) and not `Counter` (the spec), `c[1]` in 01 traps or does not compile on any test with no 1s or no 2s.
3. H-IFLET-ELSE: if the `else:` of `if let j = x, cond:` runs only when the binding fails, 07 prints nothing when the guard fails, a wrong answer.
4. H-SCAN-INT-INDEX: if `scan(+)` on a mapped `List` view is not indexed by cut count with `pre[k]` = sum of the first k, 02 is off by one.
5. H-PIECE-AS-RANGE: if `x.after` is not accepted as a `Range<C>` by `Fenwick.fold`, or means something other than the positions strictly after `x`, 05 fails to build or miscounts.

## Holes

### silent

**H-WRAP-NEG**: what does `V.wrap(x)` return for negative `x`?
- Readings: (a) Euclidean, `x mod V.count` in `0..count`; (b) C-style truncated remainder, traps or yields a bad position for negative `x`.
- Chose (a): `core/04-indices.md` R04.2.13 says only "integer to position, mod `V.count`", and `/`, `%` are Euclidean elsewhere; `std/seq.cp:3274` `Ring.step` double-mods, which hints (b) is a real concern.
- Cost: 09 is wrong on every counterclockwise throw (`Int(p) - r < 0`), or traps.
- Inside: R04.2 (supported).

**H-IFLET-ELSE**: in `if let j = x, cond: A else: B`, does `B` run when the binding succeeds but `cond` is false?
- Readings: (a) `else` runs whenever the whole header fails; (b) `else` only when the binding fails.
- Chose (a). `core/01-lexical-grammar.md:247` shows `if let v = m.get(k), v > 0:` but never with an `else`.
- Cost: 07 prints nothing for queries whose last different element is left of `l`: wrong answer.
- Inside: R01.6 (supported).

**H-SCAN-INT-INDEX**: how is the `Prefix` from `xs.map(f).scan(+)` indexed, when `xs` is a `List`?
- Readings: (a) by `Int` cut count, `pre[k]` = sum of the first `k`; (b) by an inclusive position, `pre[i]` = sum through `i`; (c) only by a typed cut, `pre[List.cuts(k)]`.
- Chose (a), from `core/07-sequences.md` (scan over cuts; integer keys take the literal as the cut).
- Cost: 02 off by one (wrong answer) under (b), compile error under (c).
- Inside: R07.9 (supported).

**H-PIECE-AS-RANGE**: is `v.after` (a piece) accepted where `Fenwick.fold(r: Range<D>)` wants a `Range<D>`?
- Readings: (a) yes, a piece is a `Range<D>` (`v.r..`); (b) no, write `x.r..C.cuts.last!` or `fold(C.cuts...)`.
- Chose (a): R04.2.19 calls it "the piece `v.r..`"; `std/fenwick.cp:66` takes `Range<D>`; nothing states the conversion for a stored piece (R04.4 says stored cut ranges do not convert to pieces, which is the opposite direction).
- Cost: 05 does not compile; a reading that includes `x` is harmless here, since `x` is added after the query.
- Inside: R04.2, std-fenwick (supported).

**H-COUNT-EXPR**: may a counted array type use an expression count, `read<[n - 1](u: V, v: V)>()`?
- Readings: (a) any `Int` expression; (b) only a name, so `domain E = 0..n - 1` first.
- Chose (a); every example in `core/10-io.md` and `lib/graphs-trees.md` uses a bare name (`[n]`, `[m]`, `[E]`).
- Cost: 10 does not compile.
- Inside: R10.2 (supported).

**H-MOD-PER-TEST**: is a runtime modulus `type Z = Mod(m)` legal below `@!multitest`, once per test, including `m = 1`?
- Readings: (a) yes, a fresh scoped type per case; (b) the modulus must be read once, above the divider.
- Chose (a): `core/10-io.md` R10.1.1 says a fresh type below the divider cannot leak into the next case; `core/03-numbers-layout.md:280` says "with `p` read once", which I read as once per scope; `lib/math.md:192` has `one = Mod(1 % P)` so `m = 1` gives 0.
- Cost: 08 does not compile; fallback is plain `Int` with `% m`.
- Inside: R03, std-mod (supported).

**H-PRINT-VIEW**: does `print` accept a lazy view such as `out.rev` as one sequence argument?
- Readings: (a) any `Seq` prints space-separated; (b) only `List`/arrays, so `List(out.rev)`.
- Chose (a); `core/10-io.md` says "a sequence argument prints" without naming views.
- Cost: 08 does not compile.
- Inside: R10.3 (supported).

**H-SUM-GENERATOR**: can `sum(of:)` consume a `yields` member such as `Counter.values`?
- Readings: (a) generators are `Seq` and every fold applies; (b) only stored sequences.
- Chose (a), from `core/07-sequences.md` (generators are push sequences) and `std/seq.cp:34` `sum<S: Seq>`.
- Cost: 01 does not compile.
- Inside: R07.9, std-hashmap (supported).

**H-RANGE-WIDEN**: does `max(lo - k, s.l)` mix `Int` and a range-typed field (`0..=1e9`) into an `Int`?
- Readings: (a) range values widen to `Int` in arithmetic and `min`/`max`; (b) an explicit `Int(s.l)` is needed.
- Chose (a); `core/03-numbers-layout.md` says arithmetic is `Int` and ranges constrain storage, but `min`/`max` of mixed types is not shown.
- Cost: 03 does not compile.
- Inside: R03.1 (supported).

**H-FN-BELOW-MULTITEST**: may a named `fn` declared below `@!multitest` capture that test's `let`s (`seg`, `a`, `t`, `k`)?
- Readings: (a) yes, like a nested function capturing by lending; (b) only lambdas capture; a top-level `fn` sees only globals.
- Chose (a): R06.2 nested functions capture, R10.1.1 makes everything below the divider per-test; no example shows a top-level `fn` there.
- Cost: 03, 04, 06, 10 do not compile; fallback is a lambda `let f = x => ...`.
- Inside: R06.2, R10.1 (supported).

**H-RANGE-EXPR-VS-TYPE**: is `(0..=1e9).first_true(f)` a value range of `Int`, given that `0..=1e9` is also a range type?
- Readings: (a) in expression position it is a `Range<Int>`; (b) it names a type, so a value range needs `Range(0, 1e9)` or similar.
- Chose (a); `core/04-indices.md` uses `(p..=x).rev` and `r.first_true`, but never with an exponent literal bound.
- Cost: 03 and 10 do not compile.
- Inside: R04.4 (supported).

**H-MAXK-ORDER**: does `Day.max_k(3, by:)` return the three largest (any order) and break ties arbitrarily?
- Readings: (a) yes; (b) it returns the k smallest by a descending key.
- Chose (a) from `std/seq.cp:1853`; neither `spec/core` nor `spec/lib` documents `max_k`, so its meaning is from the name and stub only.
- Cost: 04 picks wrong days: wrong answer.
- Inside: R04.2.20 (a domain is a `Seq`), std-seq (supported).

### contradiction

**H-TALLY-TYPE**: what does `s.tally()` return?
- `spec/core/07-sequences.md:342` (R07.11.4): `Counter<K>`. `std/seq.cp:1807`: `-> HashMap<K, Int>`.
- Readings: (a) `Counter`, missing keys read 0 (`std/hashmap.cp:255`, `get` at 304); (b) `HashMap`, `c[k]` on a missing key traps or is `?Int`.
- Chose (a), the spec.
- Cost: 01 traps or fails to compile on `c[1] * c[2]` whenever 1 or 2 is absent.
- Inside: R07.11, std-seq (supported).

**H-DESC-FLAG**: how do you sort descending?
- `spec/core/07-sequences.md:345`: "Descending is a key, not a flag: `by: x => desc(x.score)`". `std/seq.cp:225` and `:273`: `sort`/`sorted(..., desc: Bool = false)`. `desc` itself lives in `std/algebra.cp:134`, a module the subset marks `documented` (std-algebra).
- Readings: (a) `desc(...)` key; (b) `desc: true` flag.
- Chose neither: 04 uses `max_k` to avoid the question (see H-MAXK-ORDER).
- Cost: compile error with the wrong spelling.
- Inside: R07.11 (supported).

**H-RING-RECEIVER**: is `ring` on the domain or on an array?
- `spec/core/04-indices.md:69`: `D.ring.step(x, d)`. `std/seq.cp:3264`: `@get pub fn ring<D, T>(self: [D]T)`, an array method.
- Readings: (a) domain method; (b) array method.
- Chose neither: 09 uses `P.wrap` (R04.2.13), which has a table row.
- Cost: compile error with the wrong receiver.
- Inside: R04.2 (supported).

**H-ROOTED-ROOT-ARG**: is the root of `Rooted` positional or named?
- `spec/lib/containers.md:685`: `Rooted(Graph.undirected(edges), V.first!)`. `spec/lib/graphs-trees.md:246` and `std/tree.cp:84`: `Rooted(g, root: V.first!)`.
- Readings: (a) named `root:`; (b) positional; (c) either.
- Chose (a), the graphs chapter and stub. (`std/tree.cp:83` also notes that `Rooted` resolves to a compiler built-in.)
- Cost: 10 does not compile.
- Inside: std-tree (supported).

**H-COMPRESS-SUBSET**: is `compress` inside Milestone 1?
- `spec/core/07-sequences.md:343` (R07.11.5) lists `compress(s)` under R07.11, which `spec/SUBSET.md` marks supported, and the function lives in `std/seq.cp:860` (std-seq, supported). But `spec/SUBSET.md` also has a row `std-compress: documented, outside the Milestone 1 contest staples`, and there is no `std/compress.cp`.
- Readings: (a) inside (R07.11, std-seq); (b) outside (std-compress).
- Chose to use it, counted as outside to be conservative.
- Cost: if outside, 05 relies on a feature that may not work; fallback is sorting the `b` values and using `bsearch`.
- Outside (contested): std-compress row; inside by R07.11.

### missing-stdlib

**H-ABS**: where is integer `abs`?
- `spec/core/01-lexical-grammar.md:45`, `spec/core/06-control-functions.md:22` and `spec/lib/math.md:387` all call `abs(x)` on `Int`, but no `std/*.cp` stub defines it (only `Ratio.abs`, `BigInt.abs` in `std/math.cp:1464,1570`), and `std/prelude.cp` imports no `abs`.
- Readings: (a) a core built-in; (b) a missing stub.
- Chose to avoid it: 06 needs `|a[i + k] - a[i]|`, and passes the signed difference to `gcd`, which `spec/lib/math.md:117` says returns `>= 0`.
- Cost: none in these solutions; the natural spelling may not resolve.
- Inside: std-math (supported).

## Per problem

| # | Problem | Holes | Left subset? |
|---|---|---|---|
| 01 | 1899D Yarik and Musical Notes | H-TALLY-TYPE, H-SUM-GENERATOR | no |
| 02 | 1904B Collecting Game | H-SCAN-INT-INDEX | no |
| 03 | 1907D Jumping Through Segments | H-RANGE-WIDEN, H-FN-BELOW-MULTITEST, H-RANGE-EXPR-VS-TYPE | no |
| 04 | 1914D Three Activities | H-MAXK-ORDER, H-DESC-FLAG, H-FN-BELOW-MULTITEST | no |
| 05 | 1915F Greetings | H-PIECE-AS-RANGE, H-COMPRESS-SUBSET | possibly: `compress` (std-compress row says documented) |
| 06 | 1920C Partitioning the Array | H-ABS, H-FN-BELOW-MULTITEST | no |
| 07 | 1927D Find the Different Ones! | H-IFLET-ELSE | no |
| 08 | 1932C LR-remainders | H-MOD-PER-TEST, H-PRINT-VIEW | no |
| 09 | 1941D Rudolf and the Ball Game | H-WRAP-NEG, H-RING-RECEIVER | no |
| 10 | 1946C Tree Cutting | H-COUNT-EXPR, H-ROOTED-ROOT-ARG, H-FN-BELOW-MULTITEST, H-RANGE-EXPR-VS-TYPE | no |
