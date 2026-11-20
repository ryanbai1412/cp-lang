// Each of m roads can be kept or reversed; road i must not point the same
// way as road j for every listed conflict. Print one valid choice or "NO".
// "These two differ" is an edge of a graph that must stay two-colourable.
use lib.ds.bipartite_dsu.{BipartiteDsu}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var d = BipartiteDsu<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    d.add_edge(Road(a), Road(b))
if d.bipartite:
    print("YES")
    print([if d.find(r).1: 1 else: 0 for r in Road])
else:
    print("NO")
