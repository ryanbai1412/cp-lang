# Friction Backlog — Scout Batch 34: Hard USACO & DMOPC Contest Classics (v2 Spec)

## Scope and Corpus Coverage

Scout Batch 34 partitions the corpus strictly to problem codes starting with `usaco` or `dmopc` so parallel scout batches have zero overlap. Inspecting `corpus/submissions/info.json` (836 unique problem codes) shows **0 `usaco*` problems** and **58 `dmopc*` problems** (45 with AC submissions). After excluding every DMOPC problem already ported anywhere under `proto/ports/`, `scout_batch_*`, and `blind_critic_*` (including `dmopc21c6p4`, which landed in `scout_batch_31`), we selected 10 diverse hard DMOPC problems covering offline parallel binary search with dual Fenwick sweeps (`dmopc21c2p6`), interactive protocol design (`dmopc20c6p6`), rooted tree knapsack DP under a non-prime modulus (`dmopc21c2p5`), sparse hash-memoized divide-and-conquer DP (`dmopc21c8p5`), custom 5-field segment-tree monoids with offline compression (`dmopc21c7p4`), bidirectional permutation bijections (`dmopc21c2p2`), retrograde directed-graph game solving (`dmopc20c6p3`), run-length rolling DP (`dmopc21c2p4`), truncated square/octagon perimeter geometry (`dmopc20c7p2`), and order-statistic constructive permutations (`dmopc21c8p3`).

Every problem has both an idiomatic **v2 spec** port under `proto/ports/scout_batch_34/<code>.cp` (written strictly against `docs/one-page-cheat-sheet.md` at `3723dca`) and a runnable port under `proto/ports/scout_batch_34/today/<code>.cp` verified by `proto/ports/scout_batch_34/check.py` against the compiled original AC C++ submission on 30 randomized and edge-case inputs per problem (**300/300 passed, 0 failed**).

## Dual LOC Measurement

Across all 10 problems, raw non-blank/non-comment C++ totals **1,253 lines**. Stripping each submission's template header (`#include`, `dbgh`, `#ifdef DEBUG`, macros), `main()` boilerplate, and generic reusable data structures (the Fenwick `struct DS` in `dmopc21c2p6`, the `#ifdef LOCAL` grader in `dmopc20c6p6`, the custom open-addressing hash table in `dmopc21c8p5`, the generic recursive `struct ST` in `dmopc21c7p4`, and the `pb_ds` `ordered_set` typedef in `dmopc21c8p3`) leaves **757 library-stripped C++ lines**.

The locked v2 spec ports total **461 lines** — a **63.2% reduction vs raw C++** and a **39.1% reduction vs library-stripped C++** (42.0% library-stripped reduction when excluding the 22-line co-located `@interactor` block added to `dmopc20c6p6.cp`). The runnable `today/` ports total **548 lines** — a **56.3% reduction vs raw C++** and a **27.6% reduction vs library-stripped C++**.

Per-problem breakdown (raw C++ -> library-stripped C++ -> v2 spec -> runnable `today/`):

- `dmopc21c2p6` (`3968585.cpp`): 198 -> 130 -> 89 (-55.1% raw, -31.5% stripped) -> 101 runnable.
- `dmopc20c6p6` (`5200785.cpp`): 165 -> 99 -> 80 with `@interactor`, 58 solver-only (-51.5% raw, -19.2% / -41.4% stripped) -> 68 runnable.
- `dmopc21c2p5` (`3968384.cpp`): 119 -> 81 -> 42 (-64.7% raw, -48.1% stripped) -> 60 runnable.
- `dmopc21c8p5` (`4533533.cpp`): 91 -> 37 -> 19 (-79.1% raw, -48.6% stripped) -> 32 runnable.
- `dmopc21c7p4` (`4825739.cpp`): 177 -> 93 -> 77 (-56.5% raw, -17.2% stripped) -> 84 runnable.
- `dmopc21c2p2` (`3946515.cpp`): 97 -> 60 -> 28 (-71.1% raw, -53.3% stripped) -> 30 runnable.
- `dmopc20c6p3` (`3917634.cpp`): 118 -> 81 -> 32 (-72.9% raw, -60.5% stripped) -> 46 runnable.
- `dmopc21c2p4` (`3949962.cpp`): 90 -> 53 -> 25 (-72.2% raw, -52.8% stripped) -> 42 runnable.
- `dmopc20c7p2` (`3686612.cpp`): 107 -> 70 -> 35 (-67.3% raw, -50.0% stripped) -> 44 runnable.
- `dmopc21c8p3` (`4537755.cpp`): 91 -> 53 -> 34 (-62.6% raw, -35.8% stripped) -> 41 runnable.

