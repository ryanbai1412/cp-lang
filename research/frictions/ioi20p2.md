# Friction report: ioi20p2

## Dream

```cp
@grader fn construct(p: [V, V]0..=3) -> ?Graph<V>
let tree = V.classes(by: (u, v) => p[u, v] == 1)       // each class named by its first
let comp = V.classes(by: (u, v) => p[u, v] > 0)
let rings = tree.names.group_by(comp)
if any(r.len == 2 for r in rings) or any(p[i, j] != expect(i, j) for i in V for j in V):
    return None
Graph.undirected(tree.edges_to_name ++ rings.flat_map(r => r.cycle_edges()))
```

## The grader lives in the solution

`construct` returns 0 or 1 and hands the bridges to the grader's `build`. The port prints
`0`, or `1` and the matrix, itself. A `@grader` entry that takes the matrix and serialises a
returned `?Graph<V>` as an adjacency matrix would remove the output code.

## Classes by a relation are spelled by hand

`tree` and `comp` are `V.find(u => p[v, u] == 1) ?? v`: each tower named by the first
tower it is equivalent to. The `?? v` covers the diagonal, which the statement fixes at 1
but the type `0..=3` does not. A std `classes(by:)` (or a `Dsu` built from a relation) is
the one spelling every "p says these are together" problem wants.

## An undirected edge list to an adjacency matrix

The bridges are collected as pairs, then written into a `[V, V]0..=1` once for each order.
`Graph.undirected(bridges).matrix()` (or printing a graph as a 0/1 matrix) would say it.

## Verification

30 + 10 random cases agree with the v0.2 port, 30 with the accepted C++ (`orig.cpp`, the AC
submission plus a stdin `main` standing in for the grader), the corpus tests pass, and the
max case (n = 1000) is byte-identical to the C++ and runs in 0.04 s.
