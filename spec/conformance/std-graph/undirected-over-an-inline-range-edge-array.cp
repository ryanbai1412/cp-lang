// expect: ok, output "2 2 YES"
// input: "3 2\n1 2 5\n2 3 7\n"
// rule: 08.1.2
// `Graph.undirected` over a read `[1..n]` edge array keys its arcs by that array's own keys, which start at 1 (coci21c4p4).
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain V = 1..=n
let g = Graph.undirected(read<[1..n](u: V, v: V, w: 1..=1e9)>())
print(k, g.deg(V(2)), yes(g.out(V(1)).collect().len == 1))
