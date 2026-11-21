// OCC '19 G4 — Rotated-Manhattan 2D Range Counting (CP Lang v2 spec)
// original: corpus/submissions/3578469.cpp (TLE in corpus due to sparse gp_hash_table
//           2D BIT and an unshifted negative-index infinite loop when x - y + maxc - d < 0)
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Rethought via CDQ divide-and-conquer + 1D coordinate-compressed Fenwick tree:
// O(N log^2 N) time and O(N) memory, safe for arbitrary 64-bit coordinates.
// For each pair (2k, 2k+1), rotate to Chebyshev (u, v) = (x - y, x + y), set
// d = max(|du|, |dv|) - 1, and count other points j with chebyshev(i, j) <= min(d_i, d_j).

let n: 1..=6e4 = read()
domain P = 0..(2 * n)
let raw: [P](x: Int, y: Int) = read()

type Item = (u: Int, v: Int, d: Int, id: P)
var pts: List<Item> = []
var all_v: List<Int> = []
for k in 0..n:
    let (a, b) = (P.from_idx(2 * k)!, P.from_idx(2 * k + 1)!)
    let (ua, va) = (raw[a].x - raw[a].y, raw[a].x + raw[a].y)
    let (ub, vb) = (raw[b].x - raw[b].y, raw[b].x + raw[b].y)
    let d = max(abs(ua - ub), abs(va - vb)) - 1
    pts.push((u: ua, v: va, d:, id: a))
    pts.push((u: ub, v: vb, d:, id: b))
    for v in [va, vb]:
        all_v.push(v)
        if d >= 0:
            all_v.push(v - d)
            all_v.push(v + d)
            all_v.push(v + d + 1)

// FRICTION MC4: Sparse/jagged 2D Fenwick (`[X]Fenwick<Y_x, Int>`) cannot be typed
// with per-bucket domains in v2, pushing 2D sparse counting cleanly to CDQ + 1D Fenwick.
let ys = compress(all_v)
var bit = Fenwick<ys.D, Int>()
var ans: [P]Int = 0

pts.sort(by: p => (desc(p.d), p.id))

// Pass 1 (descending d): earlier points b have d_b >= d_a; query a's square [u-d..u+d] x [v-d..v+d].
type RectQ = (u: Int, sgn: Int, lo: ys.D, hi: ys.D, id: P)

fn cdq_query(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_query(l, m)
    cdq_query(m, r)
    var left = [(u: pts[i].u, y: ys.at(pts[i].v)) for i in l..m]
    var right: List<RectQ> = []
    for i in m..r:
        let p = pts[i]
        if p.d < 0: continue
        let (lo, hi) = (ys.at(p.v - p.d), ys.at(p.v + p.d))
        right.push((u: p.u + p.d, sgn: 1, lo:, hi:, id: p.id))
        right.push((u: p.u - p.d - 1, sgn: -1, lo:, hi:, id: p.id))
    left.sort(by: e => e.u)
    right.sort(by: e => e.u)
    var li = 0
    for q in right:
        while li < left.len and left[li].u <= q.u:
            bit.add(left[li].y, 1)
            li += 1
        ans[q.id] += q.sgn * bit.sum(q.lo..=q.hi)
    for j in 0..li: bit.add(left[j].y, -1)

cdq_query(0, 2 * n)

// Pass 2 (ascending d): earlier points b have d_b <= d_a; b stamps its box, a queries point (u_a, v_a).
pts.reverse()
type BoxEv = (u: Int, sgn: Int, lo: ys.D, hi1: ys.D)

fn cdq_cover(l: Int, r: Int):
    if r - l <= 1: return
    let m = (l + r) / 2
    cdq_cover(l, m)
    cdq_cover(m, r)
    var left: List<BoxEv> = []
    for i in l..m:
        let p = pts[i]
        if p.d < 0: continue
        let (lo, hi1) = (ys.at(p.v - p.d), ys.at(p.v + p.d + 1))
        left.push((u: p.u - p.d, sgn: 1, lo:, hi1:))
        left.push((u: p.u + p.d + 1, sgn: -1, lo:, hi1:))
    var right = [(u: pts[i].u, y: ys.at(pts[i].v), id: pts[i].id) for i in m..r]
    left.sort(by: e => e.u)
    right.sort(by: e => e.u)
    var li = 0
    for q in right:
        while li < left.len and left[li].u <= q.u:
            bit.add(left[li].lo, left[li].sgn)
            bit.add(left[li].hi1, -left[li].sgn)
            li += 1
        ans[q.id] += bit.sum(..=q.y)
    for j in 0..li:
        bit.add(left[j].lo, -left[j].sgn)
        bit.add(left[j].hi1, left[j].sgn)

cdq_cover(0, 2 * n)

for p in P: print(ans[p])
