use std.tree.{Rooted}
domain V = 1..=3
let t = Rooted(Graph.undirected(read<Tree<V>>()), V.first!)
print(t.size(V.first!))
