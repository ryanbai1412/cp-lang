// Codeforces 2150 F: Cycle Closing (dream).
// Take any spanning tree T, of diameter D, and let L = ceil(D / 2).
// Operation 1 (k = 3) joins the two ends of every tree path j - i - k, so after it
// any two vertices at tree distance 2 are adjacent: one step can skip a vertex.
// Operation 2 (k = L + 1) closes each remaining pair (i, j) with a path of exactly
// L steps. The tree path i..j has d steps, 3 <= d <= D <= 2L.
// - d >= L: walk it, skipping d - L vertices.
// - d < L: let far be the diameter end farther from i (so dist(i, far) >= L) and m
//   the vertex where i..j turns toward far. Replace m by a detour over the first
//   L - d + 1 vertices of the stem m..far: out on every other vertex, back on the
//   rest. Every step is a tree edge or a skip, and no vertex repeats.
with io(base: 1)

@!multitest
domain V = read(3..=200)
let g = Graph<V>.undirected(edges: read<[read(V.count - 1..=V.count * (V.count - 1) / 2)](V, V)>())
let t = g.spanning_tree()
let d = t.diameter()
let L = ceil_div(d.len, 2)

let skips = [(j, i, k) for i in V for (j, k) in t.adj[i].pairs if not g.has(j, k)]
fn joined(i: V, j: V) -> Bool = g.has(i, j) or t.dist(i, j) == 2

fn route(i: V, j: V) -> List<V>:
    let p = t.path(i, j)
    let s = p.len - 1 - L
    if s >= 0: return p[..2 * s].even ++ p[2 * s..]
    let far = max(d.u, d.v, by: f => t.dist(i, f))
    let m = t.median(i, j, far)
    let stem = t.path(m, far)[..=-s]
    let detour = if m == j: stem.odd ++ stem.even.rev else: stem.even ++ stem.odd.rev
    t.path(i, m).drop_last ++ detour ++ t.path(m, j).drop_first

print(2)
print(3)
print(skips.len)
for s in skips: print(s)
let rest = [route(i, j) for (i, j) in V.pairs if not joined(i, j)]
print(L + 1)
print(rest.len)
for r in rest: print(r)
