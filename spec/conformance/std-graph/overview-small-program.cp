// expect: ok, output "12"
// input: "3 2\n1 2 5\n2 3 7\n"
domain V = 1..=read<1..=2e5>()
let m = read<0..=2e5>()
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p.dist[V.last!] ?? -1)
