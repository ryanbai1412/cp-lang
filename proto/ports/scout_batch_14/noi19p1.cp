// NOI 2019 Route: reach station n from station 1 with minimum arrival time plus
// quadratic waiting cost A*dt^2 + B*dt + C at intermediate stations. Event-sweep
// DP with one dynamic Li Chao tree per station.
// original: corpus/submissions/3790564.cpp
use local.ds.li_chao.{Line, LiChaoArena}                          // WISH P8, P12

let n: 1..=100000 = read()
let m: 1..=200000 = read()
let ax: 0..=10 = read()
let bx: 0..=10000 = read()
let cx: 0..=10000 = read()
domain Station = 1..=n
type Route = (u: Station, v: Station, s: 0..=1000, e: 0..=1000)
let input_routes: [m]Route = read()

var routes: List<Route> = [r for r in input_routes]
routes.push((Station.first, Station.first, 0, 0))
let max_t = max(r.e for r in routes)

// Shared arena of Li Chao trees indexed by station.
var arena = LiChaoArena<Line, Int>.new(Station, 0, max_t)         // WISH P12

// Sweep backwards in time: at equal timestamps, ADD (0) runs before COMPUTE (1).
type Event = (neg_t: Int, kind: Int, ind: Int)
var events: List<Event> = []
for i in 0..routes.len:
    events.push((-routes[i].s, 0, i))
    events.push((-routes[i].e, 1, i))

var dp: [0..routes.len]?Int = none
for ev in sorted(events):
    let r = routes[ev.ind]
    if ev.kind == 0:
        if let val = dp[ev.ind]:
            arena.add(r.u, Line(-2 * ax * r.s, ax * r.s * r.s + bx * r.s + cx + val))
    else:
        if r.v == Station.last:
            dp[ev.ind] = r.e
        elif let q = arena.min_at(r.v, r.e):
            dp[ev.ind] = q + ax * r.e * r.e - bx * r.e

print(dp[m]!)
