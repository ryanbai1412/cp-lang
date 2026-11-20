// Codeforces 2245 E: glue every tree edge to its even-degree endpoints.  A
// glued piece with e edges and v vertices contributes C(e + 1 - v, 2).
@!multitest
let n: 2..=2e5 = read()
let edges: [0..n - 1](1..=n, 1..=n) = read()
domain X = 0..2 * n - 1   // vertices 0..n, then one node per edge
var odd: [0..n]Bool = false
for (u, v) in edges:
    odd[u - 1] = not odd[u - 1]
    odd[v - 1] = not odd[v - 1]
var pieces = Dsu<X>()
for e in 0..n - 1:
    let (u, v) = edges[e]
    if not odd[u - 1]: pieces.unite(X(n + e), X(u - 1))
    if not odd[v - 1]: pieces.unite(X(n + e), X(v - 1))
var ans = 0
for g in pieces.groups():
    let extra = count(x.ord >= n for x in g) + 1 - count(x.ord < n for x in g)
    ans += extra * (extra - 1) / 2
print(ans)
