# Friction backlog, batch 14: advanced DP

A Stage-1 language-ergonomics scout over ten hard dynamic programming
submissions from `corpus/submissions/`, targeting digit/carry DP, interval DP
(including Knuth optimization), SOS and broken-profile bitmask DP, ternary
prime-mask DP, probability/expected-value DP, and WQS/event-sweep DP with Li
Chao trees.

The ports live in
[`proto/ports/scout_batch_14/`](../proto/ports/scout_batch_14/): `noip21p2`,
`noi15p3`, `csps21p2`, `dmopc18c1p6`, `noip00p4`, `noi23p2`, `noi05p4`,
`ccc20s5`, `ioi16p6io`, and `noi19p1`. Every problem has an ideal CP Lang
sketch marked with `// WISH Pn` on lines the current prototype rejects. All
eight integer and modular problems also have working ports in
[`proto/ports/scout_batch_14/today/`](../proto/ports/scout_batch_14/today/)
that compile with `proto/cp.py build` at commit `91fb701` plus `g++ -O2
-std=c++17` and match their original C++ submissions across 213 differential
test cases (`python3 proto/ports/scout_batch_14/today/stress.py`). The two
floating-point problems (`noi05p4` and `ccc20s5`) are sketch-only because
`Float` is not implemented yet.

Items are numbered `P1` through `P14` so they do not collide with earlier
batches (`F`, `G`, `B`, `D`, `H`, `S`). Earlier batch items that overlap are
cited directly and only the new evidence or broader mechanism is added. Every
error message quoted below was observed directly by running `proto/cp.py build`
at commit `91fb701`. Corpus counts come from scanning the latest accepted C++
submission for each of the 710 C++ problems in `corpus/submissions/info.json`.

## Suggested order for a Stage-2 sprint

1. **P1, P2, P3 (compiler trust bugs)**: silent miscompilation of top-level
   `while` input loops, `using namespace std;` collisions breaking `g++`, and
   `var (a, b) = ...` silently binding immutable locals. Fixing these removes
   real contest-pressure traps.
2. **P8 and P12 (`struct` methods, `use local.*`, and generic `LiChao`)**:
   these two items alone account for 155 lines of boilerplate across just
   `ioi16p6io` and `noi19p1`, and `struct` appears in 255/710 corpus C++
   submissions.
3. **P4 (`Float`, floating-point `@memo`, and formatted precision)**: unblocks
   an entire class of problems (2 of 10 here; 53/710 corpus C++ problems).
4. **P7 and P5 (compiling `stdlib/mod.cp` directly + finite `Index` for `enum`
   and `[D]E`)**: eliminates manual modular arithmetic (`ModR`), Pascal tables
   (`Comb<P>`), and hand-packed base-3 masks (`[Small]Side`).
5. **P6, P10, P11 (interval boundaries, generic SOS transforms, and bit/window
   helpers)**: high-leverage ergonomics for interval DP, SOS DP, and
   broken-profile DP.
6. **P9, P13, P14 (`for ... if`, multiline `if/elif/else`, `min_by`/`argmin`,
   and `List.dedup` / `is_empty`)**: small, uniform cleanups that remove
   repeated friction across almost every DP file.

---

## Compiler bugs

### P1. Emitted `using namespace std;` collides with ordinary user variable names (bug; `ioi16p6io` — 1 of 10; 34/710 corpus C++ files use `norm` alone)

In `ioi16p6io`, declaring a perfectly ordinary list named `norm` passes
`proto/cpc` without warning and then fails C++ compilation:

```cp
// Today (fails g++):
var norm: List<Seg> = []
```

Emitted C++ error:

```text
error: reference to 'norm' is ambiguous
  norm = vector<Tup1>{};
note: candidates are: 'std::norm' (from <bits/stdc++.h> -> <complex>)
```

The workaround in `today/ioi16p6io.cp` was renaming `norm` to `intervals`. The
same trap hits `next`, `prev`, `count`, `rank`, `data`, `size`, `left`, and
`right` whenever `<bits/stdc++.h>` plus `using namespace std;` is emitted.

