// CSES 1130 Tree Matching: the largest set of edges with no shared vertex.
// Input: n, then n-1 edges "a b". Output: the size of a maximum matching.
domain V = 1..=read<1..=2e5>()
domain E = 0..V.count - 1
let t = Rooted(Graph.undirected(read<[E](u: V, v: V)>()), root: V.first!)
var free: [V]Int = 0    // best in v's subtree with v unmatched
var best: [V]Int = 0    // best in v's subtree
for v in t.order.rev:
    free[v] = t.children(v).map(c => best[c]).sum()
    best[v] = free[v]
    for c in t.children(v):
        best[v] max= free[v] - best[c] + free[c] + 1
print(best[V.first!])
