// CSES 1130 Tree Matching: the maximum number of disjoint edges in a tree.
// Input: n, then n - 1 lines "a b". Output: the size of a maximum matching.
let n = read<1..=2e5>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)

var used: [V]Bool = false
var pairs = 0
for v in t.order.rev:
    if let p = t.parent(v), not used[v] and not used[p]:
        used[v] = true
        used[p] = true
        pairs += 1
print(pairs)
