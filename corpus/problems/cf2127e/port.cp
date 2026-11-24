// Source: Codeforces 2127E (Ancient Tree), https://codeforces.com/contest/2127/problem/E
// limits: 4000 ms, 256 MB
// Problem: a tree rooted at 1 with vertex weights w and colors in 1..k, some lost (0). v is cutie
//   if lca(x, y) = v for some x, y of one color other than v's. Recolor the lost vertices to
//   minimise the weight of cutie vertices; print it and the coloring. n <= 2e5, sum of n <= 2e5;
//   multitest.
// Solution: the LCAs of a color's vertex pairs are the LCAs of its neighbours in preorder.
//   Such an LCA without a color takes this one; one colored differently is cutie anyway and
//   pays. Every vertex still uncolored copies its parent (the root takes 1), which makes no new
//   cutie vertex. Sorting plus LCA queries, O(n log n).
// Bounds: n: 2..=200_000; k: 1..=200_000; w: 1..=10 ** 9; edges.u, edges.v: 1..=n

@!multitest
let (n, k): (2..=200_000, 1..=200_000) = read()
domain V = 1..=n
domain Color = 1..=k
let w: [V]1..=10 ** 9 = read()
var col: [V]?Color = read(none: 0)

let edges: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), root: V(1))

var by_col: [Color]List<V> = []
for v in V:
    if let c = col[v]: by_col[c].push(v)

var is_bad: [V]Bool = false
for c in Color:
    by_col[c].sort(by: v => tree.pos(v).ord)
    for (u, v) in by_col[c].adjacent:
        let l = tree.lca(u, v)
        if col[l] == None: col[l] = c
        elif col[l] != c: is_bad[l] = true

let ans = sum(w[v] for v in V if is_bad[v])
for v in tree.at if col[v] == None:
    col[v] = if let p = tree.parent(v): col[p] else: Color(1)

print(ans)
print([col[v]! for v in V])
