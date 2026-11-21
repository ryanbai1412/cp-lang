// UCup 4 Stage 25 (Zagreb) C: build the directed graph of ordered adjacent
// pairs across all 6 permutations of each input triple, run Tarjan SCC, and
// check whether any intra-SCC edge violates DFS depth + 1 (mod 3).
use std.seq.{get_or_insert_with}

@!multitest
let m: Int = read()
var verts: Map<(Int, Int), Int> = Map()
var raw_edges: List<(Int, Int)> = []

fn id(k: (Int, Int)) -> Int:
    let next_id = verts.len
    verts.get_or_insert_with(k, () => next_id)

for _ in 0..m:
    let (a, b, c): (Int, Int, Int) = read()
    let perms = [(a, b, c), (a, c, b), (b, a, c), (b, c, a), (c, a, b), (c, b, a)]
    for (x, y, z) in perms:
        raw_edges.push((id((x, y)), id((y, z))))

let n = verts.len
domain V = 0..n
var adj: [V]List<V> = []
for (u, v) in raw_edges: adj[V(u)].push(V(v))

var depth: [V]Int = 0
var val: [V]Int = 0
var comp: [V]?Int = none
var stk: List<V> = []
var timer = 0
var ncomps = 0

fn dfs(u: V, d: Int) -> Int:
    depth[u] = d
    timer += 1
    val[u] = timer
    var low = timer
    stk.push(u)
    for e in adj[u] if comp[e] == none:
        low min= if val[e] != 0: val[e] else: dfs(e, d + 1)
    if low == val[u]:
        while true:
            let x = stk.pop()!
            comp[x] = ncomps
            if x == u: break
        ncomps += 1
    val[u] = low
    low

for u in V if comp[u] == none: dfs(u, 0)

let bad = any(
    comp[V(u)] == comp[V(v)] and depth[V(v)] % 3 != (depth[V(u)] + 1) % 3 for (u, v) in raw_edges
)
print(if bad: "yes" else: "no")
