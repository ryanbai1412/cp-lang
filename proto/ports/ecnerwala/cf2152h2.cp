struct HeapNode(v: Int, cnt: Int, l: ?Int, r: ?Int)

@!multitest
let n: Int = read()
domain V = 1..=n
let raw_edges: [0..n - 1](u: V, v: V, w: Int) = read()
var deg: [V]Int = 0
for (u, v, w) in raw_edges:
    deg[u] += w
    deg[v] += w
let edges = raw_edges.sorted(by: e => -e.w)

var nodes: [0..4 * n]HeapNode = HeapNode(0, 0, none, none)
var n_nodes = 0

fn new_node(v: Int, cnt: Int) -> ?Int:
    let id = n_nodes
    nodes[id] = HeapNode(v, cnt, none, none)
    n_nodes += 1
    id

fn meld(oa: ?Int, ob: ?Int) -> ?Int:
    var a = oa ?? return ob
    var b = ob ?? return oa
    if nodes[a].v > nodes[b].v: swap(mut a, mut b)
    nodes[a].r = meld(nodes[a].r, b)
    swap(mut nodes[a].l, mut nodes[a].r)
    a

var heaps: [V]?Int = [V] u => new_node(deg[u], 1)
var dsu = Dsu<V>()

for (u0, v0, w) in edges:
    var u = dsu.find(u0)
    var v = dsu.find(v0)
    dsu.unite(u, v)
    if dsu.find(u) == v: swap(mut u, mut v)
    heaps[u] = meld(heaps[u], heaps[v])
    heaps[v] = none
    deg[u] += deg[v] - 2 * w

    let top = heaps[u]!
    if deg[u] < nodes[top].v:
        var cur_pos = nodes[top].v
        var cur_slope = 0
        var cur_val = 0
        while true:
            if let h = heaps[u], cur_val + cur_slope * (nodes[h].v - cur_pos) - nodes[h].v < -deg[u]:
                let nd = nodes[h]
                cur_val += cur_slope * (nd.v - cur_pos)
                cur_pos = nd.v
                cur_slope += nd.cnt
                heaps[u] = meld(nd.l, nd.r)
            else:
                let diff = cur_pos - cur_val - deg[u]
                let nx = cur_pos + diff / (cur_slope - 1)
                let gap = diff % (cur_slope - 1)
                if gap > 0: heaps[u] = meld(new_node(nx + 1, gap), heaps[u])
                heaps[u] = meld(new_node(nx, cur_slope - 1 - gap), heaps[u])
                heaps[u] = meld(new_node(deg[u], 1), heaps[u])
                break

var deltas: List<Int> = []
let rt = dsu.find(V(1))
while let h = heaps[rt]:
    let nd = nodes[h]
    for _ in 0..nd.cnt: deltas.push(nd.v)
    heaps[rt] = meld(nd.l, nd.r)

let pref = prefix_n(deltas)
let q: Int = read()
for _ in 0..q:
    let t: Int = read()
    let cnt = deltas.first_ge(t) ?? n
    print(t * cnt - pref.prefix(cnt))
