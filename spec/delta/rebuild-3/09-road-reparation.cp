// CSES 1675 Road Reparation: cheapest set of roads connecting every city (minimum spanning tree).
// In: n m, then m roads "a b c". Out: the minimum total cost, or IMPOSSIBLE.
domain City = 1..=read<1..=1e5>()
let m = read<1..=2e5>()
let mst = kruskal(read<[m](u: City, v: City, w: 1..=1e9)>())
if mst.take.count(t => t) == City.count - 1: print(mst.total)
else: print("IMPOSSIBLE")
