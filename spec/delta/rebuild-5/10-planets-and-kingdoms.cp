// CSES 1683 Planets and Kingdoms: label planets by strongly connected component.
// Input: n m, then m lines "a b" (a teleport a -> b). Output: k, then each planet's kingdom 1..k.
let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
let (domain K, kingdom) = scc(Graph.directed(read<[m](u: V, v: V)>()))

print(K.count)
print([Int(kingdom[v]) + 1 for v in V])
