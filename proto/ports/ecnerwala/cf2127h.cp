type Edge = [0..3, 0..3]Int
type Vert = [0..3]Int

let init_e: Edge = [[0, 0, 0], [0, 1, 1], [0, 1, 1]]

fn edge_key(a: Int, b: Int) -> (Int, Int) = if a < b: (a, b) else: (b, a)

fn rev_edge(e: Edge) -> Edge = [0..3, 0..3] (i, j) => e[j, i]

fn merge_parallel(a: Edge, b: Edge) -> Edge:
    var r: Edge = 0
    for ai in 0..3:
        for bi in 0..3 - ai:
            for aj in 0..3:
                for bj in 0..3 - aj:
                    r[ai + bi, aj + bj] max= a[ai, aj] + b[bi, bj]
    r

fn merge_series(a: Edge, v: Vert, b: Edge) -> Edge:
    var r: Edge = 0
    for ai in 0..3:
        for aj in 0..3:
            for bj in 0..3 - aj:
                for bk in 0..3:
                    r[ai, bk] max= a[ai, aj] + v[2 - (aj + bj)] + b[bj, bk]
    r

fn merge_leaf(v: Vert, e: Edge, l: Vert) -> Vert:
    var r: Vert = 0
    for a in 0..3:
        for i in 0..3 - a:
            for j in 0..3:
                r[a + i] max= v[a] + e[i, j] + l[2 - j]
    r[1] max= r[0]
    r[2] max= r[1]
    r

@!multitest
let (n, m): (Int, Int) = read()
var mp: Map<(Int, Int), Edge> = Map()
var adj: [1..=n]Set<Int> = Set()
var vert: [1..=n]Vert = 0

for _ in 0..m:
    let (u, v): (Int, Int) = read()
    mp[edge_key(u, v)] = init_e
    adj[u].add(v)
    adj[v].add(u)

var in_q: [1..=n]Bool = false
var q: Deque<Int> = Deque()
fn enqueue(v: Int):
    if not in_q[v] and adj[v].len <= 2:
        in_q[v] = true
        q.push(v)

fn pop_edge(a: Int, b: Int) -> Edge:
    let raw = mp.remove(edge_key(a, b))!
    if a < b: raw else: rev_edge(raw)

for v in 1..=n: enqueue(v)

while let cur = q.pop_front():
    if adj[cur].len == 2:
        let u = adj[cur].pop_first()!
        let v = adj[cur].pop_last()!
        adj[u].remove(cur)
        adj[v].remove(cur)
        let e = merge_series(pop_edge(u, cur), vert[cur], pop_edge(cur, v))
        if let old = mp.get((u, v)): mp[(u, v)] = merge_parallel(old, e)
        else: mp[(u, v)] = e
        adj[u].add(v)
        adj[v].add(u)
        enqueue(u)
        enqueue(v)
    elif adj[cur].len == 1:
        let o = adj[cur].pop_first()!
        adj[o].remove(cur)
        vert[o] = merge_leaf(vert[o], pop_edge(o, cur), vert[cur])
        enqueue(o)
    else:
        print(vert[cur][2])
