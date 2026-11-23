// Source: IOI 2016 Day 2 (Aliens), standard I/O version, https://dmoj.ca/problem/ioi16p6io
// Problem: n <= 1e5 points (r, c) in an m x m grid, m <= 1e6. Take at most k <= n photos, each
//   a square with two opposite corners on the main diagonal, covering every point; print the
//   least number of cells photographed at least once.
// Solution: point (r, c) needs the diagonal range [min, max]; drop ranges inside others and the
//   rest sort with both ends increasing, so each photo covers a run of them. Aliens trick: charge
//   lam per photo and binary search the least lam whose optimum uses <= k photos, then answer
//   cost - k * lam. For fixed lam a DP from the right takes a run i..j-1 for (r_{j-1}+1 - l_i)^2
//   minus the overlap with range i-1; the square is a line in l_i, so a Li Chao tree over 0..=m
//   (ties toward fewer photos) makes it O(n log m) per lam, O(n log m log m^2) overall.
// original: corpus/submissions/4730558.cpp
let n: 1..=100000 = read()
let m: 1..=1000000 = read()
let k: 1..=100000 = read()
let raw: [n](r: 0..=1000000, c: 0..=1000000) = read()

// Normalize points into intervals [min(r, c), max(r, c)] and strip dominated ones.
type Seg = (l: Int, r: Int)
var intervals: List<Seg> = [(min(p.r, p.c), max(p.r, p.c)) for p in raw]
let ordered = intervals.sorted(by: s => (s.l, -s.r))
var segs: List<Seg> = []
var max_r = -1
for s in ordered:
    if s.r > max_r:
        max_r = s.r
        segs.push(s)
let cnt_s = segs.len

// Dynamic Li Chao tree over 0..=m storing lines (slope, intercept, photo_count),
// minimizing (slope * x + intercept, photo_count) lexicographically.
var slope: List<Int> = []
var icept: List<Int> = []
var lcnt: List<Int> = []
var left_c: List<Int> = []
var right_c: List<Int> = []

fn new_node(ms: Int, mb: Int, mc: Int) -> Int:
    slope.push(ms)
    icept.push(mb)
    lcnt.push(mc)
    left_c.push(-1)
    right_c.push(-1)
    slope.len - 1

fn better(s1: Int, b1: Int, c1: Int, s2: Int, b2: Int, c2: Int, x: Int) -> Bool:
    let v1 = s1 * x + b1
    let v2 = s2 * x + b2
    v1 < v2 or (v1 == v2 and c1 < c2)

fn lc_add(ms0: Int, mb0: Int, mc0: Int):
    if slope.len == 0:
        let _ = new_node(ms0, mb0, mc0)
        return
    var at = 0
    var lo = 0
    var hi = m
    var ms = ms0
    var mb = mb0
    var mc = mc0
    while true:
        let mid = (lo + hi) / 2
        if better(ms, mb, mc, slope[at], icept[at], lcnt[at], mid):
            let os = slope[at]
            let ob = icept[at]
            let oc = lcnt[at]
            slope[at] = ms
            icept[at] = mb
            lcnt[at] = mc
            ms = os
            mb = ob
            mc = oc
        if lo == hi: return
        if better(ms, mb, mc, slope[at], icept[at], lcnt[at], lo):
            if left_c[at] == -1:
                left_c[at] = new_node(ms, mb, mc)
                return
            at = left_c[at]
            hi = mid
        else:
            if right_c[at] == -1:
                right_c[at] = new_node(ms, mb, mc)
                return
            at = right_c[at]
            lo = mid + 1

fn lc_query(x: Int) -> (cost: Int, used: Int):
    var at = 0
    var lo = 0
    var hi = m
    var best_v = slope[0] * x + icept[0]
    var best_c = lcnt[0]
    while true:
        let v = slope[at] * x + icept[at]
        let c = lcnt[at]
        if v < best_v or (v == best_v and c < best_c):
            best_v = v
            best_c = c
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
    (best_v, best_c)

fn eval_penalty(lam: Int) -> (cost: Int, used: Int):
    slope = []
    icept = []
    lcnt = []
    left_c = []
    right_c = []
    var dp_cost: [0..=cnt_s]Int = 0
    var dp_cnt: [0..=cnt_s]Int = 0
    let last_r = segs[cnt_s - 1].r + 1
    lc_add(-2 * last_r, last_r * last_r, 0)
    for i in (0..cnt_s).rev:
        let q = lc_query(segs[i].l)
        var cost = q.cost + lam + segs[i].l * segs[i].l
        let used = q.used + 1
        if i > 0 and segs[i].l <= segs[i - 1].r:
            let ov = segs[i - 1].r - segs[i].l + 1
            cost -= ov * ov
        dp_cost[i] = cost
        dp_cnt[i] = used
        if i > 0:
            let r = segs[i - 1].r + 1
            lc_add(-2 * r, cost + r * r, used)
    (dp_cost[0], dp_cnt[0])

let lam = (0..=m * m).first_true(pen => eval_penalty(pen).used <= k) ?? (m * m)
let res = eval_penalty(lam)
print(res.cost - k * lam)
