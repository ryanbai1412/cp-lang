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
let down_f: (Int, V) -> Int = (d, v) => d + 1 // within: tree.Rooted.down
let up_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.up
let rr_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.reroot
let ab_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.above
var s = 0
for _ in 0..12:
    s += t.down(0, down_f)[V(n - 1)] // timed: tree.Rooted.down
    s += t.up(+, finish: up_f)[V(0)] // timed: tree.Rooted.up
    s += t.reroot(+, finish: rr_f)[V(0)] // timed: tree.Rooted.reroot
    s += t.above(+, finish: ab_f)[V(1)] ?? 0 // timed: tree.Rooted.above
for _ in 0..40:
    s += t.diameter().len // timed: tree.Rooted.diameter
    s += t.layers().pos(V(1)).ord // timed: tree.Rooted.layers
print(s)
