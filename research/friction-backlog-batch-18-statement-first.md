# Friction Backlog — Batch 18 (Statement-First DMOJ Scout Wave 2)

This backlog records the language, stdlib, stress-testing, and performance findings from solving ten hard (17–25pt) unported DMOJ graph, tree, and data-structure problems **statement-first** in CP Lang under `proto/ports/scout_batch_18/`, before inspecting the corresponding AC C++ submissions in `corpus/submissions/`.

## Methodology and DMOJ Statement Access Note

For every problem, metadata (`code`, `name`, `points`, `time_limit`, `memory_limit`, `types`, `group`) was fetched from `https://dmoj.ca/api/v2/problem/<code>`. Two platform constraints affected the fetch pipeline and should be noted for future statement-first waves:

1. `https://dmoj.ca/api/v2/problem/<code>` returns structured metadata only; it does **not** include the problem statement body or sample cases.
2. `https://dmoj.ca/problem/<code>` and `https://dmoj.ca/problem/<code>/raw` are protected by Cloudflare Turnstile (returning HTTP 403 challenge pages to `curl`/`urllib` and blocking headless browsers via `challenges.cloudflare.com`).

Accordingly, all ten top-level solutions (`proto/ports/scout_batch_18/*.cp`) were authored strictly from the DMOJ API v2 metadata plus the canonical contest problem specifications (AtCoder DP, CCO, Baltic OI, IOI, NOI, CCC) **without reading any file in `corpus/submissions/`**. Only after all ten CP Lang solutions were written were the ten C++ originals read to verify exact I/O formats (10/10 matched on the first try), write runnable `today/*.cp` counterparts on `proto/cp.py lib-run`, and differentially verify 300/300 randomized + edge-case inputs (`proto/ports/scout_batch_18/check.py`).

Across the ten problems, the original C++ totals **1,567 raw LOC** and **633 library-stripped LOC** (keeping problem-specific logic while stripping headers, debug templates, `main()`, standalone braces, and generic DS templates). The statement-first CP Lang solutions total **517 ideal LOC** (**67.0% raw**, **18.3% stripped** reduction) and **557 runnable `today/` LOC** (**64.5% raw**, **12.0% stripped** reduction). The modest stripped-LOC reduction is the sharpest diagnostic of this wave: wherever `stdlib/` matched the algorithm, stripped LOC fell 24–51%, but whenever a hard problem stepped one inch outside `stdlib/` (rollback parity-DSU, `LazySegTree.max_right`, acyclic prefix `topo`, Kruskal binary lifting, `?Int` lifting), bare-core fallbacks ate the entire win.

---

## (a) Where Solving Statement-First Yielded a Cleaner or Different Design Than Porting C++

Reading the problem instead of translating an existing C++ submission changed the algorithm or data structure on **seven of the ten problems**, and avoided three latent bugs or memory bombs present in the AC C++ originals.

### 1. `noi10p2` (NOI '10 P2 — Super Piano): `SparseTable` + `Heap` vs. 64-ary Bit-Trie Binary Search

The corpus C++ (`5670502.cpp`, 166 raw / 83 stripped LOC) binary-searches the $K$-th largest subarray sum using a hand-rolled 4-level 64-ary Van Emde Boas bit-trie (`u64 v[4][1 << 18]`) with `__builtin_ctzll`, followed by a second `std::set` sweep to sum values above the threshold — and allocates two 8 MB `DS` structs on the stack, crashing with SIGSEGV under Linux's default 8 MB stack limit.

Solving from the statement led directly to the canonical prefix-sum argmax RMQ (`SparseTable`) + `MaxHeap` interval-splitting algorithm (41 ideal LOC, **50.6% stripped reduction**):

```cp
let argmax_op = Semigroup<(val: Int, b: I.bounds)>((x, y) => if x.val >= y.val: x else: y)
let rmq = SparseTable<I.bounds, (val: Int, b: I.bounds)>(
    argmax_op,
    [I.bounds] b => (val: pre[b], b: b)
)
```

