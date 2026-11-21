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
let arcs = [(u, a.to, a.e) for u in V for a in g.out(u)]
domain Arc = 0..arcs.len
let first_arc = prefix([V] u => g.deg(u))

let q: 1..=150_000 = read()
let asks: [0..q](l: V, r: V, k: Int) = read()
let windows = [
    Arc.bounds.nth(first_arc.prefix(x.l.l))..Arc.bounds.nth(first_arc.prefix(x.r.r)) for x in asks
]

// Multiset of values in 0..vmax: add/remove O(1), k-th smallest O(sqrt vmax).
struct Bucketed(size: Int):
    count: List<Int>
    block: List<Int>

    fn new(vmax: Int) -> Bucketed:
        let size = max(1, isqrt(vmax))
        Bucketed(size, [0 for _ in 0..vmax], [0 for _ in 0..ceil_div(vmax, size)])

    fn add(mut self, x: Int, by: Int):
        count[x] += by
        block[x / size] += by

    fn kth(self, k: Int) -> Int:
        var left = k
        var b = 0
        while left > block[b]:
            left -= block[b]
            b += 1
        var x = b * size
        while left > count[x]:
            left -= count[x]
            x += 1
        x

var f: [V]Int = 0
var values = Bucketed.new(bit_ceil(n + 1))
values.add(0, n)
var arcs_in: [E]Int = 0

fn relabel(u: V, by: V):
    values.add(f[u], -1)
    f[u] ^= by.label
    values.add(f[u], 1)

fn step(p: Arc, by: Int):
    let (u, v, e) = arcs[p.to_idx]
    let was_inside = arcs_in[e] == 2
    arcs_in[e] += by
    if was_inside != (arcs_in[e] == 2):
        relabel(u, by: v)
        relabel(v, by: u)

let answer = qi => values.kth(asks[qi].k + n - (asks[qi].r - asks[qi].l + 1))
print(mo(windows, add: p => step(p, 1), remove: p => step(p, -1), answer: answer), sep: "\n")
