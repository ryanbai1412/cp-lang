// Codeforces 2207 D — Tree Leaf-Pair Distance Collapse
//
// Root the tree at `v_in`. In bottom-up order `t.at.rev`, compute `dist[u]`:
// the minimum edge distance from `u` to an escaping leaf in `u`'s subtree
// (capped at `k + 2`, meaning no leaf within `k + 1` edges).
//
// A leaf (`g[u].len == 1`) has `dist[u] = 0`. For an internal node `u`, if any
// two child subtrees yield leaves at combined distance `d + nd <= k + 1`
// through `u`, `u` itself becomes safe (`dist[u] = 0`); otherwise `dist[u]`
// keeps the minimum child distance `nd`. The root is safe iff `dist[V(v_in)] == 0`.

@!multitest
let (n, k, v_in): (2..=200_000, 1..=200_000, 1..=200_000) = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V(v_in))

var dist: [V]Int = k + 2
for u in t.at.rev:
    if g[u].len == 1:
        dist[u] = 0
        continue
    var d = k + 2
    for c in t.children(u):
        let nd = dist[c] + 1
        if d + nd <= k + 1:
            d = 0
            break
        d min= nd
    dist[u] = d

print(if dist[V(v_in)] == 0: "YES" else: "NO")
