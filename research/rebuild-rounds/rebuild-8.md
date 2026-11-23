# Rebuild round 8: holes

I read commit `e254eb449` ("lint: every lint has a machine fix or says why not") of ryanbai1412/cp-lang. I used only `spec/` (README, SUBSET, RATIONALE headings, GLOSSARY, diagnostics, `core/`, `lib/`) and `std/` (stubs, `std/README.md`, `std/GAPS.md`). I did not open `spec/conformance/`, `plan/`, `corpus/` or the compiler, and I ran nothing. One broad grep also matched a few table lines in `spec/idioms/lints.md`, which is outside the list; nothing below relies on it. Every problem fit the Milestone 1 subset. All ten files use only rows marked supported in SUBSET.md (R01, R02, R04, R06, R07, R10, std-seq, std-prefix, std-math, std-io), so every hole is inside the subset.

## Summary

| Class | Inside subset | Outside | Total |
|---|---|---|---|
| Contradiction | 12 | 0 | 12 |
| Silent | 7 | 0 | 7 |
| Missing stdlib | 4 | 0 | 4 |
| **Total** | 23 | 0 | 23 |

The ten most likely to hit a contestant, most likely first:

1. H-INT-KEY-PREFIX: `pre[k]` on an `Int`-keyed prefix can mean a cut or a position (an off-by-one that gives a silent WA).
2. H-PREFIX-ELEM-TYPE: does `a.scan(+)` over `[n]1..=1e4` store sums in `1..=1e4`?
3. H-SORT-DESC: is descending order `desc: true` or `by: x => desc(x)`?
4. H-RANGE-ARG-NARROW: what happens when an `Int` argument goes to a range-typed parameter (`odd - 1` into `0..=100`)?
5. H-BOOL-TO-INT: there is no `Int(b)` for counting flags.
6. H-MEMO-LIFETIME: when the `@memo` cache is allocated and whether it resets per test.
7. H-SCAN-MAX-IDENTITY: can `a.scan(max)` be written, and what is at cut 0?
8. H-LIST-CORE: `push`, `pop`, `len`, `is_empty`, `clone`, `List<T>()` and `pop_front` have no stubs.
9. H-QUESTION-EXTENT: does `Int(x?)` propagate through `Int`?
10. H-SCI-LITERAL: is `1.5e5` an `Int` or an `F64`?

## Contradiction

### H-SORT-DESC
- Question: how do you sort in descending order?
- Readings: (a) a key: `core/07-sequences.md` line 340, "Descending is a key, not a flag: `by: x => desc(x.score)`". (b) a flag: `std/seq.cp:225` `sort(..., desc: Bool = false)` and `:273` `sorted(..., desc: Bool = false)`, whose doc example is `[3, 1, 2].sorted(desc: true)`. (c) R06.1.5 lists "`desc`" next to `by:` as a named option, which supports (b).
- Chose: (a), because core is the chapter and the stub follows it (01, 10).
- Cost if wrong: none if both compile. If only one form exists, every descending sort is a compile error, and the two spellings break "one way per job".
- Subset: inside (R07.11, std-seq).

### H-PREFIX-ELEM-TYPE
- Question: what type do the folds of `a.scan(+)` have when `a` holds range-typed elements?
- Readings: (a) `Int`: R03.1.7 says a `Prefix` and a `.sum()` over range-typed elements hold `Int` sums, "never values of `1..=1e9`". (b) the element type: `std/prefix.cp:31` returns `Prefix<D, T>`, and its body writes `var p: [D.cuts]T` (line 33). `std/seq.cp:34` `sum(...) -> U`, where `U` defaults to `T`.
- Chose: (a) (01, 02, 03, 10).
- Cost if wrong: under (b), checked builds trap on the first sum above the element bound, and release truncates it to a silent WA (1676E sums reach 1.5e9 over `1..=1e4`).
- Subset: inside (R03.1, std-prefix).

