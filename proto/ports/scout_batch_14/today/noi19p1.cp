// NOI 2019 Route: reach station n from station 1 with minimum arrival time plus
// quadratic waiting cost A*dt^2 + B*dt + C at intermediate stations. Event-sweep
// DP with one dynamic Li Chao tree per station.
// original: corpus/submissions/3790564.cpp
let n: 1..=100000 = read()
let m: 1..=200000 = read()
let ax: 0..=10 = read()
let bx: 0..=10000 = read()
let cx: 0..=10000 = read()
domain Station = 1..=n
type Route = (u: Station, v: Station, s: 0..=1000, e: 0..=1000)
let input_routes: [m]Route = read()

// Append synthetic starting route (1, 1, 0, 0) at index m.
var u_arr: List<Int> = []
var v_arr: List<Int> = []
var s_arr: List<Int> = []
var e_arr: List<Int> = []
var max_t = 0
for r in input_routes:
    u_arr.push(r.u.idx)
    v_arr.push(r.v.idx)
    s_arr.push(r.s)
    e_arr.push(r.e)
    max_t max= r.e
u_arr.push(0)
v_arr.push(0)
s_arr.push(0)
e_arr.push(0)
let total_m = m + 1

// Shared pool of Li Chao nodes across all n stations (root[st] = -1 when empty).
const INF = 4000000000000000000
var root: [0..n]Int = -1
var slope: List<Int> = []
var icept: List<Int> = []
var left_c: List<Int> = []
var right_c: List<Int> = []

fn new_node(k: Int, b: Int) -> Int:
    slope.push(k)
    icept.push(b)
    left_c.push(-1)
    right_c.push(-1)
    slope.len - 1

fn add_line(st: Int, k0: Int, b0: Int):
    if root[st] == -1:
        root[st] = new_node(k0, b0)
        return
    var at = root[st]
    var lo = 0
    var hi = max_t
    var k = k0
    var b = b0
    while true:
        let mid = (lo + hi) / 2
        if k * mid + b < slope[at] * mid + icept[at]:
            let ok = slope[at]
            let ob = icept[at]
            slope[at] = k
            icept[at] = b
            k = ok
            b = ob
        if lo == hi: return
        if k * lo + b < slope[at] * lo + icept[at]:
            if left_c[at] == -1:
                left_c[at] = new_node(k, b)
                return
            at = left_c[at]
            hi = mid
        else:
            if right_c[at] == -1:
                right_c[at] = new_node(k, b)
                return
            at = right_c[at]
            lo = mid + 1

fn query_min(st: Int, x: Int) -> Int:
    if root[st] == -1: return INF
    var at = root[st]
    var lo = 0
    var hi = max_t
    var best = INF
    while true:
        best min= slope[at] * x + icept[at]
        if lo == hi: break
        let mid = (lo + hi) / 2
        if x <= mid:
            if left_c[at] == -1: break
            at = left_c[at]
            hi = mid
        else:
            if right_c[at] == -1: break
            at = right_c[at]
            lo = mid + 1
    best

// Sweep backwards in time: at equal timestamps, ADD (0) runs before COMPUTE (1).
type Event = (neg_t: Int, kind: Int, ind: Int)
var events: List<Event> = []
for i in 0..total_m:
    events.push((-s_arr[i], 0, i))
    events.push((-e_arr[i], 1, i))

var dp: [0..total_m]Int = INF
for ev in sorted(events):
    let i = ev.ind
    if ev.kind == 0:
        let s = s_arr[i]
        if dp[i] < INF:
            add_line(u_arr[i], -2 * ax * s, ax * s * s + bx * s + cx + dp[i])
    else:
        let e = e_arr[i]
        if v_arr[i] == n - 1:
            dp[i] = e
        else:
            let q = query_min(v_arr[i], e)
            if q < INF: dp[i] = q + ax * e * e - bx * e

print(dp[m])
