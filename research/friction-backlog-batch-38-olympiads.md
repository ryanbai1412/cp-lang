# Friction Backlog — Scout Batch 38 (Hard Olympiad & DP Classics)

Scout Batch 38 ports 10 previously unported hard olympiad and DP classics from the strict prefix partition (`dp*`, `btoi*`, `ceoi*`, `egoi*`, `apio*`, `joisc*`, `joi*`) against the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md`, commit `3723dca`), with runnable counterparts in `proto/ports/scout_batch_38/today/` verified by `proto/ports/scout_batch_38/check.py`.

All 10 runnable ports pass 414 total differential, brute-force, and semantic validation cases against their accepted C++ originals (9 compiled via `proto/cpc` + `g++ -O2`; `dpf` executed via `proto/cp.py lib-run` because `proto/cpc` currently treats `Str` as opaque). Across the 10 problems, CP Lang v2 drops raw C++ LOC by **64.0%** (1,507 to 542 lines) and library-stripped C++ LOC by **42.2%** (937 to 542 lines). Even the `today/` ports, which hand-roll missing prototype helpers (sparse table, array-backed skew heap, 2D prefix sums, manual matrix comparison, and a `SegTree` stand-in for `Set.last_le`), come in at 811 lines (46.2% below raw C++ and 13.4% below library-stripped C++).

One unexpected highlight: differential fuzzing caught a genuine out-of-bounds heap read in the accepted C++ submission `5536569.cpp` (`apio09p1`). When `k == 1`, `solve_max_down` allocates `vector<int> ans(n)` and evaluates `ans[n - 1] = ans[n]`, pulling uninitialized heap memory (`1086` instead of `148` on a 12x5 grid with values in `0..50`). In CP Lang v2, indexing the boundary array by `Row.bounds` and writing `down[rw.first.l]` / `down[rw.first.r]` makes that entire bug class unrepresentable.

---

## Dual LOC Breakdown

Non-blank, non-comment, non-`dbg(...)` lines were counted for each original C++ file twice: raw total, and library-stripped (removing `#include`/`using`/`#define`/debug macros, `int main()` fast-IO boilerplate, unused brute-force or local checker structs such as `solve_bf` in `5536701.cpp`, `solve1` in `5325518.cpp`, and `Checker` in `5678057.cpp`, plus reusable data structures like `DS1` 2D prefix sum, `ST` sparse table, `DS2` mergeable heap, `DSU`/`CmpByKey`, and `transposed`/`reversed`).

- **apio09p1** (`5536569.cpp`, Digging for Oil — 2D prefix sums + 4-orientation sweep): 117 raw C++ / 57 stripped C++ -> **49 v2** (-58.1% raw, -14.0% stripped) / 65 `today/` (41 verified cases; brute-force oracle for `k == 1`, diff + brute for `k >= 2`).
- **apio09p2** (`5536701.cpp`, Siruseri Convention Centre — sparse table + functional binary lifting + interval splitting): 174 raw / 109 stripped -> **61 v2** (-64.9% raw, -44.0% stripped) / 114 `today/` (40 verified cases).
- **btoi04p3** (`5270736.cpp`, Sequence — L1 isotonic regression via PAV + mergeable max-heap): 172 raw / 52 stripped -> **34 v2** (-80.2% raw, -34.6% stripped) / 59 `today/` (44 verified cases with strict-monotonicity & L1-cost validator).
- **btoi08p6** (`5325518.cpp`, Gloves — subset dominance boxes + Pareto skyline sweep): 104 raw / 36 stripped -> **24 v2** (-76.9% raw, -33.3% stripped) / 34 `today/` (40 verified cases).
- **btoi17p4** (`4523876.cpp`, Friends — bounded search tree + submodular cut uncrossing): 170 raw / 121 stripped -> **73 v2** (-57.1% raw, -39.7% stripped) / 100 `today/` (40 verified cases with partition & cut-bound validator).
- **btoi18p4** (`4656863.cpp`, Alternating Current — circular-arc domination + cyclic odd-cycle 2-coloring): 263 raw / 232 stripped -> **117 v2** (-55.5% raw, -49.6% stripped) / 195 `today/` (50 verified cases with 2-color full-circle coverage validator).
- **dpf** (`4131645.cpp`, AtCoder DP F — LCS DP + reconstruction): 62 raw / 28 stripped -> **19 v2** (-69.4% raw, -32.1% stripped) / 29 `today/` (34 verified cases with subsequence & optimal-length validator).
- **egoi23p2** (`5678050.cpp`, Padel Prize Pursuit — offline time-tree rollback DFS + ordered leaderboard): 82 raw / 53 stripped -> **32 v2** (-61.0% raw, -39.6% stripped) / 34 `today/` (40 verified cases).
- **egoi23p4** (`5678052.cpp`, Bikes vs. Cars — dual bottleneck Floyd-Warshall closures + min/max Kruskal MST): 160 raw / 103 stripped -> **60 v2** (-62.5% raw, -41.7% stripped) / 82 `today/` (40 verified cases with full dual-closure graph validator).
- **egoi23p7** (`5678057.cpp`, Find the Tree — component contraction + complement-graph DFS): 203 raw / 146 stripped -> **73 v2** (-64.0% raw, -50.0% stripped) / 99 `today/` (45 verified cases with rooted-arborescence & required/forbidden edge validator).
- **Batch 38 Totals**: 1,507 raw C++ / 937 stripped C++ -> **542 v2** (**-64.0% raw**, **-42.2% stripped**) / **811 `today/`**.

