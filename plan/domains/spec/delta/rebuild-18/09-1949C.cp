// CF1949C Annual Ants' Gathering. Gather at the centroid: every vertex must
// absorb its child subtrees smallest first, each no larger than what it holds.
let n: 1..=2e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: Rooted(g, root: V.first!).centroid())
var ok = true
for v in V:
    var cur = 1
    for c in t.children(v).sorted(by: c => t.size(c)):
        if t.size(c) > cur: ok = false
        cur += t.size(c)
print(yes(ok))
