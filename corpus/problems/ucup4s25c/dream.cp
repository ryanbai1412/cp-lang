// UCup 4 Stage 25 (Zagreb) C — Non-3-Divisible Directed Cycle in Pair Graph
//
// Each input triple `(a, b, c)` consists of three distinct integers in
// `1..=10^9`, so its 6 permutations yield directed transitions
// `(x, y) -> (y, z)` between ordered pairs with `x != y` and `y != z`.
//
// Intern all seen ordered pairs with `std.marks.Interner<(Int, Int)>`, compute
// the graph's SCCs, pick the first vertex `rep[c]` of each SCC as its BFS
// root, and run an intra-SCC BFS. An SCC contains a directed cycle whose
// length is not divisible by 3 iff some intra-SCC edge `(u, v)` violates
// `dist[v] == dist[u] + 1 (mod 3)`.

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

let bad = any(comp[u] == comp[v] and dist[v]! % 3 != (dist[u]! + 1) % 3 for (u, v) in edges)
print(if bad: "yes" else: "no")
