// let: k = n; depth = n; children = n; size = n; path = n
use std.rand.{Rng}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
var adj: [V]List<V> = []
for v in V:
    if let p = rpar[v]:
        adj[v].push(p)
        adj[p].push(v)
let none: [V]List<V> = []
let t = Rooted(adj, root: V(0))
let ft = Rooted.forest(none)
var s = 0
for _ in 0..12:
    s += Rooted(adj, root: V(0)).size(V(0)) // timed: tree.Rooted.new
    s += Rooted.from_parents(rpar).size(V(0)) // timed: tree.Rooted.from_parents
    s += Rooted.try_from_parents(rpar)!.size(V(0)) // timed: tree.Rooted.try_from_parents
    s += Rooted.forest(adj).size(V(0)) // timed: tree.Rooted.forest
for _ in 0..400:
    s += t.reorder(rpar).len // timed: tree.Rooted.reorder
for k in 0..200_000:
    s += t.subtree_sizes[V(k % n)] // timed: tree.Rooted.subtree_sizes
    s += t.order[k % n].ord // timed: tree.Rooted.order
for _ in 0..16:
    for r in ft.roots: // timed: tree.Rooted.roots
        s += r.ord // timed: tree.Rooted.roots
print(s)
