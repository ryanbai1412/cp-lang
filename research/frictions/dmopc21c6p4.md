# Friction report: dmopc21c6p4

No gap. The search is std `bfs` over the product domain `State = (at: City, met_c: Bool, met_d: Bool)` on `Graph.undirected(roads)`. The best end state is an optional record, and the program has no `solve()` wrapper for an early exit: the C-D road test is the first branch. Many colourings are optimal, so `checker.py` checks K, that no one-colour C-D path exists, and the shortest one-colour A-B path length.

## Measurements

gen.py --max (n = m = 2e5): port 0.24 s, accepted C++ 0.12 s.