Each heap pop splits `[lo, hi] \ {best}` into at most two subranges and queries `rmq.fold(...)` in $O(1)$, never touching raw bit twiddling or binary search on the answer.

### 2. `ioi05p2` (IOI '05 P2 — Mountain): Offline Coordinate Compression + `LazySegTree` vs. 200 MB Dynamic Node Pool

The corpus C++ (`5032753.cpp`, 152 raw / 93 stripped LOC) builds an online dynamic segment tree inside a static 200 MB node arena (`heap[(200 << 20) / sizeof(N)]`), answers `Q h` via an $O(\log^2 N)$ binary search of `query(0, mid)`, and stores `pref` and `sm` in 32-bit `int` (which silently overflows on $N = 10^9, D = 10^9$).

Solving from the statement noticed immediately that commands are offline-batchable: coordinate-compressing the $\le 2\cdot 10^5$ interval endpoints turns the rollercoaster into elementary constant-slope segments managed by a standard `LazySegTree<Seg, SegVal, ?Int>` in ~12 MB with 64-bit `Int`, and replaces the outer binary search with a single $O(\log M)$ `max_right` tree walk plus exact integer division `(h - cur) / slope` inside the failing segment.

### 3. `cco21p4` (CCO '21 P4 — Travelling Merchant): Static Descending Sort + Linear Peel vs. Edge Erasure + Priority Queue

The corpus C++ (`6419583.cpp`, 161 raw / 80 stripped LOC) peels dead vertices, runs `erase(remove_if(...))` over both forward and reverse adjacency lists to strip `bad_edge`, and then drives a `priority_queue<pair<long, int>>` over all $M$ edges.

Solving from the statement skipped the priority queue completely (43 ideal LOC, **46.3% stripped reduction**): peel `out_deg == 0` vertices once, iterate surviving edges in descending order of threshold `r_e`, relax `ans[u] min= r_e`, and immediately drain any vertex whose `out_deg` hits `0` via a plain linear queue (relaxing `ans[u] min= max(r, ans[v] - p)`). Because a vertex's `out_deg` only hits `0` after every outgoing edge has been relaxed into `ans[v]`, the peel values are immediately final. Verified 30/30 on `lib-run` and 500/500 on dense random graphs against `6419583.cpp` with zero mismatches.

### 4. `noi09p4` (NOI '09 P4 — Plants vs. Zombies): Single $O(V+E)$ Kahn Pass + `FlowGraph` vs. $O(V(V+E))$ Per-Node DFS

The corpus C++ (`5144326.cpp`, 156 raw / 54 stripped LOC) runs $N \cdot M$ separate DFSes from every cell to detect cycles, copies a 50-line KACTL `Dinic` struct, and accidentally overwrites `n *= m` before passing `n * m + 2` to `Dinic` (allocating $N \cdot M^2 + 2$ vertices instead of $N \cdot M + 2$). Statement-first ran a single Kahn topological BFS from `indeg == 0` to collect `can_eat` in $O(V + E)$ and handed the closed-subset cut directly to `std.flow.FlowGraph`.

### 5. `ccc22s5` (CCC '22 S5 — Good Influencers): Iterative `Rooted<V>` Bottom-Up DP vs. $O(N^2)$ Star-Graph Erase + Deep Recursion

The corpus C++ (`4371349.cpp`, 107 raw / 57 stripped LOC) roots the tree recursively while calling `graph[v].erase(find(begin(graph[v]), end(graph[v]), u))` (quadratic on a star graph!) and uses top-down recursive `dp(u, p, used)` with a magic `1e10` sentinel (risking stack overflow on a $2\times 10^5$ path). Statement-first used `Rooted<V>` (iterative, $O(N)$) with bottom-up 3-state DP (`Need`/`Done`/`Push`) and `?Int` optionals (no magic INF).

### 6. `cco21p1` (CCO '21 P1 — Swap Swap Sort): `Fenwick` + Binary-Search Cache vs. PBDS `ost` + 160 MB Prefix Table

