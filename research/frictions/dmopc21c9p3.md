# Friction report: dmopc21c9p3

No gap. The v0.2 port built adjacency lists by hand and ran its own queue BFS into a `[V][V]Int` filled with `-1`. Now it is `Graph.undirected(routes)` plus one std `bfs` per station, stored in a `[Station, Station]0..n` table (the graph is connected, so every distance is set), and `yes(fits)`.

## Measurements

gen.py --max (n = m = 3000, q = 1e6): port 0.37 s, accepted C++ 0.73 s.
