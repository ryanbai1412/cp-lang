# cco24p1 friction

The old port built `adj: [V]List<(V, Int)>` by hand and ran its own heap Dijkstra with a
`c > cost[u]` staleness check. The port reads the routes as `[Route](u, v, w)`, builds
`Graph.undirected` and calls std `dijkstra` with every island as a source at `-coins[y]`;
the whole solution is the one line the editorial states.

## Dream

The existing `dream.cp` is the target; the port is the dream in today's syntax.

No gaps.

Measured: `cpl stress` agrees with the old port and with the accepted C++ on 30 cases each;
on three max shapes (N = M = 1e6: random, path, star) the port matches the C++ in 0.46-0.54 s
against its 0.69-0.76 s (the C++ needs a large stack).
