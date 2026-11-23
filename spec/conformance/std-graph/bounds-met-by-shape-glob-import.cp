// expect: ok, output "2 3"
// A graph by shape (`[V]List<V>`, `Graph`) meets `Next`/`WeightedNext` also when a glob
// import brings the trait names into scope.
use std.graph.*
domain V = 0..4
domain E = 0..3
let g: [V]List<V> = [[1, 2], [3], [3], []]
let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 0, v: 2, w: 1), (u: 2, v: 1, w: 2)]
print(bfs(g, from: [V(0)])[3]!, dijkstra(Graph.directed(es), from: [(V(0), 0)])[1]!)
