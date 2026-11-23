// CSES 1679 Course Schedule: an order of n courses respecting m "a before b" rules.
// Input: n m, then m lines "a b". Output: an order, or IMPOSSIBLE on a cycle.
let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
let g = Graph.directed(read<[m](u: V, v: V)>())

let order = topo(g) else:
    print("IMPOSSIBLE")
    return
print(order)
