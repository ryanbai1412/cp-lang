// Source: NOIP 2014 Day 1 P2 (Joint Weights); DMOJ's noip14p2 is not public, statement from
//   Luogu P1351
// Problem: a tree of n <= 2e5 vertices with weights W_i <= 10000. Each ordered pair (u, v) at
//   distance 2 gives W_u * W_v; print the largest such product and the sum of all of them
//   mod 10007.
// Solution: a pair at distance 2 is two neighbours of a middle vertex. Per middle vertex, the
//   ordered sum over its neighbours is (sum of w)^2 - sum of w^2, and the best product is the
//   top two neighbour weights. O(n).
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
