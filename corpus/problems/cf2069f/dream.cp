// Codeforces 2069 F: Graph Inclusion (dream).
// Making every component of B lie inside one of A takes
//   answer(t) = comps(A) - comps(A ∪ B)
// added edges, with both graphs as they stand after query t. Edges come and go, so
// work offline: each edge is present on a few ranges of T; `over_time` puts every
// range on O(log q) segment-tree nodes over T and walks the tree, adding edges on
// the way down and rolling the DSUs back on the way up.
use std.timetree.{offline_over}

let (n, q): (2..=4e5, 1..=4e5) = read()
domain V = 1..=n
domain T = 0..q
let qs: [T](graph: Char, x: V, y: V) = read()

// each toggled edge lives from the query that adds it to the one that removes it, or the end
struct Edge(in_a: Bool, x: V, y: V, lo: Int, hi: Int)
var edges: List<Edge> = []
var since: Map<(Char, V, V), T> = Map()
for t in T:
    let (g, x, y) = qs[t]
    let key = (g, min(x, y), max(x, y))
    if let s = since.remove(key): edges.push(Edge(g == 'A', x, y, s.ord, t.ord))
    else: since[key] = t
for ((g, x, y), s) in since: edges.push(Edge(g == 'A', x, y, s.ord, q))

var (a, ab) = (Dsu<V>(), Dsu<V>())
fn add(e: Edge):
    if e.in_a: a.unite(e.x, e.y)
    let _ = ab.unite(e.x, e.y)

let ans = offline_over(
    q,
    edges,
    by: e => (e.lo, e.hi),
    mark: () => (a.mark(), ab.mark()),
    apply: add,
    answer: _ => a.components - ab.components,
    undo: m => (a.undo_to(m.0), ab.undo_to(m.1)),
)
print(ans, sep: "\n")
