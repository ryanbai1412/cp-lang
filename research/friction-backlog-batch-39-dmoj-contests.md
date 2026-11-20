# Friction Backlog — Scout Batch 39: Hard DMOJ Contest Classics (`aac`, `yac`, `wac`, `sac`, `dmpg`)

Scout Batch 39 ported ten unported hard problems from the `aac*`, `yac*`, `wac*`, `sac*`, `bts*`, and `dmpg*` corpus bucket (zero `bts*` submissions exist in `corpus/`) against the locked **v2 spec** (`docs/one-page-cheat-sheet.md`, commit `3723dca`), alongside runnable `today/` implementations differentially verified against the original AC C++ submissions on 1,500 random small/medium inputs under ASan + UBSan (`proto/ports/scout_batch_39/check.py`).

## 1. Dual LOC Summary

Non-blank, non-comment LOC across all ten problems, separating raw C++ from library-stripped C++ (excluding debug macros, fast I/O, `main()` boilerplate, and standard reusable CP data structures: `DSU`, `RMQ`, `Tree`, `LazySegTree`, `chash`, and rolling-hash helpers):

- **`aac4p6`** (`4227368.cpp`, deque sweep + rotation): 181 raw / 48 stripped -> **27 v2** (-85.1% raw, -43.8% stripped), 46 today.
- **`aac4p5`** (`4140767.cpp`, constructive SCC graph via DSU + heap merge): 169 raw / 114 stripped -> **49 v2** (-71.0% raw, -57.0% stripped), 84 today.
- **`yac7p4`** (`5705324.cpp`, dynamic active subtree with Euler order + lifting): 226 raw / 118 stripped -> **58 v2** (-74.3% raw, -50.8% stripped), 150 today.
- **`yac6p6`** (`5150025.cpp`, offline 2D grid DP with up to 5 skips): 170 raw / 137 stripped -> **72 v2** (-57.6% raw, -47.4% stripped), 126 today.
- **`yac7p3`** (`5705218.cpp`, functional graph cycle decomposition + rerouting): 122 raw / 95 stripped -> **58 v2** (-52.5% raw, -38.9% stripped), 83 today.
- **`wac6p6`** (`5122001.cpp`, multi-test interactive 3-color bin strategy): 113 raw / 79 stripped -> **43 v2** (-61.9% raw, -45.6% stripped), 70 today.
- **`sac22cc4p4`** (`4411200.cpp`, rethought from 280-LOC `LazySegTree` to O(n) `MonoDeque` two-pointers): 280 raw / 34 stripped -> **19 v2** (-93.2% raw, -44.1% stripped), 33 today.
- **`sac22cc5p5`** (`4575631.cpp`, distinct `k x k` subgrids via 2D double rolling hash): 140 raw / 45 stripped -> **14 v2** (-90.0% raw, -68.9% stripped), 62 today.
- **`dmpg16g2`** (`4653887.cpp`, 5-letter inclusion-exclusion over DSU zero-runs): 166 raw / 113 stripped -> **58 v2** (-65.1% raw, -48.7% stripped), 98 today.
- **`dmpg18s5`** (`3476383.cpp`, online divisor-count queries via sqrt decomposition): 132 raw / 116 stripped -> **36 v2** (-72.7% raw, -69.0% stripped), 79 today.

**Totals:** 1,699 C++ raw / 899 C++ stripped -> **434 CP Lang v2** (**-74.5% vs raw, -51.7% vs library-stripped**) and **831 runnable `today/`** (**-51.1% vs raw, -7.6% vs stripped**).

The **397-LOC gap** between v2 (434) and `today/` (831) is itself diagnostic: almost all of that inflation comes from six unimplemented v2 features in `proto/cpc` — `Rooted<V>` (+92 LOC in `yac7p4`), `RollingHash` (+48 LOC in `sac22cc5p5`), 3D/2D product-domain arrays (+54 LOC in `yac6p6`), `Deque` / `MonoDeque` (+33 LOC across `aac4p6` and `sac22cc4p4`), `Subset<Letter>` + struct methods (+40 LOC in `dmpg16g2`), and labeled `break 'label` + `MaxHeap` + `compress`/comprehensions (+35 LOC in `aac4p5`, +43 LOC in `dmpg18s5`).

---

## 2. Locked v2 Spec Friction & Multi-Feature Collisions

These issues exist in the locked v2 spec (`docs/one-page-cheat-sheet.md`, `3723dca`) itself — not just in `proto/cpc`.

### F39.1 — `Rooted<V>` Euler-order domain spelling, inverse lookup, and `lift_while` polarity (`yac7p4`)

