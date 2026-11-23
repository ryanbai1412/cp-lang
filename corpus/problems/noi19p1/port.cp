// NOI 2019 Route (CP Lang 0.2.2): event-sweep DP with dynamic per-station Li Chao
// trees using Tropical Optionals `?Int` and optional node pointers (zero `INF`/`-1`).
// original: corpus/submissions/3790564.cpp
let (n, m, ax, bx, cx): (1..=100_000, 1..=200_000, 0..=10, 0..=10_000, 0..=10_000) = read()
domain Station = 1..=n
type Route = (u: Station, v: Station, s: 0..=1000, e: 0..=1000)

var routes: List<Route> = []
var max_t = 0
for _ in 0..m:
    let r: Route = read()
    routes.push(r)
    max_t max= r.e
routes.push((u: Station.first, v: Station.first, s: 0, e: 0))
let total_m = m + 1

var root: [Station]?Int = None
var slope: List<Int> = []
var icept: List<Int> = []
var left_c: List<?Int> = []
var right_c: List<?Int> = []

fn new_node(k: Int, b: Int) -> Int:
    slope.push(k)
    icept.push(b)
    left_c.push(None)
    right_c.push(None)
    slope.len - 1

fn add_line(st: Station, k0: Int, b0: Int):
    if root[st] == None:
        root[st] = new_node(k0, b0)
        return
    var at = root[st]!
    var (lo, hi) = (0, max_t)
    var (k, b) = (k0, b0)
    while true:
        let mid = (lo + hi) / 2
        if k * mid + b < slope[at] * mid + icept[at]:
            (slope[at], k) = (k, slope[at])
            (icept[at], b) = (b, icept[at])
        if lo == hi: return
        if k * lo + b < slope[at] * lo + icept[at]:
            if let lc = left_c[at]:
                at = lc
                hi = mid
            else:
                left_c[at] = new_node(k, b)
                return
        else:
            if let rc = right_c[at]:
                at = rc
                lo = mid + 1
            else:
                right_c[at] = new_node(k, b)
                return

fn query_min(st: Station, x: Int) -> ?Int:
    var cur = root[st]
    var (lo, hi) = (0, max_t)
    var best: ?Int = None
    while let at = cur:
        best min= slope[at] * x + icept[at]
        if lo == hi: break
        let mid = (lo + hi) / 2
        if x <= mid:
            cur = left_c[at]
            hi = mid
        else:
            cur = right_c[at]
            lo = mid + 1
    best

var events: List<(Int, Int, Int)> = []
for i in 0..total_m:
    events.push((-routes[i].s, 0, i))
    events.push((-routes[i].e, 1, i))

var dp: [0..total_m]?Int = None
for (_, kind, i) in events.sorted():
    let r = routes[i]
    if kind == 0:
        if let cost = dp[i]:
            add_line(r.u, -2 * ax * r.s, ax * r.s * r.s + bx * r.s + cx + cost)
    else:
        if r.v == Station.last:
            dp[i] = r.e
        else:
            dp[i] = query_min(r.v, r.e) + (ax * r.e * r.e - bx * r.e)

print(dp[m]!)
