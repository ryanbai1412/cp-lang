# agc077c friction

## `components(complement(g))` is quadratic, so the complement BFS is hand-written

- **Where:** `corpus/problems/agc077c/port.cp`, `complement_components()`: a BFS that keeps
  `unvisited: List<V>`, stamps `near[v] = u` for the neighbours of `u` and
  `unvisited.partition(v => near[v] != Some(u))` to split off the vertices `u` reaches.
- **What felt wrong:** `std.graph` has `complement(g)` and documents that `components` over a
  `Complement` runs in O(|V| + m), but `components` just calls `out(u)`, and
  `Complement.out(u)` scans all of `V` (O(|V| + deg u)), so the documented one-liner is
  O(n^2): 28.6 s against orig.cpp's 0.10 s on n = m = 2e5, K = 3 (one big complement
  component). The port has to write the standard unvisited-list BFS itself.
- **Goal it hurts:** performance, express (the obvious code is the slow code).
- **Ideal code:** `let (domain C, comp) = components(complement(Graph.undirected(edges)))`,
  then per component `sz` and `odd` (the dream's `complement_components(forbidden)`).
- **General fix:** give `Complement` searches the unvisited-set walk the doc promises: a
  `components`/`bfs`/`reachable` path for `Complement` (or a `Next` hook that lets a search
  hand its unvisited set to the graph, so `out` yields only unvisited non-neighbours),
  with a perf test on a sparse n = 2e5 graph. Until then the doc's cost line is wrong.
- **Severity:** medium (a cold reader would write the one-liner and TLE).
- **Status:** open; repro `use std.graph.{complement, components}` +
  `components(complement(Graph.undirected(edges)))` on n = m = 2e5 (stdlib worklog).

## Measurements

Max inputs, best of 5, port built with `cpl build --judge codeforces` and g++ -O2; orig.cpp
compiled as C++17 with its deducing-`this` lambda changed to `std::function` (g++ 11 has no
C++23):

| input | orig.cpp | port |
| --- | --- | --- |
| n = m = 2e5 random missing pairs, K = 3 | 0.099 s | 0.033 s |
| n = 2e5, missing pairs a 632-clique, K odd | 0.079 s | 0.020 s |
| n = 633, all missing pairs, K = 5 | 0.020 s | 0.010 s |
