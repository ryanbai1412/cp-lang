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
    s += t.shapes().of[V(0)].ord // timed: tree.Rooted.shapes
    s += t.events().enter(V(1)).ord // timed: tree.Rooted.events
    s += t.long_paths().slot(V(0), 0).ord // timed: tree.Rooted.long_paths
print(s)
