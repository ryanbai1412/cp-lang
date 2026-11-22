// expect: ok, output "YES YES"
use std.tree.{Centroids}
domain V = 0..1
let adj: [V]List<V> = [[]]
let cd = Centroids(adj)
print(yes(cd.root == V(0)), yes(cd.parent(V(0)) == None))
