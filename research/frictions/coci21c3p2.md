# Friction report: coci21c3p2

Rethought. The v0.2 port pushed each edge into an adjacency list by hand, then ran a recursive `down(u, parent)` that wrote into the globals `seen` and `longest` and reset `longest` per tree. The v0.3 port reads the edges in one go into a `Graph.undirected`, roots every tree with `Rooted.forest`, and sweeps `forest.order.rev` (children before parents) to keep `deepest`/`topped` per vertex. It sums each tree's best through `forest.root_of`. It has no recursion and no mutable globals, and n and m are typed.

No language gap.

## Measurements

gen.py --max: port 0.01 s, accepted C++ 0.03 s, same output.
