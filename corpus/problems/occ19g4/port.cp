// Source: OCC '19 G4 (Willson and Fighting), https://dmoj.ca/problem/occ19g4
// limits: 1000 ms, 256 MB
// Problem: N <= 60000; 2N geese at distinct points with coordinates in 1..=60000, geese 2k-1
//   and 2k are mates. Goose p honks at goose r if the Manhattan dist(p, r) < dist(p, mate of p);
//   two geese that honk at each other fight once. Print each goose's number of fights.
// Solution: rotate to (x - y, x + y) so Manhattan distance becomes Chebyshev; with radius
//   d = dist to mate - 1, two geese fight iff their distance is <= both radii (mates never
//   are). Sort by radius, largest first. A CDQ divide and conquer over that order gives each
//   goose the larger-radius geese inside its square (sweep the first coordinate, Fenwick over
//   the compressed second); on the reversed order, the smaller-radius squares covering each
//   goose (range add, point query). O(N log^2 N).
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
    pts.push((key_d: -d, id: 2 * k, u: ua, v: va, d:))
    pts.push((key_d: -d, id: 2 * k + 1, u: ub, v: vb, d:))
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
    let lo = ys.first_ge(val) ?? ys.len
    Y(lo)

// each goose's compressed second coordinate and square edges, looked up once
let geese = [
    (
        key_d: p.key_d,
        id: p.id,
        u: p.u,
        d: p.d,
        y: y_at(p.v),
        lo: y_at(p.v - max(p.d, 0)),
        hi: y_at(p.v + max(p.d, 0)),
        hi1: y_at(p.v + p.d + 1),
    ) for p in pts
].sorted()

var bit = Fenwick<Y, Int>()
var ans: [P]Int = [P] _ => 0

fn cdq_query(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_query(l, m)
    cdq_query(m, r)
    var left: List<(u: Int, y: Y)> = []
    for i in l..m:
        left.push((u: geese[i].u, y: geese[i].y))
    var right: List<(u: Int, sgn: Int, lo: Y, hi: Y, id: Int)> = []
    for i in m..r:
        let p = geese[i]
        if p.d < 0: continue
        right.push((u: p.u + p.d, sgn: 1, lo: p.lo, hi: p.hi, id: p.id))
        right.push((u: p.u - p.d - 1, sgn: -1, lo: p.lo, hi: p.hi, id: p.id))
    left.sort(by: e => e.u)
    right.sort(by: e => e.u)
    var li = 0
    for q in right:
        while li < left.len and left[li].u <= q.u:
            bit.add(left[li].y, 1)
            li += 1
        ans[P(q.id)] += q.sgn * bit.fold(q.lo..=q.hi)
    for j in 0..li:
        bit.add(left[j].y, -1)

cdq_query(0, 2 * n)

let rev = geese.rev.collect()

fn cdq_cover(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_cover(l, m)
    cdq_cover(m, r)
    var left: List<(u: Int, sgn: Int, lo: Y, hi1: Y)> = []
    for i in l..m:
        let p = rev[i]
        if p.d < 0: continue
        left.push((u: p.u - p.d, sgn: 1, lo: p.lo, hi1: p.hi1))
        left.push((u: p.u + p.d + 1, sgn: -1, lo: p.lo, hi1: p.hi1))
    var right: List<(u: Int, y: Y, id: Int)> = []
    for i in m..r:
        let p = rev[i]
        right.push((u: p.u, y: p.y, id: p.id))
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
