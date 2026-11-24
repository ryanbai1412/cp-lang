// expect: ok, output "YES 5 7 1 5 2 7"
// rule: std.tree.Rooted.up_arc
// `up_arc` and `child_arcs` hand back the arcs `next` gave, with their edge fields, so a weighted
// tree reads its edge weights from the arcs and builds no `[V]` side array.
use std.graph.{Graph}
use std.tree.{Rooted}
domain V = 0..3
let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 7)]
let t = Rooted(Graph.undirected(es), root: V(0))
let kids = [[(a.to.ord, a.w) for a in t.child_arcs(v)] for v in V]
print(yes(t.up_arc(V(0)) == None), t.up_arc(V(1))!.w, t.up_arc(V(2))!.w, kids[0], kids[1], kids[2])
