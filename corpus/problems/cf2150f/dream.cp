// Codeforces 2150 F: Cycle Closing.
// Take any spanning tree T of diameter D, and let L = ceil(D / 2).
// Two operations close every missing edge into a cycle:
//   - Operation 1 (path length 3): join the ends of every tree path j - i - k,
//     making all vertices at tree distance 2 adjacent (one step can skip a vertex).
//   - Operation 2 (path length L + 1): connect each remaining pair (i, j) by an
//     L-step path. If `dist = t.dist(i, j) >= L`, skip `skip = dist - L`
//     vertices by taking every other vertex on `p[..2 * skip]` then `p[2 * skip..]`.
//     If `dist < L`, let `far` be the diameter end farther from `i` (so
//     `dist(i, far) >= L`) and `mid = t.median(i, j, far)`. Replace `mid` with
//     a detour over the first `stem_len = L - dist + 1` vertices of `mid..far`:
//     out on every other vertex, back on the rest (adding `stem_len - 1` steps).
//   - Building `t.path(...)` per pair is O(n) each, O(n^3) total at n <= 200.

with io(base: 1)

@!multitest
let n: 3..=200 = read()
let m: n - 1..=n * (n - 1) / 2 = read()
domain V = 1..=n
let g = Graph<V>.undirected(edges: read<[m](V, V)>())
let t = g.spanning_tree()
let d = t.diameter()
let half = ceil_div(d.len, 2)

let skips = [[j, i, k] for i in V for (j, k) in List(chain([t.parent(i)].present(), t.children(i))).pairs() if not g[j].contains(k)]
fn joined(i: V, j: V) -> Bool = g[i].contains(j) or t.dist(i, j) == 2

fn route(i: V, j: V) -> List<V>:
    let p = t.path(i, j)
    let dist = p.len - 1
    if dist >= half:
        let skip = dist - half
        return p[..2 * skip].even.collect() + p[2 * skip..].collect()
    let far = [d.u, d.v].max(by: f => t.dist(i, f))!
    let mid = t.meet(i, j, far)
    let stem_len = half - dist + 1
    let stem = t.path(mid, far)[..stem_len]
    let out = if mid == j: 1 else: 0
    let detour = [x for (z, x) in stem.items if z % 2 == out] + [x for (z, x) in stem.items.rev if z % 2 != out]
    t.path(i, mid).drop_last() + detour + t.path(mid, j).drop()

let rest = [route(i, j) for (i, j) in V.pairs if not joined(i, j)]
let ops = [(3, skips), (half + 1, rest)]
print(ops.len)
for (k, paths) in ops:
    print(k)
    print(paths.len)
    for path in paths: print(path)
