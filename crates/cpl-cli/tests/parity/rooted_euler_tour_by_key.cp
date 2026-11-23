// `by:` stably puts the key-true child (1) last; the first expectation here predated the walk's documented order.
use std.tree.{Rooted}
domain V = 0..4
let adj: [V]List<V> = [[V(1), V(2)], [V(0), V(3)], [V(0)], [V(1)]]
let t = Rooted(adj, root: V(0))
print(t.euler_tour(by: c => c == V(1)).collect(), t.euler_tour().collect())
