# Rebuild round 13: holes

I read commit `ff7970052` (`spec: case for V.cuts as a domain`) and only `spec/` (README, SUBSET, RATIONALE, GLOSSARY, diagnostics, `core/`, `lib/`, `tools/`; not `spec/conformance/`) and the root `std/` stubs. I did not open `plan/`, `corpus/`, `decisions.md` or anything else, and did not run the compiler. Statements came from the Luogu mirror (codeforces.com returned 403).

| class | count |
|---|---|
| silent | 8 |
| contradiction | 2 |
| missing-stdlib | 3 |
| found-late | 11 |
| **total** | **24** |
| inside the Milestone 1 subset | 24 |

## Top risks

1. H-ARRAY-WIDEN-READ: if `var a: [I]Int = read()` were refused and `[I]0..=n` used instead, `a[j] += 1` past `n` traps (checked) or wraps (release): 07 answers wrong.
2. H-COUNTED-INT: if a counted `[k]` array were not indexed by plain 0-based `Int`, `v[adds % k]` in 07 and `b[m - 1 - i]` in 06 read the wrong element.
3. H-FN-IN-MULTITEST: if a `fn` after `@!multitest` were hoisted to the shared section, `high`/`low` in 06 would capture another test's `a`, `b`.
4. H-GRID-ROWS: if `[R, C]Char` read column by column, 04 walks the transposed grid.
5. H-TALLY-TYPE: `tally` gives a `HashMap` (std) not a `Counter` (spec); `m[k]` on a missing key then traps instead of reading 0, so a window counter written from the spec breaks 01.

## Holes

### silent

- **H-FIXED-WINDOW** (01). Q: what is the idiom for a fixed-length sliding window whose state is a `Track` (`Counter`)? Readings: `a.windows(m)` with a fold (re-walks each window, O(nm)); `Sweep.move` per window; a hand loop with push at `i`, pop at `i - m`. Chose the hand loop over `a.keys` with two `Counter`s. Cost: none for correctness; idiom/perf only (a wrong pick of `windows` is TLE). spec/lib/sequence-patterns.md covers `maximal`/`minimal`/`mo` only. Inside: std-window, std-hashmap supported.
- **H-COUNTER-OF-ARRAY** (01). Q: does `Counter.of` accept a `[m]T` array? `std/hashmap.cp:276` takes `List<K>`; nothing says arrays convert. Readings: `Counter.of(b)`; `Counter.of(List(b))`; a loop. Chose `need[x] += 1` in a loop. Cost: compile error only. Inside: std-hashmap supported.
- **H-COUNTER-KEY-TYPE** (01). Q: with elements typed `1..=1e6`, is an unannotated `Counter(0)` keyed by the range type or `Int`, and does a range value index a `Counter<Int>`? Readings: K inferred as the range; K is `Int`. Chose an explicit `Counter<Int>`. Cost: compile error, or a key-type mismatch between `have` and `need`. Inside: R02.10, std-hashmap supported.
- **H-ARRAY-WIDEN-READ** (07). Q: can a read keep the statement's contract and store wider, e.g. `var a: [I]Int = read<[I]0..=n>()`? R03.1 says `[V]Int` widens by annotation, R02.11 says nothing converts implicitly; no rule for an array-of-range to array-of-`Int` move. Readings: implicit widening; error; read as `[I]Int`. Chose `var a: [I]Int = read()`, losing input validation. Cost: high if the range is kept (overflow past `n` on `+= 1`). Inside: R03.1, R10.2 supported.
- **H-MOD-INT-OPERAND** (08). Q: is `before[i] + 1` with `before: [n + 1]Z` Mod arithmetic? R04.4.28 converts an `Int` "where a `Mod<P>` is expected" (assignment, fill); an operand of `+` is not obviously "expected". Readings: operator expects `Z`; an error asking for `Z(1)`. Chose the bare `1`. Cost: compile error. Inside: R03.5, R04.4 supported.
- **H-OPT-RUNNING-MIN** (03). Q: how to keep a running minimum with no start value? Ordering does not lift over `?T` (chapter 2), so `best min= cost` on `?Int` is out; no `Int.max` sentinel is specified. Readings: sentinel `1e18`; `?Int` with an explicit `None` test; collect costs then `.min()!`. Chose the `None` test. Cost: none if accepted; verbosity. Inside: R02.4, R01.6 supported.
- **H-GRID-TUPLE-INDEX** (04). Q: does a `[R, C]T` array accept a `(R, C)` tuple key (`g[p]`), as `adj4`/`step` yield tuples? Only `grid[r, c]` is shown. Readings: tuple key works; must destructure. Chose destructuring (`for (r, c) in p.adj4`, `g[r, c]`). Cost: compile error. Inside: R04.2, std-grid supported.
- **H-FN-IN-MULTITEST** (02, 06). Q: is a `fn` declared after `@!multitest` per test, so it may capture that test's `let`s? R06.1.7 says functions hoist within their block and captures must be initialized before the first call; R10.1 does not say whether the per-test part is a block. Readings: per-test block (capture fine); hoisted to file scope (stale or error). Chose per-test; `high`/`low` in 06 capture `a`, `b`, `n`, `m`. Cost: high (wrong test's arrays) or compile error. Inside: R10.1, R06.1 supported.

