# Friction Backlog — Scout Batch 36: Hard CCC & CCO Contest Classics

Scout Batch 36 ported 10 diverse, previously unported hard problems from the `ccc*` / `cco*` prefix bucket against the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md`, commit `3723dca`), paired with runnable `today/` ports verified differentially against the original accepted C++ submissions across 1,085 random and adversarial test cases.

The ten problems span monotone-stack lazy segment tree DP (`ccc02s4hard`), triangular 2D sparse-table DP (`ccc19s5hard`), divide-and-conquer in-place merge sort (`cco18p6`), floating-point Li Chao trees (`cco19p4`), offline segment-tree-over-time with rollback Fenwick trees and lazy-deletion heaps (`cco20p5`), run-length-encoded greedy stacks (`cco22p2`), binary-tree modular combinatorics DP (`cco22p4`), two-graph small-to-large DSU rollback (`cco22p5`), modular ordered-set sweep with successor queries (`cco23p2`), and binary forbidden-substring avoidance (`cco23p4`).

All nine integer/string `today/` ports compile via `proto/cp.py build` into C++17, run cleanly under `-fsanitize=address,undefined -fno-sanitize-recover=all`, and match the original accepted C++ (plus independent constructive checkers for `ccc02s4hard`, `cco18p6`, and `cco22p2`). Because `proto/cpc` does not yet implement floating-point `F64`, `today/cco19p4.cp` was verified through the reference interpreter (`proto/cp.py lib-run`) against `3617807.cpp` within `1e-5` across 45 random instances.

---

## Dual LOC Measurement (Raw vs Library-Stripped)

Every line count below counts non-blank, non-comment lines of code. For the library-stripped C++ baseline, we removed each submission's `#include`/macro/debug header, `main()` fast-I/O wrapper, and generic standard-library-equivalent data structures that a complete C++ CP library already provides (`struct ST` lazy segment tree in `ccc02s4hard`, `min_pq`/`chash`/`avec` wrappers in `cco20p5`, `bpow`/`fact`/`ifact` in `cco22p4`, generic `struct DSU` in `cco22p5`, and `struct mint` in `cco23p2`), while keeping every problem-specific data structure (the triangular buffer in `ccc19s5hard`, the per-rank Li Chao tree in `cco19p4`, the rollback Fenwick + time segtree in `cco20p5`, and the RLE stack in `cco22p2`).

Across all 10 problems, total code shrank from **1,586 raw C++ LOC** and **1,038 library-stripped C++ LOC** down to **615 ideal v2 LOC** (**-61.2%** vs raw, **-40.8%** vs library-stripped) and **1,107 runnable `today/` LOC** (**-30.2%** vs raw):

