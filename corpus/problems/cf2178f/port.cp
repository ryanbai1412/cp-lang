// Source: Codeforces 2178 F (Conquer or of Forest), https://codeforces.com/contest/2178/problem/F
// limits: 2000 ms, 512 MB
// Problem: a tree rooted at 1, vertex white iff its subtree size is even. It is conquered if all
//   white vertices lie on one path from the root. An operation cuts the edge above a white
//   non-root vertex, adds any edge keeping a tree, and recolours. Count the distinct conquered
//   trees reachable, modulo 998244353. Multitest, n >= 2, sum of n <= 2e5.
// Solution: cutting every edge above an even subtree gives a root part s_0 and k even parts
//   s_1..s_k. A conquered tree chains the even parts below the root part: s_0 places to hang
//   the chain, an entry and an exit vertex in each part but the last. Summing over orders:
//   s_0 * (k - 1)! * prod s_i^2 * sum 1/s_i, or 1 if k = 0. One bottom-up pass, O(n).

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
