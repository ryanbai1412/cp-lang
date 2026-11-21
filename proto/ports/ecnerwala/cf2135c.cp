use std.graph.{two_edge_components}

type Z = Mod<998_244_353>

@!multitest
let (n, m, lim): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let a: [V]?Int = read(none: -1)
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

// Values agree across every 2-edge-connected component, and an odd cycle forces 0.
// A component is entered through one bridge, so BFS depth parity inside it is its own.
let (domain C, comp) = two_edge_components(g)
let depth = bfs(g, [V(1)])

var want: [C]?Int = none
var ans: Z = 1
fn force(c: C, x: Int):
    if (want[c] ?? x) != x: ans = 0
    want[c] = x
for (u, v) in edges:
    if comp[u] == comp[v] and depth[u]! % 2 == depth[v]! % 2: force(comp[u], 0)
for u in V:
    if let x = a[u]: force(comp[u], x)
for c in C:
    if want[c] == none: ans *= Z(lim)
print(ans)