- **`ccc02s4hard`** (`4157744.cpp`): 200 raw / 86 stripped -> **66 v2** (-67.0% raw, -23.3% stripped), 133 `today/`. `LazySegTree` with an explicit `Action` deletes 87 lines of manual push/pull tree code, though the dual-field `Entry` + `?Entry` merge still costs 20 lines.
- **`ccc19s5hard`** (`5119931.cpp`): 82 raw / 50 stripped -> **27 v2** (-67.1% raw, -46.0% stripped), 31 `today/`. Flat triangular indexing `tri(i, j)` plus tuple swap `(cur, nxt) = (nxt, cur)` eliminates the custom `struct A` wrapper and `vector<vector<int>>` input copy.
- **`cco18p6`** (`4575245.cpp`): 101 raw / 71 stripped -> **40 v2** (-60.4% raw, -43.7% stripped), 73 `today/`. Slice sorting `a[l..r].sorted()`, `a[l..mid].first_ge(med)`, and nested closures `flip`/`merge`/`dnc` replace the `Solver` class boilerplate cleanly.
- **`cco19p4`** (`3617807.cpp`): 122 raw / 93 stripped -> **60 v2** (-50.8% raw, -35.5% stripped), 86 `today/`. Clean `F64` arithmetic and typed per-rank Li Chao trees remove manual pointer/offset sizing, though optional `?F64` comparisons still add friction.
- **`cco20p5`** (`4562195.cpp`): 259 raw / 187 stripped -> **99 v2** (-61.8% raw, -47.1% stripped), 277 `today/`. Stateful `struct RollbackBit` methods, tuple-key `HashMap<(Int, Int), List<Int>>`, and `Heap<(Int, Int, Int)>` cut nearly half the stripped C++ — and account for 178 of the 492-line `today/` gap.
- **`cco22p2`** (`4801882.cpp`): 132 raw / 102 stripped -> **64 v2** (-51.5% raw, -37.3% stripped), 82 `today/`. Typed `Bus`/`Mkt` domains and direct `stack` mutation remove the intermediate `vector<pair<int, long>>` return allocation from `ds.pop(need)`.
- **`cco22p4`** (`4801884.cpp`): 150 raw / 92 stripped -> **64 v2** (-57.3% raw, -30.4% stripped), 99 `today/`. `Mod<MOD>`, `Comb<MOD>`, and `@memo` erase modular exponentiation, inverse factorials, and manual `-1` memo initialization.
- **`cco22p5`** (`4801885.cpp`): 174 raw / 113 stripped -> **76 v2** (-56.3% raw, -32.7% stripped), 112 `today/`. Compact rollback history plus `HashMap<(V, V), Int>` replaces manual packed-key hashing and raw pointer arrays.
- **`cco23p2`** (`5579928.cpp`): 192 raw / 97 stripped -> **50 v2** (-74.0% raw, -48.5% stripped), 125 `today/`. `Mod<1_000_003>`, `Map<Int, List<V>>`, and `Set.first_gt` replace a 79-line `struct mint` and cut `today/`'s 75-line coordinate-compressed min-segtree workaround down to two lines.
- **`cco23p4`** (`5579929.cpp`): 174 raw / 147 stripped -> **69 v2** (-60.3% raw, -53.1% stripped), 89 `today/`. `match t:` over string literals plus slice equality `w[i..i + pat.len] == pat` collapses 14 `else if` branches and 4 duplicate 3-char substring counters into one helper.

---

## Friction Findings and Multi-Feature Collisions

### F1. Multi-feature collision: small-to-large DSU merge under lexical borrowing (`cco22p5`)

In `cco22p5`, merging component `u` into component `v` after checking `if u != v:` naturally reads as iterating `mem[u]` while pushing each vertex into `mem[v]`:

```cp
if u != v:
    if mem[u].len > mem[v].len: (u, v) = (v, u)
    for x in mem[u]:
        rep[x] = v
        mem[v].push(x)
    mem[u].clear()
```

When compiled today in `proto/cpc`, this fails with a container-level borrow error: `` `mem_b[V(v)]` can't be changed here: the `for` loop is going over `mem_b`, and that lends it for reading until the loop ends``. The workaround in `today/cco22p5.cp` drops back to index looping (`for idx in 0..mem_b[V(u)].len: let x = mem_b[V(u)][idx]`), which defeats `for x in ...`.

In locked v2, `law disjoint` covers simultaneous disjoint mutable borrows via projections, but it does not prove that `mem[u]` (immutably lent by `for`) and `mem[v]` (mutably borrowed by `.push`) are disjoint inside an `if u != v:` guard. Because small-to-large DSU / sack-on-tree merging is one of the highest-frequency Gold/Platinum patterns in CP, adding a 3-line stdlib `List.take() -> List<T>` (compiling to `std::exchange(v, {})`) solves this cleanly with zero borrow-checker special cases: `for x in mem[u].take(): mem[v].push(x)` moves the source bucket out in O(1), leaves `mem[u]` empty automatically, and never aliases `mem` during iteration.

### F2. C++ codegen global-namespace collisions with `<bits/stdc++.h>` and POSIX headers (`ccc02s4hard`, `cco22p5`)

Two completely ordinary CP variable names in this single 10-problem batch caused hard `g++` compilation failures after `proto/cpc` emitted clean-looking C++:

- In `today/ccc02s4hard.cp`, `let tm: Int = read()` emitted global `ll tm;`, colliding with POSIX `struct tm` from `<ctime>`.
- In `today/cco22p5.cp`, `var dup = 0` emitted global `ll dup;`, colliding with POSIX `extern int dup(int)` from `<unistd.h>` (transitively pulled in by `<bits/stdc++.h>` -> `<csignal>`).

