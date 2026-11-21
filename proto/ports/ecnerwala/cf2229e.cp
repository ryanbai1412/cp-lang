// Codeforces 2229 E: root the tree at N to record each vertex's highest
// child-subtree maximum, then run a prefix-sum modular DP from max_leaf to N.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V(n))

let max_leaf = max(u.to_idx for u in V if g[u].len == 1)!
var sub_max = [V] u => u.to_idx
var prv_val: [0..n]?Int = none
for u in t.at.rev:
    var max_ch: ?Int = none
    for c in t.children(u):
        if u == V(n) and sub_max[c] == n - 2: continue
        max_ch max= sub_max[c]
    prv_val[u.to_idx] = max_ch
    if let mc = max_ch: sub_max[u] max= mc

var dp = Prefix<Z>.filling(n)
for i in 0..n:
    let val = if i == max_leaf: Z(1)
    elif i > max_leaf:
        if let pv = prv_val[i], pv < i: dp.sum(pv + 1..i) else: Z(0)
    else: Z(0)
    dp.set(i, val)
print(dp.sum(n - 1..n))
