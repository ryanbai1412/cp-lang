// Source: DMOJ ETSR '23 R4 P3 (ZJOI 2016 Small Stars), https://dmoj.ca/problem/etsr23r4p3
// Problem: a simple graph with n <= 17 vertices and m <= n(n-1)/2 edges, and a tree on the
//   same n labels. Count the bijections from tree vertices to graph vertices that send every
//   tree edge to a graph edge.
// Solution: for a vertex set S, count the maps of the tree into S by tree DP: dp[u, v] = ways
//   to place u's subtree with u on v, the product over children c of the sum of dp[c, w] over
//   w in S adjacent to v. Inclusion-exclusion over S with sign (-1)^(n - |S|) keeps only the
//   onto maps: O(2^n n^3). The map counts reach 17^17 > 2^64, so they wrap in U64; the
//   answer itself is at most 17!, so the wrapped sum is exact.
// original: corpus/submissions/5635626.cpp
// Bounds: n: 1..=17; m: 0..=n * (n - 1) / 2; tree_edges.u, tree_edges.v: 1..=n; u, v: 1..=n

let n: 1..=17 = read()
let m: 0..=n * (n - 1) / 2 = read()
domain V = 1..=n
domain TE = 0..(n - 1)

var adj: [V]Subset<V> = Subset<V>.empty
for _ in 0..m:
    let (u, v): (V, V) = read()
    adj[u] = adj[u].with(v)
    adj[v] = adj[v].with(u)
let tree_edges: [TE](u: V, v: V) = read()
let tree = Rooted<V>(Graph<V, TE>.undirected(tree_edges), root: V.first!)

var ans: U64 = 0
var dp: [V, V]U64 = 0
for mask in Subset<V>:
    if mask.len == 0: continue
    for u in tree.at.rev:
        for v1 in mask:
            var ways: U64 = 1
            for c in tree.children(u):
                ways *= sum(dp[c, v2] for v2 in adj[v1] & mask)
            dp[u, v1] = ways
    let total = sum(dp[V.first!, v] for v in mask)
    if (n - mask.len) % 2 == 0: ans += total else: ans -= total

print(ans)
