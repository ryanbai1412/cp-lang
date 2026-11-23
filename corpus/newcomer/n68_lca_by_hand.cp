let (n, q): (1..=200000, 1..=200000) = read()
let par: [0..n - 1]1..=n = read()
var parent: List<Int> = [0, 0]
var depth: List<Int> = [0, 0]
for v in 2..=n:
    parent.push(par[v - 2])
    depth.push(depth[par[v - 2]] + 1)  // expect: missing depth_by_hand
for _ in 0..q:
    let (u0, v0): (1..=n, 1..=n) = read()
    var u = u0
    var v = v0
    while depth[u] > depth[v]:  // expect: missing lca_by_hand
        u = parent[u]
    while depth[v] > depth[u]:
        v = parent[v]
    while u != v:
        u = parent[u]
        v = parent[v]
    print(u)