### contradiction

- **H-TALLY-TYPE** (01). Q: what does `xs.tally()` return? spec/core/07-sequences.md, Ordering and grouping (R07.11) lists it as `Counter<K>`; std/seq.cp:1807 declares `-> HashMap<K, Int>`. Readings: Counter (absent key reads 0); HashMap (absent key: no default). Chose neither: built the counts with `Counter` by hand. Cost: a trap on missing keys if the std signature wins. Inside: R07.11, std-seq supported.
- **H-SORT-DESC** (06). Q: is descending order a key or a flag? spec/core/07-sequences.md R07.11 says "Descending is a key, not a flag" (`by: x => desc(x)`) and R06.1.5 names `desc` as an option; std/seq.cp:225 (`sort`) and :273 (`sorted`) take `desc: Bool = false`, while std/algebra.cp:134 has the `desc(x)` key. Readings: `desc: true`; `by: x => desc(x)`. Chose plain ascending `a.sort()` and mirrored indices, avoiding both. Cost: compile error or reversed order. Inside: R07.11, std-seq supported.

### missing-stdlib

- **H-ABS** (06). Q: where is `abs(x: Int)`? Used in spec/core/01 and 06 examples (`abs(a.x - b.x)`), not in `std/prelude.cp`; std has only `Ratio.abs` (std/math.cp:1464) and `BigInt.abs` (:1570). Readings: builtin; must write `if x < 0: -x else: x`. Chose `abs(...)`. Cost: compile error. Inside: std-math supported.
- **H-MIN2** (03). Q: where is binary `min(x, y)` on `Int`? std/seq.cp:64/79 define only the sequence `min`/`max(by:)`; R03.1.8 mentions "`min`/`max` of those" as values. Readings: builtin two-argument `min`; `x min= y` on a copy. Chose `min(a[i], b[i])`. Cost: compile error. Inside: std-seq supported.
- **H-LIST-STUB** (08, 10). Q: what are `List<T>`'s `push`, `pop`, `last`, `len`? No `struct List` in std/; only the generic `last` getter (std/seq.cp:200) and spec uses (`stack.pop()` in spec/core/04b-keys.md). Readings: builtin with these names; `pop -> ?T` vs unit. Chose `push`, `pop()` (result ignored), `last` as `?T`, `len`. Cost: compile error. Inside: R02.8 supported.

### found-late

