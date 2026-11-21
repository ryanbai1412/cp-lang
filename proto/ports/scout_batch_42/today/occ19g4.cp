// Runnable port (proto/cp.py lib-run) of occ19g4.cp
// original: corpus/submissions/3578469.cpp

let n: Int = read()
domain P = 0..(2 * n)

var raw_x: [P]Int = [P] _ => 0
var raw_y: [P]Int = [P] _ => 0
for p in P:
    let (x, y): (Int, Int) = read()
    raw_x[p] = x
    raw_y[p] = y

var pts: List<(key_d: Int, id: Int, u: Int, v: Int, d: Int)> = []
var all_v: List<Int> = []
for k in 0..n:
    let a = P(2 * k)
    let b = P(2 * k + 1)
    let ua = raw_x[a] - raw_y[a]
    let va = raw_x[a] + raw_y[a]
    let ub = raw_x[b] - raw_y[b]
    let vb = raw_x[b] + raw_y[b]
    let d = max(abs(ua - ub), abs(va - vb)) - 1
    pts.push((key_d: -d, id: 2 * k, u: ua, v: va, d: d))
    pts.push((key_d: -d, id: 2 * k + 1, u: ub, v: vb, d: d))
    all_v.push(va)
    all_v.push(vb)
    if d >= 0:
        all_v.push(va - d)
        all_v.push(va + d)
        all_v.push(va + d + 1)
        all_v.push(vb - d)
        all_v.push(vb + d)
        all_v.push(vb + d + 1)

let sv = all_v.sorted()
var ys: List<Int> = []
for v in sv:
    if ys.len == 0 or ys.last! != v: ys.push(v)
domain Y = 0..ys.len

fn y_at(val: Int) -> Y:
    var lo = 0
    var hi = ys.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if ys[mid] < val: lo = mid + 1
        else: hi = mid
    Y(lo)

var bit = Fenwick<Y, Int>()
var ans: [P]Int = [P] _ => 0

pts.sort()

fn cdq_query(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_query(l, m)
    cdq_query(m, r)
    var left: List<(u: Int, y: Y)> = []
    for i in l..m:
        left.push((u: pts[i].u, y: y_at(pts[i].v)))
    var right: List<(u: Int, sgn: Int, lo: Y, hi: Y, id: Int)> = []
    for i in m..r:
        let p = pts[i]
        if p.d < 0: continue
        let lo = y_at(p.v - p.d)
        let hi = y_at(p.v + p.d)
        right.push((u: p.u + p.d, sgn: 1, lo: lo, hi: hi, id: p.id))
        right.push((u: p.u - p.d - 1, sgn: -1, lo: lo, hi: hi, id: p.id))
    left.sort(by: e => e.u)
    right.sort(by: e => e.u)
    var li = 0
    for q in right:
        while li < left.len and left[li].u <= q.u:
            bit.add(left[li].y, 1)
            li += 1
        ans[P(q.id)] += q.sgn * bit.sum(q.lo..=q.hi)
    for j in 0..li:
        bit.add(left[j].y, -1)

cdq_query(0, 2 * n)

pts.reverse()

fn cdq_cover(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_cover(l, m)
    cdq_cover(m, r)
    var left: List<(u: Int, sgn: Int, lo: Y, hi1: Y)> = []
    for i in l..m:
        let p = pts[i]
        if p.d < 0: continue
        let lo = y_at(p.v - p.d)
        let hi1 = y_at(p.v + p.d + 1)
        left.push((u: p.u - p.d, sgn: 1, lo: lo, hi1: hi1))
        left.push((u: p.u + p.d + 1, sgn: -1, lo: lo, hi1: hi1))
    var right: List<(u: Int, y: Y, id: Int)> = []
    for i in m..r:
        let p = pts[i]
        right.push((u: p.u, y: y_at(p.v), id: p.id))
    left.sort(by: e => e.u)
    right.sort(by: e => e.u)
    var li = 0
    for q in right:
        while li < left.len and left[li].u <= q.u:
            bit.add(left[li].lo, left[li].sgn)
            bit.add(left[li].hi1, -left[li].sgn)
            li += 1
        ans[P(q.id)] += bit.prefix(q.y.r)
    for j in 0..li:
        bit.add(left[j].lo, -left[j].sgn)
        bit.add(left[j].hi1, left[j].sgn)

cdq_cover(0, 2 * n)

for p in P: print(ans[p])
