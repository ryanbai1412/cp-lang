# Friction Backlog — Scout Batch 43: Hard ETSR '23 Classics (`etsr*`)

**Scope and provenance.** Batch 43 was originally scoped to `dmopc14*`–`dmopc19*`, but auditing `corpus/submissions/info.json` showed only two entries in that range (`dmopc18c1p6`, `dmopc19c4p2`), both already ported in Batches 14 and 31. The batch was redirected to the unclaimed `etsr*` bucket: 10 diverse, previously unported hard problems from `etsr23r1p3` through `etsr23r4p3` (excluding `etsr23r1p2` from Batch 6 and `etsr23r3p2` from Batch 5).

Every problem was rethought idiomatically against the locked **v2 spec** (`docs/one-page-cheat-sheet.md`, commit `3723dca`) under `proto/ports/scout_batch_43/<problem>.cp`, paired with a runnable port under `proto/ports/scout_batch_43/today/<problem>.cp`. All 10 runnable ports passed **300/300** randomized and edge-case differential checks against the compiled original `g++ -O2` AC C++ submissions via `python3 proto/ports/scout_batch_43/check.py`.

---

## Dual LOC Measurement

Counting non-blank, non-comment lines across all 10 submissions, raw C++ totals **1,455 LOC** and library-stripped C++ totals **798 LOC** (after stripping `#include`/typedef/fast-IO headers, `main()` boilerplate, generic lazy segment trees in `5527625.cpp` and `5529698.cpp`, the persistent segment-tree DSU in `5521298.cpp`, and the `SPow` precomputation cache in `5529698.cpp`; the Segment Tree Beats in `5495163.cpp` is kept in library-stripped LOC because a standard lazy segment tree cannot perform node-extrema-guarded division).

Against those two baselines, the 10 locked-v2 CP Lang ports total **376 LOC** — a **74.2%** reduction vs raw C++ and a **52.9%** reduction vs library-stripped C++ — while the 10 runnable `today/` ports total **509 LOC** (**65.0%** raw reduction, **36.2%** library-stripped reduction).

Per-problem breakdown (raw C++ -> library-stripped C++ -> v2 CP Lang -> `today/` runnable):

- **`etsr23r1p3`** (`5494303.cpp`, subset DP over backward occurrence table): 79 -> 41 -> **24** v2 (**69.6%** raw / **41.5%** stripped) / 37 `today/`.
- **`etsr23r1p4`** (`5495163.cpp`, Segment Tree Beats for range add + range Euclidean floor division): 216 -> 176 -> **89** v2 (**58.8%** raw / **49.4%** stripped) / 85 `today/`.
- **`etsr23r2p2`** (`5494433.cpp`, two-layer interval expression DP): 90 -> 53 -> **33** v2 (**63.3%** raw / **37.7%** stripped) / 58 `today/`.
- **`etsr23r2p3`** (`5494453.cpp`, `ctz` divisibility test on sorted triples): 64 -> 29 -> **6** v2 (**90.6%** raw / **79.3%** stripped) / 9 `today/`.
- **`etsr23r2p4`** (`5494510.cpp`, `@memo(hash)` bounded-window XOR scheduling DP): 87 -> 48 -> **22** v2 (**74.7%** raw / **54.2%** stripped) / 38 `today/`.
- **`etsr23r3p1`** (`5527625.cpp`, clamped affine + linear-in-initial-x `LazySegTree` with `max_right`/`min_left`): 185 -> 112 -> **53** v2 (**71.4%** raw / **52.7%** stripped) / 66 `today/`.
- **`etsr23r3p3`** (`5521298.cpp`, NOI 2018 Return — Dijkstra + Kruskal Reconstruction Tree + `lift_while`): 194 -> 60 -> **34** v2 (**82.5%** raw / **43.3%** stripped) / 52 `today/`.
- **`etsr23r4p1`** (`5521420.cpp`, `Perm<V>.cycles()` + `Subset<Big>` bracket construction + co-located `@checker`): 117 -> 83 -> **36** v2 (24 solution + 12 `@checker`; **69.2%** raw / **56.6%** stripped) / 53 `today/`.
- **`etsr23r4p2`** (`5529698.cpp`, Extended Euler totient tower + runtime `Mod<p>` + vector-shift `LazySegTree`): 317 -> 124 -> **54** v2 (**83.0%** raw / **56.5%** stripped) / 67 `today/`.
- **`etsr23r4p3`** (`5635626.cpp`, ZJOI 2016 Small Stars — `Subset<V>` inclusion-exclusion + `Rooted<V>` tree DP in `U64`): 106 -> 72 -> **25** v2 (**76.4%** raw / **65.3%** stripped) / 44 `today/`.

