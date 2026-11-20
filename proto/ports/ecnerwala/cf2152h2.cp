struct HeapNode(v: Int, cnt: Int, l: ?Int, r: ?Int)

@!multitest
let n: Int = read()
var edges: List<(Int, Int, Int)> = []
var deg: [0..n]Int = 0
for _ in 0..(n - 1):
    let (u, v, w): (Int, Int, Int) = read()
    edges.push((-w, u - 1, v - 1))
    deg[u - 1] += w
    deg[v - 1] += w
edges.sort()

var nodes: List<HeapNode> = []

fn new_node(v: Int, cnt: Int) -> ?Int:
    let id = nodes.len
    nodes.push(HeapNode(v: v, cnt: cnt, l: none, r: none))
    some(id)

fn meld(oa: ?Int, ob: ?Int) -> ?Int:
    if oa == none: return ob
    if ob == none: return oa
    var a = oa!
    var b = ob!
    if nodes[a].v > nodes[b].v: swap(mut a, mut b)
    let old_l = nodes[a].l
    nodes[a].l = meld(nodes[a].r, some(b))
    nodes[a].r = old_l
    some(a)

var heaps: [0..n]?Int = none
for i in 0..n: heaps[i] = new_node(deg[i], 1)

var par: [0..n]Int = -1

fn find_root(start: Int) -> Int:
    var a = start
    while par[a] >= 0:
        if par[par[a]] >= 0: par[a] = par[par[a]]
        a = par[a]
    a

for (nw, u0, v0) in edges:
    let w = -nw
    var u = find_root(u0)
    var v = find_root(v0)
    if par[u] > par[v]: swap(mut u, mut v)
    par[u] += par[v]
    par[v] = u
    heaps[u] = meld(heaps[u], heaps[v])
    heaps[v] = none
    deg[u] += deg[v] - 2 * w

    let top = heaps[u]!
    if deg[u] < nodes[top].v:
        var cur_pos = nodes[top].v
        var cur_slope = 0
        var cur_val = 0
        while true:
            let cur_h = heaps[u]
            let done = if cur_h == none: true else: cur_val + cur_slope * (nodes[cur_h!].v - cur_pos) - nodes[cur_h!].v >= -deg[u]
            if done:
                let diff = cur_pos - cur_val - deg[u]
                let nx = cur_pos + diff / (cur_slope - 1)
                let gap = diff % (cur_slope - 1)
                if gap > 0: heaps[u] = meld(new_node(nx + 1, gap), heaps[u])
                heaps[u] = meld(new_node(nx, cur_slope - 1 - gap), heaps[u])
                heaps[u] = meld(new_node(deg[u], 1), heaps[u])
                break
            let h = cur_h!
            cur_val += cur_slope * (nodes[h].v - cur_pos)
            cur_pos = nodes[h].v
            cur_slope += nodes[h].cnt
            heaps[u] = meld(nodes[h].l, nodes[h].r)

var deltas: List<Int> = []
let rt = find_root(0)
while let h = heaps[rt]:
    for _ in 0..nodes[h].cnt: deltas.push(nodes[h].v)
    heaps[rt] = meld(nodes[h].l, nodes[h].r)

var pref: [0..=n]Int = 0
for i in 0..n: pref[i + 1] = pref[i] + deltas[i]

let q: Int = read()
for _ in 0..q:
    let t: Int = read()
    let cnt = bsearch_first(0..n, i => deltas[i] >= t) ?? n
    print(t * cnt - pref[cnt])
