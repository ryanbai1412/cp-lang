// Codeforces 2222 F: Building Tree.
// Divide-and-conquer on weight range [lo, hi] with rollback connectivity.
// Invariant: `conn` holds all edges with weight outside [lo, hi], carrying one
// representative colored vertex per component. Adding all edges in one half
// merges colored components at the other half's minimum weight (lo or mid + 1),
// which builds the minimum spanning forest over colored vertices in O(m log m).
@!multitest
use std.dsu.DsuOf

let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
domain W = 0..=m
let edges: [0..m](u: V, v: V, w: W) = read()
let colors: [0..q]V = read()

let good = Bitset<V>.of([v for v in colors])
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
