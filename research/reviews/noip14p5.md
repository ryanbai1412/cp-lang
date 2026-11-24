# noip14p5

Score: 3/5
Dream: 4/5

Cold review: reverse BFS from t, the safe test, then BFS on the safe subgraph (lines 25-28), the header's two passes, over hand-built adjacency lists.

## Checklist failures

- R3: `out` and `back` are pushed from `edges` by hand (lines 19-23).

## Owed rewrites

- None the language allows yet: `Graph.directed` has no `rev`, and its `only` view does not feed `bfs`.

## Language gaps

- A reversed and a vertex-filtered graph view for `bfs`: in research/frictions/noip14p5.md, now with the `cpl run` errors; the friction's "`Graph.directed(edges)` gives both directions" is corrected.

## Dream notes

- None.
