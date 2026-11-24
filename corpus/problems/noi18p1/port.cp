// Source: NOI 2018 D1P1 (Return), https://dmoj.ca/problem/noi18p1
// Problem: T <= 3 tests, each a connected undirected graph, 1 <= n <= 2e5, n - 1 <= m <= 4e5,
//   roads with length 1 <= l <= 1e4 and altitude 1 <= a <= 1e9; home is vertex 1. Each of
//   1 <= q <= 4e5 days gives start v and water level 0 <= p <= S <= 1e9: drive free over roads
//   with a > p, leave the car anywhere and walk the rest; print the least walking length.
//   Online when K = 1: v, p are shifted by the last answer, mod n and S + 1.
// Solution: Dijkstra from 1 gives each vertex's walk home. A Kruskal reconstruction tree over
//   the roads by decreasing altitude makes the vertices reachable by car above p one subtree,
//   `kt.reach(v, desc(p + 1))`, and each node keeps the least walk under it, folded over its children.
//   O((n + m) log + q log n).
// original: corpus/submissions/6512702.cpp

@!multitest
let n: 1..=2e5 = read()
let m: 0..=4e5 = read()
domain V = 1..=n
let roads: [m](u: V, v: V, w: 1..=1e4, alt: 1..=1e9) = read()
let home = dijkstra(Graph.undirected(roads), from: [(V.first!, 0)])
let (domain N, kt) = kruskal_tree(roads, w: r => desc(roads[r].alt))
let t = kt.tree
// least[x]: the shortest walk home from any vertex under node x; a node merges after its children
var least = t.down(0, (d, x) => 0)
for v in V: least[kt.leaf[v]] = home[v]!
for x in t.order.rev:
    if kt.w[x] != None: least[x] = t.children(x).map(c => least[c]).min()!
let q: 1..=4e5 = read()
let k: 0..=1 = read()
let s: 0..=1e9 = read()
var last = 0
for _ in 0..q:
    let v0: V = read()
    let p0: 0..=s = read()
    let v = V.nth((v0.ord + k * last) % n)
    let p = (p0 + k * last) % (s + 1)
    last = least[kt.reach(v, desc(p + 1))]
    print(last)