**Collision (`Rooted<V>` × nominal domains × `Set` / `SegTree`):** Section 10 specifies `.sub(v) -> Range<Pre>` on `Rooted<V>(g, root)`, unlike `compress(all)` which explicitly writes `xs.D`. There is no stated spelling for the preorder domain when declaring `Set<t.Pre>` or `SegTree<t.Pre, ...>`, no single-vertex entry lookup (`t.tin(v)` or `t.pos(v)`), and no stated guarantee that `t.preorder` is indexed by `t.Pre` rather than `0..n`. In `yac7p4.cp`, every Euler operation had to guess:

```cp
let t = Rooted<V>(g, V.first)
var tins = Set<t.Pre>()                // is the domain `t.Pre`, `Pre`, or `0..n`?
tins.add(t.sub(v).first!)              // awkward proxy for single-vertex tin(v)
let top = t.preorder[tins.first!]      // assumes `t.preorder: [t.Pre]V`
```

In the same function, `t.lift_while(u, pred)` is underspecified on two contest-critical edge cases: whether `pred` is tested on `u` itself or only strict ancestors, and what it returns when `pred(u)` is false (`?V` vs `V`). In `yac7p4.cp`, finding the first inactive edge below the active subtree collapses to two lines *only* if `lift_while` returns the highest ancestor (including `u`) satisfying `pred`:

```cp
let below = t.lift_while(u, v => not has_active_sub(v))
(t.parent(below)!, below)
```

**Recommendation:** Align `Rooted<V>` with `compress`: document `t.Pre` as the associated domain, add `t.pos(v) -> t.Pre` (so `t.preorder: [t.Pre]V` is its exact inverse), and pin `t.lift_while(u, pred) -> V` to "highest ancestor `w` of `u` (possibly `u` itself) while `pred(t.parent(w))` holds" — or explicitly return `?V` if `pred(u)` itself is checked.

### F39.2 — `?Int` arithmetic boilerplate inside multi-dimensional DP transitions (`yac6p6`)

**Collision (`?T` no-implicit-lifting × `min=` × product-domain DP):** Section 3 keeps `?T` strict (no implicit `Ord`, no lifted `+`) while Section 4 makes `min=` lift over `?T` (`?Int min= ?Int` and `?Int min= Int`). That works cleanly for direct copies (`dp[r, c, k] min= dp[nr, c, k + 1]`), but breaks down the instant a transition adds an edge or cell cost `a[r, c]`. Every weighted DP transition in `yac6p6.cp` needed a nested `if let` guard — repeated twelve times across the three DP loops:

```cp
// Direct skip transition works cleanly with ?Int min= ?Int:
if k < kv: dp[r, c, k] min= dp[nr, c, k + 1]

// Weighted transition forces manual unwrapping on every cell:
if let nxt = dp[nr, c, k]: dp[r, c, k] min= nxt + a[r, c]
```

And the final meet-in-the-middle combine needed a double `if let`:

```cp
if let p1 = dp[qry.x1, qry.y1, k] and let p2 = dp2[dr, dc, qry.kv - k]:
    qans[qry.id] min= p1 + p2
```

This is a recurring multi-problem tax across shortest-path and DP problems whenever `?Int` replaces `INF`. Notice also that `yac6p6.cp` had to mix nominal domains (`Row`, `Col`) with a structural skip-count domain (`0..=MAXK`) in `[Row, Col, 0..=MAXK]?Int` because `k` needs integer arithmetic (`k + 1`, `kv - k`) without `?` unwrapping.

**Recommendation:** Either keep the rule (explicitness prevents silent `INF + w` overflow bugs) and document `x.map(v => v + w)` / `dp[r, c, k] min= dp[nr, c, k].map(_ + a[r, c])` on the cheat sheet, or allow explicit `?.` / lifted `min=` helper — without compromising the no-silent-`Ord` guardrail.

### F39.3 — `RollingHash` is string-only; 2D and integer-sequence hashing are stranded (`sac22cc5p5`)

**Collision (`RollingHash` × `[Row, Col]Char` × hash value type):** Section 10 defines `RollingHash(s)` exclusively under Strings. In `sac22cc5p5.cp`, standard 2D hashing hashes each row of length `m` (a `[Col]Char` slice via `g.row(r)`) and then hashes each column of those `k`-wide row hashes (a `[Row]U64` or `[Row]HashVal` array):

```cp
let row_h = [Row] r => RollingHash(g.row(r))
var seen: List<U64> = []
for c in 0..=m - k:
    let col_vals = [Row] r => row_h[r].get(c..c + k)
    let col_h = RollingHash(col_vals)
    for r in 0..=n - k:
        seen.push(col_h.get(r..r + k))
```

