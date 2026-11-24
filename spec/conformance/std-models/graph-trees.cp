// expect: ok, output "ok"
// input: "1 300\n"
// model: spanning_tree
// model: dfs_tree
// model: from_prufer
// brute: prufer
// Seeded random connected graphs (one per op) against a breadth-first and a depth-first search
// over the same arcs in input order: spanning_tree's parents and dfs_tree's parents, depths and
// edge mask; and from_prufer of a seeded random code against the textbook decoding, and back
// through prufer. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.tree.{prufer, from_prufer}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..9
domain E = 0..12
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    var perm = [V(i) for i in 0..9]
    rng.shuffle(mut perm)
    var ends: List<(V, V)> = []
    for i in 1..9: ends.push((perm[i], perm[rng.int(0, i - 1)]))
    for _ in 0..4:
        let a = rng.int(0, 8)
        ends.push((V(a), V((a + rng.int(1, 8)) % 9)))
    rng.shuffle(mut ends)
    let es: [E](u: V, v: V) = [E] e => (u: ends[e.ord].0, v: ends[e.ord].1)
    var adj: [V]List<(V, E)> = []
    for e in E:
        adj[es[e].u].push((es[e].v, e))
        adj[es[e].v].push((es[e].u, e))
    let g = Graph.undirected(es)
    var bp: [V]?V = None
    var seen: [V]Bool = false
    seen[V(0)] = true
    var q = [V(0)]
    var i = 0
    while i < q.len:
        let x = q[i]
        i += 1
        for (y, _) in adj[x]:
            if not seen[y]:
                seen[y] = true
                bp[y] = Some(x)
                q.push(y)
    let st = g.spanning_tree()
    let root = V(rng.int(0, 8))
    var dp: [V]?V = None
    var used: [E]Bool = false
    var on: [V]Bool = false
    on[root] = true
    var stack: List<(V, Int)> = [(root, 0)]
    while let (x, j) = stack.pop():
        if j < adj[x].len:
            stack.push((x, j + 1))
            let (y, e) = adj[x][j]
            if not on[y]:
                on[y] = true
                dp[y] = Some(x)
                used[e] = true
                stack.push((y, 0))
    let (tree, tree_edge) = g.dfs_tree(root)
    if V.any(v => st.parent(v) != bp[v]): bad = "op {k}: spanning_tree"
    elif V.any(v => tree.parent(v) != dp[v]) or tree_edge != used: bad = "op {k}: dfs_tree from {root}"
    if bad != "":
        break
    let code = [V(rng.int(0, 8)) for _ in 0..7]
    var deg: [V]Int = 1
    for c in code: deg[c] += 1
    var want: List<(u: V, v: V)> = []
    for c in code:
        let leaf = V.find(v => deg[v] == 1)!
        want.push((u: leaf, v: c))
        deg[leaf] -= 1
        deg[c] -= 1
    let last = [v for v in V if deg[v] == 1]
    want.push((u: last[0], v: last[1]))
    let tr = from_prufer<V>(code)
    if [e for e in tr] != want or prufer(tr) != code: bad = "op {k}: from_prufer"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
