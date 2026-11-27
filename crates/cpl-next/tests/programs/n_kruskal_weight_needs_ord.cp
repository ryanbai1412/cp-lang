// expect: E0606
// rule: R08.1.5
// A Kruskal tree orders its edges by weight, so `kruskal_tree` bounds `W: Ord`, and a weight
// type with no `Ord` is reported at the call, not inside std.
use std.tree.*
struct P(f: Int -> Int)
domain V = 0..4
let es: [0..3](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3), (u: 2, v: 3, w: 9)]
let (domain N, kt) = kruskal_tree(es, w: e => P(x => x + es[e].w)) // <- here
print(yes(kt.leaf[0] == kt.leaf[1]))
