// CSES 1130 Tree Matching: the maximum number of disjoint edges in a tree.
let n: 1..=2e5 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted<V>(Graph.undirected(edges), V.first)
var free: [V]Int = 0
var used: [V]Int = 0
for v in t.at.rev:
    var gain = 0
    for c in t.children(v):
        let best = max(free[c], used[c])
        free[v] += best
        gain max= free[c] + 1 - best
    used[v] = if t.children(v).len == 0: 0 else: free[v] + gain
print(max(free[V.first], used[V.first]))
