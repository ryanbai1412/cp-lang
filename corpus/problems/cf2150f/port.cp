// Codeforces 2150 F: Cycle Closing.
// Take any spanning tree T of diameter D, and let L = ceil(D / 2).
// Two operations close every missing edge into a cycle:
//   - Operation 1 (path length 3): join the ends of every tree path j - i - k,
//     making all vertices at tree distance 2 adjacent (one step can skip a vertex).
//   - Operation 2 (path length L + 1): connect each remaining pair (i, j) by an
//     L-step path. If `dist = t.dist(i, j) >= L`, skip `skip = dist - L`
//     vertices by taking every other vertex on `p[..2 * skip]` then `p[2 * skip..]`.
//     If `dist < L`, let `far` be the diameter end farther from `i` (so
//     `dist(i, far) >= L`) and `m = t.median(i, j, far)`. Replace `m` with a
//     detour over the first `stem_len = L - dist + 1` vertices of `m..far`:
//     out on every other vertex, back on the rest.

@!multitest
let n: 3..=200 = read()
let m: n - 1..=n * (n - 1) / 2 = read()
domain V = 1..=n
let edges: [0..m](V, V) = read()
var joined_by_edge: [Sym<V>]Bool = false
var tree_edges: List<(V, V)> = []
var dsu = Dsu<V>()
for (u, v) in edges:
    joined_by_edge[u, v] = true
    if dsu.unite(u, v):
        tree_edges.push((u, v))
let tree = Graph.undirected(tree_edges)
let t = Rooted(tree, V.first!)
let d = t.diameter()
let half = ceil_div(d.len, 2)

// All unordered neighbor pairs (`List.pairs` yields adjacent windows only; see friction #7).
fn neighbour_pairs(i: V) -> List<(V, V)> =
    [(j, k) for (a, j) in tree[i].items for (b, k) in tree[i].items if a < b]
let skips = [[j, i, k] for i in V for (j, k) in neighbour_pairs(i) if not joined_by_edge[j, k]]
fn joined(i: V, j: V) -> Bool = joined_by_edge[i, j] or t.dist(i, j) == 2

fn route(i: V, j: V) -> List<V>:
    let p = t.path(i, j)
    let dist = p.len - 1
    if dist >= half:
        let skip = dist - half
        return [x for (z, x) in p[..2 * skip].items if z % 2 == 0] + p[2 * skip..]
    let far = [d.u, d.v].max(by: f => t.dist(i, f))!
    let meets = [t.lca(i, j), t.lca(i, far), t.lca(j, far)]
    let mid = meets[meets.argmax(by: x => t.depth(x))!]
    let stem_len = half - dist + 1
    let stem = t.path(mid, far)[..stem_len]
    let out = if mid == j: 1 else: 0
    let there = [x for (z, x) in stem.items if z % 2 == out]
    let back = [x for (z, x) in stem.items.rev if z % 2 != out]
    let detour = there + back
    let to_mid = t.path(i, mid)
    to_mid[..to_mid.len - 1] + detour + t.path(mid, j)[1..]

let rest = [route(i, j) for (i, j) in V.pairs if not joined(i, j)]
let ops = [(3, skips), (half + 1, rest)]
print(ops.len)
for (k, paths) in ops:
    print(k)
    print(paths.len)
    for path in paths: print(path)
