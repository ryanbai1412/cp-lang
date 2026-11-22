// UCup 4 Stage 25 (Zagreb) C — Non-3-Divisible Directed Cycle in Pair Graph
//
// Each input triple `(a, b, c)` permits all 6 ordered transitions
// `(x, y) -> (y, z)` between consecutive ordered pairs. Build the directed
// graph on distinct ordered pairs, compute its SCCs, and run a BFS within each
// SCC from one representative `rep[c]`.
//
// An SCC contains a directed cycle of length not divisible by 3 iff some
// intra-SCC edge `(u, v)` violates `dist[v] == dist[u] + 1 (mod 3)`.

use std.seq.{get_or_insert_with}

@!multitest
let m: 1..=100_000 = read()
var verts: Map<(Int, Int), Int> = Map()
var raw_edges: List<(Int, Int)> = []

fn id(k: (Int, Int)) -> Int: verts.get_or_insert_with(k, () => verts.len)

for _ in 0..m:
    let (a, b, c): (Int, Int, Int) = read()
    for (x, y, z) in [(a, b, c), (a, c, b), (b, a, c), (b, c, a), (c, a, b), (c, b, a)]:
        raw_edges.push((id((x, y)), id((y, z))))

domain V = verts.len
let edges = [(V(u), V(v)) for (u, v) in raw_edges]
let adj = Graph.directed(edges)
let (domain C, comp) = scc(adj)
var rep: [C]?V = none
for u in V: rep[comp[u]] = u
let roots = [u for c in C if let u = rep[c]]
let dist = bfs(u => [v for v in adj[u] if comp[v] == comp[u]], roots)

let bad = any(comp[u] == comp[v] and dist[v]! % 3 != (dist[u]! + 1) % 3 for (u, v) in edges)
print(if bad: "yes" else: "no")
