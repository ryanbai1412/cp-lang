// expect: E0902
// rule: R04.2.18
// A literal past the domain its bound fixes is never a position of it, whichever order the
// arguments come in.
use std.graph.{Next}
fn g<V: Domain>(root: V, next: Next<V>) -> Int = Int(root)
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
print(g(next: adj, root: 3)) // <- here
