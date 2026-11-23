// CSES 1671 Shortest Routes I
// in: n m, then m one-way flights a b c; out: the shortest distance from city 1 to each city 1..n
let (n, m) = read<(Int, Int)>()
domain City = 1..=n
let g = Graph.directed(read<[m](u: City, v: City, w: 1..=1e9)>())
let p = dijkstra(g, from: [(City.first!, 0)])
print([City] v => p[v]!)
