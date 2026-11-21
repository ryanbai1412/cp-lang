// Codeforces 2207 D: root the tree at V and compute the minimum distance to a
// leaf in each subtree, collapsing to 0 when two leaves lie within K + 1 edges.
@!multitest
let (n, k, v_in): (Int, Int, Int) = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V(v_in))

var dist: [V]Int = k + 2
for u in t.postorder:
    if g[u].len == 1:
        dist[u] = 0
        continue
    var d = k + 2
    for c in t.children(u):
        let nd = dist[c] + 1
        if d + nd <= k + 1:
            d = 0
            break
        d min= nd
    dist[u] = d

print(if dist[V(v_in)] == 0: "YES" else: "NO")
