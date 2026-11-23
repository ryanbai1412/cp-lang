// Source: DMOJ ETSR '23 R4 P3 (ZJOI 2016 Small Stars), https://dmoj.ca/problem/etsr23r4p3
// Problem: a graph with n <= 17 vertices and m edges, and a tree on the same n labels. Count
//   the bijections from tree vertices to graph vertices that send every tree edge to a graph
//   edge.
// Solution: for a vertex set S, count the maps of the tree into S by tree DP: dp[u, v] = ways
//   to place u's subtree with u on v, the product over children c of the sum of dp[c, w] over
//   w in S adjacent to v. Inclusion-exclusion over S with sign (-1)^(n - |S|) keeps only the
//   onto maps: O(2^n n^3).
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
    tree_edges.push((u:, v:))

let g = Graph<V, TE>.undirected([TE] e => tree_edges[e.idx])
let tree = Rooted<V>(g, root: V.first!)

var ans = 0
for mask in Subset<V>:
    if mask.idx == 0: continue
    var dp: [V, V]Int = 0
    for u in tree.at.rev:
        for v1 in mask:
            var ways = 1
            for c in tree.children(u):
                ways *= sum(dp[c, v2] for v2 in mask if has_edge[v1, v2])
            dp[u, v1] = ways
    let total = sum(dp[V.first!, v] for v in mask)
    if (n - popcount(mask.idx)) % 2 == 0: ans += total else: ans -= total

print(ans)
