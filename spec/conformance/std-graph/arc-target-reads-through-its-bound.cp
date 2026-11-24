// expect: ok, output "2 3"
// `a.to` on `a: G.Arc` reads through the associated type's bound `Arc: Target<V>`.
use std.graph.{Next}
domain V = 0..4
fn targets<V, G: Next<V>>(g: G, u: V) -> List<V>:
    var out = List<V>()
    for a in g.out(u):
        out.push(a.to)
    out
let g: [V]List<V> = [[2, 3], [], [], []]
let t = targets(g, V(0))
print("{t[0].ord} {t[1].ord}")
