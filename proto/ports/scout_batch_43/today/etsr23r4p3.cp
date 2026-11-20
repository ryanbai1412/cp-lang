// ETSR '23 R4 P3 (CP Lang 0.2.2): tree-to-graph homomorphism inclusion-exclusion
// over `Subset<V>` with 2D domain-indexed DP tables `[V, V]Int`.
// original: corpus/submissions/5635626.cpp

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain TE = 0..(n - 1)

var has_edge: [V, V]Bool = false
for _ in 0..m:
    let (u, v): (V, V) = read()
    has_edge[u, v] = true
    has_edge[v, u] = true

var tree_edges: List<(u: V, v: V)> = []
for _ in 0..(n - 1):
    let (u, v): (V, V) = read()
    tree_edges.push((u: u, v: v))

let g = Graph<V, TE>.undirected([TE] e => tree_edges[e.idx])
let tree = Rooted<V>(g, root: V.first)
let post = tree.postorder

var ans = 0
for mask in Subset<V>:
    if mask.idx == 0: continue
    var dp: [V, V]Int = 0
    for u in post:
        for v1 in mask:
            var ways = 1
            for c in tree.children(u):
                ways *= sum(dp[c, v2] for v2 in mask if has_edge[v1, v2])
            dp[u, v1] = ways
    let total = sum(dp[V.first, v] for v in mask)
    if (n - popcount(mask.idx)) % 2 == 0: ans += total else: ans -= total

print(ans)