Three things are unspecified in v2: (1) whether `RollingHash` accepts any slice `[D]T` where `T` is `Char`, `Int`, `U64`, or a hash token; (2) what concrete type `.get(l..r)` returns (`U64`, `(U64, U64)`, or a named `HashVal` implementing `Eq + Ord + Hash`) so users can declare `List<...>` or `HashSet<...>`; and (3) whether two `RollingHash` instances share or separate their random bases (critical for 2D hashing, where row and column bases must be independent to avoid transpose collisions!).

**Recommendation:** Generalize `RollingHash` to any `[D]T` (`Char`, `Int`, `U64`, `HashVal`), name the returned token type `HashVal` (or `U64` packed double-mod), and salt each `RollingHash` constructor (or give `RollingHash.with_base` / separate 1D vs 2D salt) so 2D hashing is 14 lines with zero manual modular arithmetic.

### F39.4 — Missing `Dsu.groups()` and functional-graph `.cycles()` (`aac4p5`, `yac7p3`)

**Gap (`Dsu<V>`):** In `aac4p5.cp`, after merging components down to `x` roots, collecting vertices per component requires 7 lines of manual root-to-bucket reindexing (`comp_id: [V]?Int`, pushing empty lists, looping over `V`):

```cp
var comp_id: [V]?Int = none
var comps: List<List<V>> = []
while let (_, r) = pq.pop():
    comp_id[r] = comps.len
    comps.push([])
for v in V:
    comps[comp_id[dsu.find(v)]!].push(v)
```

