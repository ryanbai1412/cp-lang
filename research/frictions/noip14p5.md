# Friction Report: noip14p5

## 1. Adjacency lists built by hand
- **Where:** `var out`, `var back` filled from `edges`.
- **What felt wrong:** `Graph.directed(edges)` has no reversed view and its `only` filter does not feed `bfs`, so
  both directions and the safe subgraph are spelled over lists. On a fresh `cpl run`,
  `bfs(g.rev, from: [t])` is `error[E0203]: Graph<_, _> has no field or method `rev`` and
  `bfs(g.only(safe), from: [s])` is `error[E0606]: `Only<_, Graph<_, _>>` does not implement
  `Next` required by `bfs``.
- **Goal it hurts:** express.
- **Ideal code:** `let g = Graph.directed(edges)`; `bfs(g.rev, from: [t])`; `bfs(g.only(safe), from: [s])`.
- **General fix:** vertex-filtered graph views for BFS.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and AC 5205459 on gen.py inputs; max (n = 1e4, m = 2e5) 0.021 s vs C++ 0.025 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let g = Graph.directed(edges)
let reaches = bfs(g.rev, from: [t])
let safe = [V] u => all(reaches[v] != None for v in g.out(u))
print(if safe[s]: bfs(g.only(safe), from: [s])[t] ?? -1 else: -1)
```
