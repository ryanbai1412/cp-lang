# Friction report: egoi23p4

Rethought from spec.cp. The v0.2 port kept both matrices as `[V, V]Int` with `1_000_000_000` and `-1` on the diagonal as sentinels, copied a Floyd-Warshall closure twice (`matches_mn`, `matches_mx`) and ran Kruskal by hand over `(w, u, v)` tuples with Dsu in both directions. The v0.3 port reads each lower triangle straight into a `[V, V]?Int` in bike-lane terms, so the diagonal is `None`. It checks closure with `closure(over: Semiring(max, min))` and `Semiring(min, max)` and keeps the union of `kruskal(edges)` and `kruskal(edges, by: e => desc(e.w))`. It checks only the offered streets' closures: if they close back to a matrix, that matrix is closed, so checking the full matrices too is redundant.

No language gap.

## Measurements

gen.py --max (n = 500, 2023 streets): port 0.20 s / 15 MB (0.37 s on the same machine before the two redundant full-matrix closures were dropped; first recorded as 0.50 s / 19 MB), v0.2 port 0.39 s, accepted C++ 0.37 s. checker.py accepts all three, with 997, 998 and 998 streets.
