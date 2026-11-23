// Codeforces 2183 D1 — Tree Coloring (Easy Version)
//
// Each operation picks an independent set containing at most one vertex per
// depth, so every depth layer needs at least `layer.len` operations. Moreover,
// no vertex can share an operation with its parent: if every vertex in a
// non-root layer `d > 0` shares a single parent `p0`, that parent cannot pair
// with any vertex in the layer, forcing `layer.len + 1` operations. This lower
// bound is tight by assigning colors layer-by-layer avoiding each vertex's parent.

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var ans = 0
for (_, layer) in V.group(by: t.depth):
    let p0 = t.parent(layer[0])
    let same_parent = p0 != none and layer.all(v => t.parent(v) == p0)
    ans max= layer.len + Int(same_parent)
print(ans)
