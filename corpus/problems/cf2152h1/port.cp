// Source: Codeforces 2152H1 (Victorious Coloring, Easy Version),
//   https://codeforces.com/contest/2152/problem/H1
// Problem: a tree on n <= 2.5e5 vertices with edge weights <= 1e9. For vertex weights x >= 0,
//   f(x) = min over nonempty vertex sets S of sum_{v in S} x[v] + weight of edges leaving S.
//   For each of q <= 10 queries l <= 1e9, print the min sum of x with f(x) >= l. Multitest.
// Solution: per query t, greedy bottom-up over the rooted tree (children first). up[c] is how
//   much of its parent edge par_w[c] the subtree of c already leans on; child c then covers
//   min(t - up[c], par_w[c]) of v's threshold t (covered capped at t). v draws
//   e_top = min(t - covered, par_w[v]) from its own parent edge, sets up[v] = e_top and pays
//   the rest, t - covered - e_top, as x[v]. O(n) per query, O(n q) total.

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
