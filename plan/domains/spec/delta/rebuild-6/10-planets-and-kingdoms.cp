// CSES 1683 Planets and Kingdoms: strongly connected components of a directed graph.
// Input: n m, then m teleporters "a b". Output: k, then the kingdom 1..k of each planet.
domain V = 1..=read<1..=1e5>()
let m = read<1..=2e5>()
let g = Graph.directed(read<[m](u: V, v: V)>())
let (domain K, comp) = scc(g)
print(K.count)
print(comp.map(k => Int(k) + 1))
