// Codeforces 2069 F: A includes B after adding the fewest edges exactly when
// every edge of B joins two vertices that A already connects, so the answer
// is components(A) - components(A + B). Offline: each edge lives on a range
// of query times; split the time range in halves, apply the edges covering a
// half to two rollback union-finds, pass the partial ones down, and undo.
let (n, q): (2..=4e5, 1..=4e5) = read()
domain V = 1..=n
domain T = 0..q
let qs: [T](g: Char, x: V, y: V) = read()

struct Edge(on: Range<T>, in_a: Bool, x: V, y: V)
var edges: List<Edge> = []
var since: Map<(Char, V, V), T> = Map()
for t in T:
    let (g, x, y) = qs[t]
    let key = (g, min(x, y), max(x, y))
    if let s = since.remove(key): edges.push(Edge(s.l..t.l, g == 'A', x, y))
    else: since[key] = t
for ((g, x, y), s) in since: edges.push(Edge(s.l..T.last.r, g == 'A', x, y))

var (a, ab) = (Dsu<V>(), Dsu<V>())
var ans: [T]Int = 0
fn solve(span: Range<T>, live: List<Edge>):
    let snap = (a.mark(), ab.mark())
    var rest: List<Edge> = []
    for e in live:
        if e.on.lo <= span.lo and span.hi <= e.on.hi:
            if e.in_a: a.unite(e.x, e.y)
            ab.unite(e.x, e.y)
        elif e.on.lo < span.hi and span.lo < e.on.hi:
            rest.push(e)
    if span.len == 1:
        ans[span.first!] = a.components - ab.components
    else:
        let m = span.mid!
        solve(span.lo..m.r, rest)
        solve(m.r..span.hi, rest)
    a.undo_to(snap.0)
    ab.undo_to(snap.1)
solve(T.first.l..T.last.r, edges)
print(ans, sep: "\n")
