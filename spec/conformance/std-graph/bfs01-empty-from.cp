// expect: ok, output "YES"
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 1)]
let empty: List<V> = []
let p = bfs01(Graph.directed(es), from: empty)
print(yes(p[V(1)] == None))
