// Source: The 4th Universal Cup, Stage 25 (Grand Prix of Zagreb), problem C (Crooked Cycles),
//   qoj.ac/contest/3773/problem/18403
// Problem: m unordered triples of distinct numbers (m, values <= 2e5, sum of m <= 2e5,
//   t <= 2e4 tests). A cycle v_1..v_k has every three cyclically consecutive numbers form a liked
//   triple; it is crooked if 3 does not divide k. Print whether a crooked cycle exists.
// Solution: a cycle is a closed walk on ordered pairs, each triple's 6 orders giving edges
//   (x, y) -> (y, z). Intern the pairs, take SCCs, and BFS each SCC from one root inside it;
//   an SCC has a cycle of length not divisible by 3 iff some edge inside it breaks
//   dist[v] = dist[u] + 1 (mod 3). O(m) plus hashing.

use std.marks.{Interner}

@!multitest
let m: 1..=200_000 = read()
var verts = Interner<(Int, Int)>()
var raw_edges: List<(Int, Int)> = []

for _ in 0..m:
    let (a, b, c): (1..=200_000, 1..=200_000, 1..=200_000) = read()
    for (x, y, z) in [(a, b, c), (a, c, b), (b, a, c), (b, c, a), (c, a, b), (c, b, a)]:
        raw_edges.push((verts.id((x, y)), verts.id((y, z))))

domain V = verts.len
let edges = [(V(u), V(v)) for (u, v) in raw_edges]
let adj = Graph.directed(edges)
let (domain C, comp) = scc(adj)
var rep: [C]?V = None
for u in V: rep[comp[u]] ??= u
let roots = [u for c in C if let u = rep[c]]
let dist = bfs(u => [v for v in adj[u] if comp[v] == comp[u]], roots)

let bad = edges.any((u, v) => comp[u] == comp[v] and dist[v]! % 3 != (dist[u]! + 1) % 3)
print(if bad: "yes" else: "no")
