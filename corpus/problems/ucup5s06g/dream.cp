// Universal Cup 5 Stage 6 (Hyakumanben) G — Minimum Reversals from Cycles
//
// Iterative in-degree-0 peeling (`peel`) removes every vertex whose ancestors
// form a DAG (every directed path into a peeled vertex starts at a source, so
// it cannot be reached from any cycle without reversing at least one edge).
// Every unpeeled vertex keeps an incoming edge from another unpeeled vertex,
// so it lies on or below a directed cycle and costs 0 (`dist = 0`).
// A 0-1 BFS from all unpeeled vertices (forward edge cost 0, reversed edge
// cost 1) finds the minimum reversals for every vertex; print the sum or `-1`
// if any vertex is unreachable.

use std.graph.{bfs01}

@!multitest
let (n, m): (1..=500_000, 0..=500_000) = read()
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
print(if all(dist.reached(v) for v in V): sum(dist[v]! for v in V) else: -1)
