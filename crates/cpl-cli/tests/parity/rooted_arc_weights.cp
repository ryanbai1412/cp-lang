use std.graph.{arc_len}
domain V = 1..=3
domain E = 1..=2
let es: [E](u: V, v: V, w: Int) = [(u: 1, v: 2, w: 5), (u: 2, v: 3, w: 12)]
let t = Rooted(Graph.undirected(es), 1)
print(t.dist(1, 3), t.w(3), t.w(1))
let adj: [V]List<V> = [[2], [1, 3], [2]]
let u = Rooted(adj, 1)
print(u.dist(1, 3), u.w(3))
print(t.diameter().len)
