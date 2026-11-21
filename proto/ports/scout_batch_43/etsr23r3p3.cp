// ETSR '23 Round 3 P3 — Return Home (Dijkstra + Kruskal Reconstruction Tree) (CP Lang v2 spec)
// original: corpus/submissions/5521298.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Rethought idiomatically: instead of a 110-line persistent segment-tree DSU,
// run Dijkstra from vertex 1 and build a Kruskal Reconstruction Tree in
// descending altitude order, answering online queries via `Rooted.lift_while`.

struct KNode:
    left: ?Int = none
    right: ?Int = none
    gate: ?Int = none
    mn: Int = 0

@multitest
let (n, m): (1..=2e5, 0..=4e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, len: Int, alt: Int) = read()

let g = Graph.undirected([(u: e.u, v: e.v, w: e.len) for e in edges])
let dist = dijkstra(g, from: V.first)

// FRICTION ET5: Building a Kruskal Reconstruction Tree requires dynamic node
// allocation before freezing into a `Rooted<K>` tree. Packing all node fields
// into one `List<KNode>` lets a single `.freeze()` create one shared domain `K`,
// but child indices inside `KNode` remain raw `?Int` until converted to `K`.
var nodes: List<KNode> = [KNode(none, none, none, dist[v] ?? Int.MAX) for v in V]
var dsu = Dsu<V>()
var root_of: [V]Int = [V] v => v.to_idx

for e in edges.sorted(by: e => -e.alt):
    let ru = dsu.find(e.u)
    let rv = dsu.find(e.v)
    if ru != rv:
        let (a, b) = (root_of[ru], root_of[rv])
        let k = nodes.push_id(KNode(a, b, e.alt, min(nodes[a].mn, nodes[b].mn)))
        dsu.unite(e.u, e.v)
        root_of[dsu.find(e.u)] = k

let (K, kn) = nodes.freeze()
let ktree = Rooted<K>(u => [K(c) for c in [kn[u].left, kn[u].right] if let c = c], root: K.last)

let (q, online, tmax): (1..=4e5, 0..=1, Int) = read()
var last_ans = 0
for _ in 0..q:
    let (u0, t0): (Int, Int) = read()
    let u = K((u0 + online * last_ans - 1) % n)
    let t = (t0 + online * last_ans) % (tmax + 1)
    let top = ktree.lift_while(u, p => if let gate = kn[p].gate: gate > t else: true)!
    last_ans = kn[top].mn
    print(last_ans)