**Proposed general fix**: Never emit `using namespace std;` in generated C++17
(qualify runtime helpers with `std::`, or wrap user globals in a dedicated
`namespace cp_prog` / collision-safe suffix when an identifier matches a C++
standard-library name). Zero language-surface change, directly serving P0
zero-compiler-fights trust.

### P2. `var (a, b) = (x, y)` destructuring binds variables as immutable `let` (bug; `noip21p2`, `ioi16p6io`, `noi19p1` — 3 of 10)

Writing parallel mutable state initialization fails on the very first mutation:

```cp
// Wanted (also already used in stdlib/mod.cp lines 23, 32, 68, 77):
var (v, c) = (x, 0)
while v > 0:
    c += v & 1
    v = v / 2
```

Compiler error:

```text
error: `c` is not a var
```

A minimal probe confirms even the first variable (`a` in `var (a, b) = (1, 2)`)
is bound as immutable (`error: 'a' is not a var`). In `today/ioi16p6io.cp` and
`today/noi19p1.cp`, this forced expanding single-line Li Chao loop state into
six separate `var` declarations plus manual `let os = ...; ...; ms = os` swap
blocks.

**Proposed general fix**: Propagate the `var` mutability flag to every
identifier in a destructuring binding pattern, and support parallel tuple
reassignment `(a, b) = (b, a)` consistently so swaps need no temp variables.

### P3. Top-level `read()` inside a `while` loop silently swallows the rest of `main()` into the loop (bug; extends `S1`; `noip00p4` — 1 of 10; 44/710 corpus C++ files use sentinel/EOF `while` input loops)

Batch 6 (`S1`) caught this codegen bug for a word `read()` inside a `for` loop
of a `@multitest`. `noip00p4` shows it also silently miscompiles a plain
top-level sentinel-terminated `while` loop outside `@multitest`:

```cp
// Today (compiles cleanly, silently runs the entire program once per input row!):
while true:
    let x: 0..=9 = read()
    let y: 0..=9 = read()
    let z: 0..=1000 = read()
    if x == 0 and y == 0 and z == 0: break
    cell_id[x - 1, y - 1] = weights.len
    weights.push(z)
// Everything below here is emitted INSIDE the while (true) { ... } body!
```

On the sample input with 8 triples, the binary printed 8 cumulative answers
(`13`, `19`, `26`, `34`, `48`, `52`, `67`, `67`) instead of `67`. Wrapping the
loop inside a helper function `fn read_cells()` worked around it in
`today/noip00p4.cp`.

**Proposed general fix**: Insert the `CP_GEN` early-return guard strictly at
top-level statement boundaries (never inside a nested `while`/`for` block).
Separately, consider a small userland/builtin `read_until` helper for sentinel-
and EOF-terminated input streams so stress generators know how to generate
bounded input streams too.

---

## Language and type-system ergonomics

### P4. `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)

Expected-value and probability DP are completely blocked today (`error: unknown
type Float`), making `noi05p4` and `ccc20s5` the only two problems in this batch
without a buildable `today/` port.

```cp
// Wanted (noi05p4):
@memo
fn expected(cat: Node, mouse: Node) -> Float:
    if cat == mouse: return 0.0
    let c1 = step[cat, mouse]
    if c1 == mouse: return 1.0
    let c2 = step[c1, mouse]
    if c2 == mouse: return 1.0
    let deg = g.adj(mouse).len
    var total = expected(c2, mouse)
    for nxt in g.adj(mouse): total += expected(c2, nxt)
    1.0 + total / Float(deg + 1)

print(expected(start_cat, start_mouse), prec: 3)
```

Notice what probability DP specifically stresses beyond basic arithmetic:
1. `@memo` returning `Float` (must track presence via a `Bool` visited array or
   quiet `NaN` rather than integer sentinel values).
