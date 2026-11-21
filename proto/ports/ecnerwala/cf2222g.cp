// Codeforces 2222 G: root at a tree centroid, aggregate non-dominant subtree
// sizes by frequency at each vertex, and merge centroid-excluded vs non-excluded
// component sizes via prefix sums.
use std.seq.{max_by}

@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let t0 = Rooted(g, root: V(1))
var sz: [V]Int = 1
for u in t0.postorder:
    for c in t0.children(u): sz[u] += sz[c]

var rt = V(1)
while true:
    var next_rt: ?V = none
    for c in t0.children(rt) if sz[c] * 2 > n:
        next_rt = c
    if let nr = next_rt: rt = nr
    else: break

let t = Rooted(g, root: rt)
sz = [V] _ => 1
for u in t.postorder:
    for c in t.children(u): sz[u] += sz[c]

var ans: [0..=n]Int = 0
ans[n] = n
if n > 1:
    let excl = max_by(t.children(rt), c => sz[c])!

    for v in V:
        var mp: Map<Int, Int> = Map()
        for c in t.children(v):
            if v == rt and c == excl: continue
            mp[sz[c]] += 1
        let freqs = [(k, c) for (k, c) in mp]
        for i in 0..freqs.len:
            let (s1, c1) = freqs[i]
            ans[n - s1] += s1 * c1
            if c1 >= 2: ans[n - 2 * s1] += (c1 * (c1 - 1) / 2) * s1 * s1
            for j in i + 1..freqs.len:
                let (s2, c2) = freqs[j]
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

    var p_excl = 0
    var p_easy = 0
    for i in 0..=n:
        ans[i] += p_excl * easy_sz[i] + p_easy * excl_sz[i] + easy_sz[i] * excl_sz[i]
        p_excl += excl_sz[i]
        p_easy += easy_sz[i]

print(ans[1..=n])
