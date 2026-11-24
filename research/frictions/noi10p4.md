# Friction Report: noi10p4

## 1. `reachable` is O(n + m) per call
- **Where:** `for step in dfs(back, [u]): if let Visit.Enter(p, _) = step: first[p] = true`.
- **What felt wrong:** the obvious `for p in reachable(back, [u]): first[p] = true` rebuilds the
  adjacency of the whole graph on every call: 0.62 s on the max case against 0.05 s with `dfs`.
- **Goal it hurts:** the obvious way is the fast way.
- **Ideal code:** `for p in reachable(back, [u]): first[p] = true`.
- **General fix:** `reachable` walks `g.out` lazily (sent to Content stdlib).
- **Severity:** medium.

## Measurements
The schedule is not unique, so `checker.py` checks line 1 against k and the rules and line 2
exactly. Stress: 30/30 agree with the v0.2 port and with AC; max (n = 2000, m = 1e4) passes the
checker, 0.08 s vs C++ 0.06 s.
