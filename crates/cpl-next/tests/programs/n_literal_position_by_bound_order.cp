// expect: ok, output "2\n2\n1\n1"
// rule: R04.2.18
// Which domain makes a literal a position is the signature's bound, not the arguments' order:
// naming them in either order gives the same verdict.
use std.graph.{Next}
fn f<V: Domain>(next: Next<V>, root: V) -> Int = Int(root)
fn g<V: Domain>(root: V, next: Next<V>) -> Int = Int(root)
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
print(f(adj, root: 2))
print(f(root: 2, next: adj))
print(g(1, adj))
print(g(next: adj, root: 1))
