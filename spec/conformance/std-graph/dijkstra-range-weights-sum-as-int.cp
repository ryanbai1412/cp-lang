// expect: ok, output "2000000000"
// rule: std.graph.dijkstra
// input: "3 2\n1 2 1000000000\n2 3 1000000000\n"
// A weight read as `1..=1e9` is a read contract; distances add as `Int` past 1e9.
domain V = 1..=read<1..=2e5>()
let m = read<0..=2e5>()
let g = Graph.directed(read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p[V.last!] ?? -1)
