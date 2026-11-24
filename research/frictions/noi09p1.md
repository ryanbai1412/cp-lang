# Friction Report: noi09p1

## 1. Forced values and cycles on a std graph
- **Where:** `Graph.undirected(ends)` over positions as edges; `g[cur].find(a => at[a.e] == None)`.
- **What felt wrong:** the hand-built adjacency, degree array and `head` queue index are gone; the
  forced values are a worklist `while let u = forced.pop()`. Failure is `solve() -> ?[Pos]Val`
  returning `None`, not an `impossible` flag.
- **Goal it hurts:** none left.
- **Ideal code:** as written.
- **General fix:** a lint for `var head = 0; while head < xs.len: let u = xs[head]; head += 1`
  when order does not matter: `while let u = xs.pop()`.
- **Severity:** none.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC on permutation-derived (solvable) inputs, and
30/30 with the v0.2 port on random D (`gen.py SEED SIZE rand`, mostly "No Answer"; the AC C++
loops on some of those, which the judge never gives). Max (n = 1e4): 0.004 s vs C++ 0.005 s.