The 87-line gap between ideal v2 (461) and runnable `today/` (548) is concentrated in five concrete prototype/stdlib features that already belong to the v2 spec: `@memo(hash)`, `Set.nth`, `runs()`, `if let` as an expression, and `bsearch_first` / `min=` on `?Int`.

## What Worked Cleanly in v2

Several v2 choices paid off immediately across hard contest problems without fighting the type system:

- **Typed domains caught index mix-ups naturally.** Separating `Pos`, `Val`, and `Q` in `dmopc21c2p6`, `Item` vs `Slot` vs `Tag` in `dmopc21c7p4`, and `V` vs `Deg` in `dmopc21c2p5` made the parallel binary search and coordinate-compressed segment tree dramatically easier to read than the original C++ arrays of raw `int`.
- **Optional values (`?Int`, `?Bool`, `?Jug`, `?Pt`) eliminated sentinel bugs.** None of the 10 ports needed `const inf = 1e9`, `-1` unvisited flags, or `memset(state, -1, sizeof state)`.
- **`Mod<MOD>` works cleanly even on non-prime moduli.** `dmopc21c2p5` uses modulus `998_244_352` (composite, because it is `998_244_353 - 1` in the exponent); `std.mod.Mod<998_244_352>` handled `+` and `*` with zero friction since it does not require primality unless `.inv()` or `/` is called.
- **`Graph.transpose()`, `Rooted<V>`, `Fenwick`, `SegTree`, and `Pt` operator overloading** all ran unchanged in `proto/cp.py lib-run` and matched C++ output on all 300 differential tests.

## Remaining Friction and Multi-Feature Collisions in v2

### UD1 — Shifted / Potentially Empty Domain Ranges on `Fenwick` and `SegTree` (`dmopc21c2p6`, `dmopc21c7p4`)

Domain indices prevent out-of-bounds bugs, which is great when indices are known to be in range, but two common contest patterns compute an arithmetic offset `pos - passes` or a split index `k in 0..=n` where the resulting prefix `[0..k)` or suffix `[k..n)` may be empty or start before `0`. Because `Fenwick.fold(l..=r)` and `SegTree.fold(l..=r)` require valid domain endpoints, both `dmopc21c2p6` and `dmopc21c7p4` had to wrap range queries in defensive helpers:

```cp
// dmopc21c2p6 — repeated in both parallel BS and suffix sweep
let tail = if let s = Pos.from_idx(max(0, mid - k)): cnt_bit.sum(s..) else: 0

// dmopc21c7p4 — splitting a segment tree at k in 0..=total
fn split_fold(k: Int) -> (Node, Node):
    let left  = if k <= 0:     Node.empty() else: st.fold(Slot(0)..=Slot(k - 1))
    let right = if k >= total: Node.empty() else: st.fold(Slot(k)..=Slot(total - 1))
    (left, right)
```

Notice the boundary mechanism from v2 (`Range<D>` formed from boundaries `b0..bk` where `bk` can be `D.first.l` or `D.last.r`) ALREADY has a clean representation for empty ranges! Making `Fenwick.fold` and `SegTree.fold` accept half-open boundary ranges (`Range<D>`, including empty ranges returning `monoid.id`) plus a clamped helper `D.clamp_boundary(idx: Int)` eliminates both wrappers without weakening bounds safety on element indexing.

### UD2 — Dynamic Domains Created Mid-Program After Preprocessing (`dmopc21c7p4`, `dmopc21c8p5`)

In `dmopc21c7p4`, the segment tree's compressed coordinate domain `Slot` cannot be declared until after reading both the initial `n` items and all `q` queries, merging them, and sorting. In `dmopc21c8p5`, the small dense base table `domain Small = 0..=max(2 * k + 2, 4)` depends on a runtime expression rather than a single input `n`. Both worked cleanly in `proto/cpi`, confirming that runtime-sized local `domain` declarations are essential in real contest code. The spec should state explicitly that `domain D = 0..expr` is legal at any block scope after `expr` is bound, and clarify whether `D` can be referenced in helper functions nested in that scope (closure over a local domain).

