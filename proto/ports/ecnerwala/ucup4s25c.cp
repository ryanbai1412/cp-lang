// UCup 4 Stage 25 (Zagreb) C: build the directed graph of ordered adjacent
// pairs across all 6 permutations of each input triple, compute SCCs, and
// check whether any intra-SCC edge violates BFS depth + 1 (mod 3).
use std.seq.{get_or_insert_with}

@!multitest
let m: Int = read()
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
