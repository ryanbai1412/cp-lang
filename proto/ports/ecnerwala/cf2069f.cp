// Codeforces 2069 F: A includes B after adding the fewest edges exactly when
// every edge of B joins two vertices that A already connects, so the answer
// is components(A) - components(A + B). Offline: each edge lives on a range
// of query times; TimeTree.run walks the segment tree over time and rolls the
// two union-finds back on exit from each node.
use std.dsu.{Mark}
use std.timetree.{TimeTree}

let (n, q): (2..=4e5, 1..=4e5) = read()
domain V = 1..=n
domain T = 0..q
let qs: [T](g: Char, x: V, y: V) = read()

struct Edge(in_a: Bool, x: V, y: V)
var tt = TimeTree<Edge>(q)
var since: Map<(Char, V, V), T> = Map()
for t in T:
    let (g, x, y) = qs[t]
    let key = (g, min(x, y), max(x, y))
    if let s = since.remove(key): tt.add_interval(s.ord, t.ord, Edge(g == 'A', x, y))
    else: since[key] = t
for ((g, x, y), s) in since: tt.add_interval(s.ord, q, Edge(g == 'A', x, y))

var (a, ab) = (Dsu<V>(), Dsu<V>())
fn add(e: Edge):
    if e.in_a: a.unite(e.x, e.y)
    let _ = ab.unite(e.x, e.y)
fn rollback(m: (Mark, Mark)):
    a.undo_to(m.0)
    ab.undo_to(m.1)

let ans = tt.run(
    mark: () => (a.mark(), ab.mark()),
    apply: add,
    answer: _ => a.components - ab.components,
    undo: rollback,
)
print(ans, sep: "\n")
