// CF 1975D: P_B meets P_A's red trail at the vertex ceil(d/2) steps from b toward a,
// then walks the whole tree from there: 2(n - 1) - (farthest distance from it).
@!multitest
let n: 1..=2e5 = read()
domain V = 1..=n
let (a, b): (V, V) = read()
let edges = read<[n - 1](u: V, v: V)>()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V.first!)

let d = t.dist(a, b)
let meet = t.jump(b, a, (d + 1) / 2)!
let far = bfs(g, from: [meet])
print((d + 1) / 2 + 2 * (n - 1) - V.map(v => far[v]!).max()!)
