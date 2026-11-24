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
let t = Rooted(adj, root: V(0))
var s = 0
for _ in 0..12:
    for u in t.subtree(V(0)): s += u.ord // timed: tree.Rooted.subtree
    for p in t.preorder(V(0)): s += p.depth // timed: tree.Rooted.preorder
    s += t.virtual([V(v) for v in 0..n by 4]).at.len // timed: tree.Rooted.virtual
for _ in 0..16:
    for e in t.tour: // timed: tree.Rooted.tour
        s += 1 // timed: tree.Rooted.tour
    for v in t.euler_tour(): // timed: tree.Rooted.euler_tour
        s += v.ord // timed: tree.Rooted.euler_tour
print(s)
