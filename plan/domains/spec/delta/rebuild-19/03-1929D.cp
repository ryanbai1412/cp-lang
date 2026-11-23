// CF 1929D Sasha and a Walk in the City: dp[v] = sets in v's subtree whose top-most chosen
// vertices are an antichain below v (or v itself); answer = sum dp + 1 (empty set).
type Z = Mod<998244353>

@!multitest
let n: 2..=3e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: V.first!)
var dp: [V]Z = 1
for v in t.order.rev:
    for c in t.children(v): dp[v] *= dp[c] + 1
print(dp.sum() + 1)
