// Codeforces 2127 H: maximum-size subgraph of a series-parallel graph where
// every vertex has degree at most 2.
//
// Repeatedly eliminate degree-1 and degree-2 vertices until one vertex remains:
// - Edge[du, dv] is the max edges chosen inside a collapsed generalized edge
//   (u, v) using at most du incident edges at u and at most dv at v.
// - Vert[d] is the max edges chosen inside leaves already absorbed into a
//   vertex using at most d incident edges at that vertex.
// When the final vertex reaches degree 0, vert[cur][Deg(2)] is the answer.

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
let n: 1..=300_000 = read()
let m: 0..=600_000 = read()
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
