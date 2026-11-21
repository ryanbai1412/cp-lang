// Codeforces 2222 G: root at a tree centroid, aggregate non-dominant subtree
// sizes by frequency at each vertex, and merge centroid-excluded vs non-excluded
// component sizes via prefix sums.
@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let rt = Rooted(g, root: V(1)).centroid
let t = Rooted(g, root: rt)
let sz = t.subtree_sizes

var ans: [0..=n]Int = 0
ans[n] = n
if n > 1:
    let excl = max_by(t.children(rt), c => sz[c])!

    for v in V:
        var mp: Map<Int, Int> = Map()
        for c in t.children(v) if not (v == rt and c == excl):
            mp[sz[c]] += 1
        let freqs = [(k, c) for (k, c) in mp]
        for (i, (s1, c1)) in freqs.items:
            ans[n - s1] += s1 * c1
            if c1 >= 2: ans[n - 2 * s1] += (c1 * (c1 - 1) / 2) * s1 * s1
            for (s2, c2) in freqs[i + 1..freqs.len]:
                ans[n - s1 - s2] += (s1 * c1) * (s2 * c2)

    ans[n - sz[excl]] += sz[excl]
    var excl_sz: [0..=n]Int = 0
    var easy_sz: [0..=n]Int = 0
    var best: [V]Int = 0
    var in_excl: [V]Bool = false
    in_excl[excl] = true
    for c in t.children(rt) if c != excl:
        best[c] = n - sz[excl] - sz[c]
    for u in t.preorder if u != rt:
        let k = max(best[u], sz[u])
        if in_excl[u]: excl_sz[k] += 1
        else: easy_sz[k] += 1
        for c in t.children(u):
            in_excl[c] = in_excl[u]
            best[c] = max(best[u], sz[u] - sz[c])

    var (p_excl, p_easy) = (0, 0)
    for i in 0..=n:
        ans[i] += p_excl * easy_sz[i] + p_easy * excl_sz[i] + easy_sz[i] * excl_sz[i]
        p_excl += excl_sz[i]
        p_easy += easy_sz[i]

print(ans[1..=n])
