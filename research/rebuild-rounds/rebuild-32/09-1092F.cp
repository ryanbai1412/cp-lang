// cf1092f Tree with Maximum Cost: reroot (subtree weight, cost) pairs; moving one edge
// away adds the far side's whole weight to its cost.
let n: 1..=2e5 = read()
domain V = 1..=n
let a = read<[V]1..=2e5>()
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: V.first!)
let at = t.reroot(
    Monoid<(w: Int, cost: Int)>.of(each(w: +, cost: +)),
    lift: (x, _, _) => (w: x.w, cost: x.cost + x.w),
    finish: (s, v) => (w: s.w + a[v], cost: s.cost),
)
print(max(r.cost for r in at)!)