2. Explicit `Int -> Float` conversion (`Float(n)`) so integer Euclidean `/` is
   never silently confused with floating-point division.
3. Fixed-decimal output (`print(x, prec: 3)` or `"{x:.8f}"`).

**Proposed general fix**: Add IEEE-754 64-bit `Float` with explicit `Float(x)`
conversion, `@memo` support for arbitrary value types via an explicit `seen`
bit/bool table (or niche encoding), and precision formatting on `print` /
string interpolation.

### P5. Auto-derived `Index` for fieldless `enum`s and finite arrays `[D]E` (extends `B11`; `noi15p3` — 1 of 10; generalizes to all k-ary bitmask, plug/profile, and digit-DP state spaces)

In `noi15p3`, each of the 8 primes `<= sqrt(500)` is assigned to `Neither`, `A`,
or `B`, yielding `3^8 = 6561` states. Today, neither `enum Side` nor `[Small]Side`
can index an array:

```text
error: array dimension must be a domain, a range, or a struct with `impl Index`, got Side
```

So `today/noi15p3.cp` has to fall back to manual base-3 arithmetic everywhere:

```cp
// Today (noi15p3):
let pow3 = [1, 3, 9, 27, 81, 243, 729, 2187, 6561]
for m3 in 0..total3:
    ...
    let digit = (m3 / pow3[prime_id[c] - 1]) % 3
```

With `Index` auto-derived for fieldless `enum`s and finite arrays `[D]E` (radix
encoding `E.len ^ D.len`, under the same small-cardinality guard as
`Subset<D>`):

```cp
// Wanted (noi15p3):
enum Side:
    Neither
    A
    B
type Assign = [Small]Side
var ways: [Assign]Int = 0
for assign in Assign:
    ...
    if assign[i] != Side.A: ok_a = false
```

**Proposed general fix**: Derive `Index` automatically for (1) fieldless
`enum`s (`len = variant count`) and (2) fixed-size arrays `[D]E` when both `D`
and `E` implement `Index` and `E.len ^ D.len` fits the compile-time table bound.
Under this rule, `Subset<D>` is simply the bit-packed specialization of
`[D]Bool`, and ternary/quaternary/plug-DP masks need zero special compiler
constructs and zero manual `pow3[]` arithmetic.

### P6. Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)

Open-ended ranges like `for r in l..:` now work in `proto/cpc`, which is great.
However, interval DP constantly needs **strictly inner** ranges `(l, r)` and
**distance between two domain indices**. Writing `(l + 1)..=Pos.last` fails:

```text
error: cannot iterate a range of ?Pos..Pos
```

Because `l + 1` produces `?Pos`, `today/csps21p2.cp` needs three separate `if let
l1 = l + 1:` / `if let r1 = r - 1:` unwraps AND drops to `.idx` six times just
to check interval widths:

```cp
// Today (csps21p2):
if let r1 = r - 1:
    for i in l..r:
        if i == l: continue
        if i.idx - l.idx - 1 > k: break
        wrap[l, r] += seq[i, r1]
        if not ok(s[i], '*'): break
```

With the approved `.l` / `.r` boundary shorthands from `decisions.md` (where
`l.r..r.l` is the half-open slice of positions strictly between `l` and `r`,
empty when `l + 1 >= r` with no `?Pos` unwrapping) plus signed distance `r - l:
Int` between two indices of the same domain:

```cp
// Wanted (csps21p2):
for i in l.r..r.l if i - l - 1 <= k:
    wrap[l, r] += seq[i, r - 1]
    if not ok(s[i], '*'): break
```

**Proposed general fix**: Implement the approved `.l`/`.r` boundary range
expressions (`l.r..`, `..r.l`, `l.r..r.l`) in the compiler so strict subranges
never require `?Pos` unwrapping, and define `b - a -> Int` for two indices of
the same domain `D` (eliminating `.idx` escapes in interval-length formulas).

