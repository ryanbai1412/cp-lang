// CSES 1679 Course Schedule: an order of courses respecting prerequisites.
// Input: n m, then m lines "a b" (a before b). Output: an order, or IMPOSSIBLE on a cycle.
domain V = 1..=read<1..=1e5>()
let m = read<1..=2e5>()
let g = Graph.directed(read<[m](u: V, v: V)>())
let order = topo(g) else:
    print("IMPOSSIBLE")
    return
print(order)
