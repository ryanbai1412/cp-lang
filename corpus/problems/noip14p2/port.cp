// NOIP 2014 Joint Weights: over all ordered pairs of vertices at distance 2
// in a tree, the largest product of weights and the sum of products mod
// 10007. Such a pair is two neighbours of a middle vertex: the sum is
// (sum of neighbours)^2 - sum of squares, the best product the top two.
const P = 10007
type Z = Mod<P>
let n: 2..=2e5 = read()
domain V = 1..=n
domain E = 1..n
fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))
let edges: [E](u: V, v: V) = read(gen: random_tree)
let w: [V]1..=10000 = read()

var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)

var best = 0
var total: Z = 0
for mid in V:
    var top = 0
    var second = 0
    let around = sum(w[u] for u in adj[mid])
    for u in adj[mid]:
        second max= min(top, w[u])
        top max= w[u]
    best max= top * second
    total += Z(around) * Z(around) - sum(Z(w[u] * w[u]) for u in adj[mid])
print(best, total)