### H-SCAN-MAX-IDENTITY
- Question: does `a.scan(max)` compile without `from:`, and what is its value at the first cut?
- Readings: `std/prefix.cp:19` lists the ops that have an identity as `+`, `*`, `min` or a `Monoid`, and leaves out `max`. R07.9.2 says `max` has an identity "on a bounded type". If it has one, it could be `lo` of the range, `Int.MIN`, or `None`.
- Chose: it compiles. 02 only reads positions (`tallest[i]`, a fold through `i`), so the value at cut 0 never matters.
- Cost if wrong: a compile error. `from: 0` is not a fix either, because 0 is outside `1..=1e9`, so the contestant has to remap the elements with `a.map(x => Int(x))`.
- Subset: inside (R07.9, std-prefix).

### H-DIV-SEMANTICS
- Question: what does `/` do with a negative divisor?
- Readings: R01.5.2 says "`/` is floor division and `%` is never negative". `std/math.cp:163` says built-in `/` "is Euclidean division", and `:174` says `0 <= r < |d|`. The two agree for `d > 0` and differ for `7 / -2` (floor gives -4, Euclidean gives -3). Floor division and a never-negative `%` cannot both hold when `d < 0`.
- Chose: Euclidean. Every divisor in these ten files is positive (04, 09, 10).
- Cost if wrong: a silent off-by-one in negative-divisor arithmetic, for example the `ceil_div` body at `std/math.cp:170`.
- Subset: inside (R01.5).

### H-ZIP-KEYS
- Question: is `zip(a, b)` an indexed view keyed by `a`, or a walk-only generator?
- Readings: R07.4.10, R07.2.13 and R07.3.5 say it is `Indexed`, keyed by `a`'s keys. `std/seq.cp:570` declares `zip(...) -> yields (T, U)`, which is `Seq` only.
- Chose: either one works for 06, because `.map(...).sorted()` only walks it.
- Cost if wrong: `zip(a, b)[i]`, `.keys` or a scan on a zip fails to compile.
- Subset: inside (R07.4, std-seq).

### H-MEMO-LIFETIME
- Question: when is the `@memo(array)` table allocated, and does it reset per test case?
- Readings: R06.4.1 says the table is allocated "on the first call" and the cache "lives as long as the declaring block (per test case at top level below `@!multitest`)", and R06.5.1 agrees. `lib/search-dp.md:153` says it is "allocated when the enclosing block starts". `:164` says "`ways.clear()` empties the cache (multitest)", which implies a contestant must clear it by hand.
- Chose: R06.4.1. I declared the memo above `@!multitest` so one cache is shared by every test (08). The DP does not depend on the input.
- Cost if wrong: a memo below `@!multitest` that reads per-test data returns answers from the previous case (a WA) if it needs `.clear()`, or rebuilds a table per case (a TLE) if it should have been shared.
- Subset: inside (R06.4, std-dp).

### H-SUM-OF
- Question: is the sum of a function `xs.sum(of: f)` or `sum(f(x) for x in xs)`?
- Readings: `std/seq.cp:34` has `sum(self, of: T -> U)` with the example `[1, 2, 3].sum(of: x => x * x)`. `std/README.md` §3 (around line 227) says "`sum`, `prod`, `min` and `max` of a function stay `sum(f(x) for x in xs)`: one spelling per idea".
- Chose: the free generator form (03).
- Cost if wrong: low, since both probably compile. It is a second spelling for one idea.
- Subset: inside (R07.9, std-seq).

### H-SCI-LITERAL
- Question: is `1.5e5` an `Int` literal?
- Readings: R01.4.1 says scientific literals "that denote an exact integer are `Int`", and 1.5e5 is 150000. R01.4.2 says floats "need a fraction or a non-integer value", and `1.5e5` has a fraction in its mantissa.
- Chose: `Int`, written straight from the statement's bound (01).
- Cost if wrong: a type mismatch in a range bound, a compile error on a very common statement bound.
- Subset: inside (R01.4).

### H-QUESTION-EXTENT
- Question: in `Int(pre.first_ge(x)?) ?? -1`, does `?` propagate through `Int(...)`?
- Readings: R01.8.6 says "`f(x?)` propagate[s] `None`", so the whole call becomes `?Int`. R02.4.7 says the extent is "the nearest enclosing ... call argument", and for `x?` that is `Int`'s own argument slot, which would hand `?Cut` to `Int`. SUBSET also lists the known gaps `question-extent-let-stays-optional` and `question-not-early-return`.
- Chose: avoid it and write `.map(c => Int(c)) ?? -1` (01).
- Cost if wrong: a compile error, or a stray `?Int` that is not `Show`.
- Subset: inside (R02.4).

