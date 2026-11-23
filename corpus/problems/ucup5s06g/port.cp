// Source: The 5th Universal Cup, Stage 6 (Grand Prix of Hyakumanben), problem G (Source to
//   Sink), qoj.ac/contest/4617/problem/23675
// Problem: a simple directed graph, n <= 1e5 and m <= 2e5 (2-cycles allowed). One operation
//   picks a vertex of indegree 0 and reverses every edge at it. Print the most operations
//   possible, or -1 if they can go on forever. Multitest, sums of n <= 1e5 and m <= 2e5.
// Solution: with x_u the times u is flipped, a vertex on a cycle is never flipped, an edge
//   u -> v forces x_v <= x_u <= x_v + 1, and every such x is reachable; a weak component
//   with no cycle goes on forever. peel removes the vertices not reachable from a cycle; a
//   bfs01 from the rest (edge forward cost 0, backward cost 1) gives the largest x, and
//   any vertex it misses means -1. O(n + m).

use std.graph.{bfs01}

@!multitest
let (n, m): (2..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()

var peeled: [V]Bool = false
for v in peel(Graph.directed(edges)): peeled[v] = true

domain Arc = 0..2 * m
let arcs: [Arc](u: V, v: V, w: Int) = [Arc] k =>
    let e = edges[E(Int(k) / 2)]
    if Int(k) % 2 == 0: (u: e.u, v: e.v, w: 0) else: (u: e.v, v: e.u, w: 1)
let dist = bfs01(Graph.directed(arcs), from: [v for v in V if not peeled[v]])
print(if V.all(v => dist.reached(v)): sum(dist[v]! for v in V) else: -1)
