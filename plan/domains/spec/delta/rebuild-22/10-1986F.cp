// cf1986F Non-academic Problem: only a bridge disconnects. Shrink two-edge-connected
// components into the bridge tree; cutting the tree edge above component c splits the
// vertices into below[c] and n - below[c].
use std.graph.{two_edge_components}
@!multitest
let (n, m): (2..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain E = 0..m
let g = Graph.undirected(read<[E](u: V, v: V)>())
let (domain C, comp) = two_edge_components(g)
var weight: [C]Int = 0
for v in V: weight[comp[v]] += 1
let t = Rooted(condense(g, comp), root: comp[V.first!])
let below = t.up(+, finish: (s, c) => s + weight[c])
fn pairs(k: Int) -> Int = k * (k - 1) / 2
print(C.map(c => if t.parent(c) == None: pairs(n) else: pairs(below[c]) + pairs(n - below[c])).min()!)
