// Tuned: port.cp walking the tree breadth-first by hand instead of a Rooted tree or bfs's
// Paths with their dist and back links, as orig.cpp walks adj directly.
// Source: Codeforces 2229 E, Deconstruction Tree, https://codeforces.com/contest/2229/problem/E
// limits: 2000 ms, 256 MB
// Problem: multitest (sum n <= 2e5): a tree on n vertices. n-1 times: add the largest-index
//   leaf x to a set S, then delete any leaf other than x. Count the distinct sets S you can
//   end with, modulo 998244353.
// Solution: Root at n. In increasing order S is a chain from the largest initial leaf up to n,
//   and w < u can precede u iff w exceeds every label in u's child subtrees (at the root the
//   child subtree holding n-1 is exempt). With pv[u] that largest label, dp[u] = sum of dp[w]
//   for pv[u] < w < u, dp[largest leaf] = 1, answer dp[n]; a prefix-sum array makes it O(n).
// Bounds: n: 2..=300_000; edges.u, edges.v: 1..=n
// The order is a flat list and par an array; the edges go straight into the graph, so no
// edge list, bfs dist or back links stay alive.
const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[0..n - 1](u: V, v: V)>())
let root = V.last!
var par: [V]V = root
var order: [0..n]V = root  // breadth-first: each vertex after its parent
var seen = 1
for h in 0..n:
    let u = order[h]
    for c in g[u] if c != par[u]:
        par[c] = u
        order[seen] = c
        seen += 1

let max_leaf = V.filter(u => g[u].len == 1).max()!
var sub_max = [V] u => u
var prv_val: [V]?V = None
for u in order.rev:
    let max_ch = max(
        sub_max[c] for c in g[u] if c != par[u] and not (u == root and sub_max[c] == root.prev!)
    )
    prv_val[u] = max_ch
    if let mc = max_ch: sub_max[u] max= mc

var dp = Prefix<V, Z>.filling()
for u in V:
    let val = if u == max_leaf: Z(1)
    elif u > max_leaf:
        if let pv = prv_val[u], pv < u: dp.fold(pv.next!..u) else: Z(0)
    else: Z(0)
    dp.set(u, val)
print(dp.fold(root..=root))
