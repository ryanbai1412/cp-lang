// expect: ok, output "YES YES 1"
use std.tree.{Centroids}
domain V = 0..1
let adj: [V]List<V> = [[]]
let cd = Centroids(adj)
print(yes(cd.root == V(0)), yes(cd.parent(V(0)) == None), cd.piece(V(0)).count())
