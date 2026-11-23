// expect: ok, output "2"
// rule: 02.6.3
// input: "3\n0 1\n1 2\n"
// A custom constructor lends its parameters for the call only, so `Rooted` may take a temporary graph (rebuild round 6 H-ROOTED-TEMP).
domain V = 0..read<Int>()
let edges = read<[0..V.count - 1](u: V, v: V)>()
let t = Rooted(Graph.undirected(edges), V.first!)
print(t.depth(V(2)))
