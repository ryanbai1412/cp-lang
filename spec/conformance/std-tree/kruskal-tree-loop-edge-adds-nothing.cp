// expect: ok, output "3"
use std.tree.{kruskal_tree}
domain V = 0..2
let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 1, w: 1)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
print(N.count)
