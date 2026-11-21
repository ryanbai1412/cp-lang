# Friction: `ucup4s25c`

## `Map<(Int, Int), Int>` hashing and per-vertex `bfs` neighbor list allocations (2.1x slowdown)

- **Severity:** medium
- **Goal it hurts:** performance
- **Measured:** `49.88 ms` vs `23.93 ms` (`2.08x`) on `gen size=25000` (`research/scoreboard.json`).
- **Cause (stdlib cost):** `Map<(Int, Int), Int>` lowers to `std::unordered_map` (chained node allocations on every pair insert), and `bfs(u => [v for v in adj[u] if comp[v] == comp[u]], roots)` allocates a fresh `std::vector` for every visited vertex.
- **Fix:** support a flat open-addressed hash map in `std` and lazy/filtered neighbor iterators in `bfs` without materializing a `List` per vertex.