### P7. Compiling `stdlib/mod.cp` (`ModR`, `Comb<P>`) directly instead of `proto/cpi`-only stand-ins (extends `B9`/`H1`; `noi15p3`, `noip21p2` — 2 of 10)

`stdlib/mod.cp` already contains clean userland implementations of `ModR` and
`Comb<P>`. Yet importing either one in `proto/cpc` fails:

```text
error: std.mod.Comb is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
error: std.mod.ModR is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
```

As a result, `today/noip21p2.cp` writes a manual 2D Pascal table `comb[i, j]`,
and `today/noi15p3.cp` writes a manual `mod_pow` plus 11 inline `% p`
expressions:

```cp
// Today (noi15p3):
let factor = (mod_pow(2, acnt) + mod_pow(2, bcnt) - 1) % p
cur_way = (cur_way * factor) % p
```

**Proposed general fix**: Compile userland `stdlib/` structs and methods
directly through the compiler backend rather than maintaining per-symbol
hardcoded stand-ins. In addition, allowing `Mod<p>` to accept a runtime-frozen
`let p = read()` (matching `let K = n` for domains in `decisions.md`) would let
runtime-modulus DP use normal `+ - *` operators with zero `z.add`/`z.mul`
verbosity.

### P8. In-file `struct` methods and `use local.*` imports (`ioi16p6io`, `noi19p1` — 2 of 10; 255/710 corpus C++ files define a `struct`)

`examples/local-lib/ds/li_chao.cp` already implements `Line` and `LiChao` in 40
clean lines. Today, neither importing it nor defining an in-file `struct` with
methods works in `proto/cpc`:

```text
error: the prototype compiler imports names from the standard library only: `use std.module.{Name}`
error: line 1:12: expected ':' (at '(')   // on `struct Line(k: Int, m: Int):`
```

Because of this single gap, both `today/ioi16p6io.cp` (115 LOC vs 36 ideal) and
`today/noi19p1.cp` (106 LOC vs 30 ideal) had to explode a Li Chao tree into 4–5
parallel global `List<Int>` arrays (`slope`, `icept`, `lcnt`, `left_c`,
`right_c`) and hand-write ~65 lines of global helper functions.

**Proposed general fix**: Support in-file `struct` declarations with methods
and local library module imports (`use local.*`) in the compiler. Since 36% of
the entire C++ corpus defines at least one `struct`, this is the single largest
LOC win for advanced DP and data-structure problems.

### P9. `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)

Comprehensions already support trailing `if cond` filters, so writing the same
clause on a `for` loop feels natural — yet `proto/cpc` rejects it:

```text
error: expected ':' (at 'if')
```

Similarly, value-returning `if ...: ... elif ...: ... else: ...` fails across
lines (`error: expected 'else' (at None)`), forcing `today/dmopc18c1p6.cp` to
pre-declare `var cost = 0` and mutate it in three statement branches:

```cp
// Today (dmopc18c1p6):
var cost = 0
if k == l - 1:
    cost = flip_cost(c, l, r, true) + prev_dp[l, r]
elif k == r:
    cost = flip_cost(c, l, r, false) + prev_dp[l, r]
else:
    cost = prev_dp[l, k] + prev_dp[k + 1, r] + flip_cost(c, l, k, false) + flip_cost(c, k + 1, r, true)
```

**Proposed general fix**: Allow an optional trailing `if cond` filter on `for`
headers (desugaring identically to a leading `if not cond: continue`, matching
comprehensions), and support multiline `if / elif / else` expressions inside
parentheses or expression blocks.

---

## Userland standard library abstractions

### P10. Generic Zeta / Möbius (SOS) transforms over `Subset<D>` parameterized by `Algebra` (`noip00p4`, `noi15p3` — 2 of 10)

Every SOS DP repeats the same bit-by-bit propagation loops — and getting the
loop order backwards (`mask` outer instead of `bit` outer) works by accident for
idempotent monoids (`or`, `max`) while silently overcounting for `Sum`:

