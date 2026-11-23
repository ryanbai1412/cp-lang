// Tree distance queries: a tree on vertices 1..n (n - 1 edges), q queries "u v";
// print dist(u, v) = depth(u) + depth(v) - 2 depth(lca(u, v)), rooted at 1.
let n = read<1..=2e5>()
let q = read<1..=2e5>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: 1)
let lt = LcaTable(t)
for _ in 0..q:
    let (u, v) = read<(V, V)>()
    print(t.depth(u) + t.depth(v) - 2 * t.depth(lt.lca(u, v)))
