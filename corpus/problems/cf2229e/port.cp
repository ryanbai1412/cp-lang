// Source: Codeforces 2229 E, Deconstruction Tree, https://codeforces.com/contest/2229/problem/E
// Problem: multitest (sum n <= 2e5): a tree on n vertices. n-1 times: add the largest-index
//   leaf x to a set S, then delete any leaf other than x. Count the distinct sets S you can
//   end with, modulo 998244353.
// Solution: Root at n. In increasing order S is a chain from the largest initial leaf up to n,
//   and w < u can precede u iff w exceeds every label in u's child subtrees (at the root the
//   child subtree holding n-1 is exempt). With pv[u] that largest label, dp[u] = sum of dp[w]
//   for pv[u] < w < u, dp[largest leaf] = 1, answer dp[n]; a prefix-sum array makes it O(n).
const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let root = V.last!
let t = Rooted(g, root:)

let max_leaf = V.filter(u => g[u].len == 1).map(u => u).max()!
var sub_max = [V] u => u
var prv_val: [V]?V = None
for u in t.at.rev:
    let ch = [sub_max[c] for c in t.children(u) if not (u == root and sub_max[c] == root.prev!)]
    let max_ch: ?V = if ch.is_empty: None else: Some(ch.max())
    prv_val[u] = max_ch
    if let mc = max_ch: sub_max[u] max= mc

var dp = Prefix<Z>.filling(n)
for u in V:
    let val = if u == max_leaf: Z(1)
    elif u > max_leaf:
        if let pv = prv_val[u], pv < u: dp.fold(pv.ord + 1..u.ord) else: Z(0)
    else: Z(0)
    dp.set(u.ord, val)
print(dp.fold(root.ord..root.ord + 1))
