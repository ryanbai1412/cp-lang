// m arcs read into a list that only `Graph.directed` reads, then a 4m-cell
// table: the arcs are freed once the graph is built.
let (n, m): (1..=1_000_000, 1..=1_000_000) = read()
domain V = 0..n
let arcs = read<[m](V, V)>()
let g = Graph.directed(arcs)
var big: List<Int> = List.filled(4 * m, 0)
for i in 0..4 * m:
    big[i] = g[V.nth(i % n)].len + i
print(big[4 * m - 1] + big[0])
