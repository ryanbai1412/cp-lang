// Codeforces 2127 H: maximum-size subgraph of a series-parallel graph where
// every vertex has degree at most 2.
//
// Termination invariant: every non-trivial series-parallel graph (treewidth <= 2)
// has a vertex of degree at most 2, and eliminating a degree-1 vertex (leaf
// absorption) or a degree-2 vertex (series reduction, merging parallel edges
// when its two neighbours are already joined) preserves series-parallel
// structure while strictly reducing the vertex count until one vertex remains.
//
// DP state:
// - Edge[du, dv] is the max edges chosen inside a collapsed generalized edge
//   (u, v) using at most du incident edges at u and at most dv at v.
// - Vert[d] is the max edges chosen inside leaves already absorbed into a
//   vertex using at most d incident edges at that vertex.
// Parallel, series, and leaf-absorption steps all apply the same capped
// max-plus convolution `max_plus_cap` over `Deg = 0..=2`. When the final vertex
// reaches degree 0, `vert[cur][Deg(2)]` is the answer.

domain Deg = 0..=2
type Edge = [Deg, Deg]Int
type Vert = [Deg]Int

let unit_edge: Edge = [Deg, Deg] (du, dv) => if du.ord >= 1 and dv.ord >= 1: 1 else: 0

// Capped max-plus convolution: res[d] = max_{i + j <= d} (a[i] + b[j]).
fn flip(e: Edge) -> Edge = [Deg, Deg] (du, dv) => e[dv, du]

fn max_plus_cap(a: Vert, b: Vert) -> Vert:
    var res: Vert = 0
    for d in Deg:
        for i in Deg if i <= d:
            res[d] max= a[i] + b[Deg(d.ord - i.ord)]
    res

fn parallel(a: Edge, b: Edge) -> Edge:
    var res: Edge = 0
    for au in Deg:
        for bu in Deg if au.ord + bu.ord <= 2:
            let du = Deg(au.ord + bu.ord)
            let row = max_plus_cap(a[au], b[bu])
            for dv in Deg: res[du, dv] max= row[dv]
    res

fn series(a: Edge, mid: Vert, b: Edge) -> Edge =
    [Deg, Deg] (au, bv) => max_plus_cap(max_plus_cap(a[au], mid), b[.., bv])[Deg(2)]

fn absorb_leaf(host: Vert, e: Edge, leaf: Vert) -> Vert:
    let best_by_host_deg: Vert = [Deg] eh => max_plus_cap(e[eh], leaf)[Deg(2)]
    let exact = max_plus_cap(host, best_by_host_deg)
    // Vert[d] means "at most d incident edges".
    [Deg] d => max(exact[i] for i in Deg if i <= d)!

@!multitest
let n: 1..=300_000 = read()
let m: 0..=600_000 = read()
domain V = 1..=n

// Undirected edge table: keyed by unordered {u, v}, automatically orienting Edge.
var edges: Map<(V, V), Edge> = Map()
var adj: [V]Set<V> = Set<V>()
var vert: [V]Vert = 0

for _ in 0..m:
    let (u, v): (V, V) = read()
    edges[minmax(u, v)] = unit_edge
    adj[u].add(v)
    adj[v].add(u)

var queued: [V]Bool = false
var ready: Deque<V> = Deque()
fn try_queue(v: V):
    if not queued[v] and adj[v].len <= 2:
        queued[v] = true
        ready.push(v)

for v in V: try_queue(v)

// edges[minmax(x, y)] is oriented from the smaller end; take(x, y) orients it x -> y.
fn take(x: V, y: V) -> Edge:
    let e = edges.remove(minmax(x, y))!
    if x < y: e else: flip(e)

while let cur = ready.pop_front():
    if adj[cur].len == 2:
        let u = adj[cur].pop_first()!
        let v = adj[cur].pop_last()!
        adj[u].remove(cur)
        adj[v].remove(cur)
        let through = series(take(u, cur), vert[cur], take(cur, v))
        let merged = if u < v: through else: flip(through)
        edges[minmax(u, v)] = edges.get(minmax(u, v)).map(old => parallel(old, merged)) ?? merged
        adj[u].add(v)
        adj[v].add(u)
        try_queue(u)
        try_queue(v)
    elif adj[cur].len == 1:
        let host = adj[cur].pop_first()!
        adj[host].remove(cur)
        vert[host] = absorb_leaf(vert[host], take(host, cur), vert[cur])
        try_queue(host)
    else:
        print(vert[cur][Deg(2)])