---

## Remaining v2 Friction & Multi-Feature Collisions

### 38.1 — 2D `flipped` axis ambiguity and polymorphic grid helpers (`apio09p1`)

The cheat sheet lists `a.transposed` and `a.flipped`, but for a 2D array `[R, C]Int` it is unclear whether `a.flipped` reverses rows (vertical flip), columns (horizontal flip), or both. Calling one helper `best_oriented` on both `[Row, Col]Int` and `[Col, Row]Int` also requires domain-generic parameters:

```cp
fn best_oriented<R, C>(grid: [R, C]Int) -> Int:
    let ps = prefix(grid)
    ...

var ans = best_oriented(g)
ans max= best_oriented(g.flipped_rows)
ans max= best_oriented(g.transposed)
ans max= best_oriented(g.transposed.flipped_rows)
```

If `flipped` is kept ambiguous on 2D arrays, contest writers will guess the wrong axis under time pressure. Naming `flipped_rows` / `flipped_cols` on 2D arrays (while keeping `flipped` for 1D arrays) removes the guesswork with zero extra syntax. The spec should also confirm that `g.transposed` yields `[C, R]T` so a domain-polymorphic helper works without flattening into a `List<Int>`.

### 38.2 — `D.windows(k)` loses non-emptiness in `.first` / `.last` (`apio09p1`)

Iterating `for rw in R.windows(k).rev:` produces a `Range<R>` of width `k >= 1`, so `rw` is guaranteed non-empty. Yet because `Range<D>.first` and `Range<D>.last` return `?D` uniformly, every boundary lookup inside a sliding-window DP needs `!` noise:

```cp
for rw in R.windows(k).rev:
    var best: ?Int = down[rw.first!.r]
    for cw in C.windows(k):
        best max= ps.sum(rw, cw)
    down[rw.first!.l] = best
```

Under contest pressure, writing `rw.first!.l` and `rw.last!.r` on every line feels like fighting the type checker over a fact `windows(k)` already proved. Adding `.l` and `.r` directly on `Range<D>` (returning `D.Bound`, which is well-defined even for an empty range where both cuts coincide) lets the author write `down[rw.l]` and `down[rw.r]` and eliminates `!` completely.

### 38.3 — Empty-range queries on `SparseTable<D, T>` (`apio09p2`)

`SegTree<D, T>` takes an identity element for empty ranges, so `st.query(r)` always returns `T`. `SparseTable<D, T>` serves idempotent semigroups (`min`, `max`, `gcd`) that have no natural identity for arbitrary tuple types like `(Int, I)`. In `apio09p2`, every interval-splitting query can easily pass an empty slice (e.g. `cl..a` when `cl == a` or `first_after(ql)..r` when no interval starts after `ql`):

```cp
let hi = first_after(ql)
if let (end_r, u) = st.query(max(hi, lo)..r) and end_r <= qr:
    ...
```