```cp
// Today (noip00p4):
for s in Mask.rev:
    for i in Item:
        if not (i in s):
            if reach[s.with(i)]: reach[s] = true

for s in Mask:
    for i in s:
        best_sub[s] max= best_sub[s.without(i)]
```

Meanwhile, `noi15p3`'s final inclusion-exclusion over the 8 small primes is a
textbook Möbius transform on `Subset<Small>`.

**Proposed general userland stdlib fix**: Add `stdlib/subset.cp` (or extend
`stdlib/algebra.cp`) with four generic, 5-line functions over `Monoid<T>` /
`Group<T>`:

- `sos_sub(m: Monoid<T>, mut f: [Subset<D>]T)` — `f[s] = fold_{t <= s} f[t]`
- `sos_sup(m: Monoid<T>, mut f: [Subset<D>]T)` — `f[s] = fold_{t >= s} f[t]`
- `mobius_sub(g: Group<T>, mut f: [Subset<D>]T)` — inverse of `sos_sub`
- `mobius_sup(g: Group<T>, mut f: [Subset<D>]T)` — inverse of `sos_sup`

Both SOS blocks in `noip00p4` collapse to two unmistakable lines:

```cp
// Wanted (noip00p4):
sos_sup(Any, reach)
sos_sub(Max, best_sub)
```

Zero compiler special cases, reusable for SOS DP, OR/AND convolution, and
subset-sum inclusion-exclusion.

### P11. Bit-level intrinsics on `Int` (`popcount`, `ctz`, `clz`) and `Subset<D>.shifted(k)` (extends `H12`; `noip21p2`, `noi23p2` — 2 of 10; 22/710 corpus C++ files call `__builtin_popcount`, 145/710 shift bitmasks)

`Int` still has no `popcount` (`error: unknown name popcount`, `error: Int has
no '.popcount'`), so `today/noip21p2.cp` writes a 7-line `while` loop and
`today/noi23p2.cp` precomputes a `2^k` popcount array `pcnt[mask]`.

More deeply, broken-profile DP (`noi23p2`) shifts a `k`-bit sliding window by 1
on every transition. `stdlib/bitset.cp` already provides `Bitset<D>.shifted(k)`,
but `Subset<D>` has neither `shifted(k)` nor `<<`:

```text
error: `<<` between Subset<D> and Int[1..=1]; sets support `s | t`, `s & t`, `s - t`, `s ^ t`, and `s.with(x)` / `s.without(x)` for one element
```

That forced `today/noi23p2.cp` out of typed `Subset<Win>` entirely into raw
`Int` bit-twiddling:

```cp
// Today (noi23p2):
next[mask] = dp[(mask << 1) ^ lim]
...
var x = mask
while x > 0:
    let low = x & (-x)
    cur += dp[(mask ^ low) << 1]
    x = x ^ low
```

Compare with the typed version when `Subset<D>` mirrors `Bitset<D>.shifted(k)`:

```cp
// Wanted (noi23p2):
let shifted = mask.shifted(1)
if Win.last in mask:
    next[mask] = dp[shifted]
else:
    let cn = n + i + mask.len
    var cur = Z(2 * cn - 1) * dp[shifted] + Z(cn - 1) * dp[shifted.with(Win.first)]
    for bit in mask: cur += dp[mask.without(bit).shifted(1)]
    next[mask] = cur
```

**Proposed general fix**: Expose `x.popcount`, `x.ctz`, `x.clz`, and
`x.bit_len` as small core `Int` intrinsics (lowering to `__builtin_popcountll`
etc.), and mirror `Bitset<D>.shifted(k)` onto `Subset<D>` (dropping bits that
shift outside `D`).

### P12. Generic `LiChao<F, V>` and `LiChaoArena<F, V>` for tie-breaking values and multi-tree DP (`ioi16p6io`, `noi19p1` — 2 of 10; 9/710 corpus C++ files use CHT/Li Chao)

