// DMOPC '21 Contest 2 P5 — Tree Knapsack Convolution mod 998244352 (CP Lang v2 spec)
// original: corpus/submissions/3968384.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Modulus is composite `998_244_352 = 2^23 * 7 * 17`. Because `26! == 0 (mod MOD)`,
// every vertex with >= 26 marked incident edges vanishes when weighted by `deg!`.
// Rooted tree DP in postorder convolves subtree counts with degree capped at 25.

const MOD = 998_244_352
const MAX_DEG = 26
type Z = Mod<MOD>

let (n, k): (1..=3000, 0..=3000) = read()
domain V = 1..=n
domain E = 0..(n - 1)
domain Deg = 0..MAX_DEG

let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)
let tree = Rooted<V>(g, V.first)

var fact: [Deg]Z = 1
for d in Deg if d.to_idx > 0:
    fact[d] = fact[(d - 1)!] * d.to_idx

// memo[u][parent_used] holds a List<Z> of length `tree.size(u) + 1`, indexed by
// total chosen edges inside u's subtree (plus the parent edge if `parent_used`).
// FRICTION UD4: Dynamic per-node DP tables whose length depends on `tree.size(u)`
// cannot use a fixed `[0..=n]` domain without wasting O(n^2) space, so they fall
// back to `List<Z>` initialized via `[0..=sz] _ => Z.zero` or `List.filled(...)`.
var memo: [V, Bool]List<Z> = []

for u in tree.postorder:
    let sz_u = tree.size(u)
    for par_used in Bool:
        let start = Int(par_used)
        var cur: [Deg, 0..=sz_u]Z = 0
        cur[Deg.from_idx(start)!, start] = 1
        var csum = start

        for v in tree.children(u):
            let sz_v = tree.size(v)
            let child0 = memo[v, false]
            let child1 = memo[v, true]
            var nxt: [Deg, 0..=sz_u]Z = 0
            for d in Deg:
                for i in 0..=csum:
                    let stay = cur[d, i]
                    let prev_d = if let pd = d - 1: cur[pd, i] else: Z.zero
                    if stay == 0 and prev_d == 0: continue
                    for j in 0..=min(sz_v, sz_u - i):
                        nxt[d, i + j] += stay * child0[j]
                        if j > 0:
                            nxt[d, i + j] += prev_d * child1[j]
            csum += sz_v
            cur = nxt

        var res: List<Z> = [sum(cur[d, i] * fact[d] for d in Deg) for i in 0..=sz_u]
        memo[u, par_used] = res

print(if k <= n: memo[V.first, false][k] else: Z.zero)
