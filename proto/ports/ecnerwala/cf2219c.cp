// Codeforces 2219 C: root the tree at any '1' vertex and run tree DP where
// dp0[u] / dp1[u] is the minimum expected cost when u's parent edge is usable /
// not usable, greedily adding children sorted by dp1[c] - dp0[c].
@!multitest
let n: Int = read()
let s: Str = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let rt = if let i = s.find('1'): V(i + 1) else: V(1)
let t = Rooted(g, root: rt)

var dp0: [V]F64 = 0.0
var dp1: [V]?F64 = none
for u in t.postorder:
    var base = sum(dp0[c] for c in t.children(u))
    if s[u.ord] == '1':
        dp0[u] = base
        dp1[u] = base
        continue
    let ch = sorted([d1 - dp0[c] for c in t.children(u) if let d1 = dp1[c]])
    let deg = F64(g[u].len)
    var b0 = base + deg
    var b1: ?F64 = none
    for (i, x) in ch.items:
        base += x
        b1 min= base + deg / F64(i + 1)
        b0 min= base + deg / F64(i + 2)
    dp0[u] = b0
    dp1[u] = b1

print("{dp0[rt]:.15f}")
