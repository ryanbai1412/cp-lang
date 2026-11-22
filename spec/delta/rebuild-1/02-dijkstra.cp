// cpl 0.3.0
// Problem: single-source shortest paths (CSES 1671 style, -1 if unreachable).
// Input: n m, then m lines "u v w" (directed, 1 <= w <= 1e9), vertices 1..n.
// Output: dist(1, v) for v = 1..n on one line, -1 when unreachable.
// Guesses: H-COUNTED-IS-DOMAIN (`[m]` satisfies `E: Domain`),
// H-DIJKSTRA-W (the weight type W of `w: 1..=1e9` and the literal 0 source).

let n = read<1..=2e5>()
let m = read<0..=2e5>()
domain V = 1..=n
let g = Graph.directed(read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p.dist, none: -1)
