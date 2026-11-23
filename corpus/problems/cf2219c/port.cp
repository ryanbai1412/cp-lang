// Codeforces 2219 C — Expected Random-Walk Exit Tree DP
//
// Root the tree at any '1' vertex (which has 0 exit cost). For each vertex `u`:
// - `dp0[u]` = minimum expected cost in `u`'s subtree when `u` may exit via
//   its parent edge.
// - `dp1[u]` = minimum expected cost when `u` must exit into one of its
//   children (`none` if no child has a finite `dp1`).
//
// At a '0' vertex `u` of degree `deg`, start with all children in `dp0` mode
// (`base = sum(dp0[c])`) and greedily switch children into `dp1` mode in
// increasing order of marginal cost `dp1[c] - dp0[c]`. With `k` switched
// children, `u` has `k` valid exit edges without the parent (expected wait
// `deg / k`) and `k + 1` with the parent (expected wait `deg / (k + 1)`).

@!multitest
let n: 2..=200_000 = read()
let s: Str = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

let rt = if let i = s.find('1'): V(i + 1) else: V(1)
let t = Rooted(g, root: rt)

var dp0: [V]F64 = 0.0
var dp1: [V]?F64 = None
for u in t.at.rev:
    var base = sum(dp0[c] for c in t.children(u))
    if s[u.ord] == '1':
        dp0[u] = base
        dp1[u] = base
        continue
    let ch = [d1 - dp0[c] for c in t.children(u) if let d1 = dp1[c]].sorted()
    let deg = F64(g[u].len)
    var b0 = base + deg
    var b1: ?F64 = None
    for (i, x) in ch.items:
        base += x
        b1 min= base + deg / F64(i + 1)
        b0 min= base + deg / F64(i + 2)
    dp0[u] = b0
    dp1[u] = b1

print("{dp0[rt]:.15f}")
