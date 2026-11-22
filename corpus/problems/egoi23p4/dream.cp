// EGOI 2023 P4: rebuild a weighted graph from two matrices: for each pair, the best bottleneck
// over paths (max of path minimum) and k minus the worst (min of path maximum). Print its edges
// or NO. Both matrices must already be closed under their semiring; a pair can carry an edge
// only where the two agree it may; the max and min spanning trees of those candidates are the
// whole answer if any graph is.
domain V = [read Int]
let k = read Int
let widest = read lower_triangle [V, V]Int
let narrowest = (read lower_triangle [V, V]Int).map(b => k - b)

let fits = (g: Mat<Int>) => g == g.closure(Semiring(max, min)) 
if not fits(widest) or not narrowest.closed(Semiring(min, max)):
    print("NO")
    return

let candidates = [for (u, v) in V.pairs() if widest[u, v] >= narrowest[u, v]
                  for w in [widest[u, v], narrowest[u, v]]: (u:, v:, w:)]
let edges = kruskal(candidates, by: desc(.w)) ++ kruskal(candidates, by: .w)
let g = Graph.undirected(edges)
if g.bottleneck(Semiring(max, min)) != widest or g.bottleneck(Semiring(min, max)) != narrowest:
    print("NO")
    return
print(edges.len)
for e in edges: print(e.u, e.v, e.w)
