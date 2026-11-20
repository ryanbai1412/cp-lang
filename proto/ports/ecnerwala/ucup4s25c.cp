// UCup 4 Stage 25 (Zagreb) C: build the directed graph of ordered adjacent
// pairs across all 6 permutations of each input triple, run Tarjan SCC, and
// check whether any intra-SCC edge violates DFS depth + 1 (mod 3).
@!multitest
let m: Int = read()
var verts: Map<(Int, Int), Int> = Map()
var edges: List<(Int, Int)> = []

fn get_id(mut mp: Map<(Int, Int), Int>, k: (Int, Int)) -> Int:
    if let id = mp.get(k): return id
    let id = mp.len
    mp[k] = id
    id

for _ in 0..m:
    let (a, b, c): (Int, Int, Int) = read()
    let perms = [(a, b, c), (a, c, b), (b, a, c), (b, c, a), (c, a, b), (c, b, a)]
    for (x, y, z) in perms:
        let u = get_id(mut verts, (x, y))
        let v = get_id(mut verts, (y, z))
        edges.push((u, v))

let n = verts.len
var adj: List<List<Int>> = List.filled(n, [])
for (u, v) in edges: adj[u].push(v)

var depth: [0..n]Int = 0
var val: [0..n]Int = 0
var comp: [0..n]?Int = none
var stk: List<Int> = []
var timer = 0
var ncomps = 0

fn dfs(j: Int, d: Int) -> Int:
    depth[j] = d
    timer += 1
    val[j] = timer
    var low = timer
    stk.push(j)
    for e in adj[j]:
        if comp[e] == none:
            let nxt = if val[e] != 0: val[e] else: dfs(e, d + 1)
            low min= nxt
    if low == val[j]:
        while true:
            let x = stk.pop()!
            comp[x] = ncomps
            if x == j: break
        ncomps += 1
    val[j] = low
    low

for i in 0..n:
    if comp[i] == none: dfs(i, 0)

var found = false
for (u, v) in edges:
    if comp[u] != comp[v]: continue
    if depth[v] % 3 != (depth[u] + 1) % 3: found = true
print(if found: "yes" else: "no")
