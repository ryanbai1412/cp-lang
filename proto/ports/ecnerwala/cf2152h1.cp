// Codeforces 2152 H1 — Victorious Coloring (Easy Version)
//
// For a query threshold t, we assign non-negative vertex weights x[v] so that
// every non-empty cut S satisfies sum_{v in S} x[v] + cut(S) >= t, minimizing
// sum x[v].
//
// Solve bottom-up on a rooted tree in O(n) per query (O(n * q) total, with
// n, q <= 2500). Let up[v] be the budget v's subtree consumes from its parent
// edge par_w[v] (0 <= up[v] <= par_w[v]). Each child c can contribute at most
// min(t - up[c], par_w[c]) toward v's threshold t; capping `covered` at t per
// child keeps it in 0..=t without changing the shortfall t - covered.
//
// Then v draws e_top = min(t - covered, par_w[v]) from its parent edge, sets
// up[v] = e_top, and pays the unavoidable remainder t - covered - e_top in
// vertex weight.

@!multitest
let n: 2..=2500 = read()
domain V = 1..=n
domain E = 0..n - 1
let raw: [E](u: V, v: V, w: 1..=10 ** 9) = read()
let tree = Rooted(Graph.undirected([E] e => (u: raw[e].u, v: raw[e].v)), root: V(1))

var par_w: [V]Int = 0
for e in E:
    let (u, v, w) = raw[e]
    if tree.parent(u) == v: par_w[u] = w
    else: par_w[v] = w

let q: 1..=2500 = read()
let queries: [0..q]1..=10 ** 9 = read()
for t in queries:
    var ans = 0
    var up: [V]Int = 0
    for v in tree.at.rev:
        var covered = 0
        for c in tree.children(v):
            covered = min(t, covered + min(t - up[c], par_w[c]))
        let e_top = min(t - covered, par_w[v])
        ans += t - covered - e_top
        up[v] = e_top
    print(ans)
