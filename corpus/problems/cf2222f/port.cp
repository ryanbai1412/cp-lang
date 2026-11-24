// Source: Codeforces 2222F (Building Tree), https://codeforces.com/contest/2222/problem/F
// limits: 2500 ms, 512 MB
// Problem: a graph of n, m <= 3e5 with edge weights 0..=m; a path's length is the mex of its
//   weights, dis(u, v) the least over u-v paths. q vertices with colors c_i: joining two costs
//   dis(c_u, c_v), allowed only when connected. Print the cheapest way to connect all q, or -1.
//   Multitest, sums of n, m, q <= 3e5.
// Solution: dis(u, v) <= x iff u, v connect while avoiding the weight-y edges, some y <= x, so
//   Kruskal over x = 0..=m: divide and conquer on [lo, hi] keeps the edges outside [lo, hi] in a
//   rollback DSU carrying one colored vertex per component; adding one half's edges merges
//   colored components at the other half's least x, joined in a second DSU. Each edge is added
//   O(log m) times: O(m log m log n).
@!multitest
use std.dsu.DsuOf

let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
domain W = 0..=m
let edges: [0..m](u: V, v: V, w: W) = read()
let colors: [0..q]V = read()

let good = Bitset<V>.of(colors.collect())
let need = good.len - 1

var by_w: [W]List<(u: V, v: V)> = []
for e in edges if e.u != e.v:
    by_w[e.w].push((u: e.u, v: e.v))

var conn = DsuOf<V, ?V>(v => if v in good: v else: None, (a, b) => a ?? b)
var mst = Dsu<V>()
var ans = 0
var links = 0

fn add_range(lo: W, hi: W, cost: W):
    for w in lo..=hi:
        for e in by_w[w]:
            let (x, y) = (conn.get(e.u), conn.get(e.v))
            if conn.unite(e.u, e.v):
                if let a = x and let b = y and mst.unite(a, b):
                    ans += cost.ord
                    links += 1

fn solve(lo: W, hi: W):
    if links == need or lo == hi: return
    let mid = W((lo.ord + hi.ord) / 2)
    let right = mid.next!
    let snap = conn.mark()
    add_range(right, hi, lo)
    solve(lo, mid)
    conn.undo_to(snap)

    if links == need: return
    add_range(lo, mid, right)
    solve(right, hi)
    conn.undo_to(snap)

solve(W.first!, W.last!)
print(if links == need: ans else: -1)
