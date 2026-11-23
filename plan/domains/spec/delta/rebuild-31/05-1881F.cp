// CF 1881F Minimum Maximum Distance
// far[v]: distance from v to its farthest marked vertex, for every root at
// once by rerooting; None when no marked vertex lies on that side.
@!multitest
let (n, k): (1..=2e5, 1..=n) = read()
domain V = 1..=n
let marks = read<[k]V>()
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
var marked: [V]Bool = false
for v in marks: marked[v] = true
let t = Rooted(g, root: V.first!)
let far = t.reroot(
    max,
    lift: (x: ?Int, _, _) => x.map(d => d + 1),
    finish: (s: ?Int, v) => if marked[v]: Some(s ?? 0) else: s,
)
print([far[v]! for v in V].min()!)