---

## Where v2 Shone

Several locked v2 features directly erased entire classes of C++ bug sources in this batch without extra ceremony:

1. **Euclidean `/` and `%` killed negative-division footguns (`etsr23r1p4`).** The original C++ submission (`5495163.cpp`) needed a custom `divf(long a, long b)` helper with `assert(b > 0)` and manual `a / b - (a % b != 0 && a < 0)` sign patching so range floor-division worked on negative values. In CP Lang, `node.mn / x` is already Euclidean floor division, which matched `divf` across 30/30 negative-heavy randomized test cases on the first run.
2. **Coordinate compression + `LazySegTree` `max_right`/`min_left` (`etsr23r3p1`).** Rethinking `5527625.cpp` by tracking `(lo_orig, lo_val, hi_orig, hi_val)` inside each segment node turned two O(log^2 n) manual binary searches over point queries into two one-line O(log n) tree walks (`st.max_right` and `st.min_left`), with empty boundary intervals (`U.first.l..left_end`) handled cleanly without `-1`/`n` sentinel guards.
3. **Algorithmic rethinking with `Rooted<K>` and `Perm<V>` (`etsr23r3p3`, `etsr23r4p1`).** Replacing the 110-line persistent segment-tree DSU in `5521298.cpp` with a Kruskal Reconstruction Tree + `ktree.lift_while` dropped the full solution from 194 raw C++ LOC (60 stripped) to 34 v2 LOC, and `p.cycles()` in `etsr23r4p1` replaced manual visited-array DFS completely while co-locating the original submission's inline bracket verifier as a first-class `@checker`.

---

## Friction Backlog & Multi-Feature Collisions

### ET1 (Spec — Optionals): `min=` / `max=` on `?T` cannot disambiguate "uninitialized" from "absorbing failure"
**Observed in:** `etsr23r1p3.cp`, `etsr23r2p2.cp`, `etsr23r2p4.cp`.

In DP transitions (`etsr23r2p2`, `etsr23r2p4`), `var best: ?Int = none` means "+infinity / not yet relaxed", so `best min= cand` should treat `none` as the identity element (`best = cand` when `best == none`, and ignoring `cand` if `cand` is `?Int` and `none`). In `etsr23r1p3`, however, `dp[mask]: ?Int` takes the minimum over every character `c in mask` where **any** missing predecessor invalidates the whole state (`none` is absorbing, not identity):

```cp
var best: ?Int = n
for c in Alpha if c in mask:
    if let prev_pos = dp[mask.without(c)] and let hit = prv[prev_pos, c]:
        best min= hit
    else:
        best = none
        break
```

Because `best` starts non-`none` (`n`) and transitions to `none` on failure, an overloaded `min=` that silently ignores `none` works for DP relaxation but would mask bugs if a user wrote `best min= prv[prev_pos, c]` expecting `none` to poison `best`.

**Recommendation:** Keep `x min= y` on `?T` strictly defined for `x: ?T, y: T` (where `none` on the left is the identity and `y` is non-optional), and reject `x min= y` when `y` is `?T` so the programmer must write `if let v = y: x min= v` (ignore-none) or handle the `else` branch (absorbing-none). That gives zero ambiguity and zero surprises under contest pressure.

---

