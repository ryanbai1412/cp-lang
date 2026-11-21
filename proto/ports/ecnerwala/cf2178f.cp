const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var sz: [V]Int = 1
var comps = 0
var ans: Z = 1
var s_inv: Z = 0
for u in t.postorder:
    for c in t.children(u): sz[u] += sz[c]
    if u == V(1):
        ans *= sz[u]
    elif sz[u] % 2 == 0:
        comps += 1
        ans *= sz[u]
        ans *= sz[u]
        ans *= comps
        s_inv += Z(sz[u]).inv()
        sz[u] = 0

print(if comps > 0: ans * s_inv / comps else: Z(1))
