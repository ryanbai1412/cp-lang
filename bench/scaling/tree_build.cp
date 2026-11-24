// n: 2048
// let: size = n
use std.rand.{Rng}
use std.tree.{prufer, from_prufer}
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
var rng = Rng(11)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)
var s = 0
let code = prufer(es)
for _ in 0..1600:
    s += from_prufer<V>(code).len // timed: tree.from_prufer
let g = Graph.undirected(es)
for _ in 0..100:
    s += g.spanning_tree().size(V(0)) // timed: tree.spanning_tree
    s += g.dfs_tree(V(0)).tree.size(V(0)) // timed: tree.dfs_tree
    s += prufer(es).len // timed: tree.prufer
print(s)
