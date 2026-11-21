// Divide-and-conquer on weight range [lo, hi] with rollback connectivity.
// Invariant: `conn` holds all edges with weight outside [lo, hi], carrying one
// representative colored vertex per component. Adding all edges in one half
// merges colored components at the other half's minimum weight (lo or mid + 1),
// which builds the minimum spanning forest over colored vertices in O(m log m).
@!multitest
let (n, m, q): (1..=3e5, 1..=3e5, 1..=3e5) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
let colors: [0..q]V = read()

var is_good: [V]Bool = false
for v in colors: is_good[v] = true
let need = count(V, v => is_good[v]) - 1

let by_w = edges.filter(e => e.u != e.v).group(by: e => e.w)
var conn = DsuOf<V, ?V>(v => if is_good[v]: v else: none, (a, b) => a ?? b)
var mst = Dsu<V>()
var ans = 0
var links = 0

fn add_range(r: Range<Int>, cost: Int):
    for w in r:
        for (u, v, _) in by_w[w]:
            let (a, b) = (conn.get(u), conn.get(v))
            if conn.unite(u, v):
                if let (x, y) = (a, b), mst.unite(x, y):
                    ans += cost
                    links += 1

fn solve(lo: Int, hi: Int):
    if links == need or lo == hi: return
    let mid = (lo + hi) / 2
    with conn.checkpoint():
        add_range(mid + 1..=hi, lo)
        solve(lo, mid)
    if links == need: return
    with conn.checkpoint():
        add_range(lo..=mid, mid + 1)
        solve(mid + 1, hi)

solve(0, m)
print(if links == need: ans else: -1)