### UD3 — Interactive Protocols, `@interactor`, and Upfront Stdin Buffering (`dmopc20c6p6`)

Writing `dmopc20c6p6` exposed two distinct gaps for interactive problems:

1. **Spec ergonomics:** Every interactive helper repeats `print(...); flush()` and parses a single response token, and there is no standard way to tag an interactive query budget (`queries <= 4 * n`) inside `@interactor` other than manual counters and `assert`.
2. **Prototype/runner gap:** `proto/cpi/runner.py` reads the entire stdin upfront (`text.split()`), so a real interactive solver cannot talk over pipes to a grader process in `proto/cp.py lib-run`. In `today/dmopc20c6p6.cp`, we had to mirror the original C++ `#ifdef LOCAL` pattern by inlining the hidden-jug grader. Interactive sessions need streaming line/token reads in both the interpreter and `cpl run` / `cpl stress` harness so `@interactor` blocks execute the exact same solver code that is submitted to judges.

### UD4 — Rooted-Tree DP: Postorder Iteration and Jagged Subtree Dimensions (`dmopc21c2p5`)

`stdlib/tree.cp` exposes `tree.at: [Pre]V` and `tree.sub(v)`, which gives heavy-first preorder, so a bottom-up tree DP had to materialize a reverse-preorder list manually in `today/dmopc21c2p5.cp`:

```cp
var pre_order: List<V> = []
for p in tree.sub(V.first):
    pre_order.push(tree.at[p])
for u in pre_order.rev:
    ...
```

Adding `tree.postorder` (or allowing `for p in tree.sub(root).rev:`) removes this boilerplate immediately. Second, knapsack tree DP (`dp[u]` has size `tree.size[u] + 1`, merged across children in O(n^2) total via `csum`) naturally uses jagged `[V]List<Z>` tables indexed by raw `Int` sizes `0..=sz_u` alongside a fixed domain `Deg = 0..26`. Trying to force `0..=sz_u` into a per-vertex dependent domain would be unergonomic; mixing domain-indexed outer dimensions (`[Deg]`) with `List<Z>` inner knapsack dimensions felt clean, though `cur: [Deg]List<Z> = [Deg] _ => [Z(0) for _ in 0..=sz_u]` was slightly verbose.

### UD5 — Hybrid Dense-Base + Sparse `@memo(hash)` DP (`dmopc21c8p5`)

In `dmopc21c8p5`, the original C++ needed 25 lines of a hand-rolled open-addressing hash table (`keys[1<<23]`, linear probing) because `std::unordered_map` TLEs or blows memory. In v2, `@memo(hash)` turns the whole solver into 19 lines:

```cp
@memo(hash)
fn dp(x: Int) -> Int:
    if x <= base: return sm[Small(x)]!
    let lo = x / 2 - k / 2 - 1
    min(dp(i) + dp(x - i) for i in lo..=(lo + k))
```

Two design points surfaced here: first, `@memo(hash)` is not yet implemented in `proto/cpi` (forcing manual `var cache = Map<Int, Int>()` in `today/dmopc21c8p5.cp`), and second, `@memo(hash)` must lower in C++17 to a fast flat/anti-hash table (not `std::unordered_map`) so contest code never has to fall back to hand-rolled linear probing for large sparse recursion trees.

### UD6 — Multi-Feature Collision: Offline Compression + Custom Monoid + Descending Sort (`dmopc21c7p4`)

`dmopc21c7p4` combines four features simultaneously: dynamic domain `Slot`, descending sort by a derived key `a - b`, a 5-field non-commutative `Monoid<Node>`, and binary search over a split fold. Two small frictions compounded here:

1. Sorting descending by a key function required negating the sort key into a leading tuple field (`key: -(a - b)`) in `today/dmopc21c7p4.cp` because `sorted()` sorts ascending lexicographically without a `by:` / `rev:` comparator parameter.
2. Inside the query loop, `bsearch_first` needs to destructure `(l, r) = split_fold(mid)` before checking `l.sm + l.mxb >= x or l.sm - l.lastd + r.mxa >= x`, which collides with UD7 (single-expression lambdas).

### UD7 — Single-Expression Lambdas vs Multi-Line Search Predicates (`dmopc21c2p6`, `dmopc21c7p4`)