Inspecting `examples/local-lib/ds/li_chao.cp` against two real OI Li Chao DP
problems exposed two general requirements that a fixed `Line(k, m) -> Int`
single-tree class does not cover:

1. **General comparable return values (`ioi16p6io`)**: WQS (Aliens) binary
   search needs each line to return `(cost: Int, used: Int)` and compare
   lexicographically, breaking ties by minimum segment count. If `LiChao<F, V>`
   is generic over any callable/line type `F` with `f.at(x: Int) -> V` where
   `V: Ord` (or takes an `Order<V>`), the exact same Li Chao tree handles
   scalar `Int`, min vs max envelopes, and WQS `(cost, count)` tie-breaking
   with zero code duplication.
2. **Arena-backed multi-tree forests (`noi19p1`)**: Event-sweep DP over graphs
   keeps one Li Chao tree per station (`n = 10^5` trees, `m = 2e5` total
   insertions). Allocating `n` independent `List` structs wastes memory and
   breaks cache locality, whereas a `LiChaoArena<F, V>` indexed by a domain
   `Station` stores one flat node pool (`root: [Station]?NodeId`) and supports
   O(1) `clear()` across WQS iterations.

**Proposed general userland stdlib fix**: Generalize `LiChao` in `local-lib` /
`stdlib` to `LiChao<F, V>` (defaulting `F = Line`, `V = Int`) plus a thin
`LiChaoArena<D, F, V>` sharing one pool across domain `D`.

### P13. `min_by` / `argmin` returning both the best value and winning key (`dmopc18c1p6`, `noi05p4` — 2 of 10; applies to Knuth DP, D&C DP, and DP path reconstruction)

Both Knuth-optimized interval DP (`dmopc18c1p6`, storing `opt[l, r] = best_k`)
and greedy shortest-path tie-breaking (`noi05p4`, storing `step[cat, mouse]`)
need the element achieving the minimum, not just the minimum value. Today
neither `argmin` nor `min_by` exists (`error: unknown name argmin`, `error:
unknown name min_by`), forcing 6-line manual `best` + `best_k` loops:

```cp
// Today (dmopc18c1p6):
var best = 1000000000
var best_k = lo
for k in lo..=hi:
    ...
    if cost < best:
        best = cost
        best_k = k
```

While `min((cost(k), k) for k in lo..=hi)` already builds in `proto/cpc` for
simple single-expression costs, it is noisy when `cost` is a multi-branch block
and ties `Int` sentinels to tuple unpacking.

**Proposed general userland stdlib fix**: Add `min_by(iter, key)` /
`max_by(iter, key)` in `stdlib/` returning `?(value: K, item: T)` (breaking
ties by first/smallest element), matching the existing `sorted(xs, by: ...)`
convention.

### P14. `List.is_empty`, `List.dedup()`, and header-prefixed multitests (`noip00p4`, `ioi16p6io`, `noi23p2` — 3 of 10; 30/710 corpus C++ files call `unique()`)

Three small papercuts hit repeatedly across DP preprocessing:

1. `List<T>` rejects `.is_empty` (`error: List<Int> has no '.is_empty'`), even
   though `stdlib/strings.cp` calls `s.is_empty` on line 45. Both `ioi16p6io`
   and `noip00p4` had to write `.len == 0`.
2. `List<T>` has no `.dedup()` (`error: List<Int> has no method 'dedup'`),
   forcing a 4-line manual dedup loop after `sorted(...)` in `noip00p4`.
3. NOI/CSP-style multitests (`noi23p2`) begin with a test-case ID before the
   test count (`let tc_id = read(); let tcs = read()`), which `@multitest`
   cannot express without reverting to an explicit `for _ in 0..tcs:` loop.

**Proposed general userland stdlib fix**: Expose `is_empty` and `dedup()`
(consecutive deduplication on a sorted list, matching `unique`) on `List<T>` in
`stdlib/`, and allow `@multitest(count: tcs)` or a header block before
`@multitest` so test-ID-prefixed inputs keep stress-generator support.