### H-READ-ALL
- Question: can a program read to end of input?
- Readings: R10.2.7 specifies `read_all<T>()`. `std/io.cp:49` says "blocked: no end-of-input primitive" and the body is `todo`. `std/GAPS.md` lines 33, 63 and 76 say the same.
- Chose: not needed, since every problem here gives a count.
- Cost if wrong: a trap at runtime on any no-count input.
- Subset: inside (R10.2, std-io).

### H-LIST-CTOR
- Question: does `List(x)` convert a sequence or build a list from its arguments?
- Readings: `core/05-ownership.md:106` and `:172` write `List(1, 2)`, a variadic constructor. R07.8.1 writes `List(s)` and `List(0..n)`, a conversion. Under those two readings `List(xs)` with one list argument is either a copy or a list of one list.
- Chose: not used; I wrote list literals (09).
- Cost if wrong: a silently wrong shape, or a type error.
- Subset: inside (R02.8, R07.8).

### H-INTRINSICS
- Question: are `sort`, `sorted`, `sum`, `count`, `min`, `max` and `first` library code or compiler intrinsics?
- Readings: `std/README.md:224` lists them as "Intrinsics (functions not written in the language)". `std/seq.cp` gives them CP bodies with costs. R11.3 says "the compiler has no built-in list".
- Chose: library methods. This changes nothing in the code.
- Cost if wrong: low for contestants. It matters for "no stdlib privilege".
- Subset: inside (R11.3).

## Silent

### H-INT-KEY-PREFIX
- Question: on a `Prefix` over an `Int`-keyed sequence (a `List`, or a counted `[n]T`), is `pre[k]` the fold left of cut `k` or the fold through position `k`?
- Readings: R04.4.30 says that on an integer range positions are `Int` and the cut before `x` is the `Int` `x`. The scan table at `core/07-sequences.md:318` says `pre[v]` on a position is "the fold through v". With `Int` keys both readings type-check and they differ by one element.
- Chose: avoid indexing. I used `.prefix(k)` (a `Cut` parameter) and `.first_ge` on Lists (01, 10), and declared `domain Step = 0..n` where I needed both kinds (02).
- Cost if wrong: an off-by-one with no error, a silent WA.
- Subset: inside (R04.4, R07.1.6, std-prefix).

### H-BOOL-TO-INT
- Question: how do you turn a `Bool` into 0 or 1 for a prefix count?
- Readings: R02.2 lists `Int(c)` for `Char` and nothing for `Bool`. `lib/search-dp.md:144` writes `Mod(sum % k == 0)`, which suggests `Bool` converts to `Mod`, so it might convert to `Int` too.
- Chose: `good.map(b => if b: 1 else: 0).scan(+)` (03).
- Cost if wrong: verbosity only. If `Int(b)` does exist, the long form is a non-idiom.
- Subset: inside (R02.2, R02.11).

### H-RANGE-ARG-NARROW
- Question: can an `Int` (or a wider range) be passed to a range-typed parameter, for example `odd - 1` (range -1..=99) into `odd: 0..=100`?
- Readings: (a) a compile error, since R04.4.24 only allows implicit widening; (b) a checked store, as for `var x: lo..=hi` (R02.10.2); (c) proven from the guard `odd > 0`, but R02.10.2 says range inference runs no fixpoints and does not use guards. The spec does not name an explicit narrowing spelling for ranges (there is `V(x)` for domains).
- Chose: (b) (08).
- Cost if wrong: under (a), every bounded memo DP fails to compile, with no fix named in the spec.
- Subset: inside (R04.4, R06.4).

### H-CUT-FROM-INT
- Question: on `domain V = 1..=n`, does `V.cuts(k)` take the count of positions before the cut (0..=n), or a base-shifted integer?
- Readings: R04.2.17 only says "integer to cut". R04.2.16 says `Int(c)` is the count "whatever the base", so its inverse should be the count. `core/04-indices.md:139` uses `a[..V.cuts(5)]` on a 1-based domain without saying which cut that is.
- Chose: the count (03).
- Cost if wrong: an off-by-one, a silent WA.
- Subset: inside (R04.2).

