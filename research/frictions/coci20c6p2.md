# coci20c6p2 frictions

## Dream

```cp
let g = Graph.directed(arcs, weights: times)
for (a, b) in asks: print(g.dijkstra(from: a)[b] ?? "Roger")
```

## Names as vertices

Words become vertices through an `Interner` while reading, so the graph is built as a
table of lists; `print` of `?Int` with a fallback string needs an explicit `if let`.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let d: ?Int = None
print(d ?? "Roger")
```

```text
error[E0302]: expected `Int`, found `Str`
  --> probe.cp:2:12
2 | print(d ?? "Roger")
  |            ^^^^^^^
```

**Status:** open.
