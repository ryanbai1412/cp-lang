# graph.cp doc notes

One bullet per contradiction or silent choice found while bringing
`spec/std/graph.cp` to the doc standard.

## Contradictions

- `Graph.dfs_tree` / `DfsTree`: decision 147 says `g.dfs_tree(root)` returns `(tree: Rooted, tree_edge: [E]Bool)` and `DfsTree` goes; the stub still has `DfsTree`. Documented the stub as is.
- `dfs` / `Visit`: decisions 146-147 and graphs-trees.md agree on `step.prune()`, `Enter(v, via:)`, `Exit(v, via:)`, `Again(from, via)` and a shared `seen:`; the stub has `Visit: Enter(v), Exit(v)` and "`continue` after `Enter` still descends". Documented the stub.
- `Adj`, `directed_of_pairs`, `undirected_of_pairs`, `*_of_triples`, `adjacency`: graphs-trees.md names `Graph` and `[V]List` as the representations and decision 146 lists per-shape builders as the reverse option; the stub keeps them. Documented them as thin builders.
- `directed_of_triples` / `undirected_of_triples` -> `Adj<V, (V, W)>`: a positional `(V, W)` is neither a vertex nor a record with `to`/`w`, so it is not `Target`/`Weighted` and `Adj.matrix`, `next`, `transpose` cannot apply. Documented as if the tuple reads as `(to, w)`.
- `biconnected_components`, `edge_blocks`, `articulation_points`: graphs-trees.md uses `blocks` and `cut_vertices`. Names left.
- `condense`: graphs-trees.md says it returns `Graph<C, _>`; the stub returns `[C]List<(to: C)>`. Documented the stub.
- `dial` cost: graphs-trees.md says O(n * max_w + m), the stub O(n + m + W). Wrote O(n + m + max_w + d_max) with `d_max <= (n - 1) * max_w`, which both bound.
- `kruskal` `by:`: typed `E -> K` but defaults to `e => e.w`, reading a field off an edge key. Example uses `by: e => desc(es[e].w)`.
- `Only<V, G>(g, keep)` stores no `from`, though `only(from:)` needs it. Left.
- `Paths.path` had two cost lines (`O(path length)` twice). Kept one.

## Choices

