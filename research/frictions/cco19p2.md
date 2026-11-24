# Friction report: cco19p2

## Dream

```cp
domain Cell = (Row, Col)
let (domain Shape, shape) = components([Cell] p => [q for q in p.adj4 if g[q] == '#' == g[p]])
let fall = dijkstra(Graph.directed(arcs), from: floor)
```

## Changes

The v0.2 port flattened the grid to `0..n*m` with hand row/column arithmetic, a hand
union-find and a hand Dijkstra with an `INF` sentinel; it no longer passed `cpl check`
(E0904 on the 1e6 x 1e6 product). With the joint bound `cells: 1..=1e6 = n * m` the grid is
`[Row, Col]` over `domain Cell = (Row, Col)`; std `Dsu<Cell>` joins touching '#' cells via
`adj4`, objects are named by their root cell, and std `dijkstra` runs multi-source from each
column's lowest '#' (seeded with its height), so there is no floor node.

## Remaining friction

- `components` rejects a table over a product domain (`[Cell]List<Cell>` is not seen as a
  graph on `Cell`), so the port uses `Dsu` instead of the dream's `components`.
- `Graph.directed` needs a typed array, so the arc list is copied into `[Rest]`.

## Verification

30 random cases agree with the v0.2 snapshot and 30 with the accepted C++. Corpus tests pass;
the 1000 x 1000 max runs in 0.10 s. On 1e6 x 1 it matches the accepted C++; on 1 x 1e6 the
accepted C++ overflows its stack and the port prints the grid unchanged, as it should.
