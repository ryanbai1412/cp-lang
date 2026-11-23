// ETSR '23 R3 P3 (CP Lang 0.2.2): Kruskal reconstruction tree + Dijkstra from 1
// with `@multitest` and Tropical Optionals `?Int` (no `INF` sentinel).
// original: corpus/submissions/5521298.cpp

@multitest
let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
type Edge = (u: V, v: V, l: Int, w: Int)
var edges: List<Edge> = []
for _ in 0..m:
    let (u, v, l, w): (Int, Int, Int, Int) = read()
    edges.push((u: V(u), v: V(v), l:, w:))

let g = Graph<V, E>.undirected([E] e => (u: edges[e.idx].u, v: edges[e.idx].v))
let dist = dijkstra(u => g.out(u), [(V.first, 0)], a => edges[a.e.idx].l)

var dsu = Dsu<V>()
var root_of: [V]Int = [V] v => v.idx
var kids: List<List<Int>> = [[] for _ in 0..n]
var gate: List<?Int> = [None for _ in 0..n]
var mn: List<?Int> = [dist[v] for v in V]

for (_, ei) in sorted([(-edges[i].w, i) for i in 0..m]):
    let ru = dsu.find(edges[ei].u)
    let rv = dsu.find(edges[ei].v)
    if ru != rv:
        let (a, b) = (root_of[ru], root_of[rv])
        let k = kids.len
        kids.push([a, b])
        gate.push(edges[ei].w)
        var best = mn[a]
        best min= mn[b]
        mn.push(best)
        dsu.unite(edges[ei].u, edges[ei].v)
        root_of[dsu.find(edges[ei].u)] = k

domain K = 0..kids.len
let ktree = Rooted<K>(u => [K(c) for c in kids[u.idx]], root: K.last!)

let (q, online, tmax): (Int, Int, Int) = read()
var last_ans = 0
for _ in 0..q:
    let (u0, t0): (Int, Int) = read()
    let u = K((u0 + online * last_ans - 1) % n)
    let t = (t0 + online * last_ans) % (tmax + 1)
    let top = ktree.lift_while(u, p => if let gval = gate[p.idx]: gval > t else: true)!
    last_ans = mn[top.idx]!
    print(last_ans)