Under contest pressure, hitting `error: reference to 'tm' is ambiguous` or `error: 'll dup' redeclared as different kind of entity` from a completely legal CP Lang identifier directly violates the P0 "zero compiler fights" goal. Other common CP names (`y0`, `y1`, `j0`, `j1`, `next`, `prev`, `left`, `right`, `rank`, `hash`, `data`, `size`, `link`, `pipe`, `time`) carry the exact same landmine whenever user variables live in C++'s global namespace alongside `using namespace std;`.

Fix for `crates/cp-emit`: emit all user-defined globals and functions inside `namespace cp_user { ... }` (invoked from `int main()` via `cp_user::cp_main()`), and prefix any identifier matching a C++ keyword or POSIX macro (`errno`, `index`, `major`, `minor`). That keeps the emitted `.cpp` completely readable for judges while permanently eliminating host-header name collisions.

### F3. Missing `Heap<T>` and stateful `struct` methods explode multi-instance DS code (`cco20p5`)

`cco20p5` exposed the single sharpest `today/`-vs-v2 blowup in the batch: **99 lines in ideal v2 vs 277 lines in `today/` (+178 lines)**. Almost all 178 extra lines came from two exact features:

1. **No `Heap<T>` (`MinHeap`/`MaxHeap`) with lexicographic tuple ordering**: `cco20p5` maintains three lazy-deletion heaps (`by_r`, `by_l`, `by_len`). Without `Heap<(Int, Int, Int)>` in `proto/cpc`, `today/cco20p5.cp` had to write three copy-pasted binary heap push/pop pairs (~110 lines) because functions in `proto/cpc` also cannot take `inout List<HItem>` cleanly with globals.
2. **No stateful `struct` methods (`RollbackBit.insert` / `RollbackBit.rollback`)**: `cco20p5` runs two identical mirrored rollback Fenwick trees (`fwd` and `rev`). In ideal v2, `struct RollbackBit` is written once (25 lines) and instantiated twice (`var fwd = ...; var rev = ...`). Without `struct` methods or `inout` container parameters, `today/cco20p5.cp` duplicated `b1`/`b2`, `ins1`/`ins2`, and `rb1`/`rb2` verbatim (~65 extra lines).

This confirms that `Heap<T>` (default min-heap with tuple ordering, mirroring `Set`/`Map` ordering) and mutable `struct` methods (or `inout` struct parameters) are both load-bearing for CCO/ICPC-level data-structure composition — not optional sugar.

### F4. Repeated boilerplate combining optional structs and floats (`?Entry`, `?F64`) (`ccc02s4hard`, `cco19p4`)

Locked v2's `min=`/`max=` on `?Int` worked cleanly (`ans min= cand` in `cco22p5`), and `none` prevents `INF` sentinel bugs. However, when a problem carries an optional struct (`?Entry` in `ccc02s4hard`) or optional float (`?F64` in `cco19p4`), combining or comparing two optional values repeatedly forced 6-to-10-line nested `if let` ladders:

```cp
// ccc02s4hard.cp — monoid combine of two ?Entry values
fn merge_entry(a: ?Entry, b: ?Entry) -> ?Entry:
    if let x = a:
        if let y = b:
            let (dc, da) = min((x.dp_cost, x.dp_at), (y.dp_cost, y.dp_at))
            let (tc, ta) = min((x.tot_cost, x.tot_at), (y.tot_cost, y.tot_at))
            Entry(dc, da, tc, ta)
        else: x
    else: b

// cco19p4.cp — comparing two ?F64 evaluations in Li Chao
fn beats(a: ?F64, b: ?F64) -> Bool:
    if let x = a:
        if let y = b: x > y else: true
    else: false
```

Supporting tuple matching on optionals (`match (a, b):`) or a 5-line stdlib combinator `opt_merge(a, b, f)` (plus `min=`/`max=` on `?F64`) needs zero new syntax and removes this repeated nested-`if let` ceremony across DP and segment-tree problems.

### F5. `Str` slicing, literal `match`, and `List` slice algorithms (`cco23p4`, `cco18p6`)

Two string/slice ergonomics gaps showed up cleanly when comparing ideal v2 against `today/`:

- In `cco23p4`, ideal v2 handles all 14 forbidden patterns with one `count_sub(w, pat)` helper using `w[i..i + pat.len] == pat` and a 14-arm `match t:` over string literals (69 lines total). Because `proto/cpc` cannot index/slice `Str` or `match` string literals, `today/cco23p4.cp` had to convert `s` into `List<Int>`, write three separate `count1`/`count2`/`count3` helpers, and decode `t[T(0..2)]` into nested `if`/`elif` branches (89 lines).
- In `cco18p6`, ideal v2 writes `let med = a[l..r].sorted()[(r - l) / 2]` and `let m1 = l + a[l..mid].first_ge(med)` in three lines. Without slice views (`a[l..mid]`) supporting `.sorted()` and `.first_ge(x)` in `proto/cpc`, `today/cco18p6.cp` wrote a manual copy loop plus a hand-rolled binary search `lb(l, mid, med)` (+33 lines).

Making `Str` directly indexable/sliceable (`s[i]`, `s[l..r] == pat`, `match t:`) and giving `List` slices `.sorted()` / `.first_ge(x)` / `.rev()` directly preserves v2's unified indexed-slice design with zero special-case syntax.

### F6. Multi-feature collision: `LazySegTree<V, T>` with dual-field cross-overwrite + point updates (`ccc02s4hard`)

`ccc02s4hard` stresses v2's `LazySegTree<V, T>` in two subtle ways at once:

1. Its lazy range assignment (`cur_max = t` over `[l..=r]`) does **not** add to `tot_cost` — it overwrites `tot_cost = dp_cost + tag` and copies `tot_at = dp_at` from the node's *other* field. This works in v2's `Action` interface cleanly because `Action.apply(node: ?Entry, tag: Int) -> ?Entry` sees the full `Entry` aggregate.
2. Inside the same DP loop, it interleaves range updates (`seg.apply(l..=r, tag)`) with point updates (`seg.set(i, new_entry)`). That means `LazySegTree.set(pos, val)` **must** push all pending ancestor tags along the root-to-leaf path before overwriting leaf `pos`, or a stale ancestor tag will later corrupt the freshly written `dp[i]`.

Pinning `ccc02s4hard` into the `stdlib/lazy.cp` / `crates/` conformance suite will protect both invariants permanently.

### F7. Mutable `back()` / last-element mutation on stacks (`cco22p2`)

In `cco22p2`, the run-length-encoded stack repeatedly decrements `cnt` on its top element and pops when it hits zero:

```cp
stack[stack.len - 1].cnt -= take
if stack.back()!.cnt == 0: stack.pop()
```

Notice the awkward asymmetry: `stack.back()!` reads the top element concisely, yet modifying a field of the top element drops back to `stack[stack.len - 1].cnt -= take` because `back()` returns an immutable value (no stored/returned mutable borrows in v2). Supporting either `stack.last_mut` via a `lends mut` callback, or negative/boundary shorthand `stack[..].r` / `stack.top.cnt -= take` (or documenting `stack[stack.len - 1]` as the canonical zero-magic idiom) is worth clarifying so users never hesitate between `.back()!` and `[len - 1]`.

### F8. High-value stdlib / compiler priority summary from the 492-line `today/` delta

Quantifying the 492-line gap between `today/` (1,107 LOC) and ideal v2 (615 LOC) across all 10 problems yields a clear priority order for `crates/`:

1. **`Heap<T>` + stateful `struct` methods + tuple-key `HashMap`** (`cco20p5`, `cco22p5`: ~210 lines saved).
2. **`Set.first_gt` / `first_ge` / `last_le` + `Mod<P>` + `Comb<P>`** (`cco23p2`, `cco22p4`: ~110 lines saved).
3. **`LazySegTree<V, T>`** (`ccc02s4hard`: ~67 lines saved).
4. **Slice `.sorted()` / `.first_ge()` + indexable/sliceable `Str` + `F64` + `sqrt`** (`cco18p6`, `cco23p4`, `cco19p4`: ~79 lines saved).
5. **Parallel tuple swap `(a, b) = (b, a)`** (`ccc19s5hard`, `cco18p6`, `cco22p5`: removes temporary `tmp` / ping-pong buffer indexing everywhere).