Making `SparseTable.query(range)` return `?T` (returning `none` on an empty range) aligns with v2's uniform optionality rule and eliminates sentinel pairs like `(INF, -1)`. The cheat sheet currently does not state `SparseTable.query`'s return type, leaving users unsure whether empty ranges trap or return `?T`.

### 38.4 — Chained `if let` / `while let` missing from the one-page cheat sheet (`apio09p1`, `apio09p2`, `dpf`)

`decisions.md` and `research/language-architecture-v2.md` both lock chained optional bindings (`if let a = x and let b = y and cond:` and `while let ci = i and let cj = j:`), and Batch 38 relied on them across `apio09p1`, `apio09p2`, and `dpf`:

```cp
// dpf.cp — zero raw integer indices during LCS backtracking
var i: ?A = A.first
var j: ?B = B.first
while let ci = i and let cj = j:
    let cur = dp[ci.l, cj.l]
    if dp[ci.r, cj.l] == cur: i = ci + 1
    elif dp[ci.l, cj.r] == cur: j = cj + 1
    else:
        out.push(a[ci])
        (i, j) = (ci + 1, cj + 1)
```

Yet `docs/one-page-cheat-sheet.md` only shows single `if let x = e:`, so a contestant reading the cheat sheet assumes chained `if let` / `while let` does not exist and falls back to nested `if let` pyramids or raw integers. Adding one 6-word example (`if let a = x and let b = y:`) to the cheat sheet fixes this immediately.

### 38.5 — `Subset<D>` lacks fast bitmask peeling & doubling ergonomics (`btoi08p6`)

`btoi08p6` computes `(sum_a(S), sum_b(C \ S))` over all `2^n` subsets of `domain C = 0..n`. With `Subset<C>`, the natural $O(2^n)$ DP peels the lowest set bit (`lsb`) to reuse the previous subset's sum:

```cp
// What you want to write with Subset<C>:
var sa: [Subset<C>]Int = 0
for s in Subset<C> if not s.empty:
    let (c, rest) = s.pop_first()!
    sa[s] = sa[rest] + a[c]
```

Without `.first` / `.pop_first()` (or `.lsb`) on `Subset<D>`, computing subset sums in $O(2^n)$ forces either falling back to manual list-doubling (`btoi08p6.cp`) or dropping to raw bit tricks (`mask & -mask`, `ctz`). Adding `s.first -> ?D` and `s.without(c)` (or `s.pop_first() -> ?(D, Subset<D>)`) unlocks $O(2^n)$ subset DP across the corpus without adding a single language keyword.

### 38.6 — Tie-broken dynamic argmax & functional binary lifting boilerplate (`apio09p2`, `egoi23p2`)

Two recurring contest patterns still require manual boilerplate:

1. **Dynamic key-update + descending tie-break in `Set` (`egoi23p2`)**: Tracking the player with maximum holding time (and smallest index on ties) forces negating the domain index (`-p.to_idx`) and manually pairing `remove`/`add` on every rollback update:

```cp
var cnt: [Player]Int = 0
var board = Set<(Int, Int)>()
for p in Player: board.add((0, -p.to_idx))

fn add_time(p: Player, delta: Int):
    board.remove((cnt[p], -p.to_idx))
    cnt[p] += delta
    board.add((cnt[p], -p.to_idx))
```

