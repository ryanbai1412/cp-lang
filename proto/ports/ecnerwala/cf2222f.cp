// Codeforces 2222 F: Building Tree.
// Divide-and-conquer on weight range [lo, hi] with rollback connectivity.
// Invariant: `conn` holds all edges with weight outside [lo, hi], carrying one
// representative colored vertex per component. Adding all edges in one half
// merges colored components at the other half's minimum weight (lo or mid + 1),
// which builds the minimum spanning forest over colored vertices in O(m log m).
@!multitest
use std.dsu.{DsuOf}

let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
let colors: [0..q]V = read()

var is_good: [V]Bool = false
for v in colors: is_good[v] = true
let need = count(V, v => is_good[v]) - 1

var by_w: [0..=m]List<(u: V, v: V)> = []
for e in edges:
    if e.u != e.v:
        by_w[e.w].push((u: e.u, v: e.v))

var conn = DsuOf<V, ?V>(v => if is_good[v]: v else: none, (a, b) => a ?? b)
var mst = Dsu<V>()
var ans = 0
var links = 0

fn add_range(l: Int, r: Int, cost: Int):
    for w in l..=r:
        for e in by_w[w]:
            let (x, y) = (conn.get(e.u), conn.get(e.v))
            if conn.unite(e.u, e.v):
                if let a = x, let b = y, mst.unite(a, b):
                    ans += cost
                    links += 1

fn solve(lo: Int, hi: Int):
    if links == need or lo == hi: return
    let mid = (lo + hi) / 2
    let snap = conn.mark()
    add_range(mid + 1, hi, lo)
    solve(lo, mid)
    conn.undo_to(snap)

    if links == need: return
    add_range(lo, mid, mid + 1)
    solve(mid + 1, hi)
    conn.undo_to(snap)

solve(0, m)
print(if links == need: ans else: -1)
