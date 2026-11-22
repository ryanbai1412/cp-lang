// CSES 1679 Course Schedule: n courses, m prerequisites "a b" (a before b).
// Print an order of all courses that respects every prerequisite, or IMPOSSIBLE.
domain V = 1..=read<1..=1e5>()
let m = read<1..=2e5>()
let g = Graph.directed(read<[m](u: V, v: V)>())
match topo(g):
    Some(order) => print(order)
    None => print("IMPOSSIBLE")
