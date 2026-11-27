// expect: ok, output "0\n1"
// rule: R04.2.18
// An integer literal for a parameter whose domain another argument's bound fixes is a position of it.
use std.graph.{Next}
fn f<V: Domain>(next: Next<V>, root: V) -> Int = Int(root)
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
print(f(adj, root: 0))
print(f(adj, 1))
