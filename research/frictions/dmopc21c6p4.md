# Friction report: dmopc21c6p4

No gap. The search is two std `bfs` runs from A on `Graph.undirected(roads)`, one never entering D and one never entering C. The better one is an optional record, and the program has no `solve()` wrapper for an early exit: the C-D road test is the first branch. Many colourings are optimal, so `checker.py` checks K, that no one-colour C-D path exists, and the shortest one-colour A-B path length.

## Dream

The port as written. Two std `bfs` runs from A, one never entering D and one never entering C; the shorter decides the colour of every other road.

## Measurements

gen.py --max (n = m = 2e5): port 0.24 s, accepted C++ 0.12 s.
