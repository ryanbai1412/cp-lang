// Codeforces 2129 E: Induced Subgraph Queries.
// f(u) is the xor of u's neighbours inside l..r; answer the k-th smallest f.
// Lay every vertex's arcs out in vertex order: the arcs leaving l..r are one
// contiguous window. An edge is inside G[l..r] iff both of its arcs are in the
// window, so Mo's algorithm over arc windows flips f at both ends exactly when an
// edge's arc count crosses 2, at O(1) per step. The window's f values live in a
// count with O(1) update and O(sqrt) k-th query; the r - l + 1 vertices of l..r
// are among the n counted values, so the k-th of the window is the
// (k + n - (r - l + 1))-th of all, the others sitting at f = 0 below them.
@!multitest
use std.mo.{mo}
let (n, m): (2..=150_000, 1..=150_000) = read()
domain V = 1..=n
domain E = 0..m
let raw: [E](u: V, v: V) = read()
let g = Graph.undirected(raw)
let (domain Arc, arcs) = [(u, a.to, a.e) for u in V for a in g.out(u)].indexed()
let first_arc = [V.bounds] b => Arc.bounds.nth(g.deg[..b].sum())

let q: 1..=150_000 = read()
let asks: [0..q](l: V, r: V, k: Int) = read()

var f: [V]Int = 0
var values = BucketCount(0..bit_ceil(n + 1))
values.add(0, n)
var arcs_in: [E]Int = 0

fn step(p: Arc, by: Int):
    let (u, v, e) = arcs[p]
    let was_inside = arcs_in[e] == 2
    arcs_in[e] += by
    if was_inside != (arcs_in[e] == 2):
        for (x, y) in [(u, v), (v, u)]:
            values.add(f[x], -1)
            f[x] ^= y.label
            values.add(f[x], 1)

let windows = [first_arc[x.l.l]..first_arc[x.r.r] for x in asks]
print(mo(windows, add: p => step(p, 1), remove: p => step(p, -1),
         answer: qi => values.kth(asks[qi].k + n - (asks[qi].r - asks[qi].l + 1))), sep: "\n")
