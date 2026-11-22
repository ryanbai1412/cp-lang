// expect: ok, output "YES YES\nYES\nYES"
use std.tree.{Centroids}
domain V = 0..5
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let cd = Centroids(adj)
print(yes(cd.root == 1 and cd.parent(2) == Some(0) and cd.parent(1) == None), yes(cd.depth(3) == 1 and cd.depth(2) == 2))
print(yes(cd.ancestors(2).collect() == [(c: 2, dist: 0), (c: 0, dist: 1), (c: 1, dist: 2)]))
print(yes(cd.piece(0).collect() == [(v: 0, dist: 0, branch: 0), (v: 2, dist: 1, branch: 2)] and cd.piece(1).count() == 5))
