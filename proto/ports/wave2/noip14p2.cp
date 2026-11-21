// NOIP 2014 Joint Weights: over all ordered pairs of vertices at distance 2
// in a tree, the largest product of weights and the sum of products mod
// 10007. Such a pair is two neighbours of a middle vertex: the sum is
// (sum of neighbours)^2 - sum of squares, the best product the top two.
type Z = Mod<10007>
let n: 2..=2e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let w: [V]1..=10000 = read()
let tree = Graph.undirected(edges)

var best = 0
var total: Z = 0
for mid in V:
    let around = sorted(w[u] for (u, _) in tree.out(mid)).rev
    if around.len >= 2: best max= around[0] * around[1]
    let all = around.sum()
    total += Z(all) * all - sum(x * x for x in around)
print(best, total)
