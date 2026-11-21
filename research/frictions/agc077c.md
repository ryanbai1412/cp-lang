# agc077c friction

## No complement-graph connected components in `std.graph`

- **Goal it hurts:** express
- **Severity:** medium
- **Area:** graphs
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/agc077c.cp:37` (`for comp in complement_components(forbidden):`)
- **Actual:** `proto/ports/ecnerwala/agc077c.cp:32-54` sorts each adjacency list and hand-rolls the O((n + m) log n) complement-graph DFS using `unvisited: Set<V>` + `first_ge` + a sorted-adjacency pointer because `std.graph` has `components` only for explicit adjacency, not for the complement of a sparse graph.
