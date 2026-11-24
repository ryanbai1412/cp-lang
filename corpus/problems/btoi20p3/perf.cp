// Tuned: the accepted C++'s recursion, which keeps the left part's edges for the right half and
// re-adds only the scanned right edges for the left, so each level pushes fewer edges; and the
// rollback DSU logs one entry per pushed edge.
//
// Source: BOI 2020 P3 (Joker), https://dmoj.ca/problem/btoi20p3
// limits: 1000 ms, 256 MB
// Problem: a graph with n, m, q <= 2e5; each query l r removes edges l..=r. Print YES if the
//   remaining edges contain an odd cycle, else NO.
// Solution: limit[l] = the least R such that edges [1, l) and (R, m] are bipartite; it is
//   non-decreasing in l, and a query says YES iff r < limit[l]. Divide and conquer on l with
//   limit's range narrowing, over a parity DSU with union by size and rollback (no path
//   compression): each level adds the edges its halves share, then undoes them.
//   O(m log m log n).
// original: corpus/submissions/4656866.cpp
// Bounds: edges.u, edges.v: 1..=n; queries.l, queries.r: 1..=m

let (n, m, q): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 1..=m
domain Q = 0..q
let edges: [E](u: V, v: V) = read()
let queries: [Q](l: E, r: E) = read()

var up: [V]V = [V] v => v
var sz: [V]Int = 1
var par: [V]Bool = false
var bipartite = true
// one entry per pushed edge: the root it hung (if it joined two), the old size of the root
// above it, and the old flag
var (kid, joined, old_sz, was) = (List<V>(), List<Bool>(), List<Int>(), List<Bool>())

fn find_root(v: V) -> (root: V, flip: Bool):
    var (x, f) = (v, false)
    while up[x] != x:
        f = f != par[x]
        x = up[x]
    (root: x, flip: f)

// edge E(i + 1)
fn push(i: Int):
    let e = edges[E(i + 1)]
    let (ru, rv) = (find_root(e.u), find_root(e.v))
    was.push(bipartite)
    if ru.root == rv.root:
        if ru.flip == rv.flip: bipartite = false
        kid.push(ru.root)
        joined.push(false)
        old_sz.push(0)
        return
    var (a, b) = (ru.root, rv.root)
    if sz[a] < sz[b]: swap(mut a, mut b)
    kid.push(b)
    joined.push(true)
    old_sz.push(sz[a])
    up[b] = a
    par[b] = ru.flip == rv.flip
    sz[a] += sz[b]

fn pop():
    bipartite = was.pop()!
    let c = kid.pop()!
    let s = old_sz.pop()!
    if joined.pop()!:
        sz[up[c]] = s
        up[c] = c
        par[c] = false

// last[l] = the greatest r (0-based) such that removing edges l..=r leaves an odd cycle
var last = List.filled(m, -1)

fn solve(l: Int, r: Int, ql: Int, qr: Int):
    if l > r: return
    let mid = (l + r) / 2
    for i in l..mid: push(i)
    var cr = qr
    while cr >= ql and bipartite:
        push(cr)
        cr -= 1
    last[mid] = cr
    for _ in cr + 1..=qr: pop()
    push(mid)
    solve(mid + 1, r, cr, qr)
    for _ in l..=mid: pop()
    for i in cr + 1..=qr: push(i)
    solve(l, mid - 1, ql, cr)
    for _ in cr + 1..=qr: pop()

for i in 0..m: push(i)
let odd = not bipartite
for _ in 0..m: pop()
if odd: solve(0, m - 1, 0, m - 1)
for qi in Q:
    print(if odd and queries[qi].r.ord <= last[queries[qi].l.ord]: "YES" else: "NO")