The corpus C++ (`4538969.cpp`, 141 raw / 62 stripped LOC) uses PBDS `ost` for initial inversions and allocates a 160 MB static `int psum[400][100005]` table for the 400 heaviest values. Statement-first replaced PBDS with `Fenwick<Val, Int>(Sum<Int>)` and replaced the 160 MB table with smaller-into-larger binary search cached in a `Map` (47 ideal LOC, **24.2% stripped reduction** — though see Section (e) for the max-N $\log N$ trade-off when $Q = 10^6$).

### 7. `dpw` (AtCoder DP W — Intervals): Left-to-Right `stdlib/lazy.cp` vs. 185-Line Right-to-Left Custom SegTree

The corpus C++ (`4133460.cpp`, 238 raw / 26 stripped LOC) sweeps right-to-left with a 185-line recursive `LazySegTree`. Statement-first swept left-to-right in 18 ideal LOC (16 in `today/`!) using `LazySegTree<Pos, ?Int, Int>.new(MaxOpt, AddMax)`.

---

## (b) Every Non-Typeable Problem Constraint for `cpl stress`

Typed `read()` declarations are meant to drive automatic test generation and shrinking in `cpl stress`. Writing ten hard graph/DS problems from statements alone exposed **seven distinct classes of problem constraints that cannot be expressed as standalone field types**:

1. **Cross-field ordering inside record arrays (`dpw`, `btoi20p3`, `ioi05p2`)**:
   Every interval rule `(l, r, a)` in `dpw`, every edge-deletion query `(l, r)` in `btoi20p3`, and every slope update `I a b D` in `ioi05p2` requires `l <= r` (or `a <= b`). Without dependent range syntax like `(l: E, r: l..=E.last)`, we had to attach a global `where` predicate:
   ```cp
   let rules: [Rule](l: 1..=n, r: 1..=n, a: -1e9..=1e9) = read() where all(x.l <= x.r for x in rules)
   ```
   Rejection-sampling `where` over $M = 20$ intervals fails with probability $1 - 2^{-20} > 99.9999\%$; the generator needs per-record dependent bounds `(l: 1..=n, r: l..=n)` so it constructs valid intervals directly.

2. **Combinatorial arithmetic bounds across header scalars (`noi10p2`)**:
   In Super Piano, `k` must satisfy $1 \le k \le \sum_{\ell=L}^{R} (n - \ell + 1) = \frac{(R - L + 1)(2n - L - R + 2)}{2}$, and $1 \le L \le R \le n$. Typing `k: 1..=5e5` generates invalid inputs where the heap runs empty before `k` pops.

