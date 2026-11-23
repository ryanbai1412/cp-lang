// CF 2018C Tree Pruning (revision: also print the smallest leaf depth d that is optimal): keeping all leaves at depth d keeps exactly the vertices with
// depth(v) <= d <= deepest(v), deepest(v) the largest depth in v's subtree.
@!multitest
let n = read<3..=5e5>()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: V.first!)
let deepest = t.up(max, finish: (s, v) => max(s, t.depth(v)))
var starts: [0..=n + 1]Int = 0
for v in V:
    starts[t.depth(v)] += 1
    starts[deepest[v] + 1] -= 1
var kept = 0
var best = 0
var at = 0
for d in 0..=n:
    kept += starts[d]
    if kept > best:
        best = kept
        at = d
print(n - best, at)
