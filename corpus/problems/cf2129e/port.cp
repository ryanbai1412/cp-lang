// Source: Codeforces 2129E (Induced Subgraph Queries),
//   https://codeforces.com/contest/2129/problem/E
// Problem: a simple undirected graph, n, m <= 1.5e5. Each of q <= 1.5e5 queries (l, r, k): in the
//   subgraph induced by l..=r, f(u) is the XOR of u's neighbours' labels; print the k-th
//   smallest f(u) over u in l..=r. Multitest, sums of n, m, q <= 1.5e5.
// Solution: list every vertex's arcs in vertex order; the arcs leaving l..=r form one window, and
//   an edge is induced iff both its arcs are in it. Mo's algorithm over arc windows flips f at
//   both ends when an edge's arc count crosses 2, O(1) per step. All n values of f sit in a
//   sqrt-bucketed count (O(1) update, O(sqrt n) k-th); the vertices outside l..=r have f = 0,
//   so the answer is the (k + n - (r - l + 1))-th overall. O(m sqrt q + q sqrt n).
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
    Arc.cuts.nth(first_arc.prefix(x.l.l))..Arc.cuts.nth(first_arc.prefix(x.r.r)) for x in asks
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
    let (u, v, e) = arcs[p.ord]
    let was_inside = arcs_in[e] == 2
    arcs_in[e] += by
    if was_inside != (arcs_in[e] == 2):
        relabel(u, by: v)
        relabel(v, by: u)

let answer = qi => values.kth(asks[qi].k + n - (asks[qi].r - asks[qi].l + 1))
print(mo(windows, add: p => step(p, 1), remove: p => step(p, -1), answer:), sep: "\n")
