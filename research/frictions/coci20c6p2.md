# coci20c6p2 frictions

## Dream

```cp
let g = Graph.directed(arcs, weights: times)
for (a, b) in asks: print(g.dijkstra(from: a)[b] ?? "Roger")
```

## Names as vertices

Words become vertices through an `Interner` while reading, so the graph is built as a
table of lists; `print` of `?Int` with a fallback string needs an explicit `if let`.
