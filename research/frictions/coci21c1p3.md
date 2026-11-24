# Friction report: coci21c1p3

Rethought. The v0.2 port built the tree's adjacency by hand during the DSU pass, with `V.first!` placeholders for the extra edge. It found children with a head-index BFS and ran a recursive five-argument `@memo`. The v0.3 port reads the edges typed and records the extra edge as `?E`. It roots the tree at one end of that edge, skipping the edge in the `Next` it gives `Rooted`. Then it fills a `[V, Bool, Bool]?Int` table bottom-up over `tree.order.rev`, so there is no recursion.

## Gap

`Graph.undirected` takes only a `[_]_` array, so the tree without one edge cannot be `Graph.undirected([edges[e] for e in E if e != cut])`. It goes through `v => [x.to for x in g.out(v) if x.e != cut]` instead.

## Measurements

gen.py --max: port 0.22 s, accepted C++ 0.24 s, same output.
