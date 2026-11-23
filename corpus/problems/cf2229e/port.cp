// Codeforces 2229 E: count valid ways to orient / partition the tree rooted
// at `root = V.last` (vertex `n`).
//
// Invariant:
// - Every leaf `l` must appear before any strict ancestor in the order, so no
//   valid prefix can end below `max_leaf = max(u for u in V if g[u].len == 1)`:
//   `max_leaf` is the unique starting state (`dp[max_leaf] = 1`).
// - For any non-root vertex `u > max_leaf`, the last chosen vertex `w < u`
//   must exceed every label in `u`'s proper child subtrees, so `w > prv_val[u]`
//   where `prv_val[u] = max(sub_max[c] for c in t.children(u))`.
// - At `root = V.last` (`n`), the child subtree holding `n - 1` (`root.prev!`)
//   is excluded from `prv_val[root]`: the penultimate chosen vertex `w` is
//   allowed to lie inside that subtree (even `w = n - 1`), whereas every other
//   child subtree of `root` has maximum `< n - 1` and must be cleared before
//   `w`, so `w` only needs to exceed the largest label in the *other* child
//   subtrees of `root`.
// - Therefore `dp[u] = sum(dp[w] for w in pv.next!..u)` when `pv = prv_val[u] < u`,
//   maintained in O(1) per vertex via `Prefix<Z>.filling(n)`.
const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let root = V.last!
let t = Rooted(g, root:)

let max_leaf = V.filter(u => g[u].len == 1).map(u => u).max()!
var sub_max = [V] u => u
var prv_val: [V]?V = None
for u in t.at.rev:
    let ch = [sub_max[c] for c in t.children(u) if not (u == root and sub_max[c] == root.prev!)]
    let max_ch: ?V = if ch.is_empty: None else: Some(ch.max())
    prv_val[u] = max_ch
    if let mc = max_ch: sub_max[u] max= mc

var dp = Prefix<Z>.filling(n)
for u in V:
    let val = if u == max_leaf: Z(1)
    elif u > max_leaf:
        if let pv = prv_val[u], pv < u: dp.fold(pv.ord + 1..u.ord) else: Z(0)
    else: Z(0)
    dp.set(u.ord, val)
print(dp.fold(root.ord..root.ord + 1))
