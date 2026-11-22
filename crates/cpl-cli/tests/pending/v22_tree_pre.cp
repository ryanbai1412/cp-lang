// `tree.Pre`, the preorder domain a `Rooted` tree owns, as a type
// argument, an array head and a loop; and `read::<Int>()` in an expression.
use std.graph.{Graph}
use std.tree.{Rooted}
use std.fenwick.{Fenwick}
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
let edges: [E](u: V, v: V) = [E] _ => (u: V(read::<Int>() - 1), v: V(read::<Int>() - 1))
let g = Graph.undirected(edges)
let tree = Rooted(g, V.first)
var bit: Fenwick<tree.Pre, Int> = Fenwick()
var on: [tree.Pre]Bool = false
for u in V:
    bit.add(tree.pos(u), u.idx + 1)
    on[tree.pos(u)] = true
for u in V:
    print(bit.sum(tree.sub(u)))
print(count(on[p] for p in tree.Pre))
