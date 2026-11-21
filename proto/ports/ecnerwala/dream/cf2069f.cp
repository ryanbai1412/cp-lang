// Codeforces 2069 F: Graph Inclusion (dream).
// Making every component of B lie inside one of A takes
//   answer(t) = comps(A) - comps(A ∪ B)
// added edges, with both graphs as they stand after query t. Edges come and go, so
// work offline: each edge is present on a few ranges of T; `over_time` puts every
// range on O(log q) segment-tree nodes over T and walks the tree, adding edges on
// the way down and rolling the DSUs back on the way up.
use std.seq.toggles
use std.timetree.over_time

domain V = read(2..=4e5)
domain T = read(1..=4e5)
let qs: [T](graph: Char, u: V, v: V) = read()

let present = qs.toggles(key: q => (q.graph, minmax(q.u, q.v)))

var dsu = (a: Dsu<V>(), ab: Dsu<V>())
fn add(q: (graph: Char, u: V, v: V)):
    if q.graph == 'A': dsu.a.unite(q.u, q.v)
    dsu.ab.unite(q.u, q.v)

let ans = over_time(present, add, answer: t => dsu.a.components - dsu.ab.components, rollback: mut dsu)
print(ans, sep: "\n")
