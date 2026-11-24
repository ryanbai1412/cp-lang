// Stoer-Wagner: 600 vertices, 20000 random undirected edges, weights up to 100.

domain V = 0..600
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var edges = List<(u: V, v: V, w: Int)>()
for _ in 0..20000:
    let u = next(V.count)
    let v = next(V.count)
    edges.push((u: V(u), v: V(v), w: next(100) + 1))
let (w, side) = global_min_cut(edges)
print(w, [v for v in V if side[v]].len)
