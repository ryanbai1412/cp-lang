// Codeforces 2229 I: rerooted max-plus tree knapsack.
// Rooted anywhere, the score is the sum over a in S of w(a) * |S in subtree(a)|:
// a's weight counts once for every chosen node at or below it.
//
// down[v][i]: the best total of the terms of subtree(v)'s nodes when exactly i
// of S lie inside. Children join by max-plus convolution, then v is chosen or
// not: f[i] max= f[i - 1] + w(v) * i. Every count 0..=size is reachable, so
// down has no "impossible" entry.
//
// up[v][i]: rerooted at v, the best total of the terms of the nodes outside
// subtree(v), given i of S inside; none where impossible. At the root it is
// "all k inside". The part a child c contributes is split off a functional
// by the transposed convolution split(f, b)[i] = max over j of f[i + j] + b[j].
// For each child c of v: split off c's siblings (the later ones are already
// gone, the earlier ones are their prefix join), then v itself, whose subtree
// rerooted at c holds the k - i chosen nodes outside subtree(c). With every
// child split off, up[v][1] + w(v) * k is the answer with v as the root.
@!multitest
let (n, k): (2..=4000, Int) = read()
domain V = 1..=n
let w: [V]1..=1_000_000_000 = read()
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V.first)
let kids: [V]List<V> = [V] u => t.children(u).sorted(by: c => t.size(c))

use std.ntt.{max_plus_convolve, max_plus_correlate}

var down: [V]List<Int> = []
var joined_before: [V]List<List<Int>> = []
for v in t.at.rev:
    var f = [0]
    for c in kids[v]:
        joined_before[v].push(f.clone())
        f = max_plus_convolve(f, down[c])
    down[v] = [
        if i == 0: f[0] elif i == f.len: f[i - 1] + w[v] * i else: max(
            f[i],
            f[i - 1] + w[v] * i,
        ) for i in 0..=f.len
    ]

var ans: [V]Int = 0
var up: [V]List<?Int> = []
up[t.root] = [if i == k: 0 else: none for i in 0..=n]
for v in t.at:
    var f = up[v]
    for (z, c) in kids[v].items.rev:
        var g = max_plus_correlate(f, joined_before[v][z])
        for i in 0..g.len - 1:
            if let x = g[i + 1]: g[i] max= x + w[v] * (k - i)
        g.pop()
        up[c] = g
        f = max_plus_correlate(f, down[c])
    ans[v] = f[1]! + w[v] * k

print(ans)
