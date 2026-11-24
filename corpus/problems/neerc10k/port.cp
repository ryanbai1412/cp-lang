// Source: NEERC 2010 K (K-Graph Oddity), https://dmoj.ca/problem/neerc10k
// Problem: a connected graph, odd n <= 9999 vertices, m <= 1e5 edges. Print the least odd
//   k >= the maximum degree and a proper colouring of the vertices with colours 1..=k.
// Solution: n and k odd means not every degree is k, so start a BFS at a vertex of degree
//   < k. Colour greedily in reverse BFS order: every other vertex still has an uncoloured BFS
//   parent, so at most k - 1 colours are taken around it. O(n log n + m).
// original: corpus/submissions/5325567.cpp
// Bounds: n: 1..=9999; m: 0..=1e5; edges.u, edges.v: 1..=n
let (n, m): (1..=9999, 0..=1e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

let k = max(g.deg(v) for v in V)! / 2 * 2 + 1  // the least odd k >= every degree
let dist = bfs(g, from: [V.find(v => g.deg(v) < k)!])
var colour: [V]?Int = None
for u in V.sort(by: v => dist[v]!).rev:
    let taken = Set(colour[v] for v in g.next(u))
    colour[u] = (1..=k).find(c => Some(c) not in taken)
print(k)
for v in V: print(colour[v]!)
