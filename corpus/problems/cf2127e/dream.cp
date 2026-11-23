// Codeforces 2127 E — Ancient Tree
//
// A vertex `l` is "cute" (bad, costing w[l]) iff two distinct vertices of the
// same color c != col[l] have LCA equal to `l`.
//
// For each color c, sorting its initially colored vertices by DFS preorder
// means every LCA of two color-c vertices already appears as `lca(u, v)` of
// an adjacent pair `(u, v)` in that order. For each such LCA `l`:
// - If `col[l]` is unset (`none`), assigning `col[l] = c` avoids paying `w[l]`
//   for color c.
// - If `col[l]` is already set to a different color, `l` is unavoidably bad
//   (`is_bad[l] = true`).
// Finally, any still-uncolored vertex inherits its parent's color (or Color(1)
// at the root) in top-down preorder so no new bad LCAs are created.

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
