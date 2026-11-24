// expect: ok, output "1 5\n2 7\n12"
// rule: 06.4.1
// `g.weighted(f)` is std's `Reweighted`, whose `impl Next` yields each arc with `w: f(a)`,
// read directly and through a fn bounded by `Next`.
use std.graph.{Graph, Next}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2)]
let g = Graph.directed(es).weighted(a => 5 + 2 * a.e.ord)
for x in g.out(0): print(x.to.ord, x.w)
fn total<G: Next<V>>(h: G, u: V) -> Int:
    var s = 0
    for a in h.out(u): s += a.w
    s
print(total(g, 0))
