// expect: ok, output "3\n3"
// rule: std.graph.Next
use std.graph.{Next}
fn arcs<V>(g: Next<V>) -> Int:
    var t = 0
    for v in V: t += g.out(v).count()
    t
domain N = 0..3
let adj: [N]List<N> = [[N(1)], [N(0), N(2)], []]
print(arcs::<N>(u => adj[u]))
print(arcs(adj))