- Examples build graphs as `[V]List<V>` literals and `[E](u: V, v: V, ..)` literals with integer literals as positions (04-indices), so they need no input parsing.
- `Graph.directed` / `undirected`: each vertex's arcs in edge order (stable counting sort); an undirected loop's two arcs adjacent. Needed for deterministic DFS and first-found answers.
- `Graph.at`: the slice keeps arc-list keys (`start[u]..start[u + 1]`), as every slice keeps its source's keys.
- `Graph.matrix` / `Adj.matrix`: parallel arcs keep the least weight; the diagonal is the least loop weight or `None` (`floyd_warshall` sets the one itself).
- `Graph.spanning_tree`: BFS from `V.first`; traps when not connected or `V` is empty (it returns a single `Rooted`).
- `Graph.hops`: traps when some pair is unreachable (`[V, V]Int` has no absent value).
- `Graph.edges`: increasing `e`, ends as given; the "source arc per edge" in the stub is read as an `[E]` table.
- `Graph.transpose` / `Adj.transpose`: new arcs by original source in vertex order, then input order (what a counting-sort build gives).
- `peel` with `ready:`: queue starts with ready vertices in vertex order; removing `u` calls `on_remove(u, a)` per arc then queues `a.to` if ready and not yet queued; each vertex removed once. Matches the dream ports that use it.
- `topo` / `peel` ties: FIFO, in-degree-zero vertices in vertex order (the reference implementation).
- `scc`: Tarjan, roots in vertex order, components in reverse completion order; ties among unrelated components follow from that.
- `components`, `two_edge_components`: `C` numbered by smallest vertex. On a directed graph `components` is defined as the forward search the reference implementation runs.
- `two_edge_components`, `bridges`, `edge_blocks`: undirectedness is checked in checked builds (decision 146 calls it a debug-checked precondition).
- `biconnected_components`: blocks deepest first as the DFS closes them, vertices as they leave the stack with the closing vertex last; isolated vertices are in no block (reference implementation).
- `block_cut_tree`: `B` follows `biconnected_components`' order.
- `edge_blocks`: loops each form their own block, numbered after the DFS blocks in edge order.
- `two_color` / `odd_cycle`: BFS from each uncoloured vertex in vertex order, smallest vertex `false`; the odd cycle is the BFS-tree cycle through the first conflicting arc, from the LCA.
- `Paths` back pointers: the arc that first gave each vertex its final distance (only strict improvements replace it); `path(t)` follows them, `[t]` for a source.
- `Paths.tight`: weight is `a.w` for `Weighted` arcs, `1` otherwise.
- `bfs` `enter:` also filters the sources; `until: t` stops when `t` gets its distance (discovery). Weighted searches stop when `t` is settled.
- `bfs01`, `dial`, `dijkstra`: a bad weight traps when it is read; `dijkstra` keeps the smaller start for a repeated source.
- `fixed_cost_paths`: equal candidates go to the earlier move; a negative cost traps.
- `bellman_ford`: rounds over reached vertices in vertex order; negative cycles no source reaches are ignored.
- `johnson`: `None` on any negative cycle, reachable or not (the virtual source reaches every vertex).
- `find_cycle`: the first cycle the DFS closes, starting at the vertex the closing arc enters (reference implementation).
- `euler_walk`: the walk is Hierholzer's on `Graph.directed/undirected(edges)` arc order; no edges gives a walk at `V.first`, so an empty `V` traps.
- `TwoSat.solve`: implication graph node `2i` false, `2i + 1` true; clause `a or b` adds `not a -> b` then `not b -> a`; a variable is true when its true node's `scc` component is later. Makes the assignment a function of the clause sequence.
- `reachable`: discovery order is `dfs`'s preorder.
- `k_shortest`: the empty walk counts when `to` is a source; negative weights trap (Eppstein needs them non-negative).
- `negative_cycle`: Bellman-Ford from every vertex at 0, the first vertex relaxed in round `n` walked back `n` times; arcs from there.
- `min_mean_cycle`: among least-mean cycles the shortest, so `(sum, len)` is unique.
- `girth` undirected: needs symmetric arcs and at least three edges, so loops and the way back are excluded.
- `count_paths`: a repeated source counts once; `None` on a cycle anywhere, not only reachable ones.
- `closure` (bitset): reflexive, every vertex reaches itself (like `reachable` and the zero diagonal).
- `scc_times`: `t[e]` is the first index `i >= e` at which `e`'s ends share a component.
- `triangles`: rank-oriented enumeration with a fixed order (rank `(degree, vertex)`), so output order is defined.
- `max_clique`: the lexicographically least maximum clique, ascending.
- `de_bruijn`: the lexicographically least sequence (Lyndon concatenation); `k < 1` or `n < 1` traps.
- `kruskal`: stable sort by `by:`, ties in edge key order; loops never taken.
- `prim`: root `V.first`; equally cheap candidates join smallest vertex first; reads `w[u, v]` with `u` in the tree.
- `boruvka`: labels `0..c` by smallest vertex; rounds stop at one component or no progress; a wrong-length answer from `cheapest` traps.
- `arborescence`: left which minimum arborescence `parent` describes open when several exist; only `total` is unique. Open item.
- `spanning_forest`: BFS from each unvisited vertex in vertex order.
- `Functional.after`: negative `k` traps. `cycles`: each from its smallest vertex, ordered by it.
- `IncrementalBridges.add`: a loop changes nothing; a parallel edge makes its ends two-edge-connected.
- `DfsTree.depth` / `path` / `orient_down`: trap off the tree; `parent` gives `None` there.
- `DfsTree.euler_tour`: children with equal `by` keep vertex order.
- `Only.out`: none when `u` is dropped, otherwise arcs whose `to` is kept.
