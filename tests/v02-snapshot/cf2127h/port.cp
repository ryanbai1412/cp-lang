// Source: Codeforces 2127H (23 Rises Again), https://codeforces.com/contest/2127/problem/H
// Problem: a simple connected graph, 3 <= n <= 30 vertices, n - 1 <= m <= n(n - 1)/2 edges,
//   each vertex on at most 5 simple cycles (t <= 50 tests, sum of n^2 <= 900). Print the most
//   edges of a subgraph in which every vertex has degree at most 2.
// Solution: reduce the graph series-parallel style: fold a degree-1 vertex into its
//   neighbour, splice out a degree-2 vertex, merge parallel edges. A generalised edge keeps
//   best[du, dv] (most edges inside it using at most du, dv edges at its ends), a vertex
//   best[d] over the leaves folded into it; series, parallel and fold are max-plus products
//   over Deg = 0..=2. A queue holds vertices of degree <= 2 and the last one left gives
//   best[2]: O((n + m) log m) per test. One always remains: a K4 subdivision puts each branch
//   vertex on 6 simple cycles, so the graph has no K4 minor, and reduced K4-minor-free graphs
//   keep a vertex of degree <= 2.

domain Deg = 0..=2
type Edge = [Deg, Deg]Int
type Vert = [Deg]Int

let unit_edge: Edge = [Deg, Deg] (du, dv) => if du.ord >= 1 and dv.ord >= 1: 1 else: 0

fn flip(e: Edge) -> Edge = [Deg, Deg] (du, dv) => e[dv, du]

fn parallel(a: Edge, b: Edge) -> Edge:
    var res: Edge = 0
    for au in Deg:
        for bu in 0..=2 - au.ord:
            for av in Deg:
                for bv in 0..=2 - av.ord:
                    res[Deg(au.ord + bu), Deg(av.ord + bv)] max= a[au, av] + b[Deg(bu), Deg(bv)]
    res

fn series(a: Edge, mid: Vert, b: Edge) -> Edge:
    var res: Edge = 0
    for au in Deg:
        for ac in Deg:
            for bc in 0..=2 - ac.ord:
                let rem = Deg(2 - ac.ord - bc)
                for bv in Deg:
                    res[au, bv] max= a[au, ac] + mid[rem] + b[Deg(bc), bv]
    res

fn absorb_leaf(host: Vert, e: Edge, leaf: Vert) -> Vert:
    var res: Vert = 0
    for dh in Deg:
        for eh in 0..=2 - dh.ord:
            for el in Deg:
                let rem = Deg(2 - el.ord)
                res[Deg(dh.ord + eh)] max= host[dh] + e[Deg(eh), el] + leaf[rem]
    // Keep Vert monotone so Vert[d] means "at most d incident edges".
    res[Deg(1)] max= res[Deg(0)]
    res[Deg(2)] max= res[Deg(1)]
    res

@!multitest
let n: 3..=30 = read()
let m: n - 1..=n * (n - 1) / 2 = read()
domain V = 1..=n

fn canon(u: V, v: V) -> (V, V) = if u < v: (u, v) else: (v, u)

var edges: Map<(V, V), Edge> = Map()
var adj: [V]Set<V> = Set<V>()
var vert: [V]Vert = 0

for _ in 0..m:
    let (u, v): (V, V) = read()
    edges[canon(u, v)] = unit_edge
    adj[u].add(v)
    adj[v].add(u)

var queued: [V]Bool = false
var ready: Deque<V> = Deque()
fn try_queue(v: V):
    if not queued[v] and adj[v].len <= 2:
        queued[v] = true
        ready.push(v)

fn pop_edge(u: V, v: V) -> Edge:
    let raw = edges.remove(canon(u, v))!
    if u < v: raw else: flip(raw)

fn add_edge(u: V, v: V, e: Edge):
    let key = canon(u, v)
    let oriented = if u < v: e else: flip(e)
    if let old = edges.get(key):
        edges[key] = parallel(old, oriented)
    else:
        edges[key] = oriented

for v in V: try_queue(v)

while let cur = ready.pop_front():
    if adj[cur].len == 2:
        let u = adj[cur].pop_first()!
        let v = adj[cur].pop_last()!
        adj[u].remove(cur)
        adj[v].remove(cur)
        let merged = series(pop_edge(u, cur), vert[cur], pop_edge(cur, v))
        add_edge(u, v, merged)
        adj[u].add(v)
        adj[v].add(u)
        try_queue(u)
        try_queue(v)
    elif adj[cur].len == 1:
        let host = adj[cur].pop_first()!
        adj[host].remove(cur)
        vert[host] = absorb_leaf(vert[host], pop_edge(host, cur), vert[cur])
        try_queue(host)
    else:
        print(vert[cur][Deg(2)])
