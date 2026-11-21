// AtCoder WTF 2026 A: build the Kruskal reconstruction tree of edges sorted by
// decreasing weight, propagate cut costs downward (clamped to max_w without
// 128-bit overflow), then run a second Kruskal DSU to sum pair contributions.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
domain U = 1..=n
let raw: [0..n - 1](u: U, v: U, w: Int) = read()
let edges = raw.sorted(by: e => e.w, desc: true)
let max_w = edges[0].w * (n - 1)

let m = 2 * (n - 1)
var cut_w: [0..m]Int = 0
var cut_sz: [0..m]Int = 0
var cut_ng: [0..m]Int = 0

var dsu1 = Dsu<U>()
var sz: [U]Int = 1
var label = [U] u => u.ord
for (e, edge) in edges.items:
    let (a, b) = dsu1.merge(edge.u, edge.v)!
    let (la, lb) = (label[a], label[b])
    cut_w[la] = edge.w * sz[b]
    cut_sz[la] = sz[a]
    cut_ng[la] = n + e
    cut_w[lb] = edge.w * sz[a]
    cut_sz[lb] = sz[b]
    cut_ng[lb] = n + e
    sz[a] += sz[b]
    label[a] = n + e

for i in (0..m).rev if cut_ng[i] < m:
    cut_w[i] += cut_w[cut_ng[i]]

let cuts = sorted(
    [
        (
            w: if cut_w[i] > max_w / cut_sz[i]: max_w else: cut_w[i] * cut_sz[i],
            u: i,
            v: cut_ng[i],
        ) for i in 0..m
    ],
    by: c => c.w,
    desc: true,
)

domain V = 0..(2 * n - 1)
var dsu = Dsu<V>()
var freq = [V] v => Int(v.ord < n)
var ans: Z = 0
for c in cuts if let (kept, merged) = dsu.merge(V(c.u), V(c.v)):
    ans += Z(freq[kept]) * freq[merged] * c.w
    freq[kept] += freq[merged]
print(ans)
