// expect: ok, output "7 9"
domain V = 0..3
let g: [V]List<(to: V, w: Int)> = [[(to: 1, w: 4), (to: 2, w: 9)], [(to: 2, w: 3)], []]
print(dijkstra(g, from: [(V(0), 0)])[2]!, g.out(0).collect()[1].w)