### ET2 (Stdlib — Data Structures): Segment Tree Beats breaks `LazySegTree`'s stateless `Action<T, F>` contract
**Observed in:** `etsr23r1p4.cp` (89 v2 LOC — nearly 25% of the entire batch's v2 LOC!).

Standard `LazySegTree<D, T, F>` applies a tag `f: F` to any node fully covered by the query range unconditionally. Segment Tree Beats (`etsr23r1p4`, range add + range floor-division) cannot use `LazySegTree` at all because range division must inspect `node.mn` and `node.mx` at the covered node to decide whether division reduces to a uniform add (`node.mn - node.mn / x == node.mx - node.mx / x`), a uniform set (`node.mn / x == node.mx / x`), or must recurse into both children:

```cp
if node.mn / x == node.mx / x:
    put(o, l, r, Tag(node.mn / x, 0))
    return
if node.mn - node.mn / x == node.mx - node.mx / x:
    put(o, l, r, Tag(none, node.mn / x - node.mn))
    return
// Otherwise must recurse even though [l, r] is fully inside [ql, qr]!
```

Because `LazySegTree` exposes no conditional-apply hook, `etsr23r1p4.cp` had to hand-roll the entire 4N heap-indexed recursive tree (`build`, `push`, `put`, `range_add`, `range_div`, `query`), wiping out the abstraction gain.

**Recommendation:** Add a single method to `LazySegTree` — `st.apply_beats(range, try_tag: (Node) -> ?F)` — which recurses past a covered internal node whenever `try_tag(node)` returns `none` and applies the returned `F` tag when it returns `Some(f)` (and at leaves). That keeps `Action<T, F>` unchanged for normal problems while shrinking Segment Tree Beats solutions like `etsr23r1p4` from 89 LOC down to ~28 LOC.

---

### ET3 (Multi-feature collision): `@memo(hash)` lifetime across `@multitest` + sliding-window `Subset<D>`
**Observed in:** `etsr23r2p4.cp`.

When `@memo(hash)` is attached to a nested recursive function inside a `@multitest` script, the memoized function captures test-local inputs (`n`, `items`):

```cp
@multitest
let n: 1..=1000 = read()
domain Item = 0..n
domain Win = 0..15
let items: [Item](val: Int, tol: 0..=7) = read()

@memo(hash)
fn dp(ind: Int, prv: ?Item, used: Subset<Win>) -> Int:
    if ind >= n: return 0
    if Win.first in used:
        return dp(ind + 1, prv, used.shifted(-1))
```

Two issues collided here:
1. **Per-test reset guarantee:** If the compiler lowers `@memo(hash)` to a function-static `unordered_map` in C++17 without clearing it at the start of each `@multitest` iteration (or tying the cache lifetime to the enclosing block), test case 2 silently reads cached answers from test case 1. The v2 spec should explicitly state that `@memo` caches live in the scope where the `fn` is declared (so any `@memo` inside `@multitest` is fresh per test case, with zero stale-state risk).
2. **Sliding-window bit operations on `Subset<D>`:** Rolling-window bitmask DPs (like `etsr23r2p4`'s 15-element lookahead window) naturally want `used.shifted(-1)` (or `used >> 1`) to advance the window by one position while dropping out-of-range bits. `docs/one-page-cheat-sheet.md` lists `mask.with(i)` / `mask.without(i)` / set operators, but has no shift operation on `Subset<D>`, forcing either a new method (`shifted(k)`) or dropping to raw `Int` bitmasks.

---

### ET4 (Spec — Domains & `Str`): Mixing `Str` indices with a named position domain forces `.to_idx` and `(r - 1)!`
**Observed in:** `etsr23r2p2.cp`, `etsr23r1p3.cp`.

`etsr23r2p2` defines `domain Pos = 0..n` for its DP tables (`num[l, r]`, `dp1[l, r, v]`, `dp2[l, r, v]`), while `s: Str` is indexed by `0..s.len`. Inside the precomputation loop, every character lookup had to unwrap `Pos` back to a raw index, and stepping left by 1 inside `for r in l.r..:` needed a force-unwrap `(r - 1)!`:

```cp
for l in Pos.rev:
    num[l, l] = s[l.to_idx] - '0'
    for r in l.r..:
        num[l, r] = min(CAP + 1, num[l, (r - 1)!] * 10 + (s[r.to_idx] - '0'))
```

Two small improvements would clean this up without adding new concepts:
1. Let `domain Pos = s.D` (matching `domain U = xs.D` from `compress`) so `s[l]` and `s[r]` accept `Pos` directly without `.to_idx`.
2. Let the range checker prove `r - 1 in Pos` automatically whenever `r` iterates over `l.r..` (since `l.r >= Pos.first + 1`), eliminating the noisy `(r - 1)!` force-unwrap inside every bottom-up interval DP.

---

### ET5 (Multi-feature collision): Dynamically constructed trees (`Kruskal Reconstruction Tree`) leak raw `?Int` node IDs before `.freeze()`
**Observed in:** `etsr23r3p3.cp`.

Building a Kruskal Reconstruction Tree creates `n` leaf nodes and up to `n - 1` internal merge nodes dynamically in sorted edge order, then roots the finished tree with `Rooted<K>` for online `lift_while` queries:

```cp
var nodes: List<KNode> = [KNode(none, none, none, dist[v] ?? Int.MAX) for v in V]
...
let k = nodes.push_id(KNode(a, b, e.alt, min(nodes[a].mn, nodes[b].mn)))
...
let (K, kn) = nodes.freeze()
let ktree = Rooted<K>(u => [K(c) for c in [kn[u].left, kn[u].right] if let c = c], root: K.last)
```

Because domain `K` only comes into existence *after* `nodes.freeze()`, the child pointers stored inside `KNode` (`left`, `right`) must be raw `?Int`, and every read back into `Rooted<K>` casts `K(c)`. This is the exact same pattern that appears in tries, suffix automata, Cartesian trees, and segment-tree node pools. Documenting `nodes.push_id(...)` + `let (K, kn) = nodes.freeze()` (or a small `std.tree.kruskal_tree` helper for KRTs) would make this idiom uniform and zero-boilerplate across tree problems.

---

### ET6 (Multi-feature collision): Fixed-size array `[Slot]Z` inside `LazySegTree` + clamping domain index shifts
**Observed in:** `etsr23r4p2.cp`.

`etsr23r4p2` stores a runtime-sized fixed-width vector `[Slot]Z` (where `domain Slot = 0..(phis.len + 1)`, at most ~30 slots) as the monoid value of a `LazySegTree`, with a lazy shift-and-clamp action:

```cp
let shift_act = Action<[Slot]Z, Int>(
    0,
    (a, b) => min(a + b, K - 1),
    (k, x, _) => [Slot] s => x[Slot(min(s.to_idx + k, K - 1))],
    over: node_m,
)
```

Two subtle points to lock down for the Rust compiler and C++17 codegen:
1. **Value semantics + small-buffer layout for `[D]T` as a generic type parameter:** Passing `[Slot]Z` as the `T` parameter of `LazySegTree<I, [Slot]Z, Int>` must deep-copy cleanly (or lower cleanly when `Slot` is small/bounded) without aliasing bugs or quadratic heap allocations per pushdown.
2. **Clamping domain arithmetic:** `Slot(min(s.to_idx + k, K - 1))` repeats the domain's upper bound `K - 1` manually; a helper `s.sat_add(k)` (or `Slot.clamp(s.to_idx + k)`) avoids off-by-one typos when clamping indices into a domain. (Note: runtime `type Z = Mod<p>` now works end-to-end in `stdlib/mod.cp` as of `84bdddf` and passed 30/30 differential tests live in `today/etsr23r4p2.cp`!)

---

### ET7 (Numeric Core & Prelude): Wrapping `U64` is genuinely irreducible, and `prod(...)` belongs next to `sum(...)`
**Observed in:** `etsr23r4p3.cp` and `today/etsr23r4p3.cp`.

In `etsr23r4p3` (ZJOI 2016 Small Stars, `n <= 17`), intermediate DP values on a dense graph (`K_17`) can reach `16^16 = 2^64`, overflowing signed 64-bit `Int` (`2^63 - 1`), while the inclusion-exclusion result is guaranteed to fit in 64 bits — so `5635626.cpp` uses `unsigned long long` (`u64`) wrapping arithmetic.

When we tried to emulate wrapping `U64` in `today/etsr23r4p3.cp` using signed `Int` and `% (1 << 64)`, `proto/cpi` caught **two** immediate runtime traps:
- `1 << 64` raised `shift by 64`, and
- `(ans - total) % 18446744073709551616` raised `Int overflow: -4 % 18446744073709551616 does not fit in 64 bits` (because Euclidean `%` returns `2^64 - 4 > Int.MAX`).

This proves empirically that `U64` (with native wrapping `+`, `-`, `*`) is a **non-negotiable core type**, not something users can emulate cleanly in signed `Int`. Two companion ergonomics noticed in the same loop:
- Alternating inclusion-exclusion signs cannot write `-total` on `U64`, requiring `if parity == 0: ans += total else: ans -= total` (actually clearer, worth noting explicitly in the spec).
- Adding `prod(...)` to the prelude alongside `sum(...)` turns the 4-line inner child product loop into a single expression: `dp[u, v1] = prod(sum(dp[c, v2] for v2 in allowed if has_edge[v1, v2]) for c in tree.children(u))`.

---

### ET8 (`proto/cpi` Prototype Gaps Surfaced During Verification)
After rebasing onto `origin/main` (`84bdddf` / `e39686c`), `LazySegTree.max_right`/`min_left`, runtime `type Z = Mod<p>`, and `Rooted.lift_while`/`postorder`/`children` all ran live in `today/` and passed 300/300 differential checks. Four remaining `proto/cpi` parser/runtime gaps still required workarounds in `today/` (without touching `proto/cpc/` or `crates/`):

1. **Compound shift assignments `>>=` / `<<=` do not parse** (`today/etsr23r2p4.cp`, `today/etsr23r4p2.cp` — had to write `e = e >> 1`).
2. **Multi-line `if ... elif ... else:` expressions fail to parse** (`today/etsr23r3p1.cp` — had to fall back to `var tag = ...; if ... elif ...`).
3. **`Str` lacks `.join(...)`** (`today/etsr23r4p1.cp` — fell back to string concatenation `out = out + "("`).
4. **No `U64` type in `proto/cpi`** (`today/etsr23r4p3.cp` verified exact signed `Int` for `n <= 8`; full `n = 17` requires wrapping `U64`).
5. **Subtle `Rooted.lift_while` contract worth highlighting in the cheat sheet:** `stdlib/tree.cp`'s `lift_while(v, pred)` returns `none` when `pred(v)` is false on the starting vertex itself (so KRT leaves whose `gate` is `none` must make `pred` return `true` on `none`, not `false`).

---

## Priority Recommendations for the Language Architect

1. **High priority — Add `LazySegTree.apply_beats(range, try_tag: (Node) -> ?F)` (ET2):** Biggest single LOC and ergonomic lever in this batch (~60 LOC saved on `etsr23r1p4` alone, and covers every Segment Tree Beats problem without complicating standard `Action<T, F>`).
2. **High priority — Lock `@memo` lexical lifetime inside `@multitest` (ET3) and keep native wrapping `U64` (ET7):** Both prevent silent wrong answers or impossible workarounds on real contest problems.
3. **Medium priority — Lock `min=`/`max=` on `?T` to non-optional RHS only (ET1), expose `s.D` on `Str` (ET4), add `Subset<D>.shifted(k)` (ET3), and add `prod(...)` to the prelude (ET7):** Small, zero-concept-budget polish items that directly eliminate remaining boilerplate and footguns.
4. **Low priority (frontend/prototype) — Parse `>>=`/`<<=` and multi-line `if`/`elif` expressions, add `Str.join`, and document `lift_while`'s starting-vertex predicate semantics in `docs/one-page-cheat-sheet.md` (ET8).**
