// Source: Codeforces 2222G (Statistics on Tree), https://codeforces.com/contest/2222/problem/G
// Problem: a tree on n <= 1e5 vertices. The value of a pair u <= v is the size of the largest
//   component left after deleting every edge on the u-v path; for each i in 1..=n print how many
//   pairs have value i. Multitest, sum n <= 5e5.
// Solution: root at a centroid rt; let excl be its largest child. Unless one endpoint is in excl
//   and the other in another child of rt, the component holding the path's top x dominates:
//   n - s1 (x and one child subtree) or n - s1 - s2 (two of them), counted by grouping x's child
//   sizes (O(sqrt n) distinct). For the rest the value is max(k[u], k[v]), k[w] being the largest
//   piece hanging off the path rt..w; histograms of k on the two sides and prefix sums count
//   them. O(n sqrt n) worst case for the size pairs, O(n) otherwise.
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

fn size_counts(v: V, excl: V) -> List<(Int, Int)>:
    var mp: Map<Int, Int> = Map()
    for c in t.children(v) if not (v == rt and c == excl):
        mp[sz[c]] += 1
    [(k, c) for (k, c) in mp]

var ans: [Size]Int = 0
ans[Size(n)] = n
if n > 1:
    let excl = t.children(rt).max(by: c => sz[c])!

    for v in V:
        let freqs = size_counts(v, excl)
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