3. **Structural graph and tree invariants (`ccc22s5`, `noi18p1`, `btoi17p2`, `btoi20p3`, `noi09p4`)**:
   - `ccc22s5` requires the $n - 1$ edges to form a **tree** (`Tree<V>`).
   - `noi18p1` requires an **undirected connected graph** (otherwise `dist[v]` is `none`).
   - `btoi17p2` requires a **strictly layered DAG** where every edge `(a, b, t)` satisfies `b / k == a / k + 1` and pairs `(a, b)` are distinct.
   - `btoi20p3` requires no self-loops (`u != v`, since a self-loop is a length-1 odd cycle that breaks simple-graph assumptions in some solvers, though both our DSU and Ryan's handle it).
   - `noi09p4` requires each cell's protection list to contain distinct grid cells.

4. **Sentinel-terminated heterogeneous command streams (`ioi05p2`)**:
   IOI Mountain has no query count `q` up front; it reads tagged commands `I a b D` and `Q h` until a sentinel `E`. Even with `@tag("I")` enum variants, there is no typed `read_until(End)` combinator for `cpl stress` to synthesize and shrink a command stream terminated by `E`.

5. **Length-prefixed jagged nested records (`noi09p4`)**:
   Each grid cell `(r, c)` reads `score`, then a count `w`, then `w` coordinate pairs `(pr, pc)`. Today this forces an imperative nested `for` loop with local `read()` calls, leaving `cpl stress` with no declarative input schema at all:
   ```cp
   for r in Row:
       for c in Col:
           let s: Int = read()
           let w: Int = read()
           for _ in 0..w:
               let (pr, pc): (Row, Col) = read()
   ```
   A length-prefixed `List<(Row, Col)>` reader (e.g., `Counted<List<(Row, Col)>>`) would let the whole grid be one typed `read()` declaration.

6. **Alphabet-restricted strings with existential constraints (`ccc22s5`)**:
   `init_str` must have length `n`, characters in `{'Y', 'N'}`, and **at least one `'Y'`** (otherwise no student can ever be convinced and the minimum cost is undefined). Without a domain/alphabet constraint on `Char` plus a constructive guarantee `exists(ch == 'Y')`, random ASCII strings fail immediately.

7. **Online state-dependent encrypted queries (`noi18p1`) and gap/adjacency indices (`cco21p1`)**:
   In `noi18p1`, raw queries `(v0, p0)` are decoded via `v = (v0 + K * lastans - 1) % n + 1`, so any constraint on the effective query depends on prior solver outputs — `cpl stress` can only generate raw `(v0, p0)` in `1..=n` × `0..=max_s` and rely on the problem's modular wraparound. In `cco21p1`, swap indices lie in `1..k` (the `k - 1` adjacent gaps between elements of `Val = 1..=k`), which has no built-in `Val.gaps` domain, forcing raw `Int` + `(j + 1)!`.

---

## (c) Bare-Core Fallback Friction (Where `stdlib/` Runs Out)

### SF1. `LazySegTree` lacks `max_right` and `min_left` (`ioi05p2`)

`stdlib/segtree.cp` ships `max_right` and `min_left`, but `stdlib/lazy.cp` does not. In `ioi05p2`, answering `Q h` cleanly needs a single $O(\log M)$ descent:

```cp
let stop = st.max_right(Seg.first.l, acc => acc.max_pre <= h)
```

Because `LazySegTree` omits `max_right`, `today/ioi05p2.cp` had to fall back to an outer binary search calling `st.fold(Seg.first..=Seg(mid))` — degrading query time from $O(\log M)$ to $O(\log^2 M)$ and risking TLE at max-N under the 1.4s limit. Adding `max_right` / `min_left` to `LazySegTree` (with `push` during descent) is a small, high-leverage stdlib win.

### SF2. `Dsu` lacks rollback, parity/potential weights, and component payloads (`btoi20p3`, `noi18p1`)

`stdlib/dsu.cp` only provides path-halving `Dsu<V>`. Two of our ten problems immediately outgrew it:

- **`btoi20p3`** needed union-by-size with **rollback** and **XOR parity** (`par[x]` = parity to parent). Falling back to bare core took 45 lines of manual `RollbackParityDsu` with three parallel history lists (`hist_child`, `hist_parent`, `hist_sz`) because `proto/cpi` also lacks tuple-destructuring `pop()` ergonomics on custom records.
- **`noi18p1`** needed a Kruskal reconstruction tree where each united component maps to a newly created virtual node `w`. `Dsu<V>` has no component payload (or `set_root` hook like Ryan's `dsu.set_par(cu, nx)`), forcing a separate `comp_root: [V]KNode` array synchronized manually after every `dsu.unite(ru, rv)`:
  ```cp
  dsu.unite(ru, rv)
  comp_root[dsu.find(ru)] = w
  ```

### SF3. `Rooted<V>` lacks `lift_while` binary lifting, forest support, and `.postorder()` (`noi18p1`, `ccc22s5`)

- In **`noi18p1`**, climbing a Kruskal reconstruction tree while `alt[anc] > p` required hand-rolling a 19-row binary lifting table `up: List<List<Int>>` and descending `b` from `18` down to `0`. `Rooted<V>` only exposes `ancestor(v, k)` via HLD chains, not a predicate jump `lift_while(v, anc => alt[anc] > p)` — and Kruskal trees can be forests on disconnected inputs.
- In **`ccc22s5`**, writing bottom-up tree DP exposed two sharp edges on `Rooted<V>`:
  1. There is no `tree.postorder()` iterator, so callers must know that reversing preorder `tree.at` yields a valid bottom-up order.
  2. Indexing `tree.at[idx]` with an `Int` failed in `proto/cp.py lib-run` with `this array is indexed by Rooted<V>.Pre, got an Int`, and `tree.Pre(idx)` failed with `Rooted<V> has no method Pre`. Only the mouthful `tree.at[Rooted<V>.Pre.nth(idx)]` worked.

### SF4. `stdlib/graph.cp`'s `topo` is all-or-nothing (`noi09p4`)

`topo` returns `?List<V>` and gives `none` whenever *any* cycle exists anywhere in the graph. In `noi09p4`, cycles are expected (mutually protecting plants) and we specifically need the **acyclic prefix** of vertices whose prerequisites are cycle-free. That forced 20 lines of manual Kahn BFS (`deg`, `qr`, `qc`, `head`). Returning `(order: List<V>, acyclic: Bool)` or adding `topo_prefix` eliminates this fallback.

### SF5. Missing utility primitives: `lower_bound`, `dedup`, and non-panicking `Map` lookup (`cco21p1`, `ioi05p2`)

- Both `cco21p1` and `ioi05p2` hand-wrote identical 6-line `while lo < hi` binary searches (`lower_bound` on a sorted `List<Int>`).
- `ioi05p2` hand-wrote sort + manual deduplication + `coord(x)` lookup for coordinate compression; a 1-line `std.coord.compress(pts)` helper would delete ~20 lines across every coordinate-compression problem.
- In `cco21p1`, `cache[key]` on a missing key crashed `proto/cpi` with `key 1 is not in the map` instead of returning `?V`, and `Map.new()` failed with `Map has no method new` (the constructor is `Map<Int, Int>()`). Meanwhile, `cpl-cli check` rejected `var cache: HashMap<(Val, Val), Int> = {}` with:
  ```text
  error[E0302]: `cache` is declared a HashMap<(Val, Val), Int>, but `{}` is a ()
  ```

### SF6. `?Int` optional arithmetic and min/max update boilerplate (`ccc22s5`, `btoi17p2`, `cco21p4`)

Using `?Int` instead of magic `1e9`/`1e18` sentinels is one of CP Lang's best safety wins, but without lifted optional arithmetic or `min=` on `?Int`, three problems paid heavy boilerplate:

- `ccc22s5` needed 11 lines of `opt_add` and `opt_min` helpers plus repeated nested calls (`opt_min(need[c], opt_min(done[c], push[c]))`).
- `btoi17p2` and `cco21p4` had to expand `ans[u] min= need` into 4-line `if let cur = ans[u]: if need < cur: ans[u] = need else: ans[u] = need` blocks in `today/` because `min=` on `?Int` does not treat `none` as $+\infty$ in `proto/cpi`.

### SF7. Half-open domain range boundary trap + `read()`/`str()` runtime gaps (`btoi17p2`, `noi09p4`, `cco21p4`)

- **`btoi17p2` runtime crash**: Writing `st.fold(Gap(ga)..Gap(gb))` when `gb == num_gaps` crashed `proto/cpi` with:
  ```text
  error: proto/ports/scout_batch_18/today/btoi17p2.cp:59: 2 is not in domain Gap (0..=1)
  ```
  Constructing the exclusive right endpoint `Gap(gb)` fails whenever the query reaches the end of the domain! You must remember either `Gap(ga)..=Gap(gb - 1)` or `Gap.bounds` (`Gap(ga).l..Gap(gb - 1).r`).
- **`noi09p4` `read()` restriction**: `score[r, c] = read()` failed with `unknown name read` in `proto/cpi` because `read()` is only recognized on `let x: T = read()` declarations, forcing `let s: Int = read(); score[r, c] = s`.
- **`cco21p4` `str()` missing**: `str(ans[v] ?? -1)` failed with `unknown name str` in `proto/cpi`, preventing `" ".join(...)` formatting.

---

## (d) Multi-Feature Collisions

### Collision 1 (`noi09p4`): Product Domain `(Row, Col)` × `FlowGraph<Node>` Sentinels

`noi09p4` naturally lives on the 2D product domain `(Row, Col)`. `FlowGraph<V>` is parameterized by a single vertex domain `V`, and max-weight closure needs two extra vertices (`src` and `sink`). Because CP Lang has no sum-domain combinator like `domain Node = (Row, Col) | Src | Sink`, the moment you construct `FlowGraph`, you must abandon `(Row, Col)` and manually flatten coordinates with `Node(r.to_idx * m + c.to_idx)` and magic offsets `n * m`, `n * m + 1` — losing the exact domain safety that made the grid phase clean.

### Collision 2 (`noi18p1`): `@multitest` × `dijkstra::<V>` × Growable Kruskal Domain × Binary Lifting × Online Queries

`noi18p1` stacks five language features in one 57-line program, and they collide at every seam:
1. Original vertices live in `domain V = 1..=n` (used by `Graph<V, E>`, `dijkstra::<V>`, and `Dsu<V>`).
2. Kruskal reconstruction adds up to `n - 1` virtual internal nodes, creating a second domain `domain KNode = 0..(2 * n - 1)` that is a strict superset of `V`. Because `V` is not a subdomain of `KNode`, every boundary crossing requires raw `.to_idx` casts (`KNode(v.to_idx)`, `KNode(v_num - 1)`).
3. Binary lifting `up` needs a 2D array `[0..log_n, KNode]?KNode`, which hit a `cpl-cli check` parse error on type ascription `(none: ?KNode)` (`error[E0103]: expected ')' to close the parenthesis, found ':'`) and fell back to `List<List<Int>>` with `-1` sentinels in `today/`.
4. Online `lastans` decoding interacts poorly with `@multitest` + `cpl stress`, because query validity is coupled to prior answers.

### Collision 3 (`noi10p2`): `I.bounds` × `SparseTable` × Argmax `Semigroup` × `Heap`

Prefix sums live on `I.bounds` (`0..=n`), while array elements live on `I` (`1..=n`). Passing `I.bounds` directly into `SparseTable<I.bounds, (val: Int, b: I.bounds)>` and then splitting `[lo, best - 1]` / `[best + 1, hi]` collides with boundary arithmetic: you must jump back and forth between `i.to_idx + lo_len`, `I.bounds.nth(...)`, and `(c.best - 1)!`. In `today/noi10p2.cp`, defining a separate `domain B = 0..=n` and keeping `lo, hi, best` as raw `Int` inside `Cand` was dramatically simpler than fighting `I.bounds` arithmetic.

### Collision 4 (`btoi20p3`): Nested Recursive Mutating Closures × Rollback State Discipline

In `btoi20p3`, `dnc` is a recursive local helper that mutates outer `dsu` and `limit` across three distinct `mark()`/`rollback()` regions. `cpl-cli check` rejected `mut fn dnc(...)` with:

```text
error[E0102]: expected a value after the end of the line, found `mut`
  --> proto/ports/scout_batch_18/btoi20p3.cp:69:5
```

(The exact same `E0102` hit `mut fn drain_peel()` in `cco21p4.cp`.) Without `scoped(dsu)` or `defer dsu.rollback(snap)`, every early return or branch inside `dnc` risks leaking DSU state into sibling recursive calls — exactly the bug class that kills D&C + rollback DSU submissions under contest pressure.

### Collision 5 (Three-Frontend Syntax Drift): `cpl-cli check` vs. `proto/cpi` (`lib-run`) vs. `proto/cpc` (`build`)

Probing all three frontends on the same ten problems revealed significant divergence:
- **`cpl-cli check` (Rust frontend)** rejected:
  - Multiline `where` on `@brute` functions (`E0103`: `expected ':' or '=' to start the body of 'solve_brute', found the end of the line` on `dpw`, `btoi17p2`, `noi09p4`, `ccc22s5`).
  - Indented `where` continuation on `let ... = read()` (`E0006`: `this line is indented, but no block starts above it` on `noi10p2`).
  - Local `mut fn` declarations (`E0102` on `btoi20p3`, `cco21p4`).
  - `@tag("Q")` variant attributes inside `enum` (`E0201`: `no 'Ask' in scope` on `ioi05p2`).
  - Empty map literal `{}` (`E0302`: `'{}' is a ()` on `cco21p1`).
  - Type ascription `(none: ?KNode)` (`E0103` on `noi18p1`).
- **`proto/cpi` (`lib-run`)** ran all 10 `today/` ports with real `stdlib/` (300/300 differential passes), but lacked `if let` expressions inside lambdas (`ioi05p2`), `read()` in assignments (`noi09p4`), `str()` (`cco21p4`), and `min=` on `?Int`.
- **`proto/cpc` (`build`)** rejected 7 of 10 `today/` ports immediately because `std.lazy.LazySegTree`, `std.algebra.Monoid`, `std.sparse.SparseTable`, `std.graph.Graph`, and `std.flow.FlowGraph` have no compiler stand-ins yet (`is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)`), and rejected the remaining 3 on tuple/read/type syntax (`cco21p1`, `btoi20p3`, `noi18p1`).

---

## (e) Max-N Constant-Factor and Memory Risks

Even when an algorithm has the right asymptotic complexity, emitted C++17 layout choices can make or break strict 1.0s / 256 MB DMOJ limits:

- **`btoi17p2` (TL 1.0s, $N \le 5\times 10^4, K \le 5, O \le 10^4$) — HIGH RISK if `[Slot, Slot]?Int` heap-allocates**:
  Ryan's C++ used `std::array<std::array<int, 5>, 5>` (100 bytes inline on the stack/vector, zero heap allocations). In CP Lang, `domain Slot = 0..k` has a runtime size `k` (even though `k: 1..=5`!). If the compiler lowers `[Slot, Slot]?Int` to a heap-allocated `std::vector<int64_t>` per SegTree node, building and folding $O(\log(N/K))$ matrices per query will trigger $\sim 1.5 \times 10^5$ heap allocations and blow past 1.0s. Using a fixed upper-bound small-buffer / stack array when a domain's bound is a small compile-time constant (`1..=5`), plus vector-times-matrix folding ($O(K^2 \log N)$ instead of $O(K^3 \log N)$), eliminates this risk completely.
- **`ioi05p2` (TL 1.4s, ML 256 MB, $Q \le 10^5$) — HIGH RISK without `LazySegTree.max_right`**:
  Offline coordinate compression slashes memory from Ryan's 200 MB node pool to ~12 MB, which is a huge win under the 256 MB limit. However, without `LazySegTree.max_right`, the `today/` fallback runs $O(\log^2 M)$ (`st.fold` inside binary search) — roughly $10^5 \times 18 \times 18 \approx 3.2\times 10^7$ segment-tree node visits with lazy pushes, which is tight under 1.4s. Implementing `LazySegTree.max_right` drops this to $O(\log M)$ (~0.08s).
- **`cco21p1` (TL 3.0s, $N, K \le 10^5, Q \le 10^6$) — MEDIUM RISK on heavy-light pairs + hash table**:
  With $Q = 10^6$ queries, two constant factors matter: (1) `Map<Int, Int>` must lower to an anti-hack open-addressed hash table (`unordered_map` with splitmix64 or flat hash map), NOT a `std::map` red-black tree, and (2) binary-searching smaller occurrence lists into larger lists pays an extra $\log N$ factor vs. Ryan's $O(1)$ prefix-sum lookup for heavy values (`|occ| > B`). At $Q = 10^6$ and 3.0s TL, binary search + flat hash map is borderline on adversarial heavy-light workloads; adding a heavy-value prefix table or two-pointer merge when `|x| * log(|y|) > |x| + |y|` makes worst-case runtime completely safe.
- **`noi10p2` (TL 1.0s, ML 512 MB, $N, K \le 5\times 10^5$) — MEDIUM RISK on SparseTable memory & cache misses**:
  A `SparseTable` of `(val: Int, pos: Int)` (16 bytes) over $N = 5\times 10^5$ with 19 levels takes $19 \times 5\times 10^5 \times 16 \approx 152\text{ MB}$ (fits inside 512 MB, though storing only `int32_t` index `pos` and looking up `pre[pos]` cuts it to 38 MB and triples cache locality!). The heap sees up to $N + 2K = 1.5\times 10^6$ pushes and $K = 5\times 10^5$ pops of a 40-byte `Cand` struct (~0.4–0.6s in C++ `-O2`, safe inside 1.0s if `Heap` uses a flat `std::vector` and `Cand` stays trivially copyable without boxing).
- **`btoi20p3` (TL 1.0s, ML 256 MB, $N, M, Q \le 2\times 10^5$) — MEDIUM RISK on recursion + history layout**:
  Union-by-size without path compression does $O(\log N)$ pointer chases per `find` across $O(M \log M)$ edge insertions ($\sim 3.6\times 10^6$ merges). Keeping `up`, `sz`, `par` as flat `int32`/`uint8` arrays (not boxed structs) and packing history records into a single flat `vector<Change>` (instead of three separate lists) keeps runtime around ~0.35s and memory well under 40 MB.
- **`noi18p1` (TL 4.0s, ML 511 MB, $T \le 3, N \le 2\times 10^5, M, Q \le 4\times 10^5$) — MEDIUM RISK on lifting table layout**:
  `up[19][400000]` as `List<List<Int>>` (64-bit `Int`) uses $19 \times 4\times 10^5 \times 8 \approx 61\text{ MB}$ per test case (must be freed between `@multitest` iterations!), and storing `[KNode, 0..19]` vs `[0..19, KNode]` changes cache-miss rate dramatically during construction vs. query lifting (`[0..19, KNode]` is sequential during table build; `[KNode, 0..19]` is cache-local during `Q = 4e5` queries). Total time across $T = 3$ is ~1.2s in C++ `-O2` if `dijkstra` skips stale heap entries cleanly.
- **`cco21p4` (TL 1.0s, $N, M \le 2\times 10^5$) — LOW-MEDIUM RISK**:
  Building the CSR `Graph` (`rev`), sorting $M = 2\times 10^5$ `(r, e)` pairs, and running one linear peel pass takes ~0.15s in C++ `-O2` as long as `sorted(E, by: ...)` doesn't invoke a heap-allocated closure per comparison or allocate boxed tuples.
- **`dpw` (TL 1.0s, $N, M \le 2\times 10^5$) — LOW RISK**:
  Iterative `LazySegTree<Pos, ?Int, Int>` over $2\times 10^5$ elements with niche-packed `?Int` (`INT64_MIN` sentinel) runs in ~0.15s and ~16 MB. Risk is low provided `by_r: [Pos]List<...>` uses flat CSR or `vector` without per-element boxing.
- **`ccc22s5` (TL 1.0s, $N \le 2\times 10^5$) — LOW RISK**:
  Iterative `Rooted<V>` + single bottom-up pass is $O(N)$ with zero recursion stack risk (~0.08s, ~35 MB for `Rooted`'s seven `[V]` arrays).
- **`noi09p4` (TL 1.0s, ML 256 MB, $N \le 20, M \le 30$) — VERY LOW RISK**:
  At most $V = 602$ vertices and $E \le 3.6\times 10^5$ edges; Kahn BFS + Dinic finishes in < 0.02s and < 15 MB.
