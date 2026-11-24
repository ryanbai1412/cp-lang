// Source: Codeforces 2129E (Induced Subgraph Queries),
//   https://codeforces.com/contest/2129/problem/E
// limits: 5000 ms, 512 MB
// Problem: a simple undirected graph, n, m <= 1.5e5. Each of q <= 1.5e5 queries (l, r, k): in the
//   subgraph induced by l..=r, f(u) is the XOR of u's neighbours' labels; print the k-th
//   smallest f(u) over u in l..=r. Multitest, sums of n, m, q <= 1.5e5.
// Solution: list every vertex's arcs in vertex order; the arcs leaving l..=r form one window, and
//   an edge is induced iff both its arcs are in it. Mo's algorithm over arc windows flips f at
//   both ends when an edge's arc count crosses 2, O(1) per step. All n values of f sit in a
//   sqrt-bucketed count (O(1) update, O(sqrt n) k-th); the vertices outside l..=r have f = 0,
//   so the answer is the (k + n - (r - l + 1))-th overall. O(m sqrt q + q sqrt n).
@!multitest
use std.bag.{BucketCount}
use std.mo.{mo}
let (n, m): (2..=150_000, 1..=150_000) = read()
domain V = 1..=n
domain E = 0..m
let raw: [E](u: V, v: V) = read()
let g = Graph.undirected(raw)
let (domain Arc, arcs) = [(u, a.to, a.e) for u in V for a in g.out(u)].with_domain()
let degs = prefix([V] u => g.deg(u))
let first_arc = [V.cuts] b => Arc.cuts.nth(degs.prefix(b))

let q: 1..=150_000 = read()
let asks: [0..q](l: V, r: V, k: 1..=n) = read()

var f: [V]Int = 0
var values = BucketCount(0..bit_ceil(n + 1))
values.add(0, n)
var arcs_in: [E]0..=2 = 0

fn relabel(u: V, by: V):
    values.remove(f[u])
    f[u] ^= by.label
    values.add(f[u])

fn step(p: Arc, by: Int):
    let (u, v, e) = arcs[p]
    let was_inside = arcs_in[e] == 2
    arcs_in[e] += by
    if was_inside != (arcs_in[e] == 2):
        relabel(u, by: v)
        relabel(v, by: u)

let windows = [first_arc[x.l.l]..first_arc[x.r.r] for x in asks]
fn answer(qi: Int) -> Int:
    let x = asks[qi]
    values.kth(x.k - 1 + n - (x.l..=x.r).len)

print(mo(windows, add: p => step(p, 1), remove: p => step(p, -1), answer:), sep: "\n")
