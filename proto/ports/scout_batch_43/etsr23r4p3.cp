// ETSR '23 Round 4 P3 — Tree-in-Graph Bijective Embedding (Inclusion-Exclusion) (CP Lang v2 spec)
// original: corpus/submissions/5635626.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Count bijective homomorphisms from an n-vertex tree into an n-vertex graph
// (n <= 17) using inclusion-exclusion over `Subset<V>` and bottom-up tree DP
// in wrapping `U64` arithmetic (intermediate products can exceed 2^63 while
// the final count fits in 64 bits).

let (n, m): (1..=17, 0..=136) = read()
domain V = 1..=n
domain GE = 0..m
domain TE = 0..(n - 1)

var has_edge: [V, V]Bool = false
for _ in GE:
    let (u, v): (V, V) = read()
    has_edge[u, v] = true
    has_edge[v, u] = true

let tedges: [TE](u: V, v: V) = read()
let tree = Rooted<V>(Graph.undirected(tedges), root: V.first)

var ans: U64 = 0
for mask in Subset<V> if mask.len > 0:
    let allowed = [v for v in V if v in mask]
    var dp: [V, V]U64 = 0
    for u in tree.postorder:
        for v1 in allowed:
            var ways: U64 = 1
            for c in tree.children(u):
                // FRICTION ET7: `U64` wrapping arithmetic works cleanly for `+`
                // and `*`, but alternating inclusion-exclusion sign cannot negate
                // a `U64` directly (`-x`), and `prod(...)` is not in the prelude.
                ways *= sum(dp[c, v2] for v2 in allowed if has_edge[v1, v2])
            dp[u, v1] = ways
    let total: U64 = sum(dp[V.first, v] for v in allowed)
    if (n - mask.len) % 2 == 0: ans += total
    else: ans -= total

print(ans)
