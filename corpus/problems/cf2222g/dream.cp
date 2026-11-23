// Codeforces 2222 G: for each k in 1..=n, count unordered pairs {u, v} (u <= v)
// whose maximum remaining component size after deleting all edges on the u-v
// path is k.
//
// Root at a centroid rt, and let excl be rt's largest child subtree (size <= n/2).
// - u == v (0 edges cut): the whole tree stays connected -> ans[n] = n.
// - lca(u, v) = x with both endpoints in non-dominant child subtrees of x (for
//   x != rt, all children; for x == rt, all children except excl): x's own
//   component has size n - s1 (for {x, w}) or n - s1 - s2 (for w1, w2 in two
//   child subtrees of sizes s1, s2), which always dominates every other cut
//   component along the path.
// - u in excl, v == rt: rt's component n - sz[excl] >= n/2 dominates ->
//   ans[n - sz[excl]] += sz[excl].
// - u in excl, v in another child c != excl of rt: the max component on u..v is
//   max(k[u], k[v]), where k[w] is the max component hanging off the path from
//   rt to w (seeded at best[excl] = 0 and best[c] = n - sz[excl] - sz[c]).
//   Prefix sums over the two histograms count pairs with max(k[u], k[v]) == i.
use std.prefix.*

@!multitest
let n: 1..=200_000 = read()
domain V = 1..=n
domain Size = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let rt = Rooted(g, root: V(1)).centroid
let t = Rooted(g, root: rt)
let sz = t.subtree_sizes

var ans: [Size]Int = 0
ans[Size(n)] = n
if n > 1:
    let excl = t.children(rt).max(by: c => sz[c])!

    for v in V:
        let freqs = [e for e in [sz[c] for c in t.children(v) if not (v == rt and c == excl)].tally().items]
        for (i, (s1, c1)) in freqs.items:
            ans[Size(n - s1)] += s1 * c1
            if c1 >= 2: ans[Size(n - 2 * s1)] += (c1 * (c1 - 1) / 2) * s1 * s1
            for (s2, c2) in freqs[i + 1..]:
                ans[Size(n - s1 - s2)] += (s1 * c1) * (s2 * c2)

    ans[Size(n - sz[excl])] += sz[excl]

    var best: [V]Int = 0
    for c in t.children(rt) if c != excl:
        best[c] = n - sz[excl] - sz[c]
    for u in t.at if u != rt:
        for c in t.children(u):
            best[c] = max(best[u], sz[u] - sz[c])

    var excl_hist: [Size]Int = 0
    var rest_hist: [Size]Int = 0
    for u in t.subtree(excl):
        excl_hist[Size(max(best[u], sz[u]))] += 1
    for c in t.children(rt) if c != excl:
        for u in t.subtree(c):
            rest_hist[Size(max(best[u], sz[u]))] += 1

    let p_excl = Prefix(excl_hist)
    let p_rest = Prefix(rest_hist)
    for i in Size:
        ans[i] += p_excl.prefix(i.l) * rest_hist[i] + p_rest.prefix(i.r) * excl_hist[i]

print(ans)
