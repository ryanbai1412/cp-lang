// Source: ETSR '23 R3 P3, NOI 2018 Day 1 P1 (Return), https://dmoj.ca/problem/etsr23r3p3
// Problem: a connected undirected graph, n <= 2e5, m <= 4e5, edges with length l <= 1e4 and
//   altitude a <= 1e9; home is vertex 1. Each of q <= 4e5 days gives start v and water level p:
//   drive free over edges with a > p, leave the car anywhere and walk the rest; print the least
//   walking length. Online: v, p are decoded with the last answer when K = 1. Multitest.
// Solution: Dijkstra from 1 gives each vertex's walk home. A Kruskal reconstruction tree over
//   the edges by decreasing altitude makes the vertices reachable by car above p one subtree:
//   jump from leaf v up while the node's altitude > p, and each node keeps the least walk in its
//   subtree. O((n + m) log + q log n).
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

let g = Graph<V, E>.undirected([E] e => (u: edges[e.ord].u, v: edges[e.ord].v, w: edges[e.ord].l))
let dist = dijkstra(g, from: [(V.first!, 0)])

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
    let top = ktree.jump_while(u, p => if let gval = gate[p.ord]: gval > t else: true)!
    last_ans = mn[top.idx]!
    print(last_ans)
