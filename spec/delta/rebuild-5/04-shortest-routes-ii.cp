// CSES 1672 Shortest Routes II: all-pairs shortest paths on an undirected weighted graph.
// Input: n m q, m lines "a b c", q lines "a b". Output: per query the distance, or -1.
let (n, m, q) = read<(1..=500, 1..=250_000, 1..=1e5)>()
domain V = 1..=n
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1e9)>())

var d: [V, V]?Int = g.matrix()
let _ = floyd_warshall(mut d)
repeat q:
    let (a, b) = read<(V, V)>()
    print(d[a, b] ?? -1)