The locked cheat sheet allows only single-expression lambdas (`x => expr`). Whenever a predicate for `bsearch_first` or a key function needs one intermediate `let` binding, you must hoist a named nested `fn`:

```cp
// dmopc21c7p4
fn reaches(mid: Int) -> Bool:
    let (l, r) = split_fold(mid)
    l.sm + l.mxb >= x or l.sm - l.lastd + r.mxa >= x
let hit = bsearch_first(0..=pos_count, reaches)
```

This is actually quite readable — arguably cleaner than multi-line inline closures in indentation-sensitive syntax — so the recommendation is **keep single-expression lambdas** (protecting concept budget and parser simplicity) and explicitly teach the nested `fn` idiom in docs, while making sure nested `fn` can capture enclosing locals cleanly in both `proto/cpi` and the Rust compiler (confirmed working in `proto/cpi` for `dmopc21c2p6` and `dmopc21c2p4`!).

### UD8 — Same-Container `swap(a[i], a[j])` Under Borrow Checking and Two-Domain Bijections (`dmopc21c2p2`)

`dmopc21c2p2` maintains a `Pos <-> Val` bijection (`perm: [Pos]Val` and `inv: [Val]Pos`) under position swaps (`'E'`) and value swaps (`'B'`). Typed domains shine here — swapping by value (`swap_val(u: Val, v: Val)`) vs swapping by position (`swap_pos(i: Pos, j: Pos)`) can never be accidentally reversed! However, swapping two elements in `perm` hits a classic borrow-checker snag when `i == j`:

```cp
fn swap_pos(i: Pos, j: Pos):
    if i == j: return
    let (u, v) = (perm[i], perm[j])
    perm[i] = v
    perm[j] = u
    inv[u] = j
    inv[v] = i
```

Under lexical disjoint-borrow rules, `swap(perm[i], perm[j])` cannot prove `i != j` without a runtime guard or a built-in `perm.swap(i, j)` method (which is safely a no-op when `i == j`). Providing `.swap(i, j)` on every `[D]T` array and `List<T>` avoids both the manual 4-line temp dance and borrow-checker friction on same-container swaps. Similarly, reading single-character command tokens (`'B'`, `'E'`, `'Q'`) had to fall back to `let kind: Str = read()` in `today/dmopc21c2p2.cp` because `read()` for `Char` and `match` on `Char` literals are not yet uniform in `proto/cpi`.

### UD9 & UD10 — Retrograde Game BFS and Short-Circuiting Predicates Over Adjacency (`dmopc20c6p3`)

`dmopc20c6p3` is a classic retrograde game on a directed graph. `g.transpose()` in `stdlib/graph.cp` eliminated the manual reverse-graph build cleanly! The remaining friction was computing `res` from outgoing neighbors with optional `state[arc.to]`:

```cp
// Ideal v2:
let res = if out_deg[u] == 0: true
    elif want[u]: any((state[e.to] ?? false) for e in g.adj(u))
    else:         all(state[e.to]! for e in g.adj(u))
```

In `today/dmopc20c6p3.cp`, this expanded to 12 lines of manual loops because `any(...)` / `all(...)` over generator expressions with optional unwrapping were not available, and `q` used a `List<V>` as a stack (`pop()` works for retrograde DAG peeling, while general BFS queues need `Deque<V>`). Ensuring `any`/`all`/`sum`/`min`/`max` over generators are first-class in both the Rust frontend and `proto/cpi` cuts a full third of the lines on graph and DP problems.

### UD11 — Optional `min=` / `max=` Update Ergonomics and Statement-Only `if let` (`dmopc21c2p4`, `dmopc21c8p5`, `dmopc20c7p2`)

This was the **#1 most frequent friction** across the batch (occurring in 3 of 10 problems). Whenever DP tables or search trackers store `?Int` or `?Pt` instead of an `INF` sentinel, relaxing a candidate value takes 4 lines:

```cp
// Repeated in dmopc21c2p4, dmopc21c8p5, dmopc20c7p2:
if let cur = ndp[st]:
    if val < cur: ndp[st] = val
else:
    ndp[st] = val
```

Compare that to a single optional-aware `min=` / `max=` (or `ndp[st].chmin(val)` where `none` acts as the identity `+inf` for `min=` and `-inf` for `max=`):

```cp
ndp[st] min= val
```

