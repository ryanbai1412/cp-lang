// Tuned: time_tree.cp's offline dynamic connectivity with each segment-tree
// node's edges in one contiguous list, as time_tree.cpp lays them out.
// `TimeTree` chains a node's events through one shared `next` list, so `run`
// visits them scattered across memory (perf PLAN.md, general fixes owed).
let n = 30_000
let m = 60_000
let q = 30_000
domain V = 0..n
var seed = 123456789
fn next_rng() -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed

let size = bit_ceil(q)
var tree: List<List<(V, V)>> = List.filled(2 * size, [])
for _ in 0..m:
    let a = next_rng() % q
    let b = next_rng() % q
    let u = V(next_rng() % n)
    let v = V(next_rng() % n)
    var (l, r) = (min(a, b) + size, max(a, b) + 1 + size)
    while l < r:
        if l % 2 == 1:
            tree[l].push((u, v))
            l += 1
        if r % 2 == 1:
            r -= 1
            tree[r].push((u, v))
        l /= 2
        r /= 2

var dsu = Dsu<V>(rollback: true)
var res: List<Int> = []
fn dfs(node: Int, nl: Int, nr: Int):
    let snap = dsu.mark()
    for (a, b) in tree[node]: dsu.unite(a, b)
    if nr - nl == 1:
        if nl < q: res.push(dsu.parts)
    else:
        let mid = (nl + nr) / 2
        dfs(2 * node, nl, mid)
        if mid < q: dfs(2 * node + 1, mid, nr)
    dsu.undo_to(snap)

dfs(1, 0, size)
var acc = 0
for x in res: acc += x
print(acc)
