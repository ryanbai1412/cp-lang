// Codeforces 2178 F — Tree Cut Reconnection
//
// Cut every edge above a non-root vertex whose remaining subtree size is even,
// splitting the tree into the root component (size s_0) and `comps` cut
// components of sizes s_1, ..., s_k.
//
// Counting ways to reconnect the components into a valid tree yields:
// - If comps == 0 (no cuts): 1.
// - Otherwise: s_0 * (prod_{i=1..k} s_i^2) * (k - 1)! * (sum_{i=1..k} 1 / s_i)
//   (modulo 998_244_353), computed in one bottom-up pass by accumulating
//   `ans = s_0 * prod(s_i^2) * k!` and `s_inv = sum(1 / s_i)`, then dividing
//   by `comps` at the end.

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 1..=200_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var sz: [V]Int = 1
var comps = 0
var ans: Z = 1
var s_inv: Z = 0
for u in t.at.rev:
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
