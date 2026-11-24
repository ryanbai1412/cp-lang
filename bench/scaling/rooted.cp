// let: k = n; depth = n; children = n; size = n; path = n
use std.rand.{Rng}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let ppar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(v.ord - 1))
var adj: [V]List<V> = []
for v in V:
    if let p = rpar[v]:
        adj[v].push(p)
        adj[p].push(v)
domain E = 0..n - 1
let es: [E](u: V, v: V) = [E] e => (u: V(0), v: V(e.ord + 1))
let none: [V]List<V> = []
let t = Rooted(adj, root: V(0))
let pt = Rooted.from_parents(ppar)
let st = Rooted(Graph.undirected(es), root: V(0))
let ft = Rooted.forest(none)
let down_f: (Int, V) -> Int = (d, v) => d + 1 // within: tree.Rooted.down
let up_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.up
let rr_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.reroot
let ab_f: (Int, V) -> Int = (x, v) => x + 1 // within: tree.Rooted.above
let ex_f: V -> Int = c => c.ord // within: tree.Rooted.excl_fold
var s = 0
for _ in 0..12:
    s += Rooted(adj, root: V(0)).size(V(0)) // timed: tree.Rooted.new
    s += Rooted.from_parents(rpar).size(V(0)) // timed: tree.Rooted.from_parents
    s += Rooted.try_from_parents(rpar)!.size(V(0)) // timed: tree.Rooted.try_from_parents
    s += Rooted.forest(adj).size(V(0)) // timed: tree.Rooted.forest
    s += t.down(0, down_f)[V(n - 1)] // timed: tree.Rooted.down
    s += t.up(+, finish: up_f)[V(0)] // timed: tree.Rooted.up
    s += t.reroot(+, finish: rr_f)[V(0)] // timed: tree.Rooted.reroot
    s += t.above(+, finish: ab_f)[V(1)] ?? 0 // timed: tree.Rooted.above
    s += t.shapes().of[V(0)].ord // timed: tree.Rooted.shapes
    s += t.events().enter(V(1)).ord // timed: tree.Rooted.events
    s += t.long_paths().slot(V(0), 0).ord // timed: tree.Rooted.long_paths
    for u in t.subtree(V(0)): s += u.ord // timed: tree.Rooted.subtree
    for p in t.preorder(V(0)): s += p.depth // timed: tree.Rooted.preorder
    s += t.virtual([V(v) for v in 0..n by 4]).at.len // timed: tree.Rooted.virtual
for _ in 0..40:
    s += t.diameter().len // timed: tree.Rooted.diameter
    s += t.layers().pos(V(1)).ord // timed: tree.Rooted.layers
    s += pt.path(V(0), V(n - 1)).len // timed: tree.Rooted.path
    for c in st.excl_fold(V(0), +, ex_f): s += c.rest // timed: tree.Rooted.excl_fold
    for c in st.light(V(0)): s += c.ord // timed: tree.Rooted.light
for _ in 0..400:
    s += t.reorder(rpar).len // timed: tree.Rooted.reorder
    s += pt.centroid().ord // timed: tree.Rooted.centroid
    s += pt.centroids().len // timed: tree.Rooted.centroids
    s += t.subtree_sizes[V(n - 1)] // timed: tree.Rooted.subtree_sizes
for _ in 0..3000:
    s += t.order.len // timed: tree.Rooted.order
for _ in 0..16:
    for r in ft.roots: // timed: tree.Rooted.roots
        s += r.ord // timed: tree.Rooted.roots
    for e in t.tour: // timed: tree.Rooted.tour
        s += 1 // timed: tree.Rooted.tour
    for v in t.euler_tour(): // timed: tree.Rooted.euler_tour
        s += v.ord // timed: tree.Rooted.euler_tour
    for a in pt.ancestors(V(n - 1)): // timed: tree.Rooted.ancestors
        s += a.ord // timed: tree.Rooted.ancestors
    for c in st.children(V(0)): // timed: tree.Rooted.children
        s += c.ord // timed: tree.Rooted.children
    for a in st.child_arcs(V(0)): // timed: tree.Rooted.child_arcs
        s += a.to.ord // timed: tree.Rooted.child_arcs
print(s)