Adding `dsu.groups() -> List<List<V>>` (matching AtCoder Library's `dsu::groups()`) eliminates that boilerplate across almost every constructive/component-DP DSU problem.

**Gap (`Perm<V>` vs functional graph `[V]V`):** In `yac7p3.cp`, the input is a functional graph `par: [V]V` (out-degree 1, not a bijection). Section 10 provides `Perm<V>.cycles()`, which cannot be used here because `par` has trees feeding into cycles — forcing a 25-line three-color DFS `find_cycles` written twice in spirit (once before rewiring `par[u_ind] = u_nv` and once after). Even a small stdlib helper `functional_cycles(par: [V]V)` or `Graph.directed(...).scc()` (remembering that a functional graph's cycles are simply its non-trivial SCCs plus self-loops) would cut `yac7p3.cp` by ~20 LOC. Note that `Graph.directed(...).scc()` works today in v2 only if you construct a `Graph` just to find cycles and still must special-case self-loops of size 1 (`par[v] == v`).

### F39.5 — `Subset<D>` iteration, `[Mask]Struct` mutation, and `@tags` char-to-enum decoding (`dmpg16g2`, `wac6p6`)

**Collision (Algebraic `Index` × `Subset<D>` × mutable struct arrays):** In `dmpg16g2.cp`, `type Mask = Subset<Letter>` has `2^5 = 32` masks and we keep a `[Mask]ZeroRuns` array of 32 mutable DSU-backed structs. Section 3 lists `Subset<D>` as an algebraic `Index` (`D.K <= 25`), which cleanly enables `[Mask]ZeroRuns` and `for mask in Mask`, but two points are underspecified:
1. Does `for mask in Mask` iterate in ascending popcount / integer order, and can `ch in mask` test membership on an enum `Letter` directly?
2. Calling `ds[mask].erase(c)` mutates a struct stored inside an array `[Mask]ZeroRuns`. Lexical borrow rules allow `mut self` methods on an array slot, so confirming `arr[i].mut_method(...)` borrows only `arr` (and is compatible with `for mask in Mask if ch in mask: ds[mask].erase(c)`) is essential for real DP/sweep structs.

**Collision (`@tags` char-to-enum conversion & small enum-indexed array literals):**
- In `dmpg16g2.cp`, input grid cells are `'.'` or `'A'..'E'`. Because `@tags` only applies to `read()` of a whole enum, converting a `Char` already read inside `[Row, Col]Char` back to `Letter` falls back to `Letter.from_idx(g[bot, c] - 'A')!`. Allowing `enum Cell = Empty('.') | Piece(Letter)` with `@tags` or `Letter.try_from(ch)` would remove all ASCII arithmetic.
- In `wac6p6.cp`, initializing a 3-element `[Color]?Int` array with distinct values for `R`, `G`, `B` had to use a 4-line comprehension-plus-`match`:

```cp
var bin: [Color]?Int = [Color] col => match col:
    R => 1
    G => 2
    B => none
```

Allowing a keyed array literal `[R: 1, G: 2, B: none]` (or exhaustive `[Color: R => 1, G => 2, B => none]`) for small enum domains would remove that ceremony without adding a new core concept.

### F39.6 — Number-theory return shapes and missing `divisors(x)` (`dmpg18s5`)

**Gap (`sieve(n)` / `factor(x)` / `divisors`):** Section 10 lists `sieve(n)` and `factor(x)` by name only, without specifying their return types (does `sieve(n)` return primes, an SPF array, or a `Bool` mask? Does `factor(x)` return prime factors with multiplicities `List<(Int, Int)>` or flat primes?). In `dmpg18s5.cp`, we need all positive divisors of every value up to `200_000` (and in single-query problems, `divisors(x)`). Writing the manual harmonic sieve took 5 lines:

```cp
var divs: [0..=MAXV]List<Int> = [0..=MAXV] _ => []
for d in 1..=MAXV:
    for v in (d..=MAXV).step(d):
        divs[v].push(d)
```

Documenting the exact signatures of `sieve(n)` and `factor(x)` (and whether `divisors(x)` lives alongside `factor(x)`) removes guesswork under contest time pressure.

### F39.7 — Small ergonomic papercuts in `Deque`, `MonoDeque`, and `rotate` (`aac4p6`, `sac22cc4p4`)

Three minor but high-frequency friction points surfaced in our two deque problems:

1. **Discarding `Deque.pop()` / `pop_front()` results (`aac4p6`):** `dq.pop()` returns `?Int`. If v2 has an unused-value warning on `?T`, popping inside `while let x = dq.last and x >= m - i: dq.pop()` forces `let _ = dq.pop()` everywhere. Explicitly exempting mutating `.pop()` / `.pop_front()` from unused-result warnings (or permitting bare statement calls) avoids noise.
2. **Comparing `?Int` against `Int` (`aac4p6`):** `if dq.last != m - i - 1:` is far cleaner than `if dq.last != some(m - i - 1):` or `if let x = dq.last { x != m - i - 1 } else { true }`. Clarifying whether `==`/`!=` auto-lifts `T` to `?T` (note: `==`/`!=` is equality, not `Ord`, so it does not violate the no-implicit-`?T`-ordering rule!) eliminates a common contest typo.
3. **No `rotate_right` (`aac4p6`) and `MonoDeque.best!` double unwrap (`sac22cc4p4`):** `rotate_left(mut a, (-k) % n)` works nicely *because* CP Lang uses Euclidean `%` (`(-k) % n` is always in `0..n`), which is a genuine win for Euclidean modulo — though `rotate_right(mut a, k)` reads much faster under contest stress. In `sac22cc4p4.cp`, `while hi.best! - lo.best! > limit:` requires `!` on both deques even though `push(at: r, ...)` just guaranteed non-emptiness; acceptable given uniform optionality, and worth keeping as-is rather than adding unchecked `.best_unchecked` methods.

---

## 3. Prototype (`proto/cpc`) Implementation Gaps Hit While Building `today/`

None of `crates/` or `proto/cpc/` were modified. While writing the ten runnable `today/` ports, we hit the following concrete divergences between `proto/cpc` and the locked v2 spec:

- **Lexer crash on labeled loops (`aac4p5`):** `'intra: for ...` threw `ValueError: substring not found` in `proto/cpc/lexer.py` because `'` is lexed exclusively as a character literal. Workaround: nested `if eu.len == m: break` guards.
- **`Set` method naming mismatch (`yac7p4`):** v2 specifies `.add(x)` / `.remove(x)`, whereas `proto/cpc` implements `.insert(x)` / `.remove(x)` (error: `no method add on Set<V>`).
- **Missing v2 data structures in `proto/cpc`:** `Rooted<V>`, `RollingHash`, `Deque<T>`, `MonoDeque<T>`, `MaxHeap<T>`, `Subset<D>`, `rotate_left`, `@multitest`, `@tags`, and 3D arrays (`[Row, Col, 0..=MAXK]Int`) are not yet implemented in `proto/cpc`, accounting for nearly all of the 397-LOC delta between v2 (434 LOC) and `today/` (831 LOC).

---

## 4. Verification Status

All ten `today/` ports compile cleanly with `proto/cp.py build` (run on temporary copies so tracked `.cp` files are never stamped) and `g++ -std=c++17 -O2 -fsanitize=address,undefined`, and pass 150 small/medium random differential test cases each (1,500 total) via `python3 proto/ports/scout_batch_39/check.py` with zero ASan/UBSan findings:

- `aac4p5` uses a dedicated Tarjan SCC constructive validator in `check.py`.
- `aac4p6`'s original `4227368.cpp` allocates `int data[3_000_000]` (12 MB) on the stack and segfaults under the default 8 MB Linux stack limit; `check.py` raises `RLIMIT_STACK` only for the reference C++ binary (the CP Lang port passes under the default stack).
- `wac6p6` was verified offline against `5122001.cpp`'s exact command stream across randomized 100-ball inputs covering all three strategy branches (234 / 459 / 78 test cases per branch), rather than against a live DMOJ interactor process.
- Max-N runtime and peak RSS judge benchmarks were not run as part of this scout batch.
