// expect: ok, output "YES"
use std.tree.{Centroids}
domain V = 0..2
let adj: [V]List<V> = [[V(1)], [V(0)]]
let cd = Centroids(adj)
print(yes(cd.parent(cd.root) == None))
