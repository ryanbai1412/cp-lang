# Friction Report: cco16p3

## No adjacency test on `Graph`
- **Where:** `(0..k).all(i => g.out(cycle[i]).any(a => a.to == cycle[(i + 1) % k]))`.
- **What felt wrong:** "is uv an edge" walks u's arcs. A `Set<(V, V)>` of the arcs would be the
  fix, but neither `Set.of(list)` nor `let s: Set<(V, V)> = [... for ...]` nor `.collect()`
  builds one: `error[E0302]: expected Set<(V_0, V_0)>, found List<(V_0, V_0)>`.
- **Goal it hurts:** express, performance.
- **Ideal code:** `g.has_edge(cycle[i], cycle[(i + 1) % k])`, or `Set.of(xs)` / a set comprehension.
- **General fix:** `Graph.has_edge(u, v)` (sorted CSR rows make it O(log deg)) and one
  list-to-`Set` spelling (`Set.of`, as `SortedSet.of` and `Counter.of` already have).
- **Severity:** low

## A block's vertices are rebuilt from its edges
- **Where:** `let members = [B] b => [x for e in block_edges[b] for x in [edges[e].u, edges[e].v]].distinct()`.
- **What felt wrong:** `blocks(g)` partitions edges only; every block-shaped argument (cycle,
  theta, long cycle) wants each block's vertex and edge count, so each port re-derives them.
- **Goal it hurts:** express.
- **Ideal code:** `let bs = g.blocks()` with `bs.vertices(b)` and `bs.edges(b)`.
- **General fix:** return the block-cut structure (`blocks` + `cut_vertices` + vertex lists) as one
  std value, and put `blocks` in the prelude beside `bridges`.
- **Severity:** low

## Dream

```cp
let (pattern, graphs) = read<(1..=5, 1..=1000)>()
repeat graphs:
    let g = read<Graph.undirected(V = [1..=1e5], E = [0..=1e5])>()
    let bs = g.blocks()
    let exits = (cycle: List<V>) => cycle.count(u => g.out(u).any(v => v not in cycle)) >= 2 and
        cycle.flat_map(u => g.out(u)).filter(v => v not in cycle).distinct().len >= 2
    print(yes(match pattern:
        1 => bs.any(b => b.edges > b.vertices.len)
        2 => bs.any(b => b.edges >= 3)
        3 => bs.any(b => b.vertices.len >= 4)
        4 => g.vertices.any(v => g.deg(v) >= 3)
        _ => bs.any(b => if b.vertices.len > 5: b.vertices.count(v => g.deg(v) >= 3) >= 2
            else: b.vertices.cycles().any(exits))))
```

The old port no longer passed `cpl check` (E0603, a lent element stored in a frame stack), so
it was verified against the accepted C++ only: 300 `gen.py` cases at sizes 10, 40 and 1000.

## Measurements

- Tokens: port 507, original 3729
- Max input (`gen.py --max`): port 0.06 s, 22360 KB, original 0.02 s, 18688 KB
- Emitted C++: 32204 bytes
  (Both under the 0.1 s floor of `corpus/generators.py`.)
