# Friction report: noi03p4

## Dream

The port is the dream: `Rooted(...weighted)`, `diameter()` and `dist`.

## Hand-rolled walks become std tree queries

The v0.2 port read the edges one field at a time, filled an adjacency list by hand, and ran
three stack walks with `-1` as the unvisited sentinel ("the prototype's bfs has no edge
weights"). The rewrite reads typed `roads`, builds a weighted `Rooted` tree, takes the
diameter ends from `diameter()`, and maximises `min(dist(c, a), dist(c, b))`. Each dist is an
O(log n) LCA query, so O(n log n) overall.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. The max case runs in
0.04 s. Corpus tests pass.
