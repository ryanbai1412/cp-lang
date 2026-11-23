# Idiomatic CP Lang: lint families (draft 1)

Status: non-normative. This is taste guidance, not the language: `spec/core`, `spec/lib`,
`std` and `spec/tools` define CP Lang, and where this file disagrees with them, they win.
Whether it becomes Ryan's normative taste is open in `plan/QUESTIONS.md` (idioms). Changes go
through the idioms lane; a lint in `spec/lints/` should cite the practice it serves.

The aim is an exhaustive first inventory: every pattern we can name
today where legal code is non-idiomatic, slower than it looks, or a P0 bug waiting to happen.
It will still grow; see "Adding a lint". `principles.md` says why; this file says what.

## Legend

**Tier**
- `fix`: autofix that never changes meaning, cost class or trap behaviour. Safe to apply in bulk.
- `fix?`: autofix offered, reviewed by a person; it may change evaluation order, cost or which
  input traps (for example, a loop that silently skipped a bad value now traps).
- `warn`: detected, no rewrite (the right replacement depends on intent).
- `doc`: judgment. Documented with examples in the guide, never flagged.

**P**: the practice served (#1–#14 in `principles.md`, Part 2).

**Status**: `✓name` exists in `cpl-lint` today under that name; blank is new.

**Engine**: what the detector needs. `syn` syntax only; `ty` types and names; `loop` a
loop-shape recognizer over the typed IR (body is a reduction, search, push or table fill);
`flow` def-use or ownership facts; `bound` value ranges from declarations (unique to CP Lang:
we know `n <= 2e5`, so we know `O(n²)` is too slow).

## Entry schema (machine side)

Each lint is one record, owned by whoever owns the names it mentions (decision below):

```toml
[lint.hand_rolled_prefix_sum]
code       = "L0441"
family     = "S.prefix"
principle  = "#1"
tier       = "fix"
engine     = "loop"
summary    = "manual `p[i + 1] = p[i] + x`: use `Prefix(a)`"
message    = "this loop builds prefix sums by hand; `Prefix(a)` holds them and `.fold(l..r)` reads a range"
before     = "lints/hand_rolled_prefix_sum/before.cp"   # passes cpl check, prints X
after      = "lints/hand_rolled_prefix_sum/after.cp"    # passes cpl check, prints X, lint-clean
not        = ["lints/hand_rolled_prefix_sum/not-1.cp"]  # near misses that must not fire
docs       = "https://cp-lang.org/lints/hand_rolled_prefix_sum"
evidence   = ["corpus/problems/cf1234a", "skill-eval/2026-11-22/p07"]
```

`before`/`after`/`not` are tested like conformance cases: both programs check and print the same
output on the case's input, `after` is lint-clean, `cpl lint --fix before.cp` produces `after`
byte for byte (for `fix` tiers), and no `not-*` file fires. That one test makes the lint, the
autofix, the docs page, the skill's rewrite table and the site's guide impossible to drift.

## Where lints live (proposed decision)

`spec/tools/fmt-lint.md` already says: "a lint is declared by the library that owns the names it
mentions; the linter has no hard-coded stdlib names." Today's `hand_rolled_*` lints break that.
Proposal:

1. **Core lints** (families D, O, W, M, K, R, N, C, T, E) live in `cpl-lint`. They mention only
   core concepts.
2. **Library lints** (family S, most of H) are declared in the stdlib module they recommend,
   next to the API, as *rewrite rules*: a CP Lang pattern plus a template, matched on the typed IR
   after loop-shape normalization. User libraries get the same power (no stdlib privilege), so a
   personal library can teach its own idioms.
3. **Loop recognizers** (sum/count/search/push/table-fill/prefix/BFS-queue/heap-relax shapes) are
   a small core engine both kinds share. Library rules say "this recognized shape, with these
   ops, becomes this call".

---

## S. The standard library would say it shorter (the largest family)

One sub-family per module. Each says: you rebuilt something the library has.

### S.seq: reductions

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| loop_reduction | loop that only accumulates sum, count, max or min | `xs.sum()`, `xs.count(p)`, `xs.max()` | fix | #2 | loop | ✓loop_reduction |
| loop_product | `acc *= f(x)` loop | `xs.map(f).product()` | fix | #2 | loop | |
| loop_xor_fold | `acc ^= x` loop | `xs.fold(Xor)` | fix | #2 | loop | |
| loop_gcd_fold | `g = gcd(g, x)` loop | `xs.fold(gcd)` | fix | #2 | loop | |
| loop_argmax | track best value and its index | `xs.argmax()`, `D.max(by: f)` | fix | #2 | loop |✓loop_argmax |
| loop_max_by_key | track the element with largest `f(x)` | `xs.max(by: f)` | fix | #2 | loop | |
| max_then_count | `mx = xs.max()` then `xs.count(x == mx)` | `xs.max_count()` | fix | #1 | ty | |
| loop_mean | sum then divide by len | `xs.mean()` | fix | #2 | loop | |
| search_loop | flag or `?T` set then break | `xs.first(p)`, `xs.any(p)`, `xs.all(p)` | fix | #2 | loop | ✓search_loop |
| loop_contains | loop comparing to a value then break | `x in xs` / `xs.contains(x)` | fix | #2 | loop | |
| loop_find_index | first key whose value satisfies `p` | `xs.keys.find(k => p(xs[k]))` | fix | #2 | loop | |
| loop_last_index | backwards scan for last match | `xs.rfind(p)` | fix | #2 | loop | |
| loop_first_some | first `f(x)` that is `Some` | `xs.first_some(f)` | fix | #2 | loop | |
| all_some_loop | collect `?T`s, fail if any `None` | `xs.all_some()` | fix | #5 | loop | |
| majority_loop | Boyer–Moore vote by hand | `xs.majority()` | fix | #1 | loop | |
| loop_min_k | sort then take first k | `xs.min_k(k, by:)` / `max_k` | fix? | #10 | ty | |
| sort_for_extreme | `xs.sorted()[0]` or `.last` | `xs.min()` / `xs.max()` | fix | #10 | ty | ✓sort_for_extreme |
| free_fold | `sum(xs)`, `count(xs, p)`, `sort(mut xs)` | method on the sequence | fix | #13 | ty | ✓free_fold |

### S.seq: building

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| push_loop | list built by a loop that only pushes | comprehension `[f(x) for x in xs]` | fix | #2 | loop | ✓push_loop |
| filter_push_loop | push under an `if` | `xs.filter(p)` / comprehension with `if` | fix | #2 | loop | |
| flat_push_loop | nested loops that only push | `xs.flat_map(f)` / two-`for` comprehension | fix | #2 | loop | |
| zip_index_loop | `for i in keys: f(a[i], b[i])` over same domain | `a.zip(b)` | fix | #2 | loop | |
| index_only_loop | `for i in 0..xs.len` that only reads `xs[i]` | `for x in xs` | fix | #3 | loop | ✓index_only_loop |
| index_and_value_loop | reads both `i` and `xs[i]` | `for (i, x) in xs.items` | fix | #3 | loop | |
| pairs_nested_loop | `for i` / `for j > i` pairs | `xs.pairs()` / `combinations(2)` | fix | #2 | loop | |
| adjacent_index_loop | reads `xs[i]` and `xs[i + 1]` | `xs.adjacent` / `with_next` | fix | #3 | loop | |
| cyclic_index_loop | `xs[(i + 1) % n]` | `xs.cyclic_pairs` / `xs.ring` | fix | #3 | loop | |
| adjacent_diff_loop | `d[i] = a[i + 1] - a[i]` | `a.adjacent_diff()` | fix | #1 | loop | |
| chain_loops | two loops pushing into one list | `a.chain(b)` | fix | #2 | loop | |
| take_while_loop | loop pushing until a condition, then break | `xs.take_while(p)` | fix | #2 | loop | |
| drop_while_loop | skip prefix by index then iterate | `xs.drop_while(p)` | fix | #2 | loop | |
| chunks_loop | step-k index loop slicing `k` items | `xs.chunks(k)` | fix | #2 | loop | |
| windows_loop | `for i: xs[i..i + k]` | `xs.windows(k)` | fix | #2 | loop | |
| successor_while | `while u != None: ...; u = next[u]` | `iterate(from: u, step:)` | fix? | #2 | loop | |
| partition_two_lists | push to `a` or `b` by predicate | `xs.partition(p)` | fix | #2 | loop | |
| reverse_copy | build a reversed copy to iterate | `xs.rev` | fix | #10 | ty | |
| collect_then_walk | `.collect()` immediately walked once | drop the `.collect()` | fix | #10 | flow | |
| permutations_recursion | recursive permutation generator | `xs.permutations()` / `next_permutation` | fix? | #1 | loop | |
| subsequences_mask_loop | `for mask` picking elements | `xs.subsequences()` | fix? | #1 | loop | |

### S.seq: ordering and ranks

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| comparator_by_key | `sort` with a comparator that compares `f(a)` to `f(b)` | `sort(by: f)` | fix | #13 | ty | |
| negate_for_desc | `sort(by: x => -x)` / `(-a, b)` keys | `by: desc` / `(desc(a), b)` | fix | #13 | ty | |
| argsort_loop | sort pairs `(value, index)` to get an order | `D.sorted(by: k => a[k])` | fix | #3 | ty |✓argsort_loop |
| copy_sort_writeback | copy `xs[l..r]`, sort, write back | sort `xs[l..r]` in place | fix | #10 | loop | ✓copy_sort_writeback |
| hand_compress | sort, dedup, then `lower_bound` each value | `compress(xs)` | fix? | #3 | loop | |
| rank_by_sort | position of each element in sorted order | `xs.compress()` / `perm_rank` | fix? | #1 | loop | |
| inverse_perm_loop | `inv[p[i]] = i` | `p.inverse()` | fix | #1 | loop | |
| is_sorted_loop | adjacent comparison loop with flag | `xs.is_sorted()` | fix | #2 | loop | |
| merge_loop | two-pointer merge of sorted lists | `merge_sorted(a, b)` / `kway_merge` | fix | #1 | loop | |
| sorted_set_ops_loop | two-pointer union/intersection/difference | `sorted_union` / `sorted_intersect` / `sorted_diff` | fix | #1 | loop | |
| inversions_by_bit | Fenwick or merge-sort inversion count | `xs.inversions()` | fix? | #1 | loop | |
| lis_by_hand | patience sorting with `lower_bound` | `xs.lis(strict:)` / `OnlineLis` | fix? | #1 | loop | |
| counting_sort_loop | bucket count then prefix-place | `counting_sort` / `radix_sort` | warn | #1 | loop | |
| next_permutation_loop | hand-rolled next permutation | `xs.next_permutation()` | fix | #1 | loop | |
| perm_cycles_loop | visited-array cycle walk on a permutation | `p.perm_cycles()` | fix? | #1 | loop | |
| sorted_range_method | `first_ge`/`first_gt`/... spelled out by search | the sorted method | fix | #13 | ty | ✓E0203 (checker, with fix) |

### S.seq: grouping and runs

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| runs_loop | "start a new group when value changes" loop | `xs.runs(by:)` | fix | #1 | loop | |
| rle_loop | run-length encode by hand | `[(r.first!, r.len) for r in s.runs()]` | fix | #1 | loop | |
| group_into_map | `m[k].push(x)` loop | `xs.group(by: k)` | fix | #1 | loop |✓group_into_map |
| tally_loop | `cnt[x] += 1` over a Map | `xs.tally()` / `Counter` | fix | #1 | loop | |
| bucket_loop | `[D]List` filled by key | `xs.map(k).buckets()` | fix | #1 | loop | |
| dedup_adjacent_loop | skip if equal to previous | `xs.dedup()` | fix | #1 | loop | |
| distinct_via_set | Set to drop repeats while keeping order | `xs.distinct(by:)` | fix | #1 | loop | |
| prev_same_loop | `last[x]` table to find previous equal | `xs.prev_same()` / `next_same()` | fix? | #1 | loop | |
| by_twin | `min_by`, `group_by`, `runs_by`, ... | base op with `by:` | fix | #13 | ty | ✓E0201/E0203 (checker, with fix) |

### S.search: binary and ternary search

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_answer_search | `while lo < hi` midpoint loop over a predicate | `(lo..=hi).first_true(p)` / `last_true` / `bsearch` | fix? | #1 | loop | ✓hand_rolled_answer_search |
| lower_bound_on_list | search a sorted list for first `>= x` | `xs.first_ge(x)` | fix | #1 | loop | |
| real_bsearch_iters | fixed-iteration float bisection | `bsearch_real` | fix? | #1 | loop | |
| ternary_loop | two-probe narrowing loop | `ternary_max` / `ternary_min` | fix? | #1 | loop | |
| mid_overflow | `(lo + hi) / 2` on wide values | `bsearch` (also P0: overflow) | fix? | #1 | syn | |
| parallel_bsearch_by_hand | rounds of per-query midpoints | `parallel_bsearch` | warn | #1 | loop | |
| dinkelbach_by_hand | iterate ratio `λ = best(λ)` | `dinkelbach` | warn | #1 | loop | |

### S.prefix: prefix, difference and sliding structures

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_prefix_sum | `p[i + 1] = p[i] + x` | `Prefix(a)`, `.fold(l..r)` | fix | #1 | loop | ✓hand_rolled_prefix_sum |
| prefix_2d_by_hand | inclusion–exclusion 2-D table | `Prefix` over `[R, C]`, `.fold(rows, cols)` | fix | #1 | loop | |
| prefix_twin | `prefix1`/`Prefix2`/`Diff1` variants | one generic `Prefix` / `Diff` | fix | #13 | ty | ✓E0201 (checker, with fix) |
| diff_array_by_hand | `d[l] += v; d[r + 1] -= v` then scan | `Diff` / `SweepAdd` | fix | #1 | loop | |
| arith_diff_by_hand | second-order difference for arithmetic adds | `ArithDiff` | fix? | #1 | loop | |
| scan_loop | running max/min/gcd stored per index | `xs.scan(op)` | fix | #2 | loop | |
| suffix_loop | backwards running fold | `suffix_fold` / `xs.rev.scan(op)` | fix | #2 | loop | |
| record_highs_loop | "new maximum so far" filter | `xs.record_highs()` | fix | #1 | loop | |
| sliding_deque | monotone deque for window min/max | `xs.windows(k).fold(min)` / `Window(op)` | fix? | #1 | loop | |
| refold_window | window re-folded at every position | `windows(k).fold(op)` (O(n) total) | fix? | #10 | loop | |
| two_pointer_by_hand | `while r < n` grow, `while bad` shrink | `xs.maximal(state, ok:)` / `minimal` | fix? | #1 | loop | |
| mono_stack_nearest | stack loop for previous/next smaller | `prev_smaller` / `next_smaller` (`by:`) | fix | #1 | loop | |
| histogram_stack | largest-rectangle stack loop | `xs.spans()` plus a fold | fix? | #1 | loop | |
| contribution_loop | count subarrays where `a[i]` is the min/max | `xs.spans()` | warn | #1 | loop | |
| doubled_array_for_ring | `a + a` to handle wrap-around | `a.ring` | fix? | #3 | ty | |
| subarray_sum_counter | map of prefix counts, hand-built | `Counter` over `pre` (documented idiom) | doc | #1 | | |

### S.range: segment trees, Fenwick, sparse tables

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_range_tree | `p += p & -p` or `2p`/`2p + 1` loops | `Fenwick` / `SegTree` / `std.lazy` | warn | #1 | loop | ✓hand_rolled_range_tree |
| lazy_segtree_by_hand | push/pull with lazy tags | `SegTree` with an `Action` | warn | #1 | loop | |
| sparse_table_by_hand | `st[k][i] = min(st[k-1][i], ...)` | `SparseTable(op)` | fix? | #1 | loop | |
| fenwick_kth_by_hand | descending-bit walk on a BIT | `Fenwick.kth` / `first_ge` | fix? | #1 | loop | |
| segtree_descend_by_hand | recursive "first index where prefix ≥ x" | `left_while` / `right_while` / `first_ge` | fix? | #1 | loop | |
| merge_sort_tree_by_hand | sorted vectors per node | `MergeSortTree` / `WaveletMatrix` | warn | #1 | loop | |
| persistent_tree_by_hand | node pools with `lc`/`rc` arrays | `PersistentSegTree` / `RangeKth` | warn | #1 | loop | |
| beats_by_hand | max/second-max/count node fields | `BeatsSegTree` | warn | #1 | loop | |
| cht_by_hand | deque of lines with intersection checks | `LiChao` / `LineContainer` / `MonotoneHull` | warn | #1 | loop | |
| segtree_for_static_rmq | `SegTree(min, a)` never updated | `SparseTable(min, a)` (O(1) query) | fix? | #10 | flow | |
| fenwick_for_static_sums | Fenwick never updated after build | `Prefix(a)` | fix? | #10 | flow | |
| ops_as_lambdas | `SegTree((a, b) => a + b, ...)` | `SegTree(+, ...)` | fix | #13 | ty | |

### S.sorted: sets, bags, maps, intervals

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| map_as_multiset | `Map<T, Int>` used as a multiset with min/max reads | `SortedBag<T>` | fix? | #1 | ty | ✓map_as_multiset |
| counter_by_map | `Map<T, Int>` with `+= 1` / `get ?? 0` | `Counter<T>` | fix | #1 | ty | ✓counter_by_map |
| prev_of_lower_bound | "one before first `>= x`" | `last_lt(x)` / `last_le(x)` | fix | #1 | ty | |
| erase_one_by_find | find then erase a single copy | `bag.remove(x)` | fix | #1 | ty | |
| order_stat_by_hand | BIT over compressed values for k-th | `SortedBag.kth` / `count_lt` | fix? | #1 | loop | |
| interval_map_by_hand | split/merge on an ordered map of runs (ODT) | `IntervalMap.assign` / `IntervalSet` | warn | #1 | loop | |
| interval_union_loop | sort intervals, merge overlapping | `IntervalSet.add` | fix? | #1 | loop | |
| pop_min_loop | `first` then `remove(first)` | `pop_first()` | fix | #13 | ty | ✓pop_min_loop |
| domain_keyed_map | `Map`/`Set` keyed by a domain | `[D]?T` / `Marks<D>` | fix? | #3 | ty | ✓domain_keyed_map |

### S.heap

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| negated_heap | push `-x` to get a max-heap | `Heap(by: desc)` / `MaxHeap` | fix | #13 | ty | ✓negated_heap |
| lazy_delete_heap | heap plus "deleted" counter map | `ErasableHeap` | fix? | #1 | loop | |
| stale_entry_skip | `if d > dist[v]: continue` after pop | `IndexedHeap.decrease` (or `dijkstra`) | warn | #1 | loop | ✓stale_entry_skip |
| two_heap_median | max-heap + min-heap balancing | `SortedBag` / `MinMaxHeap` | doc | #1 | | |
| push_pop_pair | `push(x)` then `pop()` | `push_pop(x)` | fix | #10 | ty | ✓push_pop_pair |

### S.dsu

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_dsu | parent-pointer path compression | `Dsu` | warn | #1 | loop | ✓hand_rolled_dsu |
| dsu_side_arrays | `size[]`/`min[]`/`sum[]` kept next to a DSU | `DsuOf` with a monoid | fix? | #7 | flow | |
| rollback_dsu_by_hand | union by size, stack of changes | `Dsu.undo_to` / `RollbackArray` | warn | #1 | loop | |
| parity_dsu_by_hand | xor-to-parent array | `ParityDsu` | warn | #1 | loop | |
| potential_dsu_by_hand | weighted difference to parent | `PotentialDsu` | warn | #1 | loop | |
| components_via_dsu_groups | loop building `root -> members` map | `dsu.groups()` | fix | #1 | loop | |
| offline_connectivity_by_hand | segment tree over time with rollback DSU | `TimeTree` | warn | #1 | loop | |

### S.graph

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| adjacency_build_loop | `adj[u].push(v); adj[v].push(u)` over read edges | `Graph.undirected(...)` / `directed` / `weighted` | fix | #1 | loop | ✓manual_adjacency_list |
| bfs_by_hand | queue + dist array with `-1` | `bfs(next, from:)` | fix? | #1 | loop | ✓queue_head_index |
| multi_source_bfs_by_hand | queue seeded with many sources | `bfs(next, from: sources)` | fix? | #1 | loop | |
| zero_one_bfs_by_hand | deque with front/back pushes | `dial` / 0-1 `bfs` | fix? | #1 | loop | |
| dijkstra_by_hand | heap relaxation loop | `dijkstra(g, from:)` | fix? | #1 | loop | |
| bellman_ford_by_hand | `n - 1` rounds of relaxing all edges / SPFA | `bellman_ford` | fix? | #1 | loop | |
| floyd_by_hand | triple `k, i, j` loop | `floyd_warshall` | fix | #1 | loop | |
| topo_by_hand | Kahn's indegree queue | `g.topo()` / `topo_order` | fix? | #1 | loop | |
| cycle_by_colors | white/grey/black DFS for a cycle | `find_cycle` | fix? | #1 | loop |✓cycle_by_colors |
| scc_by_hand | Tarjan / Kosaraju | `scc` / `condense` | warn | #1 | loop | |
| bridges_by_hand | low-link DFS | `bridges` / `cut_vertices` / `blocks` | warn | #1 | loop | |
| components_by_dfs | visited loop labelling components | `components` | fix? | #1 | loop |✓components_by_dfs |
| bipartite_by_hand | 2-colouring BFS/DFS | `two_color` | fix? | #1 | loop | |
| kruskal_by_hand | sort edges + DSU | `kruskal` / `prim` / `boruvka` | fix? | #1 | loop | |
| euler_path_by_hand | Hierholzer with edge pointers | `euler_walk` | warn | #1 | loop | |
| two_sat_by_hand | implication graph + SCC | `TwoSat` | warn | #1 | loop | |
| max_flow_by_hand | Dinic / Edmonds–Karp | `max_flow` / `FlowGraph` | warn | #1 | loop | |
| matching_by_hand | Kuhn's augmenting DFS | `max_matching` / `general_matching` | warn | #1 | loop | |
| min_cost_flow_by_hand | SPFA/potential MCMF | `CostFlow` | warn | #1 | loop | |
| functional_graph_walk | `seen` + `next[]` cycle finding | `Functional` / `on_cycle` / `cycle_len` | fix? | #1 | loop | |
| diameter_two_bfs | BFS, farthest, BFS again | `diameter` | fix? | #1 | loop | |
| dag_dp_by_topo | topo order then fold over predecessors | `dag_fold` | fix? | #1 | loop | |
| reachability_by_bitsets | bitset closure by hand | `reach_sets` / `closure` | warn | #1 | loop | |
| edge_index_pairs | `2i`/`2i ^ 1` to pair an edge's two arcs | `Arc` / `.back` / `Edge` domain | warn | #3 | syn | |
| shortest_path_tree_parents | `par[v] = u` recorded during relax for paths | `.path(to:)` on the search result | fix? | #1 | flow | |

### S.tree

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| root_dfs_by_hand | DFS filling `parent`, `depth` | `Rooted(at:)`, `.parent`, `.depth` | fix? | #1 | loop | |
| hand_rolled_binary_lifting | `up[k][v] = up[k-1][up[k-1][v]]` | `Lift` / `JumpTree` | warn | #1 | loop | ✓hand_rolled_binary_lifting |
| lca_by_hand | depth-equalise then lift | `t.lca(u, v)` / `LcaTable` | fix? | #1 | loop |✓lca_by_hand |
| tree_dist_by_lca | `depth[u] + depth[v] - 2 * depth[lca]` | `t.dist(u, v)` | fix | #1 | ty | |
| subtree_size_loop | postorder `sz[p] += sz[v]` | `t.size` | fix | #1 | loop | |
| euler_tour_by_hand | `tin`/`tout` counters | `euler_tour` / `.span[v]` | fix? | #3 | loop | |
| tree_order | `t.preorder` / `t.postorder` recursion | `t.at` / `t.at.rev` | fix | #13 | ty | ✓tree_order |
| tree_dp_by_recursion | recursive post-order DP | `fold_tree` (or keep `fn`, judgment) | doc | #2 | | |
| reroot_two_pass | down pass then up pass with prefix/suffix of children | `reroot` | warn | #1 | loop | |
| centroid_by_hand | size recomputation + removed flags | `Centroids` | warn | #1 | loop | |
| hld_by_hand | heavy child, head, pos arrays | `chains` / `apply_path` / `fold_path` | warn | #1 | loop | |
| kth_ancestor_loop | climb parent `k` times | `ancestor(v, k)` / `jump` | fix | #10 | loop | |
| virtual_tree_by_hand | sort by tin, add LCAs, stack | `VirtualTree` | warn | #1 | loop | |
| children_from_parents | build child lists from a parent array | `from_parents` / `parent.fibers()` | fix | #1 | loop | |
| small_to_large_by_hand | swap child sets by size | documented pattern, `MergeSeg` where it fits | doc | #1 | |✓small_to_large_by_hand |

### S.strings

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_string_algo | prefix-function, Z, polynomial hash, trie loops | `std.strings` | warn | #1 | loop | ✓hand_rolled_string_algo |
| manacher_by_hand | palindrome radii loop | `manacher` / `Palindromes` | warn | #1 | loop | |
| suffix_array_by_hand | doubling sort | `suffix_array` / `SuffixArray` | warn | #1 | loop | |
| aho_by_hand | trie with BFS fail links | `AhoCorasick` | warn | #1 | loop | |
| palindrome_check_loop | two-index comparison loop | `s.is_palindrome()` | fix | #1 | loop | |
| letter_count_array | `[26]Int` with `c - 'a'` | `s.counts()` / `Counter<Char>` / `[Letter]` domain | fix? | #3 | ty | |
| char_offset_index | `c - 'a'` as an index | a `Letter` domain | fix? | #3 | ty | ✓char_offset_index |
| split_by_hand | scan for spaces/commas | `s.split(sep)` / `split_whitespace` | fix | #1 | loop | |
| parse_int_by_hand | `x = x * 10 + (c - '0')` | `parse_int` / `read<Int>()` | fix | #1 | loop | |
| digits_by_hand | `while x > 0: d.push(x % 10)` | `digits(x)` | fix | #1 | loop | |
| min_rotation_by_hand | Booth / doubled-string compare | `min_rotation` | warn | #1 | loop | |
| substring_compare_by_hash | hash equality + manual LCP binary search | `lcp_between` / `compare` | warn | #1 | loop | |
| find_all_loop | repeated `find` from last match | `s.find_all(p)` | fix | #1 | loop | |
| str_concat_loop | `s = s + piece` in a loop | collect pieces, `concat` / `print(sep:)` | fix? | #10 | loop | |

### S.math

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_sieve | Eratosthenes or linear sieve loop | `Sieve` | warn | #1 | loop | ✓hand_rolled_sieve |
| euclid_by_hand | `while b: a, b = b, a % b` | `gcd` / `lcm` / `ext_gcd` | fix | #1 | loop | |
| fast_pow_by_hand | square-and-multiply loop | `a ** e` / `pow_mod` | fix | #1 | loop | |
| pow_method | `a.pow(e)` | `a ** e` | fix | #13 | syn | ✓E0203 (checker, with fix) |
| float_sqrt_for_int | `floor(sqrt(x))` (`Int(sqrt(x))` is E0302) | `x.isqrt` (also P0: float error) | fix | #1 | ty | ✓float_sqrt_for_int |
| ceil_div_formula | `(a + b - 1) / b` | `ceil_div(a, b)` (correct for negatives) | fix? | #1 | syn | ✓ceil_div_formula |
| divisors_by_hand | `for i in 1..=sqrt(n)` pushing `i` and `n / i` | `divisors(n)` / `each_divisor` | fix | #1 | loop | |
| factor_by_hand | trial division loop | `factor(n)` | fix | #1 | loop | |
| phi_by_hand | totient from factors | `phi` / `phi_table` | fix | #1 | loop | |
| mobius_by_hand | Möbius by sieve | `mobius_table` | fix? | #1 | loop | |
| crt_by_hand | pairwise CRT merge | `crt` / `crt_all` | fix? | #1 | loop | |
| primality_loop | trial division `is_prime` | `is_prime` | fix | #1 | loop | |
| floor_blocks_by_hand | `for l: r = n / (n / l)` | `floor_quotients(n)` | fix? | #1 | loop | |
| popcount_loop | `while x: c += x & 1; x >>= 1` | `x.popcount` / `Subset.len` | fix | #1 | loop | |
| bit_len_loop | shift until zero | `bit_len` | fix | #1 | loop | |
| base_convert_loop | repeated `% b` | `digits(x, base:)` | fix | #1 | loop | |

### S.mod: modular arithmetic and combinatorics

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_mod_reduce | `% MOD` after every op on `Int` | `type Z = Mod<P>` | fix? | #6 | ty | ✓manual_mod_reduce |
| neg_mod_fixup | `(a - b + MOD) % MOD` | `Mod` subtraction | fix | #6 | syn | ✓neg_mod_fixup |
| fermat_inverse | `pow(a, MOD - 2)` | `a.inv` | fix | #1 | ty | ✓fermat_inverse |
| fact_tables_by_hand | `fact[]`, `inv_fact[]` arrays | `Comb<Z>` / `ModComb` | fix? | #1 | loop |✓fact_tables_by_hand |
| binom_by_hand | `fact[n] * inv_fact[k] * inv_fact[n-k]` | `comb.choose(n, k)` | fix | #1 | ty |✓binom_by_hand |
| inv_all_loop | batch inverse prefix trick by hand | `inv_all` | fix | #1 | loop | |
| loop_invariant_mod_div | `Mod` division by a loop-invariant value | hoist `inv` | fix | #10 | flow | ✓loop_invariant_mod_div |
| mixed_modulus | two `Mod<P>` types meet through `Int(...)` | one `Z`, or an explicit conversion | warn | #6 | ty | |
| mod_to_int_compare | `Int(a) == Int(b)` on `Mod` values | `a == b` | fix | #13 | ty | |
| catalan_derange_by_hand | recurrence tables for Catalan / derangements | `catalan` / `derange` / `multichoose` | fix? | #1 | loop | |

### S.poly, S.matrix, S.ntt

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| quadratic_convolution | `c[i + j] += a[i] * b[j]` at large bounds | `convolve(a, b)` | fix? | #10 | bound | |
| ntt_by_hand | butterfly loops | `convolve` / `ntt` | warn | #1 | loop | |
| xor_or_and_convolution_by_hand | FWHT / SOS loops | `fwht` / `sos` | warn | #1 | loop | |
| subset_sum_by_hand | `for mask, for bit: f[mask] += f[mask ^ bit]` | `sos` / `subset_folds` | fix? | #1 | loop | |
| matmul_loops | triple loop matrix product | `Mat` `*` | fix | #1 | loop | |
| matpow_by_hand | squaring loop on matrices | `m ** e` / `MatPow` | fix | #1 | loop | |
| gauss_by_hand | elimination loops | `solve` / `rank` / `det` / `rref` | warn | #1 | loop | |
| xor_basis_by_hand | insert-by-top-bit loop | `XorBasis` | fix? | #1 | loop | |
| linear_recurrence_by_matrix | matrix power for a k-term recurrence with large k | `bostan_mori` / `Poly` | warn | #10 | bound | |
| poly_eval_loop | Horner by hand | `Poly.at` / `eval_all` | fix | #1 | loop | |

### S.bits: masks and subsets

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| submask_loop | `for (s = m; s; s = (s - 1) & m)` | `m.submasks` / `bit_subsets` | fix | #1 | syn |✓submask_loop |
| set_bits_loop | `for b in 0..k: if m >> b & 1` | `for b in m` on a `Subset` | fix | #1 | loop | |
| mask_as_int | `Int` mask with `1 << i` tests over a domain | `Subset<D>` | fix? | #3 | ty | |
| bool_array_shift_dp | `[n]Bool` knapsack loop | `Bitset.shift_or` | fix? | #10 | loop | |
| masks_of_size_filter | all masks filtered by popcount | `masks_of_size(k)` | fix | #10 | loop | |
| bit_trick_parity | `x & 1` for parity in non-bit code | `x % 2 != 0` (judgment) | doc | B6 | | |

### S.geometry

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| cross_by_hand | `x1 * y2 - x2 * y1` | `a.cross(b)` on `Vec` | fix | #1 | syn |✓cross_by_hand |
| tuple_points | `(Int, Int)` used as points with arithmetic | `Pt` / `Vec` | fix? | #3 | ty | ✓tuple_points |
| hull_by_hand | monotone chain loop | `hull` / `OnlineHull` | warn | #1 | loop | |
| shoelace_by_hand | area loop | `poly.area` | fix | #1 | loop | |
| atan2_angle_sort | sort by `atan2` | `ByAngle` (exact; also P0 floats) | fix? | #1 | ty | |
| dist_with_sqrt_compare | comparing `sqrt` distances | compare squared (`dist2`) | fix | #10 | ty | |
| segment_intersect_by_hand | orientation case analysis | `std.geometry` predicates | warn | #1 | loop | |

### S.dp: DP optimizations and memoization

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| interval_dp_by_length | `for len in 2..=n: for l in ..: let r = l + len - 1` filling `dp[l, r]` | a table over `[I.ranges]` read in its order (Ryan, 2026-11-22: atcoder_dp_l) | fix? | #3 | loop |✓interval_dp_by_length |
| memo_table_by_hand | `-1`-initialised table + recursive check | `@memo` | fix? | #1 | flow | |
| memo_key_space | `@memo` table too big at the input bounds | `@memo(map)` / iterative DP | warn | #10 | bound | ✓memo_key_space |
| inf_seeded_dp | DP seeded with `INF` / `-INF` | `?T` with `min=` / `max=` | fix? | #5 | ty | |
| rolling_rows_copy | `cur = nxt.clone()` each layer | `swap(mut cur, mut nxt)` + `fill` | fix | #10 | flow | |
| knuth_by_hand | `opt[i][j]` bounded inner loop | `knuth` | warn | #1 | loop | |
| dnc_opt_by_hand | recursive `solve(l, r, optl, optr)` | `dnc_layer` | warn | #1 | loop | |
| aliens_by_hand | λ binary search with penalty | `aliens` | warn | #1 | loop | |
| monotone_minima_by_hand | SMAWK / monotone minima recursion | `monotone_minima` / `smawk` | warn | #1 | loop | |
| slope_trick_by_hand | two heaps with lazy offsets | `Convex` / `Concave` (`std.slope`) | warn | #1 | loop | |
| dp_over_raw_int | `[n + 1]T` DP indexed by a domain's ordinal | `[D]T` or cut-indexed `[D.cuts]T` | fix? | #3 | ty | |

### S.offline: Mo, CDQ, windows with undo

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| mo_by_hand | block-sorted queries + add/remove loop | `mo(queries, state, answer:)` | warn | #1 | loop | |
| cdq_by_hand | divide and conquer over time | `cdq` | warn | #1 | loop | |
| queue_undo_by_hand | two-stack trick with a rollback structure | `undoing(...)` | warn | #1 | loop | |
| track_by_hand | `add`/`remove` closures passed around | `track(s, add:, remove:)` | fix? | #7 | ty | |

### S.containers: deques, lists, grids, randomness

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| linked_list_arrays | `nxt[]`/`prv[]` arrays with splicing | `LinkedList` | warn | #1 | loop | |
| list_front_ops | `insert(0, x)` / `remove(0)` in a loop | `Deque` | fix? | #10 | ty | |
| rotate_by_copy | slice + concat to rotate | `rotate_left` / `rotated_left` | fix | #1 | ty | |
| dx_dy_arrays | `dx = [0, 1, 0, -1]` | `p.adj4` / `p.step(d)` with `Dir` | fix? | #3 | syn | ✓dx_dy_arrays |
| grid_bounds_check | `0 <= r < h and 0 <= c < w` | domain membership / `adj4` already filters | fix? | #3 | syn | ✓grid_bounds_check |
| grid_flatten | `r * w + c` | `[h, w]` domain | fix? | #3 | syn | |
| grid_neighbour_step | a `[Row, Col]` loop that reads neighbours as `r - 1` / `c - 1` (or `if r > 0`) | `type Cell = (Row, Col)` and `p.step(Dir4.U)` / `p.adj4`; absent neighbours are `None` (Ryan, 2026-11-22: atcoder_dp_h) | fix? | #3 | ty | ✓grid_neighbour_step |
| transpose_loop | `t[j][i] = g[i][j]` | `g.transposed()` | fix | #1 | loop | |
| ray_walk_loop | step while in bounds | `p.ray(d)` | fix? | #1 | loop | |
| shuffle_by_hand | Fisher–Yates loop | `rng.shuffle(mut xs)` | fix | #1 | loop | |
| random_perm_by_hand | shuffle of `0..n` | `rng.perm(D)` | fix | #1 | loop | |

---

#### Additions (brainstorm, round 2)

### S.seq: reductions

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_reduce | seed `acc = xs[0]` then fold `xs[1..]` with binary op | `xs.reduce(f)` (`std.seq`) | fix | #2 | loop | |
| min_then_count | `mn = xs.min()` then `xs.count(x == mn)` | `xs.min_count()` (`std.seq`) | fix | #1 | ty | |
| loop_seq_equal | `for i in D: if a[i] != b[i]: ok = false` elementwise equality | `a == b` (`std.seq`) | fix | #2 | loop | |
| loop_all_equal | check every element equals `xs[0]` | `xs.all(x => x == xs[0])` or `needs stdlib: std.seq.all_equal` | fix | #2 | loop | |
| loop_has_dup | sort and scan adjacent pairs for `a[i] == a[i-1]` | `a.sorted().deduped().len < a.len` (`std.seq`) or `needs stdlib: std.seq.has_dup` | fix? | #1 | loop | |

### S.seq: building

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_iota_loop | allocate array then `for i in 0..n: a[i] = i` | `[D] i => i` (`std.seq`) | fix | #2 | loop | |
| manual_zip_loop | `for i in D: f(a[i], b[i])` when `i` is only used to index `a` and `b` | `for (x, y) in a.zip(b): f(x, y)` (`std.seq`) | fix | #2 | loop | |
| manual_partition_in_place | two-pointer Lomuto/Hoare partition loop on a `List` | `xs.partition_in_place(p)` / `xs.stable_partition_in_place(p)` (`std.seq`) | fix? | #1 | loop | |
| manual_perm_compose | `[D] i => a[b[i]]` permutation composition | `a.compose(b)` (`std.seq`) | fix | #1 | loop | |
| manual_perm_pow | binary-lifting / doubling loop for \(p^k\) | `p.perm_pow(k)` (`std.seq`) | fix? | #1 | loop | |
| manual_rotate_right | slice-and-concat or triple-reverse to rotate right by `k` | `xs.rotate_right(k)` / `xs.rotated_right(k)` (`std.seq`) | fix | #1 | loop | |
| manual_prev_permutation | hand-written lexicographically previous permutation loop | `xs.prev_permutation()` (`std.seq`) | fix? | #1 | loop | |
| manual_pop_while | `while not xs.is_empty and p(xs.last!): xs.pop()` | `xs.pop_while(p)` (`std.seq`) | fix | #2 | loop | |
| manual_select_nth | sort whole list just to read `xs[k]` | `xs.select(k)` (`std.seq`) | fix? | #10 | ty | |
| manual_deduped_copy | `let b = a.sorted(); b.dedup()` | `a.sorted().deduped()` (`std.seq`) | fix | #2 | ty | |
| manual_covering_intervals | greedy sweep choosing minimum intervals to cover `[lo, hi]` | `covering(intervals, lo, hi)` (`std.seq`) | warn | #1 | loop | |
| manual_halve_clip | `x / 2` / `min(max(x, lo), hi)` repeated via helper on sequences or ints | `x.clip(lo, hi)` / `halve` (`std.seq`) | fix | #1 | syn | |
| manual_try_at | `if i >= 0 and i < xs.len: Some(xs[i]) else: None` | `xs.try_at(i)` (`std.seq`) | fix | #5 | syn | |

### S.seq: grouping and runs

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_most_common | build frequency `HashMap` then argmax over entries | `xs.tally().most_common()` (`std.seq`, `std.hashmap`) | fix | #1 | loop | |

### S.search: binary and ternary search

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_ternary_real | fixed-iteration `100` loop trisecting a `Float` range | `ternary_max_real` / `ternary_min_real` (`std.search`) | fix? | #1 | loop | |
| hand_rolled_gallop_search | exponential step `1, 2, 4, ...` then binary search | `gallop_first` (`std.search`) | fix? | #1 | loop | |
| hand_rolled_split_best | binary search on monotone split point minimizing `max(f(i), g(i))` | `split_best` (`std.search`) | warn | #1 | loop | |

### S.prefix: prefix, difference and sliding structures

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_cyclic_prefix_sum | duplicate array `a ++ a` or split-wrap query for circular range sums | `pref.cyclic_sum(l, r)` (`std.prefix`) | fix? | #1 | ty | |
| manual_prefix_excl_fold | build both `pref[i]` and `suff[i]` arrays just to combine all-except-`i` | `prefix_excl(op, xs)` (`std.prefix`) | fix? | #1 | loop | |
| manual_prefix_fold_monoid | manual `pref[i] = op(pref[i-1], a[i])` for non-sum monoid (e.g. `gcd`, `min`, `max`) | `prefix_fold(op, a)` (`std.prefix`) | fix | #1 | loop | |
| manual_fill_prefix | incremental prefix-sum table appended online during DP | `FillPrefix` (`std.prefix`) | warn | #1 | loop | |

### S.range: segment trees, Fenwick, sparse tables

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_fenwick_2d | nested `i += i & -i; j += j & -j` 2D BIT loops | `Fenwick2D` / `SparseFenwick2D` (`std.fenwick`) | warn | #1 | loop | |
| hand_rolled_dual_fenwick | BIT storing difference array for range-add + point-query | `DualFenwick` (`std.fenwick`) | fix? | #1 | ty | |
| hand_rolled_disjoint_sparse | divide-and-conquer table precomputing prefix/suffix folds per bit level | `DisjointSparseTable` (`std.sparse`) | warn | #1 | loop | |
| hand_rolled_sparse_table_2d | 4D `st[k1][k2][i][j]` 2D RMQ table | `SparseTable2D` (`std.sparse`) | warn | #1 | loop | |
| hand_rolled_linear_rmq | block-mask + sparse table O(n)/O(1) RMQ | `LinearRmq` (`std.sparse`) | warn | #1 | loop | |
| hand_rolled_historic_beats | segment tree tracking historic max/sum tags alongside `chmin`/`chmax` | `HistoricBeatsSegTree` (`hist_max`, `hist_sum`) (`std.beats`) | warn | #1 | ty | |
| hand_rolled_segtree_2d | outer segment tree of inner segment trees | `SegTree2D` (`std.segtree`) | warn | #1 | ty | |
| hand_rolled_dynamic_order_tree | pointer/arena dynamic segment tree for order statistics on huge range | `DynamicOrderTree` (`std.segtree`) | warn | #1 | ty | |
| hand_rolled_persistent_lazy | versioned copy-on-write lazy segment tree | `PersistentLazySegTree` (`std.persistent`) | warn | #1 | ty | |
| hand_rolled_persistent_array | versioned tree-backed array with `Version` handles | `PersistentArray` (`std.persistent`) | warn | #1 | ty | |
| hand_rolled_persistent_multiset | versioned frequency segment tree for offline/online range k-th | `PMultiset` (`std.persistent`) | warn | #1 | ty | |
| hand_rolled_lazy_treap | implicit treap / splay with split, merge, reverse, and range apply | `LazyTreap` / `Rope` (`std.lazy`) | warn | #1 | ty | |
| segtree_manual_walk_alive | binary-search on `seg.fold(l..mid)` to find first active element | `seg.first_alive` / `seg.try_fold` (`std.segtree`) | fix? | #10 | loop | |

### S.sorted: sets, bags, maps, intervals

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_bag_drain_range | loop calling `bag.lower_bound(l)` and removing elements up to `r` | `bag.drain_range(l..r)` (`std.bag`) | fix | #1 | loop | |
| manual_bag_take_first | `let x = bag.first!; bag.remove(x)` | `bag.take_first()` (`std.bag`) | fix | #1 | syn | |
| manual_bag_add_many | `for _ in 0..c: bag.add(x)` | `bag.add_many(x, c)` (`std.bag`) | fix | #10 | loop | |
| manual_sparse_interval_set | `OrderedMap` of `[l, r)` runs over huge coordinates (`10^9`) | `SparseIntervalSet` / `SparseIntervalMap` (`std.interval`) | warn | #1 | ty | |
| manual_interval_next_gap | loop stepping past covered intervals to find first uncovered point | `intervals.next_gap(x)` / `intervals.next_covered(x)` (`std.interval`) | fix? | #1 | loop | |
| manual_sparse_set | `stamp[x] == cur_stamp` array + dirty list for O(1) clear | `Marks` / `MarkMap` / `SparseSet` (`std.marks`) | fix? | #1 | ty | |
| manual_interner | `HashMap<T, Int>` + `List<T>` assigning dense sequential IDs | `Interner` (`std.marks`) | fix? | #1 | ty | |
| manual_flat_map_or_insert | `if not m.contains(k): m[k] = v; let r = m[k]` | `m.get_or_insert(k, v)` (`std.hashmap`, `std.seq`) | fix | #2 | flow | |

### S.heap

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_heap_replace_top | `h.pop(); h.push(x)` back-to-back | `h.replace_top(x)` (`std.heap`) | fix | #10 | syn | |
| manual_heap_drain | `while let x = h.pop(): body` when only consuming elements in order | `for x in h.drain(): body` (`std.heap`) | fix | #2 | loop |✓manual_heap_drain |
| manual_heap_push_all | `for x in xs: h.push(x)` on a fresh heap | `Heap.from(xs)` / `h.push_all(xs)` (`std.heap`) | fix | #10 | loop | |
| hand_rolled_meld_heap | leftist/skew heap with `meld` pointers | `MeldHeap` (`std.heap`) | warn | #1 | ty | |
| manual_settle_dijkstra | manual `dist` array + `Heap` + `if d > dist[u]: continue` boilerplate | `Settle` (`std.heap`) | fix? | #1 | loop | |

### S.dsu

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_dsu_roots | `for v in V: if dsu.find(v) == v: ...` | `dsu.roots()` (`std.dsu`) | fix | #1 | loop | |
| manual_dsu_diff | potential-DSU computing `pot[v] - pot[u]` manually | `dsu.diff(u, v)` (`std.dsu`) | fix? | #1 | ty | |
| hand_rolled_persistent_dsu | persistent array + union-by-rank for versioned connectivity | `PersistentDsu` (`std.dsu`) | warn | #1 | ty | |

### S.graph

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_graph_transpose | rebuild reversed adjacency list `for u in V: for v in g.adj(u): rev.add(v, u)` | `g.transpose()` (`std.graph`) | fix | #1 | loop | |
| hand_rolled_kcore_peel | degree queue repeatedly peeling vertices with `deg < k` | `g.peel(...)` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_dial_bfs | bucketed queue array `q[d % (W+1)]` for small integer edge weights | `g.dial(...)` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_johnson | Bellman-Ford potentials + `V` runs of Dijkstra | `g.johnson()` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_k_shortest | Eppstein / Yen / `cnt[v] <= k` Dijkstra for k-th shortest path | `g.k_shortest(...)` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_negative_cycle | Bellman-Ford `n`-th pass parent walk to extract a negative cycle | `g.negative_cycle()` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_girth | BFS from every vertex tracking shortest cycle length | `g.girth()` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_boruvka | component-cheapest-edge rounds merging with DSU | `g.boruvka()` (`std.graph`) | warn | #1 | loop | |
| hand_rolled_functional_graph | in-degree peeling + cycle decomposition on 1-out-degree graph | `functional(next)` (`std.graph`) | warn | #1 | loop | |
| manual_flow_min_cut_arcs | BFS on residual graph after max-flow to collect cut edges | `net.min_cut(s, t)` / `net.cut_arcs(s, t)` (`std.flow`) | fix? | #1 | loop | |
| hand_rolled_circulation | supersource/supersink demand reduction for lower-bounded flow | `circulation` / `max_bounded_flow` / `min_bounded_flow` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_gomory_hu | `n - 1` recursive min-cut splits to build all-pairs min-cut tree | `gomory_hu` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_global_min_cut | Stoer-Wagner phase loop for undirected global min-cut | `global_min_cut` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_max_closure | project-selection s-t min-cut reduction | `max_closure` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_min_vertex_cover | Konig's theorem alternating BFS after bipartite matching | `min_vertex_cover` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_weighted_matching | Hungarian algorithm potentials + slack array | `max_weight_matching` / `min_weight_matching` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_general_matching | Edmonds blossom shrinking BFS for general-graph matching | `max_weight_general_matching` (`std.flow`) | warn | #1 | loop | |
| hand_rolled_push_relabel | height + excess + highest-label discharge loop | `net.push_relabel(s, t)` (`std.flow`) | warn | #1 | loop | |

### S.tree

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_tree_excl_fold | rerooting prefix+suffix product over children of each node | `t.excl_fold(...)` (`std.tree`) | fix? | #1 | loop | |
| manual_tree_is_anc | `tin[u] <= tin[v] and tout[v] <= tout[u]` | `t.is_anc(u, v)` (`std.tree`) | fix | #1 | ty | |
| manual_tree_on_path | `dist(u, x) + dist(x, v) == dist(u, v)` | `t.on_path(u, v, x)` (`std.tree`) | fix | #1 | ty | |
| hand_rolled_tree_centroids | DFS finding one or two nodes with all components `<= n / 2` | `t.centroid()` / `t.centroids()` (`std.tree`) | fix? | #1 | loop | |
| hand_rolled_long_path_decomp | height-based heavy child decomposition for O(1) k-th ancestor / depth DP | `t.long_paths()` (`std.tree`) | warn | #1 | loop | |
| manual_tree_jump_while | binary-lifting `for k in (0..LOG).rev: if pred(up[k][u]): u = up[k][u]` | `jt.jump_while(u, pred)` (`std.tree`) | fix? | #1 | loop | |
| manual_tree_layers | BFS grouping tree vertices by depth level | `t.layers()` (`std.tree`) | fix | #1 | loop | |
| manual_tree_forest | loop calling `Rooted.from` on each unvisited component | `Rooted.forest(g)` (`std.tree`) | fix? | #1 | loop | |

### S.strings

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_str_starts_ends | `s.len >= p.len and s[..p.len] == p` | `s.starts_with(p)` / `s.ends_with(p)` (`std.strings`) | fix | #1 | syn | |
| manual_str_trim | two `while` loops advancing `l` and retreating `r` past spaces | `s.trim()` / `s.trim_start()` / `s.trim_end()` (`std.strings`) | fix | #1 | loop | |
| manual_str_split_once | `let i = s.search(d)!; (s[..i], s[i+1..])` | `s.split_once(d)` (`std.strings`) | fix | #1 | syn | |
| manual_str_replace | loop finding occurrences of `pat` and splicing replacement | `s.replace(pat, to)` (`std.strings`) | fix | #1 | loop | |
| manual_str_case_fold | manual `'A'..'Z'` ASCII offset loop | `s.to_lower()` / `s.to_upper()` (`std.strings`) | fix | #1 | loop | |
| manual_str_repeat | loop appending `s` `k` times | `s.repeat(k)` (`std.strings`) | fix | #1 | loop | |
| manual_str_rsearch | reverse loop searching for last occurrence of substring/char | `s.rsearch(pat)` (`std.strings`) | fix | #1 | loop | |
| manual_str_lcp_lcs | `while i < min(a.len, b.len) and a[i] == b[i]: i += 1` | `common_prefix_len(a, b)` / `common_suffix_len(a, b)` (`std.strings`) | fix | #1 | loop | |
| manual_str_period_borders | KMP `pi` chain loop computing smallest period or all borders | `period(s)` / `borders(s)` (`std.strings`) | fix? | #1 | loop | |
| manual_str_overlap | suffix-prefix matching loop finding max overlap of `a` and `b` | `overlap(a, b)` (`std.strings`) | fix? | #1 | loop | |

### S.math

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_choose2 | `n * (n - 1) / 2` or `n * (n + 1) / 2` | `choose2(n)` / `choose2(n + 1)` (`std.math`) | fix | #1 | syn | |
| manual_ceil_sqrt | `let r = isqrt(n); if r * r < n: r + 1 else: r` | `ceil_sqrt(n)` (`std.math`) | fix | #1 | syn | |
| manual_iroot | binary search or `pow(x, 1.0/k)` for integer k-th root | `iroot(x, k)` (`std.math`) | fix? | #1 | loop | |
| hand_rolled_mobius | linear sieve computing Mobius `mu[1..=n]` | `mobius(n)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_div_tables | nested harmonic loop filling divisor count or divisor sum table | `div_count_table(n)` / `div_sum_table(n)` (`std.math`) | fix? | #1 | loop | |
| hand_rolled_mult_sieve | linear sieve with prime-power transition for multiplicative function | `sieve_multiplicative(...)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_primitive_root | factoring `p - 1` and testing `g^((p-1)/q) != 1` | `primitive_root(p)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_mod_log | baby-step giant-step `HashMap` loop for discrete log | `mod_log(a, b, m)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_mod_sqrt | Tonelli-Shanks / Cipolla loop for modular square root | `mod_sqrt(a, p)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_floor_sum | Euclidean recursion for \(\sum \lfloor(ai+b)/m\rfloor\) | `floor_sum(n, m, a, b)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_prime_count | Meissel-Lehmer / Lucy DP counting primes up to `n` | `prime_count(n)` / `primes_in(lo, hi)` (`std.math`) | warn | #1 | loop | |
| hand_rolled_dirichlet | harmonic double loop for Dirichlet convolution or dujiao sieve | `dirichlet_convolve` / `dirichlet_inv` / `DirichletSum` (`std.math`) | warn | #1 | loop | |
| hand_rolled_splitmix64 | manual `0x9e3779b97f4a7c15` bit-mixing hash function | `splitmix64(x)` (`std.math`) | fix | #1 | syn | |

### S.mod: modular arithmetic and combinatorics

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_mod_try_inv | `if gcd(a, m) == 1: Some(inv_mod(a, m)) else: None` | `x.try_inv()` (`std.mod`) | fix | #5 | syn | |
| manual_mod_multinomial | `comb.fact(sum) * prod(comb.inv_fact(k))` | `comb.multinomial(ks)` (`std.mod`) | fix | #1 | loop | |
| manual_mod_arrange | `comb.fact(n) * comb.inv_fact(n - k)` | `comb.arrange(n, k)` (`std.mod`) | fix | #1 | syn | |
| manual_mod_fact_loop | `var w = Z(1); for i in 1..=k: w *= Z(i)` inside a loop | `comb.fact(k)` (`std.mod`) | fix? | #10 | loop | |
| hand_rolled_barrett | `((u128(a) * m) >> 64)` manual Barrett reduction struct | `Barrett` (`std.mod`) | warn | #1 | ty | |
| hand_rolled_prime_power_comb | Lucas/Granville factor-out-`p` factorial table for `Mod<p^k>` | `PrimePowerComb` (`std.mod`) | warn | #1 | ty | |

### S.poly, S.matrix, S.ntt

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_poly_deriv_integral | loop multiplying `a[i] * i` or `a[i] / (i + 1)` on polynomial coeffs | `p.deriv()` / `p.integral()` (`std.poly`) | fix | #1 | loop | |
| hand_rolled_poly_log_exp | Newton iteration computing formal power series `log`/`exp` | `p.log(n)` / `p.exp(n)` (`std.poly`) | warn | #1 | loop | |
| hand_rolled_poly_div_mod | reversed-poly inverse for polynomial quotient and remainder | `p.div_mod(q)` (`std.poly`) | warn | #1 | loop | |
| hand_rolled_poly_taylor_shift | factorial-scaled convolution for `P(x + c)` | `p.taylor_shift(c)` (`std.poly`) | warn | #1 | loop | |
| hand_rolled_lagrange_interp | O(n^2) or consecutive-point O(n) Lagrange interpolation loop | `lagrange_at(ys, x)` / `interpolate(xs, ys)` (`std.poly`) | warn | #1 | loop | |
| hand_rolled_subproduct_tree | divide-and-conquer polynomial tree for multipoint evaluation | `SubproductTree` (`std.poly`) | warn | #1 | ty | |
| hand_rolled_matrix_det_mod | Gaussian elimination loop computing determinant mod `P` | `mat.det_mod()` (`std.matrix`) | warn | #1 | loop | |
| hand_rolled_char_poly | Hessenberg reduction + DP for characteristic polynomial | `mat.char_poly()` (`std.matrix`) | warn | #1 | loop | |
| hand_rolled_pfaffian | skew-symmetric elimination computing Pfaffian | `mat.pfaffian()` (`std.matrix`) | warn | #1 | loop | |
| hand_rolled_real_gauss | floating-point partial-pivot elimination loop | `det_real` / `rank_real` / `solve_real` / `inverse_real` (`std.matrix`) | warn | #1 | loop | |
| hand_rolled_convolve_all | priority-queue / divide-and-conquer loop multiplying a list of polynomials | `convolve_all(polys)` (`std.ntt`) | fix? | #1 | loop | |
| hand_rolled_relaxed_convolve | online divide-and-conquer FFT (`cdq_fft`) | `RelaxedConvolve` / `SemiRelaxedConvolve` (`std.ntt`) | warn | #1 | loop | |
| hand_rolled_subset_exp_log | ranked subset convolution for set power series exp/log | `subset_exp` / `subset_log` (`std.ntt`) | warn | #1 | loop | |

### S.bits: masks and subsets

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_bitset_subset_sums | `for x in xs: bs = bs.bitor(bs.shifted(x))` (especially with duplicate counts) | `bitset.subset_sums` / `dp.subset_sums` (`std.bitset`, `std.dp`) | fix? | #1 | loop | |
| manual_bitset_range_ops | loop setting, flipping, or counting bits in `[l, r)` one by one | `bs.set_range(l..r)` / `bs.flip_range(l..r)` / `bs.count_range(l..r)` (`std.bitset`) | fix | #10 | loop | |
| manual_bitset_count_below | count set bits `< i` with a loop | `bs.count_below(i)` (`std.bitset`) | fix | #10 | loop | |
| manual_bitset_meets | `not (a & b).is_empty` or loop checking any shared bit | `a.meets(b)` (`std.bitset`) | fix | #10 | syn | |
| manual_bitset_nth | loop scanning set bits `k` times to find the k-th set bit | `bs.nth(k)` (`std.bitset`) | fix | #10 | loop | |
| hand_rolled_dyn_bitset | `List<U64>` with manual word-shift and carry loops | `DynBitset` (`std.bitset`) | warn | #1 | ty | |
| manual_supermasks_loop | `var s = m; while s < (1 << n): ...; s = (s + 1).bitor(m)` | `for s in supermasks(m, n):` (`std.dp`) | fix | #1 | loop | |

### S.geometry

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_geo_cross_dot_128 | manual `Int128(a.x) * b.y - Int128(a.y) * b.x` | `cross128(a, b)` / `dot128(a, b)` / `dist2_128(a, b)` (`std.geometry`) | fix | #1 | syn | |
| manual_geo_manhattan_chebyshev | `abs(a.x - b.x) + abs(a.y - b.y)` or `max(abs(...), abs(...))` | `manhattan(a, b)` / `chebyshev(a, b)` (`std.geometry`) | fix | #1 | syn | |
| manual_geo_turn | sign-of-cross-product helper classifying left/right/collinear turn | `turn(a, b, c)` (`std.geometry`) | fix | #1 | syn | |
| manual_geo_angle_sort | half-plane `y < 0` + cross-product comparator for polar angle sort | `ByAngle` / `ByAngleFrom` / `angle_less` (`std.geometry`) | fix? | #1 | ty | |
| manual_geo_on_segment | collinear + bounding-box check for point on segment | `on_segment(p, a, b)` (`std.geometry`) | fix | #1 | syn | |
| hand_rolled_convex_hull | Andrew's monotone chain lower+upper hull loops | `convex_hull` / `lower_hull` / `upper_hull` (`std.geometry`) | warn | #1 | loop | |
| hand_rolled_closest_pair | divide-and-conquer strip scan for closest pair of points | `closest_pair` / `closest_pair2` (`std.geometry`) | warn | #1 | loop | |
| hand_rolled_point_locate | winding-number or binary-search fan test for point in polygon | `locate(poly, p)` (`std.geometry`) | warn | #1 | loop | |

### S.dp: DP optimizations and memoization

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_monge_path | 1D/1D Monge DP with deque of `(pos, opt_range)` | `monge_path` / `monge_paths` (`std.dp`) | warn | #1 | loop | |
| manual_lis_ends | patience-sorting `tails` array tracking LIS length ending at each index | `lis_ends(xs)` (`std.dp`) | fix? | #1 | loop | |
| manual_dp_backtrack | `while cur != None: path.push(cur); cur = prev[cur]; path.reverse()` | `backtrack(start, step)` (`std.dp`) | fix | #1 | loop | |
| hand_rolled_slope_trick | dual min/max heaps with lazy offset maintaining piecewise-linear convex DP | `Concave` / `Convex` (`add_abs`, `prefix_min`, `slide_min`, `min_convolve`) (`std.slope`) | warn | #1 | ty | |

### S.offline: Mo, CDQ, windows with undo

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| hand_rolled_mo_hilbert | recursive Hilbert-curve index function sorting Mo queries | `mo_order_hilbert` (`std.mo`) | fix? | #1 | ty | |
| hand_rolled_mo_rollback | block-bucketed Mo with snapshot/undo for non-deletable structures | `mo_rollback` (`std.mo`) | warn | #1 | loop | |
| hand_rolled_mo_with_updates | 3D `(l/B, r/B, t)` Mo sorting + time pointer adjustments | `mo_with_updates` (`std.mo`) | warn | #1 | loop | |
| hand_rolled_two_stack_window | two-stack queue simulating sliding-window monoid fold without inverse | `Window` / `Tracked` / `Undoing` / `Sweep` (`std.window`) | warn | #1 | ty | |

### S.containers: deques, lists, grids, randomness

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| manual_deque_pop_front_while | `while not d.is_empty and p(d.first!): d.pop_front()` | `d.pop_front_while(p)` (`std.deque`) | fix | #2 | loop | |
| manual_deque_splice | loop popping from one deque and pushing to another | `d.splice_back(other)` / `d.splice_front(other)` (`std.deque`) | fix | #1 | loop | |
| manual_grid_dir_turn | `(dir + 1) % 4` / `(dir + 3) % 4` on integer direction index | `Dir4` / `Dir8` `.left` / `.right` (`std.grid`) | fix | #3 | syn | |
| manual_rng_pick_sample | `xs[rng.int(0..xs.len)]` or Floyd/reservoir loop | `rng.pick(xs)` / `rng.sample(xs, k)` (`std.rand`) | fix | #1 | syn | |
| manual_rng_chance | `rng.int(0..d) < n` for Bernoulli trial | `rng.chance(n, d)` (`std.rand`) | fix | #1 | syn | |
| manual_pair_other | `if e.u == x: e.v else: e.u` on an unordered endpoint pair | `pair.other(x)` (`std.pair`) | fix | #1 | syn | |
| manual_unordered_pair_swap | `if u > v: swap(mut u, mut v)` before keying an unordered edge | `Pair.new(u, v)` (`std.pair`) | fix | #3 | syn | |

---

## H. C++ habits (spelling carried over from C++)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| while_counter_loop | `var i = 0; while i < n: ...; i += 1` | `for i in 0..n` | fix | #2 | loop | |
| reverse_index | `xs[n - 1 - i]` | `xs.rev` / `D.mirror(i)` | fix | #3 | syn | ✓reverse_index |
| mod_index_ring | `(i + 1) % n` on positions | ring domain / `xs.ring.next(i)` | fix? | #3 | syn | |
| temp_swap | `let t = a; a = b; b = t` | `swap(mut a, mut b)` / `xs.swap(i, j)` | fix | #13 | flow | |
| test_count_loop | `for _ in 0..read<Int>():` wrapping the solution | `@!multitest` | fix | #11 | syn |✓test_count_loop |
| maxn_arrays | arrays sized by a literal bound | arrays over the declared domain | fix? | #6 | ty | |
| nested_list_grid | `List<List<T>>` for a fixed rectangle | `[R, C]T` | fix? | #3 | ty | ✓nested_list_grid |
| pair_first_second | `.0`/`.1` read far from construction | named record fields | warn | B6 | ty | |
| int_enum_constants | `let UP = 0, RIGHT = 1` | `enum Dir = Up \| Right \| ...` | fix? | #3 | ty | |
| bool_flag_state | several `Bool`s encoding one state | an `enum` | warn | #3 | flow | |
| bit_packed_state | `x & 2`, `x & 1` as fields | a record `(fed: Bool, blue: Bool)` | warn | #3 | syn | |
| memset_reset | loop setting every entry to 0 between uses | `fill` / declare per test / `Marks` | fix | #8 | loop | |
| comparator_bool | `sort(by:)` with a less-than lambda | key function | fix | #13 | ty | |
| turbofish_type_args | `f::<T>(...)` | `f<T>(...)` | fix | #13 | syn | ✓turbofish_type_args |
| big_inf_literal | `1e18`, `LLONG_MAX`, `0x3f3f3f3f` | `?T` / `inf` where a bound is meant | fix? | #5 | syn | |
| global_counter_id | `id[x] = cnt++` to number a subset | `numbered()` / `compress` | fix? | #3 | loop | |
| early_return_flag | `var ok = true` ... `if not ok: print(...)` | `all(p)` / early `return` | fix? | #2 | loop | |
| ternary_spelling | `a if c else b` / `c ? a : b` (parse error today) | `if c: a else: b` (diagnostic, not lint) | n/a | #13 | | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| chmin_chmax_if | `if x < best: best = x` or `best = min(best, x)` (from C++ `chmin`/`ckmin`) | `best min= x` / `best max= x` | fix | #13 | syn | ✓chmin_chmax_if |
| peek_then_pop | `let x = s.last!; s.pop()` (from C++ `.back(); .pop_back()` / `.top(); .pop()`) | `let x = s.pop()!` (same for `Deque.pop_front()`, `Heap.pop()`) | fix | #13 | syn | ✓peek_then_pop (`List` only) |
| shift_mul_div_two | `x << 1` or `x >> 1` for doubling/halving a non-bitmask integer | `x * 2` or `x / 2` | fix? | #13 | ty | |
| xor_one_bool_toggle | `var turn = 0; ...; turn ^= 1` (C++ parity toggle for 2-state flag) | `var turn = false; ...; turn = not turn` | fix? | #3 | flow | |
| count_down_while | `var i = n - 1; while i >= 0: ...; i -= 1` (from C++ `for (int i = n-1; i >= 0; i--)`) | `for i in (0..n).rev:` | fix | #2 | loop | |
| char_minus_zero_digit | `Int(c) - Int('0')` or `Int(c) - Int('a')` for digit/alphabet indexing | `parse_int` (`std.strings`) or `domain D = 'a'..='z'` | fix? | #3 | syn | |
| plus_mod_percent_mod | `((a - b) % P + P) % P` manual negative-modulo fixup | `(a - b) % P`: `%` already floors (R01.5.2), or `Mod<P>` (`std.mod`) | fix? | #1 | syn | |
| cpp_edge_xor_one | storing undirected/residual edges at `2*i` and `2*i ^ 1` in a flat list | `Graph` / `FlowGraph` (`std.graph`, `std.flow`) | warn | #1 | loop | |
| while_not_empty_pop | `while not q.is_empty: let x = q.pop()!` | `while let x = q.pop():` | fix | #5 | syn |✓while_not_empty_pop |
| check_then_insert_set | `if not seen.contains(x): seen.add(x); ...` (C++ `count` then `insert`) | `if seen.add(x): ...` (`std.marks`, `std.bitset`) | fix | #10 | flow | |
| manual_sort2_swap | `if a > b: swap(mut a, mut b)` only to get ordered `(lo, hi)` | `Pair.new(a, b)` (`std.pair`) or `(min(a, b), max(a, b))` | fix? | #13 | syn | |
| int_bool_increment | `if cond: ans += 1` inside a loop when `ans` is a counter | `xs.count(cond)` (`std.seq`) | fix | #2 | loop | |

---

## D. Domains, positions and cuts

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| position_arithmetic | `V(v.ord + 1)` | `v.next` / `v.prev` | fix | #3 | ty | ✓position_arithmetic |
| int_array_over_domain | `[n]T` where `n` is a domain's size and it is indexed by `Int(v)` | `[V]T` | fix? | #3 | flow | |
| repeated_ord_conversion | `Int(v)` / `xs[Int(t)]` more than once per value | keep the domain; key the array by it | warn | #14 | flow | |
| raw_loop_over_domain | `for i in 0..n` where `n == V.len` and `i` is converted | `for v in V` | fix | #3 | flow | |
| same_size_domains_mixed | a `V`-typed value used to index an `E`-sized raw array | separate domains | warn | #3 | flow | |
| plus_one_cut | `pre[r + 1] - pre[l]` / `a[l..r + 1]` | cuts: `.after`, `l..=r`, `pre.fold(r)` | fix | #3 | syn | |
| literal_slice_index | `w[0]` on a slice | `w.first!` / the slice's own keys | fix | #3 | ty | |
| manual_base_shift | `read()` then `- 1` on every index | `domain V = 1..=n` / `with io(base: 1)` | fix? | #6 | flow | |
| compressed_raw_mix | compressed index compared with a raw value | `compress` keeps the two types apart | warn | #3 | ty | |
| domain_spelling | `D.sorted(by:)`, `xs.first(where:)` | `D.sort(by:)`, `xs.first(p)` | fix | #13 | ty | ✓domain_spelling |
| unproven_list_index | `List` index in nested loops checked every access | a domain-keyed array | warn | #10 | ty | ✓unproven_list_index |
| symmetric_write | `m[u][v] = x; m[v][u] = x` | `Pair` / `.symmetric` | fix | #7 | syn | ✓symmetric_write |
| sentinel_node_index | `n`, `n + 1` as source/sink | labelled extra positions in the domain | warn | #3 | ty | |
| anonymous_domain_reused | the same `[n]` written in several declarations | one `domain` | fix | #7 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| domain_empty_first_last | `D.first!` / `D.last!` when `D`'s declared bound allows size `0` (`n >= 0`) | guard `if let f = D.first:` or tighten `n: 1..=N` | warn | #5 | bound | ✓domain_empty_first_last |
| cut_difference_off_by_one | `r - l + 1` on half-open cuts or `r - l` on inclusive positions | use `.len` on the range `l..r` or `l..=r` | warn | #4 | ty | |
| inclusive_range_empty_bound | `a..=b` where declared bounds allow `a > b` without emptiness check | handle empty case or use half-open cuts | warn | #4 | bound | |
| mixed_zero_one_based_read | reading `1..=n` into `Int` and manually subtracting `1` on only some uses | declare `domain V = 1..=n` and `read<V>()` | warn | #3 | flow | |
| domain_cast_roundtrip | converting `V -> Int -> V` (`V.at(v.ord)`) with no arithmetic in between | use `v` directly | fix | #3 | ty | |
| slice_key_lost_indexing | indexing a sliced `[D]T` with an integer `0..k` instead of `D` position | iterate `for p in s.keys:` or index with the cut/position | warn | #3 | ty | |
| reversed_range_literal | `for i in n..0:` (empty ascending range, meant `(0..n).rev`) | `for i in (0..n).rev:` | fix? | #4 | bound |✓reversed_range_literal |
| cut_midpoint_overflow | `(l.ord + r.ord) / 2` computed in `Int` instead of range midpoint | `(l..r).mid` | fix | #4 | syn | |
| subrange_not_checked_nonempty | indexing `a[l..r].first!` when `l <= r` (may be empty) from bounds | check `if let x = a[l..r].first:` or prove `l < r` | warn | #4 | bound | ✓unwrap_on_possibly_empty |
| unbounded_cut_step | `p.next!` / `p.prev!` in a loop without proving `p` is not at domain boundary | iterate `for p in D.init:` / `for (a, b) in a.windows(2):` | warn | #4 | bound | |
| domain_size_recomputed | `hi - lo + 1` repeated when `D` is already declared as `lo..=hi` | `D.len` | fix | #6 | ty | |

---

## O. Optionals and sentinels

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| none_check_unwrap | `x != None and ... x!` | `if x is Some(v):` | fix | #5 | flow | ✓none_check_unwrap |
| sentinel_table | `[D]Int` filled with `-1` meaning absent | `[D]?Int` | fix? | #5 | flow | ✓sentinel_table |
| sentinel_scalar | scalar `var found = -1` / `var best = -10**18` set in a `for` loop | `var found: ?Int = none`, or `.max()` | warn | #5 | flow | ✓sentinel_scalar |
| sentinel_compare | `== -1` / `!= INF` tests | `is None` / `is Some` | fix? | #5 | flow | |
| found_flag_pair | `var found = false; var val = 0` | `?T` | warn | #5 | flow | ✓found_flag_pair |
| match_to_map | `match o: Some(v) => Some(f(v)); None => None` | `o.map(f)` | fix | #13 | syn | |
| match_to_default | `if o is Some(v): v else: d` | `o ?? d` | fix | #13 | syn | |
| match_to_and_then | nested optional matches | `o.and_then(f)` | fix | #13 | syn | |
| is_some_and | `o is Some(v) and p(v)` in a value position | `o.is_some_and(p)` | fix | #13 | syn | |
| filter_map_identity | `xs.filter_map(x => x)` | `xs.present()` | fix | #13 | syn | |
| repeated_unwrap | same `x!` many times in a block | bind once | fix | #14 | flow | |
| total_after_fill | `dist[v]!` everywhere after a loop that set all | `let dist = dist!` once | fix? | #5 | flow | |
| unwrap_or_panic | `o ?? panic(...)` | `o!` (with message if needed) | fix | #13 | syn | |
| option_bool | `?Bool` used as tri-state | an `enum` | warn | #3 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| sentinel_coalesce_ternary | `if x != -1: x else: y` (sentinel fallback) | `x ?? y` on `?T` | fix? | #5 | flow | |
| option_match_is_none_or | `match x: None => true, Some(v) => p(v)` | `x.is_none_or(p)` (`std.opt`) | fix | #5 | syn | |
| sentinel_char_or_empty_str | `'?'`, `'\0'`, or `""` stored in table to mean "unset" | `?[D]Char` / `?[D]Str` | warn | #5 | flow | |
| sentinel_zero_for_unset | `0` used as "unvisited/unset" in array whose valid values can include `0` | `?[D]Int` | warn | #5 | bound | |
| map_contains_then_index | `if m.contains(k): use(m[k])` (double hash lookup + trap risk) | `if let v = m.get(k): use(v)` | fix | #5 | flow |✓map_contains_then_index |
| unwrap_on_possibly_empty | `xs.min()!`, `xs.max()!`, `xs.first!`, `xs.last!` when `xs.len` can be `0` from bounds | handle `None` or tighten `n: 1..=N` | warn | #5 | bound | ✓unwrap_on_possibly_empty |
| option_assign_none_then_branch | `var res: ?T = None; ...; if res == None: return None; Some(res!)` | `return` directly or use `?` propagation | fix? | #5 | flow | |
| option_unwrap_in_loop_invariant | `opt!` called on every iteration of a loop when `opt` is loop-invariant | unwrap once before loop: `let v = opt!` | fix | #5 | loop | |

---

## I. Input and output

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| bool_ternary_print | `print(if ok: "YES" else: "NO")` | `print(yes(ok))` | fix | #11 | syn | ✓bool_ternary_print (warn: words follow the profile) |
| answer_words_repeated | `yes(ok, answers: ("Yes", "No"))` at several calls | `with io(answers: ...)` once | fix | #7 | syn | |
| join_loop | building a string with separators to print | `print(xs, sep:)` | fix | #11 | loop | |
| print_end_loop | `print(x, end: " ")` loop then `print()` | `print(xs)` | fix | #11 | loop | |
| read_push_loop | `for _ in 0..n: xs.push(read())` | `read<[n]T>()` | fix | #6 | loop | |
| split_reads | consecutive `read()` for one record | `read<(a: A, b: B)>()` | fix | #6 | syn | |
| unbounded_read | `read<Int>()` where the statement gives a bound | `read<1..=2e5>()` | doc | #6 | | |
| read_str_then_chars | read `Str`, then index chars as a grid | `read<[n]Char>()` / `[h, w]Char` | fix? | #6 | ty | |
| eof_loop | `while has_input()` loop | `read_all` / `@!multitest(until_eof)` | fix | #11 | syn | |
| case_prefix_by_hand | `"Case #{t}: "` printed by hand | `multitest(prefix:)` | fix | #11 | syn | |
| interactive_without_ask | `print` then `read` in an interactive problem | `ask` / `Wire` (flushes) | fix? | #11 | syn | |
| float_print_default | printing floats without a precision setting | `with io(precision:)` | warn | #11 | ty | |
| bool_as_int_print | `print(Int(b))` for 0/1 output | fine when the format is 0/1; else `yes(b)` | doc | #11 | | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| interactive_missing_flush | `print(...)` followed by `read(...)` without `Wire` or `with io(flush: true)` | use `Wire.ask` (`std.io`) or `with io(flush: true)` | warn | #11 | flow | |
| interactive_exit_on_minus_one | `if res == -1: return` scattered after every `read` in interactive problem | centralize verdict handling in a `Wire` helper (`std.io`) | doc | #11 | flow | |
| interactive_read_before_flush | `read` called on a `Wire` before flushing pending query output | use `wire.ask(query)` (`std.io`) | warn | #11 | flow | |
| read_1based_then_sub_one | `let u = read<Int>() - 1` (manual 1-to-0 shift on input) | `domain V = 1..=n; let u = read<V>()` | fix? | #3 | syn | |
| print_1based_add_one | `print(v.ord + 1)` (manual 0-to-1 shift on output) | `domain V = 1..=n; print(v)` | fix? | #3 | ty | |
| read_graph_edges_loop | `for _ in 0..m: let (u, v) = read(); g.add(u, v)` | `Graph.read(V, m)` (`std.graph`) or `needs stdlib: std.graph.read_edges` | fix? | #11 | loop | |
| with_io_nested_redundant | nested `with io(...)` repeating identical settings already active in enclosing scope | remove inner `with io(...)` | fix | #11 | flow | |
| wire_created_in_loop | constructing a new `Wire` inside a query loop instead of once per test | lift `Wire` creation above the loop | warn | #11 | loop | |

---

## W. Ownership, lending and moves

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| var_never_reassigned | `var` never changed | `let` | fix | #8 | flow | ✓var_never_reassigned |
| needless_clone | `.clone()` where a lend or a move works | drop `.clone()` | fix | #8 | flow | |
| clone_in_loop | `.clone()` of a loop-invariant value per iteration | hoist / lend | fix? | #10 | flow | |
| mut_param_unused | `mut` parameter never mutated | drop `mut` (and at call sites) | fix | #8 | flow | |
| copy_to_read_field | copying a struct to read one field | read through the lend | fix | #10 | flow | |
| lent_gen_walked_twice | generator holding a `mut` lend walked twice | collect once, or restructure | warn | #8 | flow | |
| move_then_clone | value cloned at its last use | move it | fix | #10 | flow | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| cache_field_written_outside_cache | mutating a `@cache` field outside a `@cache`-updating method or clearing without invalidation | restrict mutation to cache helpers or remove `@cache` | warn | #8 | ty | |
| cache_field_observable_leak | `@cache` field read directly by observable output/return instead of pure recomputation | keep `@cache` strictly observationally transparent | warn | #8 | flow | |
| mut_param_reassigned_whole | `fn f(mut xs: List<Int>)` that overwrites `xs = ...` instead of mutating in place | return the new `List<Int>` | warn | #8 | flow | |
| lent_slice_cloned_for_read | `xs[l..r].clone()` passed to a read-only (`lent`) parameter | pass `xs[l..r]` directly | fix | #8 | ty | |
| return_local_clone | `return local.clone()` on a local value at end of scope | `return local` (move) | fix | #8 | flow | |
| move_into_loop_reconstructed | moving a container into a loop body and rebuilding an identical empty one each iteration | reuse one buffer with `.clear()` | fix? | #10 | loop | |
| large_value_copied_in_closure | closure capturing a large array/container by value when it only reads it | lend via parameter or borrow capture | warn | #8 | ty | |

---

## F. Functions, closures and generators

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| eta_lambda | `x => f(x)` | `f` | fix | #13 | syn | |
| operator_lambda | `(a, b) => a + b`, `(a, b) => min(a, b)` | `+`, `min` | fix | #13 | syn | |
| field_lambda | `x => x.len` | `.len` | fix | #13 | syn | |
| recursive_lambda | Y-combinator / self-passing closure | nested `fn` | fix? | B6 | syn | |
| list_only_iterated | `List` built only to be walked once | a generator (`-> yields T`) or lazy chain | fix? | #10 | flow | |
| lazy_chain_reused | lazy chain walked twice (recomputes) | `.collect()` once | fix | #10 | flow | |
| map_reread | `.map(f)` view read per index in a hot loop | `.collect()` | fix? | #10 | flow | |
| long_lambda_chain | lambda nesting past three levels | a named `fn` | warn | B6 | syn | |
| discarded_result | `let _ =` dropping a result of this file's fn | use it or drop the call | warn | #7 | ty | ✓discarded_result |
| out_param | `mut` list parameter used only to return results | return the value | fix? | #8 | flow | |
| bool_arg | positional `Bool` arguments | named argument or `enum` | warn | B6 | ty | |
| redundant_arg_name | `f(moves: moves)` | `f(moves:)` | fix | #13 | syn | ✓redundant_arg_name |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| memo_unbounded_key | `@memo` function with an unconstrained `Int` parameter (can blow memory/time) | bound parameter with domain/range or pass explicit table | warn | #6 | bound | |
| memo_mut_capture | `@memo` function reading a mutable outer variable not in its key (stale cache bug) | pass the varying state as a parameter | warn | #8 | flow | |
| memo_large_tuple_key | `@memo` keying on a `List` or `Str` when a small domain/index uniquely identifies it | key on domain position / integer ID | warn | #10 | ty | |
| generator_collects_then_yields | `fn ... -> yields T` that builds a `List<T>` then `for x in res: yield x` | `yield` directly or return `List<T>` | fix? | #10 | flow | |
| generator_single_yield | `fn ... -> yields T` that yields at most once | return `?T` | fix? | #5 | flow | |
| generator_mutates_external_state | generator mutating outer state during iteration (breaks on re-walk or short-circuit) | keep generators pure or materialize once | warn | #8 | flow | |
| lambda_redundant_parens | `(x) => f(x)` or `(x, y) => (x + y)` unnecessary wrapper parens | `x => f(x)` / `(x, y) => x + y` | fix | #13 | syn | |
| fn_returns_unit_trailing_expr | `fn f()` ending in a non-unit expression whose value is silently dropped | bind or discard explicitly, or return the value | warn | #8 | ty | |
| unused_closure_param | closure taking `(k, v)` when only `v` is used without `_` | `(_, v) => ...` or `.values` | fix | #13 | syn | |

---

## C. Complexity (costs that do not look like costs)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| quadratic_at_bounds | nested loops over domains whose declared sizes make it > ~1e9 steps | (warn with the computed count) | warn | #10 | bound | |
| contains_on_list_in_loop | `x in list` inside a loop | `Set` / `Marks` / `[D]Bool` | fix? | #10 | ty | |
| sort_in_loop | sorting the same data each iteration | sort once / sorted container | warn | #10 | flow | |
| count_in_loop | `xs.count(...)` over the same `xs` per element | `tally` once | fix? | #10 | flow | |
| len_of_lazy_in_loop | `.len` of a lazy filter re-walked | collect once | fix | #10 | flow | |
| deep_recursion | recursion depth bounded by a large declared size | iterative form / note the stack the profile gives | warn | #10 | bound | |
| hashmap_hackable | `HashMap` with integer keys on a hackable judge profile | domain array / sorted map / randomized hash | warn | #10 | ty | |
| repeated_power | `x ** k` recomputed inside a loop over `k` | running product | fix? | #10 | loop | |
| quadratic_string_build | appending to `Str` in a loop | collect pieces | fix? | #10 | loop | |
| refold_range | `xs[l..r].sum()` for many overlapping ranges | `Prefix` | warn | #10 | loop | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| n_log_n_inside_n_at_bounds | `sort` / `Heap` / `OrderedSet` `O(n log n)` inside an `O(n)` loop when `n^2 log n > 10^8` | maintain incremental structure or pre-sort once | warn | #10 | bound | |
| nested_domain_product_tle | nested loops over `D1`, `D2`, `D3` whose declared bound product exceeds `~3 * 10^8` | reduce asymptotic complexity or verify pruning | warn | #10 | bound | |
| pow_mod_same_base_in_loop | calling `pow_mod(base, e, P)` inside a hot loop with fixed `base` and small `e` | precompute power table `[0..=E] i => ...` | fix? | #10 | bound | |
| hashmap_small_dense_int_key | `HashMap<Int, T>` whose key has declared range `0..=N` with `N <= 10^6` | `[0..=N]T` array (`std.seq`) | fix? | #10 | bound | |
| list_remove_front_in_loop | `xs.remove(0)` / `xs.insert(0, x)` inside a loop (`O(n)` shift each time) | `Deque` (`std.deque`) | fix? | #10 | loop | |
| rebuild_segtree_per_query | `SegTree.from(a)` / `Prefix.from(a)` constructed inside the query loop | build once before the query loop and update in place | warn | #10 | loop | |
| dijkstra_missing_stale_skip | priority-queue loop popping `(d, u)` without checking `if d > dist[u]: continue` | `g.dijkstra(s)` (`std.graph`) or `Settle` (`std.heap`) | warn | #10 | loop | |

---

## A. Allocation and layout

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| alloc_in_hot_loop | new `List`/array per iteration of a hot loop | hoist and `fill` / `clear` | fix? | #10 | flow | |
| wide_array | element type wider than its proven range needs | narrower range type | warn | #10 | bound | |
| struct_padding | record padding over 20% | reorder fields / narrower types | warn | #10 | ty | |
| collect_for_len | `.collect().len` | `.count()` | fix | #10 | syn | |
| to_str_in_loop | `.to_str()` of slices in a loop | keep the view | fix? | #10 | flow | |
| big_local_array | large fixed array in a recursive fn | hoist | warn | #10 | bound | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| list_of_fixed_small_lists | `[D]List<Int>` where every inner list has constant small length `2..=4` | tuple or small struct `[D](Int, Int)` | fix? | #10 | flow | |
| unreserved_push_loop | `var xs = List.new(); for i in D: xs.push(...)` when `D.len` is known | `[D] i => ...` or `xs.reserve(D.len)` (`std.seq`) | fix | #10 | loop | |
| option_full_width_int_array | `[D]?Int` over full 64-bit `Int` (no niche, doubles storage to 16 bytes/elem) | tighten value range (e.g. `0..=10^9`) so `?T` uses a spare niche | warn | #6 | ty | |
| clone_in_recursion | cloning a `List` / `HashMap` / `Bitset` on every recursive call | mutate with undo (push/pop) or pass lent slice | warn | #10 | flow | |
| temp_alloc_in_comparator | allocating a `List` or `Str` inside a `sort(by: ...)` comparator | precompute key array (Schwartzian transform) | warn | #10 | flow | |
| sparse_2d_array_over_large_domains | `[D1][D2]T` where `D1.len * D2.len > 10^7` from declared bounds | `HashMap<(D1, D2), T>` or coordinate compression | warn | #10 | bound | |

---

## K. Arithmetic, overflow, modulus and floats

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| float_equality | `==`/`!=` on floats | `approx(a, b, abs:)` | warn | #3 | ty | ✓float_equality |
| repeated_literal | a bound or modulus literal in several places | name it once (`type Z`, `domain`) | fix | #6 | syn | ✓repeated_literal |
| parity_of_negative | `x % 2 == 1` where `x` may be negative | `x % 2 != 0` | fix | #3 | bound |✗ not a bug here: `/` and `%` floor for a positive divisor (R01.5.2), so the C++ trap does not exist |
| trunc_div_negative | `/` on values that may be negative where floor is meant | `div_floor` / `divmod` | warn | #3 | bound |✗ not a bug here: `/` and `%` floor for a positive divisor (R01.5.2), so the C++ trap does not exist |
| overflow_before_mod | `a * b % m` on `Int` that can exceed 64 bits | `Mod` / `mul_mod` | warn | #3 | bound | |
| float_for_integer | `Float` arithmetic on integer data (`pow`, `log2`) | integer ops (`**`, `bit_len`) | fix? | #3 | ty | |
| wide_shift | `1 << k` where `k` can reach 63 | `bit(k)` on a wider type | warn | #3 | bound | ✓int_shift_overflow_at_bounds |
| eps_magic | different `eps` literals in one file | one named `eps` | fix | #7 | syn | |
| int_of_mod_math | `Int(z)` then arithmetic then back to `Mod` | stay in `Mod` | fix | #6 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| int_mul_overflow_at_bounds | `a * b` where `ProvenExpr` range exceeds `Int` (`i64`) bounds | widen to `Int128`, use `Mod<P>`, or reorder division | warn | #6 | bound | ✓int_overflow_at_bounds |
| int_add_overflow_at_bounds | `a + b` or `sum` where `ProvenExpr` range exceeds target integer width | widen type or use `Mod<P>` | warn | #6 | bound | ✓int_overflow_at_bounds |
| int_shift_overflow_at_bounds | `1 << k` where declared bound allows `k >= 63` (or `k < 0`) | guard `k`, use `Bitset` (`std.bitset`), or `pow_mod` (`std.math`) | warn | #6 | bound |✓int_shift_overflow_at_bounds |
| int_abs_min_overflow | `abs(x)` when `x`'s range includes `Int.MIN` | tighten range or widen before `abs` | warn | #6 | bound | |
| int_div_possible_zero | `a / b` or `a % b` where `0` is inside `b`'s proven range | guard `b != 0` or tighten `b: 1..=N` | warn | #6 | bound |✓int_div_possible_zero |
| mod_literal_mismatch | two different prime-like moduli (e.g. `998244353` and `1000000007`, or typo `100000007`) in one file | declare one `type Z = Mod<P>` | warn | #6 | ty | |
| mod_subtract_without_mod | `(a - b) % P` where `a - b` can be negative (remainder is negative in CP Lang) | `Mod<P>` (`std.mod`) | warn | #1 | bound |✗ not a bug here: `/` and `%` floor for a positive divisor (R01.5.2), so the C++ trap does not exist |
| mod_pow_exponent_mod_p | reducing exponent of `pow_mod(a, e, P)` modulo `P` instead of `P - 1` (Fermat's little theorem bug) | reduce exponent mod `P - 1` (for prime `P`, `a != 0`) | warn | #6 | flow | |
| mod_div_non_prime | `a / b` on `Mod<M>` where `M` is statically known composite | use `b.try_inv()` (`std.mod`) or `ext_gcd` (`std.math`) | warn | #6 | ty | |
| float_cmp_without_eps_in_geo | `<` / `>` / `==` on `Float` cross products or distances in geometry | use integer `Pt` (`cross128`, `dist2_128`) (`std.geometry`) or `close(a, b)` | warn | #1 | ty | |
| float_sum_cancellation | alternating large positive/negative `Float` accumulation | sort by magnitude or use integer / rational arithmetic | doc | #6 | flow | |

---

| count_bool_seq | `count(p(x) for x in xs)` or `.count()` with no predicate on a `Bool` sequence: counts every element, not the trues | `xs.count(x => p(x))` | fix | #13 | ty | new (found porting a differential test) |

## M. Mutation and aliasing

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| same_place_read_write | `a[l] += inc(mut a[l])` | split the statement | warn | #8 | flow | |
| pop_into_same | `xs[0] = xs.pop()!` | bind the popped value first | warn | #8 | flow | |
| mutate_while_iterating | push/remove on the sequence being walked | iterate a snapshot or use indices deliberately | warn | #8 | flow | |
| swap_two_muts | `swap(mut a[i], mut a[j])` | `a.swap(i, j)` | fix | #8 | syn | |
| captured_var_mutated | closure captures a `var` that changes after capture | pass it, or `let` a copy | warn | #8 | flow | |
| compound_self_ref | `x = x + 1` | `x += 1` | fix | #13 | syn |✓compound_self_ref |
| conditional_assign | `if c: x = a else: x = b` | `x = if c: a else: b` | fix | B6 | syn | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| dp_in_place_wrong_direction | 0/1 knapsack `for w in weight..=W: dp[w] max= dp[w - weight] + val` (ascending overwrites same layer) | iterate `(weight..=W).rev` (or use `next_dp` layer) | warn | new: P17 | loop | |
| bag_cursor_invalidated | holding a `BagCursor` across `bag.add` / `bag.remove` and reusing it | re-acquire cursor or use `drain_range` (`std.bag`) | warn | #8 | flow | |
| heap_key_mutated_externally | mutating a score array used by a `Heap` without pushing/updating | use `IndexedHeap` (`std.heap`) or push new pair + stale-skip | warn | #8 | flow | |
| prefix_stale_after_write | reading a `Prefix` / `SparseTable` after mutating the source array it was built from | use `Fenwick` (`std.fenwick`) or `SegTree` (`std.segtree`) | warn | #8 | flow | |
| shared_mut_alias_two_args | passing the same mutable container twice (`f(mut a, mut a)`) or overlapping slices | split into disjoint slices or combine into one parameter | warn | #8 | flow | |
| mutate_map_key_in_place | mutating a struct field that participates in its key while stored in a set/map | remove, mutate, and re-insert | warn | #8 | flow | |

---

## T. Multitest and state

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| state_above_multitest | `var` above `@!multitest` mutated per case | declare it below | warn | #8 | flow | ✓state_above_multitest |
| reset_loop_per_case | manual reset loops at the end of a case | declare per case / `Marks.clear` | fix? | #8 | flow | |
| bound_from_shared_sum | per-case arrays sized by the global max | size by the case's own `n` | warn | #10 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| memo_cache_leaks_across_tests | `@memo` function at module scope that depends on per-test input/globals in `@!multitest` | move `@memo` inside `solve()` or include test-local data in key | warn | #9 | flow | |
| rng_reseeded_inside_multitest | `Rng.new(seed)` called inside `solve()` of `@!multitest` (repeats identical sequence every test) | lift `Rng` above `solve()` or seed per test | warn | #9 | flow | |
| static_sieve_rebuilt_per_test | `Sieve.up_to(MAX)` or `Comb.up_to(MAX)` built inside `solve()` of `@!multitest` | hoist constant-bound precomputation to top-level `let` | fix? | #9 | flow | |
| global_table_partial_reset | resetting only `0..n` of a top-level array when `n + 1` or sentinel slot was written | declare array inside `solve()` (auto-scoped per test) | warn | #9 | flow | |
| multitest_sum_bound_missing | `@!multitest` with `t <= 10^4, n <= 2*10^5` where `O(n)` work looks `O(t * n)` without `@sum(n) <= 2*10^5` | declare the statement's sum-of-`n` constraint so bound checks stay accurate | doc | #6 | bound | |
| io_context_inside_multitest_loop | repeating `with io(answers: ...)` inside every branch of `solve()` | put one `with io(...)` at file/solve top | fix | #11 | syn | |

---

## R. Redundancy and dead code

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| unread_field | record field never read | drop it | warn | #7 | flow | ✓unread_field |
| unused_binding | `let` never read | drop it | fix | #7 | flow |✓unused_binding |
| unused_fn | top-level `fn` never called (outside test attributes) | drop it | warn | #7 | flow | |
| bool_literal_branch | `if c: true else: false` | `c` | fix | B6 | syn | |
| compare_to_bool | `x == true` | `x` | fix | B6 | syn | |
| negated_compare | `not (a == b)` | `a != b` | fix | B6 | syn | |
| identical_branches | both arms equal | drop the `if` | fix | #7 | syn | |
| duplicated_arms | two arms differing only in sign or call | one helper with a parameter | warn | #7 | syn | |
| redundant_sort | `sorted()` before an order-insensitive op (`sum`, `max`, `tally`) | drop it | fix | #10 | ty | |
| redundant_collect | `.collect()` before a `Seq` op | drop it | fix | #10 | ty | |
| self_max | `max(a, a)`, `x min= x` | drop it | fix | #7 | syn | |
| shadows_top_level | local name hides a top-level `let` | rename | warn | B6 | ty | ✓shadows_top_level |
| unreachable_code | statements after `return` / `break` | drop them | fix | #7 | flow |✓unreachable_code |
| redundant_prelude_import | importing what the prelude gives | drop it | fix | #13 | ty | ✓redundant_prelude_import |
| redundant_annotation | type annotation equal to the inferred one on a local | drop it (judgment for docs value) | doc | B6 | | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| empty_range_loop_at_bounds | `for i in a..b:` where declared bounds prove `a >= b` (loop never runs) | fix bounds or remove dead loop | warn | #6 | bound | ✓reversed_range_literal |
| always_true_bounds_check | `if x >= 0:` or `if i < n:` already proven true by `x`'s declared range or domain | remove redundant check | fix? | #6 | bound | |
| redundant_min_max_at_bounds | `min(x, K)` where declared bounds prove `x <= K` (always returns `x`) | `x` (or fix typo in `K`) | warn | #6 | bound | |
| dead_match_arm_at_bounds | `match` / `else if` branch unreachable under declared range constraints | remove dead arm or widen constraint | warn | #6 | bound | |
| redundant_domain_redeclare | two identical `domain` declarations with the same bounds in the same scope when one is a copy | reuse the domain if same index space (or comment why distinct) | warn | #3 | ty | |
| noop_mod_reduce | `x % P` when `x` is already `Mod<P>` or proven in `0..P` | `x` | fix | #6 | bound | |

---

## N. Naming and spelling

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| getter_parens | `.len()`, `.first()`, `.rev()` | no parentheses | fix | #13 | syn | ✓E0304 (checker, with fix) |
| is_in_cond_header | `if x is ...` headers that should bind with `let` | `if let p = x` form | fix | #13 | syn | ✓is_in_cond_header |
| unknown_lint | `with lint(...)` names a lint that doesn't exist | the right name | warn | #13 | syn | ✓unknown_lint |
| deprecated_spelling | any spelling a codemod retired | the current spelling | fix | #13 | syn | |
| shouting_constants | `MAXN`, `MOD` names for types/domains | `Z`, a domain name | doc | #13 | | |
| single_letter_domain_clash | two domains whose names differ by case | rename | warn | B6 | syn | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| trailing_semicolon_stmt | unnecessary C++-style `;` at end of every line | drop trailing `;` | fix | #13 | syn | |
| c_style_cond_parens | `if (a < b):` / `while (x > 0):` redundant outer parens around condition | `if a < b:` / `while x > 0:` | fix | #13 | syn | |
| non_canonical_range_bound | `0..=n - 1` or `1..n + 1` | `0..n` or `1..=n` | fix | #4 | syn | |
| redundant_tail_return | `return expr` as the final statement of a function body | `expr` | fix | #13 | syn | |
| large_unseparated_int_literal | `1000000007` or `1000000000000000000` with 7+ digits and no `_` | `1_000_000_007` / `10**9 + 7` | fix | #13 | syn | |
| inconsistent_index_var_name | using `j` for outer loop and `i` for inner loop, or `x` for row and `y` for col | `i`/`j` outer/inner, `r`/`c` for grid | doc | #13 | syn | |

---

## X. Testing, stress and debugging

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| debug_output_left | `eprint`/trace calls in a build for submission | remove (`cpl build` warns) | warn | #12 | syn | |
| alt_without_gen | `@alt` with no `@gen` and no typed input to generate from | add `@gen` or read types | warn | #12 | ty | |
| alt_unbounded | `@alt` brute without a size cap | `@alt(f, n: ..=8)` | warn | #12 | ty | |
| assert_in_hot_loop | `assert` inside the innermost hot loop | hoist or `debug_assert` | warn | #10 | flow | |
| check_never_fails | `@check` that only compares to a constant | real checker | warn | #12 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| gen_misses_boundary_cases | `@gen` only draws random `n` in `1..=20` without explicitly hitting `n = 1`, `all equal`, `sorted`, `max` | include edge-case weights/branches in `@gen` | doc | #12 | flow | |
| check_ignores_input | `@check(input, out)` that validates `out` syntax without checking `out` against `input` | verify `out` satisfies `input` constraints | warn | #12 | flow | |
| alt_shares_mut_state | `@alt` reading or mutating top-level `var` shared with main `solve()` | isolate state inside each function | warn | #12 | flow | |
| alt_same_complexity_as_main | `@alt` uses the same non-trivial DS/algorithm as main (blind to shared misconception) | write a simpler brute-force `@alt` | doc | #12 | flow | |
| stress_unseeded_rand | `@gen` using non-deterministic seed instead of passed `rng` | use the framework-provided `Rng` so counterexamples replay | warn | #12 | ty | |
| assert_always_true_at_bounds | `assert(x >= 0)` when `x`'s type already guarantees `x >= 0` | remove tautological assert (or assert the real invariant) | warn | #12 | bound | |
| alt_ignores_io_context | `@alt` printing raw `"YES"`/`"NO"` while `solve` uses `with io(answers: ("Yes", "No"))` | return `Bool` or share `with io(...)` | warn | #12 | flow | |

---

## J. Judge and portability

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| output_size_risk | generated C++ near the 48 KB warning | drop unused library code / `cpl explain size` | warn | #10 | | |
| stack_depth_vs_profile | recursion deeper than the profile's stack | iterative form | warn | #10 | bound | |
| answer_words_vs_profile | `YES`/`NO` on a profile whose problems use `Yes`/`No` | `with io(answers:)` | warn | #11 | ty | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| int128_on_32bit_profile | `Int128` used under a 32-bit judge profile where `__int128` is unavailable | use `mul_mod` (`std.math`) or switch judge profile | warn | #6 | ty | |
| memory_limit_at_bounds | total static + declared-bound array allocation exceeds judge profile memory limit (e.g. 256 MB) | narrow element width, rolling DP, or sparse structure | warn | #10 | bound | |
| float_precision_vs_judge | printing `Float` without `with io(prec: ...)` when problem requires `1e-9` | `with io(prec: 12)` | warn | #11 | ty | |

---

## E. Escape hatches

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| provable_unwrap | `!` on a value the checker can prove present | drop `!` / use the total form | fix | #14 | flow | |
| raw_array_for_domain_data | raw `[n]T` holding per-position data of a named domain | `[D]T` | fix? | #14 | flow | |
| unchecked_without_reason | `unchecked` with no comment | add the reason | warn | #14 | syn | |

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| unchecked_block_too_wide | `unchecked:` wrapping an entire function or large loop instead of the single hot access | narrow `unchecked:` to the single inner statement | warn | #14 | syn | |
| raw_cpp_in_normal_code | inline C++ escape hatch used where a stdlib API exists | use the stdlib API or document why inline C++ is required | warn | #14 | syn | |
| lint_allow_file_wide | file-wide `with lint(allow: ...)` silencing a safety lint (overflow/bounds/multitest) | scope `with lint(allow: ...)` to the single expression/function | warn | #14 | syn | |
| unchecked_provable_at_bounds | `unchecked:` around an index/arithmetic op that bound analysis already proves safe | remove `unchecked:` (zero-cost already) | fix | #14 | bound | |

---

## L. Library authoring (user libraries and the stdlib)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| pub_without_cost | `pub fn` with no documented cost | `/// O(...)` | warn | #10 | syn | |
| pub_without_doc | `pub fn` with no doc comment | doc comment | warn | B6 | syn | |
| by_variant_declared | declaring `foo_by` beside `foo` | `foo(by:)` | warn | #13 | syn | |
| op_without_laws | an op used as a monoid with no `Monoid` / laws declared | declare it | warn | #1 | ty | |
| getter_declared_with_parens | zero-arg accessor declared as a method | a getter | warn | #13 | syn | |
| lint_without_cases | a library lint with no `before`/`after` pair | add them | warn | #12 | syn | |

---

#### Additions (brainstorm, round 2)

| Lint | Catches | Write instead | Tier | P | Engine | Status |
|---|---|---|---|---|---|---|
| pub_fn_takes_list_not_seq | `pub fn` taking read-only `List<T>` instead of generic `S: Seq` / `Indexed` | generalize parameter to `S: Seq` or `S: Indexed` | warn | #2 | ty | |
| pub_fn_missing_domain_generic | `pub fn` over a container hardcoded to `Int` index instead of domain-generic `D` | parameterize over domain `D` (`[D]T`) | warn | #3 | ty | |
| pub_struct_exposes_raw_index | `pub` field on a domain-keyed data structure exposing raw `Int` index instead of `D` | expose `D` position or keep raw index private | warn | #3 | ty | |
| library_lint_foreign_name | library-declared lint rule referencing a symbol owned by a different module (`fmt-lint.md` ownership rule) | move lint to the module that owns the name | warn | #13 | ty | |
| pub_cache_field_unsealed | `pub @cache` field directly writable by external callers | keep `@cache` fields private to the defining type | warn | #8 | ty | |
| pub_mut_fn_ambiguous_name | `pub fn` that mutates `self` in place with a noun/past-participle name (`sorted`, `reversed`) instead of imperative (`sort`, `reverse`) | follow `sort`/`sorted` naming convention (`std`) | warn | #13 | ty | |

---

## Additions: possible new practices and stdlib gaps

### Possible new practices

Three recurring lint groups above point to practices not directly named by `#1`–`#14` in `principles.md`:

- **P15. Let bounds prove safety and budgets** (extends `#6` Constraints once and `#10` Costs visible): declared statement constraints (`n: 1..=200_000`, `@sum(n) <= 200_000`) should drive compile-time proofs for overflow (`int_mul_overflow_at_bounds`, `int_shift_overflow_at_bounds`), division by zero (`int_div_possible_zero`), empty-range unwraps (`domain_empty_first_last`, `unwrap_on_possibly_empty`), TLE (`nested_domain_product_tle`, `n_log_n_inside_n_at_bounds`), and MLE (`memory_limit_at_bounds`). *Alternative*: fold explicitly into `#6` and `#10` rather than adding a new practice number.
- **P16. Interactive I/O is a typed protocol** (extends `#11` I/O): interactive problems fail disproportionately from missing flushes, read-before-flush deadlocks, and scattered `-1` exit checks. Framing interactive I/O around `Wire.ask` (`std.io`) rather than raw `print`/`read` catches `interactive_missing_flush`, `interactive_read_before_flush`, and `interactive_exit_on_minus_one`. *Alternative*: fold into `#11`.
- **P17. DP transitions separate read and write generations** (extends `#8` Mutation and `#9` Multitest): in-place DP direction bugs (`dp_in_place_wrong_direction`), stale `@memo` captures (`memo_mut_capture`), and `@memo` leakage across multitest cases (`memo_cache_leaks_across_tests`) all stem from mixing old and new DP state. *Alternative*: keep `dp_in_place_wrong_direction` under `#8` (mutation surprises).

---

### Stdlib gaps found

### Gaps surfaced directly by lint rewrites (`needs stdlib: X`)

1. **`std.seq.is_sorted` / `is_strictly_sorted`**: contestants frequently write adjacent-pair loops to check monotonicity; `std.seq` has `sorted()` and `windows(2)` but no `is_sorted(by:)` / `is_strictly_sorted(by:)` predicate.
2. **`std.seq.all_equal` / `has_dup`**: recurring 1-line contest checks currently require `xs.all(x => x == xs[0])` or `xs.sorted().deduped().len < xs.len`.
3. **`std.math.rem_euclid`**: when contestants need non-negative Euclidean remainder modulo a dynamic `m` without `Mod<P>` (e.g. negative coordinate wrapping), they write `((a % m) + m) % m`; a `rem_euclid(a, m)` helper eliminates the C++ idiom. (Not needed: `%` floors for a positive divisor, R01.5.2, so `a % m` is already non-negative.)
4. **`std.graph.read_edges` / `Graph.read`**: nearly every graph problem repeats `for _ in 0..m: let (u, v) = read<(V, V)>(); g.add(u, v)`; a typed edge-list reader would let `read_graph_edges_loop` autofix cleanly.

### Spec/implementation gaps in `std/GAPS.md` that affect lints

5. **`@cache` fields do not parse yet** (`GAPS.md` #1): blocks `cache_field_written_outside_cache`, `cache_field_observable_leak`, `pub_cache_field_unsealed`, and optimal `Dsu`/`Comb` implementations.
6. **`Wire` lacks stdout flush-before-read mode** (`GAPS.md` #6): needed for `interactive_missing_flush` and `interactive_read_before_flush` to autofix cleanly onto `Wire`.
7. **`read_all` lacks a public end-of-input test** (`GAPS.md` #5): limits autofixing manual `eof_loop` patterns that parse heterogeneously until EOF.
8. **Generic domain inference & slice source-key preservation** (`GAPS.md` #3, #8, #9): required for `slice_key_lost_indexing`, `pub_fn_missing_domain_generic`, and several `std.graph`/`std.tree`/`std.seq` rewrites to typecheck cleanly after autofix.
9. **Operation-value and generic `Mod<P>` inference** (`GAPS.md` #7, #10): needed for `manual_prefix_fold_monoid` and `manual_mod_try_inv` to resolve without explicit turbofish/type annotations.


## Counts

395 lints: 38 exist today, 357 are new. About two thirds are library rewrite rules (family S) that share a small set of loop recognizers, so the
real engine work is about a dozen recognizers (accumulate, search-and-break, push, table-fill,
prefix-chain, queue-BFS, heap-relax, midpoint-search, two-pointer, stack-monotone,
parent-pointer, square-and-multiply), not 357 separate analyses.

## Adding a lint

A new lint needs:
1. **Evidence**: at least one real program (corpus port, skill-eval output, user code) that
   contains the pattern. No speculative lints.
2. **A principle** it serves. If none fits, either the principle set is missing something
   (raise it) or the lint is a style preference (don't ship it).
3. **Cases**: `before`, `after`, and at least one `not-*` near miss, tested as above.
4. **A tier** with a reason; `fix` must preserve meaning, cost class and trap behaviour.
5. **An owner**: the module whose names it mentions, or `cpl-lint` for core lints.

Discovery loop: every corpus review and skill-eval run records, per line, "non-idiomatic, no
lint fired". Those lines are the queue for new lints; a pattern seen three times gets an entry.

## Open

1. **Library-declared rewrite rules.** Need a pattern language: a CP Lang snippet with holes,
   matched after loop-shape normalisation. Recommendation: start with the recognizer set above
   plus per-library templates (`recognizer = "accumulate", op = "+", becomes = "xs.sum()"`),
   and add free-form patterns later only if templates run out.
2. **`warn` versus `fix?` for the "by hand" structure lints.** Replacing a hand-rolled Dinic
   with `max_flow` changes the code a lot; I put most of them at `warn` with a docs link that
   shows the replacement.
3. **Bound-aware lints** (`quadratic_at_bounds`, `wide_array`, `deep_recursion`) are unique to
   CP Lang and probably the most valuable; they need the range analysis to expose loop trip
   counts.
4. **Default levels.** Proposal: `fix` and P0-correctness lints on by default; `fix?` on;
   `warn` structure lints on; library-authoring lints on only inside library files.
