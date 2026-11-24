# Friction Report: ccc23s4

## A growing graph is a hand-built adjacency list
- **Where:** `var kept: [V]List<(to: V, w: Int)>` and two `push`es per kept road.
- **What felt wrong:** `Graph<V, E>.undirected(edges)` is built once from a fixed edge array; a
  graph that gains edges between searches has no std form, so the port writes both arcs.
- **Goal it hurts:** brevity.
- **Ideal code:** `var kept = Graph<V, E>.undirected(); kept.add(e, roads[e])`.
- **General fix:** an incremental `Graph.add` (edges keep their `E` ids for paths).
- **Severity:** low

## "Within distance" reads an optional by hand
- **Where:** `if let d = dijkstra(...)[v]: d <= len else: false`.
- **What felt wrong:** an unreachable vertex is `None`; comparing reads as a three-line match.
- **Goal it hurts:** brevity.
- **Ideal code:** `dijkstra(kept, from: [(u, 0)], until: v)[v].is_some_and(d => d <= len)`.
- **General fix:** `is_some_and` / `is_none_or` on optionals.
- **Severity:** low

## Dream

```cp
let (n, m): (1..=2000, 1..=2000) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](u: V, v: V, len: 0..=1e9, cost: 1..=1e9) = read()
var kept = Graph<V, E>.undirected()
print(sum(roads[e].cost for e in sorted(E, by: e => (roads[e].len, roads[e].cost))
    if not dijkstra(kept, from: [(roads[e].u, 0)], until: roads[e].v)[roads[e].v].is_some_and(d => d <= roads[e].len)
    and kept.add(e, roads[e])))
```

## Measurements

- Tokens: port 225, original 891
- Max input (`gen.py --max`): port 0.01 s, 3456 KB, original 0.12 s, 3712 KB
- Emitted C++: 18369 bytes