- **H-RUNTIME-BOUND** (01, 03, 06, 07). Q: may a later read's range use an earlier local (`let m: 1..=n = read()`)? Looked first at R01.6.1 and R10.2.1; answer in spec/core/02-values-types.md R02.10.2 ("A local's range type may have runtime bounds"). Inside: R02.10 supported.
- **H-COUNTED-INT** (01, 02, 06, 07, 08, 10). Q: is `a[i - m]` with `Int` legal on `read<[n]T>()`? Looked first at R10.2.2 (base) and R04.2; answer in spec/core/04-indices.md R04.4.30 (counted `[n]` is the anonymous domain `0..n`, positions are `Int`s). Inside: R04.4 supported.
- **H-STR-INDEX** (02). Q: is `s[i]` a `Char` and `s.keys` `0..len`? Looked first at chapter 7 tables; answer at spec/core/02-values-types.md:64 (Str indexed by `0..len`). Inside: R02.3 supported.
- **H-RANGE-AS-INT** (01, 09, 10). Q: does a range-typed value pass where `Int` is expected (`n.digits()`, `self: Int`) and does `var lo = a.min()!` become `Int`? Looked first at R02.11; answer at spec/core/02-values-types.md:45 (a range type is "an `Int` with a contract") and R02.10.2 (unannotated `var` widens to `Int`). Inside: R02.10 supported.
- **H-EMPTY-LINE** (10). Q: what does `print(xs)` do for an empty list (0 operations, empty line allowed)? Looked first at `print` in std/io.cp; answer in spec/core/10-io.md R10.3.1 ("an empty one prints an empty line"). Inside: R10.3 supported.
- **H-TUPLE-FOR** (04). Q: may a `for` destructure a tuple? Looked first at R01.7; answer in spec/core/01-lexical-grammar.md R01.6.3 (`for (var left, v) in items`). Inside: R01.6 supported.
- **H-DEQUE-OF** (04). Q: how is a BFS queue built: `Deque<T>()`, `Deque.new()`, `Deque.of(xs)`? spec/lib/containers.md 1.5 names the type only; answer in std/deque.cp:20-24 (`Deque.of(sources)`, then `while let u = q.pop_front()`). Inside: std-deque supported.
- **H-WHILE-LET-AND** (08). Q: can `while let` bind and test in one condition for a monotonic stack? Looked first at R01.7; answer in spec/core/04b-keys.md `next_greater` (`while let j = stack.last and a[j] < a[i]`). Inside: R01.7, R04.2 supported.
- **H-GEN-SEQ** (05). Q: is a `-> yields Int` call a sequence with `.min()`? Chapter 6 only shows `for`; answer in spec/core/07-sequences.md R07.2.10 (calling a generator gives a sequence). Inside: R06.3, R07.6 supported.
- **H-GRID-ROWS** (04). Q: in `[R, C]Char`, is `R` the row (one line per `R` position)? R10.2.3 says "reads rows" without naming the factor; answer in spec/core/04-indices.md R04.2.9 (product walks row-major, last factor fastest). Inside: R10.2, R04.2 supported.
- **H-DIVISORS-IMPORT** (02). Q: is `divisors` in the prelude? Looked first at `std/prelude.cp` (absent; chapter 6 defines its own); answer: it is `std/math.cp:552`, so `use std.math.{divisors}` per spec/core/11-modules-prelude.md R11.3. Inside: R11.2, std-math supported.

## Per problem

| solution | holes | left subset |
|---|---|---|
| 01-inaccurate-subsequence-search | H-FIXED-WINDOW, H-COUNTER-OF-ARRAY, H-COUNTER-KEY-TYPE, H-TALLY-TYPE, H-RUNTIME-BOUND, H-COUNTED-INT, H-RANGE-AS-INT | no |
| 02-nearly-shortest-repeating-substring | H-FN-IN-MULTITEST, H-COUNTED-INT, H-STR-INDEX, H-DIVISORS-IMPORT | no |
| 03-seraphim-the-owl | H-OPT-RUNNING-MIN, H-MIN2, H-RUNTIME-BOUND | no |
| 04-arrow-path | H-GRID-TUPLE-INDEX, H-TUPLE-FOR, H-DEQUE-OF, H-GRID-ROWS | no |
| 05-yet-another-coin-problem | H-GEN-SEQ | no |
| 06-very-different-array | H-FN-IN-MULTITEST, H-SORT-DESC, H-ABS, H-RUNTIME-BOUND, H-COUNTED-INT | no |
| 07-watering-an-array | H-ARRAY-WIDEN-READ, H-RUNTIME-BOUND, H-COUNTED-INT | no |
| 08-array-collapse | H-MOD-INT-OPERAND, H-LIST-STUB, H-WHILE-LET-AND, H-COUNTED-INT | no |
| 09-good-triples | H-RANGE-AS-INT | no |
| 10-add-divide-and-floor | H-LIST-STUB, H-COUNTED-INT, H-RANGE-AS-INT, H-EMPTY-LINE | no |
