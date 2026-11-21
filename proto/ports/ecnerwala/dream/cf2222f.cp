// Codeforces 2222 F.
// Divide-and-conquer on weight range `[lo, hi]` with rollback connectivity.
// Invariant: `conn` holds all edges with weight outside `[lo, hi]`, carrying
// one representative colored vertex per component. Adding all edges in one half
// merges colored components at the other half's minimum weight (`lo` or
// `mid + 1`), building the minimum spanning forest over colored vertices in
// O(m log m).

@!multitest
let (n, m, q): (1..=300_000, 1..=300_000, 1..=300_000) = read()
domain V = 1..=n
domain W = 0..=m
let edges: [0..m](u: V, v: V, w: W) = read()
let colors: [0..q]V = read()

let good = V.mask(colors)
let need = good.count() - 1

let by_w = edges.filter(e => e.u != e.v).group(by: e => e.w)
var conn = DsuOf<V, ?V>(v => if good[v]: v else: none, (a, b) => a ?? b)
var mst = Dsu<V>()
var ans = 0
var links = 0

fn add_range(r: Range<W>, cost: W):
    for w in r:
        for (u, v, _) in by_w[w]:
            let (a, b) = (conn.get(u), conn.get(v))
            if conn.unite(u, v):
                if let (x, y) = (a, b), mst.unite(x, y):
                    ans += cost.ord
                    links += 1

fn solve(lo: W, hi: W):
    if links == need or lo == hi: return
    let mid = W((lo.ord + hi.ord) / 2)
    let right = (mid + 1)!
    with conn.checkpoint():
        add_range(right..=hi, lo)
        solve(lo, mid)
    if links == need: return
    with conn.checkpoint():
        add_range(lo..=mid, right)
        solve(right, hi)

solve(W.first, W.last)
print(if links == need: ans else: -1)
