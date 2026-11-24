// Source: Codeforces 2069 F (Graph Inclusion), https://codeforces.com/contest/2069/problem/F
// limits: 5000 ms, 500 MB
// Problem: graphs A and B on n <= 4e5 vertices start empty; each of q <= 4e5 queries toggles an
//   edge in A or B. After each query print the fewest edges to add to A so that every
//   component of B lies inside a component of A.
// Solution: the answer is components(A) - components(A + B). Offline: each edge is present
//   over an interval of query times; a segment tree over time adds edges to two union-finds
//   with rollback on the way down and undoes them on the way up. O(q log q log n).
// Bounds: n: 2..=4e5; q: 1..=4e5; qs.x, qs.y: 1..=n
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
