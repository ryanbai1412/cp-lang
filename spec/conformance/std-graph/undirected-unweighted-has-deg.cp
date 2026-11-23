// expect: ok, output "2 2"
// input: "1 2\n2 3\n"
// A graph from unweighted edges has `deg` and `next` like a weighted one (round 25, F1).
domain V = 1..=3
let g = Graph.undirected(read<[2](u: V, v: V)>())
print(g.deg(2), g.next(2).count())