### H-DISCARD-RESULT
- Question: how do you call `left.remove(i)` (which returns `T`) only for its effect?
- Readings: R06.4.8 says dropping a non-unit result is the unused-result lint, and that `let _ = f()` also drops it. The only escape is `with lint(discarded_result: allow)`. The spec gives no idiom for removing and discarding.
- Chose: a plain call, accepting the warning (07).
- Cost if wrong: a warning only. It is noise on a common shape.
- Subset: inside (R06.4).

### H-WITH-ACROSS-MULTITEST
- Question: does a bare `with io(answers: ...)` at the top of the file apply below `@!multitest`?
- Readings: R09.1.1 says a bare `with` covers "the rest of" its block. R10.1.1 says the divider starts per-test code, and the spec does not say whether that is a new block.
- Chose: it applies (08).
- Cost if wrong: the program prints `YES`/`NO` instead of `Alice`/`Bob`, a WA on every test.
- Subset: inside (R09.1, R10.1).

### H-DESC-AS-VALUE
- Question: can the generic fn `desc` be passed directly as `by: desc`?
- Readings: R06.1.8 lets a `fn` stand in for a lambda, but it does not cover generic fns. Every example writes `by: x => desc(x)`.
- Chose: the lambda (01, 10).
- Cost if wrong: verbosity only.
- Subset: inside (R06.1).

## Missing stdlib

### H-LIST-CORE
- Question: where are `List`'s core methods: `push`, `pop`, `len`, `is_empty`, `clone`, `List<T>()`, `push_front` and `pop_front`?
- Readings: `std/README.md:169` names them as a convention ("Sequences `push` and `pop` at the back, `push_front` and `pop_front` at the front"). `std/seq.cp` declares only `keys`, `insert`, `remove`, `dedup`, `pop_while` and `take_all` on `List`. No stub gives a signature or a cost for the rest.
- Chose: assume they exist. I used `.len`, `.is_empty`, `.clone()` (07, 09) and `remove(0)` as the stand-in for `pop_front` (07).
- Cost if wrong: unknown costs. `remove(0)` is \(O(n)\), and a contestant can't tell whether a cheap `pop_front` exists.
- Subset: inside (R02.8, std-seq).

### H-ABS-MIN-MAX
- Question: where are the free `abs(x)`, `min(a, b)` and `max(a, b)`?
- Readings: the spec uses them (`core/01` §1.2 and `core/06` §6.1 `abs(a.x - b.x)`, `std/math.cp:1464` `abs(p)`). R11.3.3 calls `min`/`max` "builtins", and `std/README.md:224` lists them as intrinsics. No stub declares them, and `std/seq.cp:64/79` are the one-sequence fold methods.
- Chose: assume the built-ins (03, 10).
- Cost if wrong: an unresolved name.
- Subset: inside (R11.3).

### H-FIRST-GE-SEQ
- Question: is there `a.first_ge(x)` or `a.last_le(x)` on a sorted `[D]T`?
- Readings: `std/README.md:178` promises "One four-name family everywhere a sorted order exists (sorted lists, `[D]T`, ...)": `first_ge`, `first_gt`, `last_le`, `last_lt`. `std/seq.cp` has only `first_gt` (:1129) and `last_lt` (:1143). Core R04.8.3 points to `first_true`/`last_true` instead.
- Chose: `keys.last_true(...)` and `bsearch` (02, 07). I used only `Prefix.first_ge`, which exists.
- Cost if wrong: a compile error on the most natural lower-bound call.
- Subset: inside (std-seq).

### H-FLUSH
- Question: what is the signature of `flush()`?
- Readings: `std/io.cp` calls `flush()` in `Wire.new` and `Wire.ask`, and `std/README.md:224` lists it as an intrinsic. `core/10-io.md` §10.5 gives `read`, `read_line`, `read_all`, `print` and `yes`, and no `flush`.
- Chose: not needed, since no problem here is interactive.
- Cost if wrong: hand-flushed interactive code does not resolve.
- Subset: inside (R10.4, std-io).
