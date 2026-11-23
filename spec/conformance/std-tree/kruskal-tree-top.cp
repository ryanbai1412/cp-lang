// expect: ok, output "YES\nYES\nYES"
// `kt.top(v, ok)` climbs from `v`'s leaf while each merging edge satisfies `ok`.
use std.tree.*
domain V = 0..4
let es: [0..3](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3), (u: 2, v: 3, w: 9)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
print(yes(kt.w[kt.top(0, e => es[e].w <= 5)] == Some(5)))
print(yes(kt.top(3, e => es[e].w <= 5) == kt.leaf[3]))
print(yes(kt.top(1, e => true) == kt.top(3, e => true)))
