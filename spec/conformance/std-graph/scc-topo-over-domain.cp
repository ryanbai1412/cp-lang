// expect: ok, output "3 YES\nYES"
// `scc` and `topo` take a graph over a declared domain, in both modes (rebuild round 6 H-SCC-BLOCKED).
domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 0), (u: 2, v: 3), (u: 1, v: 2)]
let g = Graph.directed(es)
let (domain C, comp) = scc(g)
print(C.count, yes(comp[0] == comp[1]))
print(yes(topo(g) == None))
