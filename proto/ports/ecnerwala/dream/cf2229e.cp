// Codeforces 2229 E: count valid ways to orient / partition the tree rooted
// at `root = V.last` (vertex `n`).
//
// Invariant:
// - Every leaf `l` must appear before any strict ancestor in the order, so no
//   valid prefix can end below `max_leaf = max(u for u in V if g.deg(u) == 1)`:
//   `max_leaf` is the unique starting state (`dp[max_leaf] = 1`).
// - For each vertex `u > max_leaf`, `prv_val[u]` is the largest vertex label in
//   any child subtree of `u` (at `root = V.last`, the child subtree containing
//   `root.prev!` — i.e., `n - 1` — is excluded because `n - 1` sits on the
//   final branch into `n` and cannot block earlier transitions).
// - `dp[u]` counts valid sequences ending at `u`: the previous chosen vertex
//   `w < u` must exceed every child-subtree maximum of `u` (`prv_val[u] < w < u`),
//   so `dp[u] = sum(dp[w] for w in (prv_val[u] + 1)..u)` when `prv_val[u] < u`,
//   maintained in O(1) per vertex via `Prefix<Z>.filling(n)`.
const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let root = V.last
let t = Rooted(g, root:)

let max_leaf = max(u for u in V if g.deg(u) == 1)!
var sub_max = [V] u => u
var prv_val: [V]?V = none
for u in t.at.rev:
    var max_ch: ?V = none
    for c in t.children(u):
        if u == root and sub_max[c] == root.prev!: continue
        max_ch = if let mc = max_ch: some(max(mc, sub_max[c])) else: some(sub_max[c])
    prv_val[u] = max_ch
    if let mc = max_ch: sub_max[u] max= mc

var dp = Prefix<Z>.filling(n)
for u in V:
    let val = if u == max_leaf: Z(1)
    elif u > max_leaf:
        if let pv = prv_val[u], pv < u: dp.sum(pv.ord + 1..u.ord) else: Z(0)
    else: Z(0)
    dp.set(u.ord, val)
print(dp.sum(n - 1..n))
