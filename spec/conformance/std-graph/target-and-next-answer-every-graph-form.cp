// expect: ok, output "2 1 1\n2 5 1\nYES 1 0"
// rule: std.graph.Next
// `Next` and `Target` are std impls on every graph form: a table of lists, a fn, weighted rows, a domain, `Int` and a product position.
use std.graph.{Next, Target}
domain V = 0..3
fn deg<G: Next<V>>(g: G, u: V) -> Int:
    var n = 0
    for _ in g.out(u): n += 1
    n
let g: [V]List<V> = [[2, 1], [], [0]]
let f = (u: V) => g[u]
let w: [V]List<(to: V, w: Int)> = [[(to: V(1), w: 5)], [], []]
print(deg(g, V(0)), deg(f, V(2)), deg(w, V(0)))
print(V(2).to, 5.to, w[V(0)][0].to)
domain R = 0..2
domain P = (R, R)
let p = P(R(1), R(0))
print(yes(p.to == p), p.to)
