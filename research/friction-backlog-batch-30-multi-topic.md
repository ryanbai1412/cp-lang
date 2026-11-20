# Friction Backlog — Scout Batch 30: Hard Multi-Topic Contest Classics

Ten unported multi-paradigm IOI, CCO, COCI, and NOI contest problems were rethought and ported to CP Lang under `proto/ports/scout_batch_30/`, compiled through the unmodified `proto/cpc` prototype, differential-tested on 100 random cases each (80 small + 20 medium, 1,000 cases total) against their original accepted C++ submissions via `proto/ports/scout_batch_30/check.py`, and verified clean under `-fsanitize=address,undefined`. All 10 have zero overlap with `scout_batch_1`–`scout_batch_31` and `blind_critic_*`.

Every problem combines two to four distinct paradigms (byte-string subsequence automaton + DP + reconstruction; game theory + shifted-origin segment tree + binary search; 2:1 tiling geometry + Euclidean step-jumping number theory + memoized HashMap DP; graph DP + lazy segment tree; Dijkstra over DP states + interval binary search; LIS decomposition + flat packed segment tree + backtracking DFS; tree + bitmask DP; envelope decomposition + rolling prefix/suffix-min DP; rooted tree + deepest-first max-heap greedy; Dijkstra + 2D orthogonal range queries + DSU). Composing these features inside a single solution exposed sharp collisions that single-topic ports rarely trigger.

---

## 1. Multi-Feature Collisions

### 1.1 Structural `[0..n]Int` Breaks Across Function Boundaries (`expected [range]Int, got [range]Int`)

**Hit in:** `noi06p3.cp`.

`docs/one-page-cheat-sheet.md` §1 defines `[0..n]Int` as a **structural** array type (distinct from nominal `domain` arrays). In `proto/cpc`, however, each syntactic occurrence of `[0..n]Int` mints a fresh anonymous range identity. Writing a helper signature for two identically sized arrays `a` and `b` failed with a baffling self-contradictory diagnostic:

```cp
var a: [0..n]Int = 0
fn solve_env(arr: [0..n]Int) -> Int: ...
// proto/cpc error: line 44: expected [range]Int, got [range]Int
```

Fixing this required introducing a nominal `domain Col = 0..n`, switching both arrays and helper signatures to `[Col]Int`, and wrapping every integer loop variable in `Col(i)` when indexing `arr[Col(i)]`. When a helper processes two homogeneous arrays of length `n` with arithmetic neighbor indices (`i - 2..=i + 2`, `i + 1`), forcing a nominal domain adds `Col(...)` casts at every read without adding safety.

**Design recommendation:** Ensure structural range arrays `[0..n]T` unify by bounds (or slice type `[Int]T`) across function signatures in the Rust frontend, and never emit `expected X, got X` with identical rendered types.

### 1.2 Opaque `Str` Collides with Byte-Indexed String Algorithms (`[domain S]Char`)

**Hit in:** `ioi99p2.cp` (IOI 1999 Hidden Codes).

CP Lang's approved string model treats `Str` as a flat ASCII byte array `[domain S]Char` with `O(1)` indexing and zero UTF-8 machinery. In `proto/cpc`, `Str` is still an opaque token that rejects **both** indexing (`s[i]` -> `cannot index Str`) and iteration (`[Int(c) for c in s]` -> `cannot iterate Str`). Writing a string-plus-DP problem therefore forces reading every word and target string with an explicit fresh domain:

```cp
for i in 0..m:
    let w: [domain W]Char = read()
    for j in (0..w.len).rev:
        by_char[Int(w[W(j)])].push((i, j))

let s: [domain S]Char = read()
```

Two follow-on collisions appear immediately: (1) because each `w` gets its own loop-local `domain W`, you cannot store `[0..m][domain W]Char` in an array of strings and must drop down to parallel `word_len: [0..m]Int` + `by_char` tables; (2) converting `Char` to a `0..256` table index requires `Int(c)` (not `ord(c)` or `c.byte`), which is undocumented in the one-page cheat sheet. Unifying `Str` as an indexable, iterable, storable ASCII byte array in the Rust compiler resolves all three friction points at once.

### 1.3 Variable-Sized Per-Layer Data Structures vs Domain-Indexed `SegTree<D, T>`

**Hit in:** `coci21c1p5.cp` (COCI Volontiranje).

Patience sorting partitions the permutation into `L` layers `layers[1..=L]` of different runtime lengths, and each layer needs prefix-min queries plus point deletions. In C++ this is `vector<ST<pair<int, int>>>`. Because `SegTree<D, T>` in CP Lang is parameterized by a static `domain D`, a `List` of segment trees of varying sizes cannot be typed.

