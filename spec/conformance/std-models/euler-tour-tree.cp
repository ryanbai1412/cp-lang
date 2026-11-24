// expect: ok, output "ok"
// input: "1 300\n"
// model: EulerTourTree
// model: LazyEulerTourTree
// An EulerTourTree over + and a LazyEulerTourTree with add tags against an adjacency list and
// plain value arrays: seeded random link, cut, set, apply_tree and apply_sub, with connected,
// len, get, fold_tree and fold_sub checked after each op. Reads `seed n`.
use std.rand.{Rng}
use std.tree.{EulerTourTree, LazyEulerTourTree}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..9
var rng = Rng(seed0)
fn side(adj: [V]List<V>, s: V, block: V) -> List<V>:
    var seen: [V]Bool = false
    seen[s] = true
    var q = [s]
    var i = 0
    while i < q.len:
        let x = q[i]
        i += 1
        for y in adj[x]:
            if not seen[y] and not (x == s and y == block):
                seen[y] = true
                q.push(y)
    q
var w1: [V]Int = [V] _ => rng.int(-5, 5)
var w2 = w1.clone()
var et = EulerTourTree(+, a: Some(w1))
var lz = LazyEulerTourTree<V, Int, Int>.new(add, Some(w2))
var adj: [V]List<V> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 5)
    let (u, v) = (V(rng.int(0, 8)), V(rng.int(0, 8)))
    let x = rng.int(-5, 5)
    if op <= 1 and not side(adj, u, u).contains(v):
        et.link(u, v)
        lz.link(u, v)
        adj[u].push(v)
        adj[v].push(u)
    elif op == 2 and adj[u].len > 0:
        let y = adj[u][rng.int(0, adj[u].len - 1)]
        if rng.int(0, 1) == 0: et.cut(u, y) else: et.cut(y, u)
        if rng.int(0, 1) == 0: lz.cut(u, y) else: lz.cut(y, u)
        adj[u] = [z for z in adj[u] if z != y]
        adj[y] = [z for z in adj[y] if z != u]
    elif op == 3:
        et.set(u, x)
        lz.set(u, x)
        w1[u] = x
        w2[u] = x
    elif op == 4:
        lz.apply_tree(u, x)
        for z in side(adj, u, u): w2[z] += x
    elif adj[u].len > 0:
        let y = adj[u][rng.int(0, adj[u].len - 1)]
        lz.apply_sub(u, y, x)
        for z in side(adj, u, y): w2[z] += x
    let comp = side(adj, u, u)
    if et.connected(u, v) != comp.contains(v) or lz.connected(u, v) != comp.contains(v): bad = "op {k}: connected {u} {v}"
    elif et.len(u) != comp.len or lz.len(u) != comp.len: bad = "op {k}: len {u}"
    elif et.get(u) != w1[u] or lz.get(u) != w2[u]: bad = "op {k}: get {u}"
    elif et.fold_tree(u) != sum(w1[z] for z in comp) or lz.fold_tree(u) != sum(w2[z] for z in comp): bad = "op {k}: fold_tree {u}"
    elif adj[u].len > 0:
        let y = adj[u][0]
        let s = side(adj, u, y)
        if et.fold_sub(u, y) != sum(w1[z] for z in s) or lz.fold_sub(u, y) != sum(w2[z] for z in s): bad = "op {k}: fold_sub {u} {y}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
