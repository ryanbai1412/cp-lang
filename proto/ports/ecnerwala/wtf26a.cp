// AtCoder WTF 2026 A: build the Kruskal reconstruction tree of edges sorted by
// decreasing weight, propagate cut costs downward (clamped to max_w without
// 128-bit overflow), then run a second Kruskal DSU to sum pair contributions.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
var edges: List<(w: Int, u: Int, v: Int)> = []
for _ in 0..n - 1:
    let (u, v, w): (Int, Int, Int) = read()
    edges.push((w, u - 1, v - 1))
edges = sorted(edges, by: e => -e.w)
let max_w = edges[0].w * (n - 1)

let m = 2 * (n - 1)
var cut_w: [0..m]Int = 0
var cut_sz: [0..m]Int = 0
var cut_ng: [0..m]Int = 0

var par: [0..n]Int = -1
var label = [0..n] i => i
fn find_root(mut p: [0..n]Int, var x: Int) -> Int:
    while p[x] >= 0:
        if p[p[x]] >= 0: p[x] = p[p[x]]
        x = p[x]
    x

for e in 0..n - 1:
    var a = find_root(mut par, edges[e].u)
    var b = find_root(mut par, edges[e].v)
    if par[a] > par[b]: swap(mut a, mut b)
    let la = label[a]
    let lb = label[b]
    cut_w[la] = edges[e].w * (-par[b])
    cut_sz[la] = -par[a]
    cut_ng[la] = n + e
    cut_w[lb] = edges[e].w * (-par[a])
    cut_sz[lb] = -par[b]
    cut_ng[lb] = n + e
    par[a] += par[b]
    par[b] = a
    label[a] = n + e

for i in (0..m).rev:
    let ng = cut_ng[i]
    if ng < m: cut_w[i] += cut_w[ng]

var cuts: List<(w: Int, u: Int, v: Int)> = []
for i in 0..m:
    let w = if cut_w[i] > max_w / cut_sz[i]: max_w else: cut_w[i] * cut_sz[i]
    cuts.push((w, i, cut_ng[i]))
cuts = sorted(cuts, by: c => -c.w)

let total_nodes = 2 * n - 1
domain V = 0..total_nodes
var dsu = Dsu<V>()
var freq: [V]Int = 0
for i in 0..n: freq[V(i)] = 1
var ans: Z = 0
for c in cuts:
    let ra = dsu.find(V(c.u))
    let rb = dsu.find(V(c.v))
    let fa = freq[ra]
    let fb = freq[rb]
    ans += Z(fa) * Z(fb) * Z(c.w)
    dsu.unite(ra, rb)
    freq[dsu.find(ra)] = fa + fb
print(ans)