We worked around this cleanly by packing all `n` elements contiguously by layer into one shared `domain Flat = 0..n` and querying `[off[k], off[k + 1])`:

```cp
domain Flat = 0..n
var off: [0..=mlen + 1]Int = 0
for k in 1..=mlen:
    off[k + 1] = off[k] + layers[k].len

var st = SegTree<Flat, (Int, Int)>((INF, -1), (a, b) => min(a, b))
let best = st.query(Flat(base), Flat(base + cnt - 1))
```

While the flat-packed layout improves cache locality over `vector<ST>`, the same collision recurs whenever a problem builds a segment tree per HLD chain, per component, or per DP layer. Offering a runtime-sized `SegTree.from(list, id, op)` avoids forcing manual offset arrays when a `List` of trees is the natural model.

### 1.4 Domain-Index Round-Tripping Across `MinHeap` + `Rooted`

**Hit in:** `noi07p6.cp` (NOI 2007 P6).

Because `proto/cpc` lacks `MaxHeap<T>`, popping the deepest vertex with tie-breaking by largest vertex id required pushing `(-tree.depth(u), -u.idx)` into `MinHeap<(Int, Int)>` and reconstructing the domain index via `let u = Node(-top.1 + 1)` (knowing `Node` is 1-based while `.idx` is 0-based). That `+ 1` offset is an exact off-by-one hazard that `domain` was designed to prevent; shipping `MaxHeap<T>` (or `MinHeap` with `Node` directly comparable in tuples, and `Domain.from_idx(k)`) eliminates manual 0-vs-1 conversion.

### 1.5 Expression-Position `if let`, `some(...)`, and `!` / `else if` Diagnostics

**Hit in:** `cco22p3.cp`, `coci21c1p5.cp`, `noi19p4.cp`.

- `if let o = oi: ... else: ...` is rejected in expression position (`expected an expression (at 'let')`), even though `docs/one-page-cheat-sheet.md` §2 explicitly uses `let below = if let p = h[i] - 1: ... else: 0`. In `cco22p3.cp` this forced expanding a one-line expression into a 4-line `var keep = 0; if other == 1: if let o = oi: ...` block.
- `some(x)` is rejected (`unknown name some`); `?T` relies strictly on implicit lifting (`if lo == 0: none else: lo - 1`).
- Prefix `!` (instead of `not`) and `else if` (instead of `elif`) yield generic syntax errors (`expected an expression (at '!')`, `expected ':' (at 'if')`) instead of "did you mean `not` / `elif`?" hints.

### 1.6 Real C++ Undefined Behavior Eliminated by `?Int`

**Hit in:** `cco22p3.cp` (`corpus/submissions/4801968.cpp`).

The accepted C++ solution uses `--lower_bound(arr[!loc], {d + k + 1, -1})`. On the valid 4-interval input `2 2 5 / 0 2 / 10 12 / 8 9 / 15 20`, `4801968.cpp` crashes under `-fsanitize=address,undefined` with `AddressSanitizer: heap-buffer-overflow` (reading 4 bytes before `begin()`). Our CP Lang port (`cco22p3.cp`) returns `3` with zero sanitizer errors because `last_starting_le` returns `?Int` and forces handling `none` explicitly.

---

## 2. Raw vs Library-Stripped LOC Savings

All counts below are non-blank, non-comment code lines only. **C++ raw** is the full accepted submission in `corpus/submissions/`. **C++ library-stripped** strips debug/fast-I/O templates, `main()` boilerplate, and generic reusable data structures (custom open-addressed hash table, generic `ST`, `AddMin` segment tree), while keeping all problem-specific logic and bespoke helper structures.

