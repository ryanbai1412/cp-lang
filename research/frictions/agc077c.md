# agc077c friction

## `components(complement(g))` was quadratic, so the complement BFS was hand-written

- **Where:** `corpus/problems/agc077c/port.cp` used to carry `complement_components()`, an
  unvisited-list BFS, because `components` over a `Complement` called `Complement.out(u)`,
  which scans all of `V`: 28.6 s against orig.cpp's 0.10 s on n = m = 2e5, K = 3.
- **Goal it hurt:** performance, express (the obvious code was the slow code).
- **Fix (landed with this port):** `std.graph` gains `ReachesUnvisited<V>`; `Complement`
  implements it by splitting the search's unvisited list into the vertices `u` reaches and the
  rest, and `components` and `reachable` take that path when `G: ReachesUnvisited`, so both
  are O(|V| + m) over a `Complement` as the doc says. Test:
  `complement_searches_match_the_explicit_complement` in `std/graph.cp`. The port is now
  `components(complement(Graph.undirected(edges)))`.
- **Still open:** `bfs` and `dfs` over a `Complement` still walk `out(u)`, O(|V|^2); their
  event order needs `out(u)` order, so they keep it (documented on `Complement`). The trait is
  module-private; a user graph with a cheap unvisited split cannot opt in yet.
- **Status:** fixed for `components` and `reachable`.

## Measurements

Max inputs, best of 5, port built with `cpl build --judge codeforces` and g++ -O2; orig.cpp
compiled as C++17 with its deducing-`this` lambda changed to `std::function` (g++ 11 has no
C++23), stack unlimited:

| input | orig.cpp | port |
| --- | --- | --- |
| n = m = 2e5 random missing pairs, K = 3 | 0.095 s | 0.033 s |
| n = 2e5, missing pairs a 632-clique, K = 5 | 0.078 s | 0.022 s |
| n = 2e5, missing pairs a star, K = 7 | 0.092 s | 0.030 s |
