// n: 2048
// let: size = n
use std.rand.{Rng}
use std.tree.{Centroids}
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
var rng = Rng(11)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)
var adj: [V]List<V> = []
for e in E:
    adj[es[e].u].push(es[e].v)
    adj[es[e].v].push(es[e].u)
var s = 0
let t = Rooted(adj, root: V(0))
for _ in 0..100:
    s += Centroids(adj).root.ord // timed: tree.Centroids.new
    s += LcaTable(t).n // timed: tree.LcaTable.new
let cd = Centroids(adj)
for _ in 0..100:
    for x in cd.piece(cd.root): s += x.dist // timed: tree.Centroids.piece
let lt = LcaTable(t)
for i in 0..64 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    s += (cd.parent(a) ?? a).ord // timed: tree.Centroids.parent
    s += cd.depth(a) // timed: tree.Centroids.depth
    for x in cd.ancestors(a): s += x.dist // timed: tree.Centroids.ancestors
    s += lt.lca(a, b).ord // timed: tree.LcaTable.lca
    s += lt.dist(a, b) // timed: tree.LcaTable.dist
    s %= 1000000007
print(s)