- **ioi99p2 (`4896839.cpp`):** 45 CP LOC vs 83 C++ raw (**-45.8%**) and 52 C++ stripped (**-13.5%**). Clean win despite `[domain W]Char` ceremony.
- **cco16p6 (`4849378.cpp`):** 38 CP LOC vs 141 C++ raw (**-73.0%**) and 56 C++ stripped (**-32.1%**). `SegTree<Slot, Node>` + `(base - 1..=n + 1).first_where(...)` replaced the 60-line segment tree and 8-line manual binary search, while tightening bounds from `r = 2 * n` (relying on recursive-ST clamping) to the exact `Slot = 0..=n + 1` domain.
- **cco17p2 (`6416978.cpp`):** 29 CP LOC vs 145 C++ raw (**-80.0%**) and 95 C++ stripped (**-69.5%**). Largest win in the batch: `HashMap<(Int, Int), (Int, Int)>` plus pure Euclidean step-jumping memoization eliminated the 30-line custom splitmix64 hash table, 35-line operator-overloaded `A` monoid, and 30-line `1005 x 1005` precomputation table.
- **cco21p5 (`4554642.cpp`):** 36 CP LOC vs 136 C++ raw (**-73.5%**) and 47 C++ stripped (**-23.4%**). `AddMax<Slot>()` eliminated the entire 65-line custom lazy segment tree.
- **cco22p3 (`4801968.cpp`):** 61 CP LOC vs 98 C++ raw (**-37.8%**) and 65 C++ stripped (**-6.2%**). Modest stripped win because 8 lines of hand-written `last_starting_le` plus 4 lines from statement-only `if let` offset most of the brevity from `MinHeap` and `?Int`.
- **coci21c1p5 (`4086660.cpp`):** 66 CP LOC vs 189 C++ raw (**-65.1%**) and 100 C++ stripped (**-34.0%**). Packing all layers into one `SegTree<Flat, (Int, Int)>` eliminated both `ST` and the 48-line `DS` iterator wrapper.
- **noi06p1 (`5032743.cpp`):** 56 CP LOC vs 102 C++ raw (**-45.1%**) and 65 C++ stripped (**-13.8%**). Clean bitmask + tree DP with `[0..sz, 0..sz]Int = INF`, `(0..=m).rev`, and `best min= cand`.
- **noi06p3 (`3868871.cpp`):** 80 CP LOC vs 110 C++ raw (**-27.3%**) and 73 C++ stripped (**+9.6% longer**). First negative library-stripped result: we intentionally traded the original's `O(n * 25^2)` `10^5 x 25 x 2` global memo table for an `O(n * 25)` rolling bottom-up prefix/suffix-min DP, and spent 7 lines on `unique_sorted` (no `dedup` in `proto/cpc`) plus 6 lines on manual paired reads.
- **noi07p6 (`3756979.cpp`):** 33 CP LOC vs 98 C++ raw (**-66.3%**) and 61 C++ stripped (**-45.9%**). `Rooted(adj, root)` replaced manual `dfs`, `p[]`, and `d[]`, leaving only the leaf check and `MinHeap` walk.
- **noi19p4 (`3800719.cpp`):** 108 CP LOC vs 136 C++ raw (**-20.6%**) and 98 C++ stripped (**+10.2% longer**). Second negative library-stripped result: without an ordered `Set` (`lower_bound`/`erase`) or `first_ge` on `List<Int>`, we had to write a 58-line static X-segment tree with y-sorted lists and path-compressed `find_alive` (which does save ~2x memory vs `std::set<pair>` on NOI 2019 Jump's tight memory limit).

**Totals across all 10 problems:** **552 CP LOC** vs **1,238 C++ raw LOC (-55.4%)** and **712 C++ library-stripped LOC (-22.5%)**.

The pattern is crisp: whenever `proto/cpc` covers the core data structures (`SegTree`, `AddMax`, `HashMap`, `Rooted`, `MinHeap`, `first_where`), CP Lang beats library-stripped C++ by **13% to 70%**. The only two negative stripped results (`noi06p3` +9.6%, `noi19p4` +10.2%) occur exactly where `dedup` or ordered `Set` / `List.first_ge` is missing.

---

## 3. Bare-Core Fallback Friction

Across all 552 CP Lang lines, roughly **96 lines (17.4%)** exist solely to compensate for missing stdlib primitives in `proto/cpc`:

1. **No ordered `Set<T>` with `lower_bound` and `erase` (`noi19p4.cp`, ~58 LOC):**
   Range-harvesting 2D points (`lower_bound(qly)` through `qry` with deletion) forced a 58-line static X-segtree + path-compressed DSU-next-alive fallback.

2. **No `first_ge` / `last_le` on `List<T>` or key-projected `[D]Struct` (`cco22p3.cp`, `coci21c1p5.cp`, `noi19p4.cp`, ~25 LOC):**
   We hand-wrote the same 7–9 line binary-search loop three times (`last_starting_le`, `upper_time`, `lower_y`):

   ```cp
   fn lower_y(o: Int, target: Int) -> Int:
       var lo = 0
       var hi = bucket_y[o].len
       while lo < hi:
           let mid = (lo + hi) / 2
           if bucket_y[o][mid] < target: lo = mid + 1
           else: hi = mid
       lo
   ```

   Making `xs.first_ge(v)` / `xs.last_le(v)` (with optional `by:` key projection) work uniformly on both `[D]T` and `List<T>` removes every one of these loops.

3. **No `dedup` / `unique_sorted` (`noi06p3.cp`, ~7 LOC):**
   Deduplicating sorted candidate heights required a 7-line `unique_sorted` helper.

4. **Thin `AddMax` API and missing `MaxHeap` (`cco21p5.cp`, `noi07p6.cp`):**
   `AddMax<Slot>()` defaults every slot to `0` and has no `AddMin`, no `init:` fill value, and no point `.set`. `cco21p5.cp` had to negate values, pre-add `-INF` on `0..n-1`, and emulate `.set` via `let cur = tree.max(s, s); tree.add(s, s, -val - cur)`. Likewise `noi07p6.cp` negated `(-tree.depth(u), -u.idx)` to use `MinHeap` as a max-heap and manually added `+ 1` to rebuild `Node`.

5. **Manual `HashMap` memoization (`cco17p2.cp`):**
   `cco17p2.cp` is already 29 lines, and `@memo` over sparse keys (once graduated from fixed-domain `@memo`) would remove another 4 lines of `let key = (n, m); if key in memo: ... memo[key] = ans` boilerplate.

---

## 4. Non-Typeable Stress Constraints

Writing `proto/ports/scout_batch_30/check.py` confirmed that **every single one of these 10 hard problems has input or output invariants that range types alone cannot express**:

1. **Prefix-dependent element bounds (`cco16p6`):** Pirate `i` needs `arr[i]` votes with `1 <= arr[i] <= i + 1` — each element's upper bound depends on its own index.
2. **Reverse-simulated geometric feasibility (`cco17p2`):** Valid `(n, m)` inputs must be tileable by 2:1 rectangles; naive random pairs like `(121, 4125)` are un-tileable (hitting the `INF`/`-INF` sentinel path where C++ shifts `-INF` by `+k`). Stress testing required reverse-simulating 2:1 cuts from `(n, 0)`.
3. **Paired interval inequalities (`noi06p3`, `noi19p4`):** `a_i < b_i` per column in `noi06p3`; `1 <= lx <= rx <= w` and `1 <= ly <= ry <= h` per portal in `noi19p4`.
4. **Permutations, trees, and simple graphs (`coci21c1p5`, `noi07p6`, `cco21p5`):** `coci21c1p5` requires a permutation of `1..=n`; `noi07p6` requires a tree on `n >= 3` vertices; `cco21p5` requires a simple undirected graph (`u != v`, no duplicate edges).
5. **Strictly disjoint sorted schedules + reference-UB guard (`cco22p3`):** Both sides require strictly non-overlapping intervals `l_0 < r_0 < l_1 < r_1 < ...`. Differential-testing against `4801968.cpp` also required `s1[0].l <= s0[0].l + k` so the reference binary wouldn't crash on `--begin()`.
6. **Global graph reachability (`noi19p4`):** Every city `2..=n` must be reachable from city 1 via portals; random rectangles frequently isolate vertices (trapping `dist[i]!`), so the generator had to plant a spanning tree of point portals first.
7. **Alphabet skew & match-density tuning (`ioi99p2`):** Over a 26-letter alphabet, random strings rarely contain dictionary words as short subsequences; restricting the stress alphabet to 4–6 letters was essential to exercise overlapping DP transitions and backtracking.
8. **Multi-answer constructive `@checker` validators (`ioi99p2`, `coci21c1p5`):** Plain stdout comparison fails immediately whenever multiple optimal answers exist; both required semantic validators checking subsequence/non-overlap/span `<= 1000` (`ioi99p2`) and pairwise-disjoint strictly increasing index/value chains (`coci21c1p5`) plus optimality parity against the reference.

---

## 5. Priority Recommendations for v2

1. **Ship ordered `Set<T>` (`first`, `last`, `lower_bound`, `erase`) and unified `first_ge` / `last_le` (with `by:`) on both `List<T>` and `[D]T`:** removes 83 lines of bare-core fallbacks across `noi19p4`, `cco22p3`, and `coci21c1p5`.
2. **Unify `Str` as an indexable, iterable, storable ASCII `[domain S]Char`:** removes the `cannot index Str` / `cannot iterate Str` split hit in `ioi99p2` and documents `Int(c)` / `c.byte`.
3. **Make `[0..n]T` genuinely structural across function parameters:** fixes `expected [range]Int, got [range]Int` hit in `noi06p3`.
4. **Ship `dedup`, `MaxHeap`, `AddMin` (with `init:` and `.set`), and `Domain.from_idx`:** removes recurring boilerplate and 0-vs-1 off-by-one hazards in `cco21p5`, `noi06p3`, and `noi07p6`.
5. **Implement expression-position `if let` (matching the cheat sheet) and add parser hints for `!` -> `not` and `else if` -> `elif`.**