One tiny rule — making `min=` and `max=` treat `none` on the left-hand side as unset (assigning the right-hand value) — eliminates the main ergonomic tax of replacing sentinel `INF` with `?Int`, without adding any new keyword!

Separately, in `proto/cpi`, `if let x = opt: a else: b` failed to parse in expression position (`expected an expression (at 'let')`), forcing rewrites to `opt ?? fallback` or statement-form `if let`. Supporting `if let` expressions uniformly (or documenting `??` and `match` as the expression forms) avoids surprise parse failures.

### UD12 — `Pt` vs `Pt<Int>`, Generic Parser Ambiguity on Tuple Args, and Struct `Ord`/`Hash` (`dmopc20c7p2`)

Three subtle discrepancies surfaced in `dmopc20c7p2`:

1. **Spec vs stdlib:** `docs/one-page-cheat-sheet.md` writes `Pt<Int>` / `Pt<F64>`, while `stdlib/geometry.cp` defines a non-generic `pub struct Pt(x: Int, y: Int)`.
2. **Parser bug in `proto/cpi`:** Writing `var nxt = Map<(Int, Int), List<Step>>()` threw `expected end of line (at ',')` because `<` followed by `(` was parsed as less-than instead of a generic type argument list. Aliasing `type Key = (Int, Int)` or using `Map<Pt, List<Step>>()` bypassed it. The Rust parser should disambiguate `Type<...>` cleanly even when a type argument starts with `(` (or consistently require `::<>` in expression position as noted in `decisions.md`).
3. **Structural `Eq`/`Ord`/`Hash` on value structs:** `proto/cpi` automatically supports `Pt` as a `Map` key and `<` comparison (`Pt(1, 2) < Pt(1, 3)` evaluates to `true`), even though `stdlib/geometry.cp` only exports `const ByXY = Order<Pt>(...)` and does not implement `Ord` or `Hash` on `Pt`. The v2 spec should make structural `Eq`/`Ord`/`Hash` derivation on plain value structs explicit (default-derived when all fields satisfy it, or explicit `@derive`) so `Map<Pt, ...>` and `min(pts)` work reliably in C++17 codegen too.

### UD13 — Order-Statistic `Set.nth(k)` vs `Fenwick.lower_bound` Fallback (`dmopc21c8p3`)

In `dmopc21c8p3`, the original C++ pulls in `<ext/pb_ds/assoc_container.hpp>` solely for `s.find_by_order(k)` and `s.erase`. In v2, an ordered `Set<Pos>` with `.nth(k)` replaces `pb_ds` cleanly in 4 lines. Because `Set.nth` is not yet implemented in `proto/cpi`, `today/dmopc21c8p3.cp` used `Fenwick<Pos, Int>.lower_bound(offsets[v] + 1)` on an array of 1s — which worked cleanly and passed 30/30 differential tests, and in fact runs faster with smaller constant factor when keys come from a dense domain `0..n`! This suggests a sharp stdlib design insight: when keys are a dense domain `D`, `Fenwick.lower_bound` already covers order-statistic extraction cleanly, while `Set.nth(k)` remains valuable specifically for sparse/unbounded `Int` keys.

## Summary of Priority Actions for the Language Architect

1. **Highest leverage (touches 3/10 problems):** Define `opt min= val` / `opt max= val` semantics on `?T` (treating `none` as identity) so `?Int` beats `INF` sentinels on both safety and conciseness.
2. **High leverage (touches 2/10 problems + broad CP utility):** Allow `Fenwick.fold` and `SegTree.fold` to accept possibly-empty `Range<D>` boundaries returning `monoid.id`, and add `tree.postorder` to `std.tree.Rooted`.
3. **High leverage (borrow-checker ergonomics):** Provide `.swap(i, j)` on every `[D]T` array and `List<T>` so same-container swaps (`i == j` no-op) never fight disjoint-borrow rules.
4. **Interpreter & compiler alignment:** Close the `proto/cpi` vs v2 gaps uncovered here (`@memo(hash)`, streaming stdin for interactive `@interactor` runs, `runs()`, `any`/`all` generators, `sorted(by:)` / descending sort, `read()` for `Char`, `Map<(T, U), V>()` parser handling, and aligning `Pt` vs `Pt<Int>` with explicit structural `Eq`/`Ord`/`Hash` rules).