Notice the domain-safety leak: `Player` has to be demoted to `-p.to_idx: Int` and reconstructed with `Player(-neg_best)` just to reverse tie-breaking! A lightweight `Rev<T>` wrapper (like Rust's `Reverse`) lets `Set<(Int, Rev<Player>)>` stay 100% domain-typed without `-p.to_idx` or unchecked `Player(...)` casts.

2. **Functional-graph binary lifting (`apio09p2`)**: `Rooted<D>` already provides `.ancestor(v, k)` for trees, yet static or online functional graphs still allocate `const LOG = 18; var lift: [0..LOG, I]?I = none` and hand-write the descending power-of-two jump loop. A small stdlib `BinaryLift<D>` (with `jump(u, k)` and `jump_while(u, pred)`) removes manual `LOG` constants and off-by-one shift bugs.

### 38.7 — `Map<K, V>.last_le(k)` return type ambiguity (`apio09p2`)

In `apio09p2`, splitting an interval block in an ordered `Map<I, Block>` immediately needs both the key `cl` and the stored tuple `(cr, ql, qr, val)`:

```cp
let (cl, (cr, ql, qr, val)) = blocks.last_le(a)!
```

The cheat sheet lists `first_ge` / `last_le` for ordered collections, but does not clarify whether `Map<K, V>.last_le(k)` returns `?K` or `?(K, V)`. Returning `?(K, V)` (matching `Map` iteration) avoids a redundant `O(log n)` lookup `blocks[cl]` right after `last_le`, while `Set<K>.last_le(k)` naturally returns `?K`.

### 38.8 — Small-to-large heap merging lacks `.absorb(mut other)` (`btoi04p3`)

Pool Adjacent Violators (`btoi04p3`) and slope-trick DP constantly merge two heaps using small-to-large draining:

```cp
fn merge_blocks(mut l: Block, mut r: Block) -> Block:
    if l.heap.len < r.heap.len: swap(mut l.heap, mut r.heap)
    while let v = r.heap.pop(): l.heap.push(v)
    let new_len = l.len + r.len
    while l.heap.len > (new_len + 1) / 2:
        l.heap.pop()
    Block(lo: l.lo, hi: r.hi, len: new_len, heap: l.heap)
```

Writing the swap-and-drain loop manually is repetitive and easy to get backwards (draining the large heap into the small one silently degrades `O(n log^2 n)` to `O(n^2)` TLE). A single stdlib method `l.heap.absorb(mut r.heap)` (on `MaxHeap`, `MinHeap`, `Set`, `HashSet`, `Map`, `HashMap`) makes small-to-large merging a one-liner that cannot be inverted by mistake.

### 38.9 — Backtracking search cleanup verbosity (`btoi17p4`)

In `btoi17p4`, the bounded search tree tries including a neighbor `v` (`state[v] = In; incl.push(v); dfs(); incl.pop()`), then excluding it (`state[v] = Out; dfs()`), then resetting (`state[v] = Undecided`):

```cp
state[v] = In
incl.push(v)
dfs()
incl.pop()
state[v] = Out
dfs()
state[v] = Undecided
return
```

Every early `return` in a backtracking `dfs` risks leaving `state` or `incl` dirty. While CP Lang intentionally avoids RAII destructors and hidden control flow, a lightweight lexical `defer` statement (or documenting this explicitly as an intentional trade-off) is worth considering: `defer state[v] = Undecided` guarantees cleanup on every exit without adding destructors or lifetime complexity.

### 38.10 — Value-copying & structural equality on 2D domain arrays (`egoi23p4`)

In `egoi23p4`, checking whether a matrix equals its bottleneck Floyd-Warshall closure is a single line in C++ (`fw_mn(arr_mn) != arr_mn`) because `std::vector<std::vector<int>>` supports both pass-by-value copy and `operator==`. In CP Lang v2:

```cp
if closure_mn(arr_mn) != arr_mn or closure_mx(arr_mx) != arr_mx:
    print("NO")
    return
```

This raises two spec questions the cheat sheet does not answer:
1. Does `[V, V]Int` support structural `==` / `!=` out of the box? (In `proto/cpc`, `today/egoi23p4.cp` had to write explicit 2D comparison loops inside `matches_mn` / `matches_mx`.)
2. How does a function that wants a mutable scratch copy of a caller's `[V, V]Int` declare its parameter without mutating the caller? In v2, `mut g: [V, V]Int` borrows from the caller in-place, whereas passing by immutable value and writing `var g = src.clone()` (or `var g = src`) makes the copy explicit. Clarifying `var g = src` copy semantics for arrays (and ensuring `==` works on `[D]T` / `[R, C]T` whenever `T: Eq`) keeps matrix/grid algorithms concise and unsurprising.

### 38.11 — Complement-graph DFS set partitioning (`egoi23p7`)

Complement-graph BFS/DFS (`egoi23p7`) visits every unvisited node *not* in `cc_adj[u]` in total $O((V + E) \log V)$ time by splitting `to_vis` into neighbors (`cant_vis`) and non-neighbors (`can_vis`). The idiom requires swapping `to_vis` into `can_vis` in $O(1)$ without copying:

```cp
var can_vis = Set<Comp>()
var cant_vis = Set<Comp>()
swap(mut can_vis, mut to_vis)
for v in cc_adj[u]:
    if v in can_vis:
        can_vis.remove(v)
        cant_vis.add(v)
swap(mut cant_vis, mut to_vis)
for v in can_vis: dfs_comp(v, u)
```

In `today/egoi23p7.cp`, because `proto/cpc` lacks `swap` on `Set`, we had to copy elements out (`for c in to_vis: can_vis.insert(c)`), turning an $O(V + E)$ complement traversal into $O(V^2)$! Ensuring `swap(mut a, mut b)` is guaranteed $O(1)$ on every heap-backed container (`List`, `Set`, `HashSet`, `Map`, `HashMap`, heaps) — and optionally offering `to_vis.take_except(cc_adj[u])` in stdlib — prevents silent quadratic regressions on complement-graph problems.

### 38.12 — Prototype (`proto/cpc`) false-positive aliasing on `Copy` array reads and API drift

Compiling the `today/` ports surfaced one critical compiler-design lesson and five prototype-vs-v2 gaps:

- **False-positive borrow-checker aliasing on `Copy` reads (`btoi04p3`)**: Writing `hrc[u] = merge_heap(hrc[u], v)` inside `fn merge_heap` was rejected with `` `hrc[u]` can't be passed to `merge_heap`: `merge_heap` also changes `hrc` directly ``, even though `hrc[u]` is `?V` (a plain `Copy` value evaluated before the call!). Lifting it to `let rc = hrc[u]; hrc[u] = merge_heap(rc, v)` compiled immediately. The Rust compiler's borrow checker MUST treat by-value `Copy` element reads as temporaries that release their borrow of `hrc` before the call begins, or every array-backed treap / skew heap / segment tree will trip false aliasing errors.
- **`Set` method drift**: v2 uses `s.add(x)` and property `s.max`, while `proto/cpc` requires `s.insert(x)` and method `s.max()`.
- **Missing `Set.last_le` / `Map.last_le` in `proto/cpc`**: only 1D sorted arrays support `.last_le` today; `today/apio09p2.cp` used a `SegTree<I, Int>` max-prefix query as an $O(\log n)$ stand-in.
- **No `Str` indexing in `proto/cpc`**: `today/dpf.cp` had to run on `proto/cp.py lib-run` (`proto/cpi`) instead of `proto/cpc`.
- **No `SparseTable` or `MaxHeap` in `proto/cpc`**: `today/apio09p2.cp` hand-rolled a 25-line sparse table, and `today/btoi04p3.cp` used an array-backed skew heap.
- **No structural `==` on 2D arrays in `proto/cpc`**: `today/egoi23p4.cp` wrote explicit nested comparison loops.

---

## What Worked Especially Well in v2

- **Domain bounds eliminated a real accepted-C++ UB (`apio09p1`)**: `[Row.bounds]?Int` with `rw.first.l` / `rw.first.r` caught the exact off-by-one bug (`ans[n]` read when `k == 1`) present in `corpus/submissions/5536569.cpp`.
- **Cyclic indexing shone on circular geometry (`btoi18p4`)**: `i.cyc(-1)`, `i.cyc(1)`, `i.cyc(2)`, `i.cyc(3)` replaced every `(i + k - 1) % k` and ternary wraparound loop cleanly, cutting 232 stripped C++ lines down to 117 v2 lines.
- **Built-in `Dsu<V>` + open-ended lower-triangle loops (`egoi23p4`)**: `for j in ..i:` and `Dsu<V>` replaced 40 lines of custom `DSU` + `CmpByKey` boilerplate cleanly, cutting 103 stripped C++ lines to 60 v2 lines.
- **Uniform `?Event` erased `-1` sentinels in tree rollback (`egoi23p2`)**: `var rep: [Player]?Event = none` and `if let root = rep[p]: dfs(root, m)` removed `memset(rep, -1, ...)` and every `if (u == -1) return;` guard.
- **Minimal boilerplate on classic DP & greedy (`dpf`, `btoi08p6`)**: LCS reconstruction dropped to 19 lines and Gloves to 24 lines with zero manual index bounds checks.
