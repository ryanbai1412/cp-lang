# Friction report: neerc10k

## Dream

```cp
let order = bfs(g, from: [start]).order          // vertices in visit order
for u in order.rev: colour[u] = (1..=k).find(c => c not in g.next(u).map(v => colour[v]))
```

## Hand BFS becomes `Graph.undirected` + `bfs`

The v0.2 port read the edges one at a time into a hand adjacency list, ran a queue BFS with a
`seen` table, and coloured with a `0` "uncoloured" sentinel and a `List<Bool>` of size k per
vertex. The rewrite reads typed `edges`, builds `Graph.undirected`, takes `bfs` distances and
walks `V.sort(by: dist)` in reverse. Colours are `?Int`, and the first free colour is a
`find` over a `Set` of the neighbours' colours. `Paths` has no visit order, so the walk sorts
by distance, O(n log n).

## Verification

`gen.py` (connected simple graphs on an odd n, max n = 9999, m = 1e5) and `checker.py` (k is
the least odd number >= the max degree, and the colouring is proper) are new. 30 random cases
are accepted against the v0.2 port and 30 against the accepted C++. Corpus tests pass.
