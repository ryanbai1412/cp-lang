// Matched benchmark for std.tree: rooting, rerooting, LCA queries and a centroid decomposition.
use std.tree.{Centroids}
let n: 1..=1_000_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let t = Rooted.new(g, V(1))
// eccentricity of every vertex by rerooting
let far = t.reroot(max, (h, c, p) => h + 1, (h, v) => max(h, 0))
let q: 0..=1_000_000 = read()
let queries: [0..q](a: V, b: V) = read()
var hash = 0
for (a, b) in queries: hash = (hash * 31 + t.lca(a, b).ord + t.dist(a, b)) % 1_000_000_007
let cd = Centroids<V>(g)
var spread = 0
for v in V:
    for x in cd.ancestors(v): spread += x.dist
print(sum(far[v] for v in V), hash, spread)
