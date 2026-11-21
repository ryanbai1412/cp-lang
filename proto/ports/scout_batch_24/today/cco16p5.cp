// Runnable port (proto/cp.py lib-run) of statement-first cco16p5.cp
// original: corpus/submissions/4580695.cpp
// Klee's Measure Problem (2D coordinate-compressed sweep-line + tag-free Cover-Count SegTree).
let (n, m): (Int, Int) = read()
let k: Int = read()
domain Z = 0..k
let zombies: [Z](r: Int, c: Int) = read()
let q: Int = read()

var ys: List<Int> = []
var cnt: List<Int> = []
var cov: List<Int> = []

fn ct_update(p: Int, lo: Int, hi: Int, ql: Int, qr: Int, delta: Int):
    if qr <= lo or hi <= ql: return
    if ql <= lo and hi <= qr:
        cnt[p] += delta
    else:
        let mid = (lo + hi) / 2
        ct_update(2 * p, lo, mid, ql, qr, delta)
        ct_update(2 * p + 1, mid, hi, ql, qr, delta)
    if cnt[p] > 0:
        cov[p] = ys[hi] - ys[lo]
    elif lo + 1 == hi:
        cov[p] = 0
    else:
        cov[p] = cov[2 * p] + cov[2 * p + 1]

fn y_id(y: Int) -> Int:
    var (lo, hi) = (0, ys.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if ys[mid] < y: lo = mid + 1
        else: hi = mid
    lo

fn union_area(rad: Int) -> Int:
    if rad < 0: return 0
    var raw_y: List<Int> = []
    var evs: List<(x: Int, y1: Int, y2: Int, d: Int)> = []
    for z in Z:
        let x1 = max(1, zombies[z].r - rad)
        let x2 = min(n, zombies[z].r + rad) + 1
        let y1 = max(1, zombies[z].c - rad)
        let y2 = min(m, zombies[z].c + rad) + 1
        raw_y.push(y1)
        raw_y.push(y2)
        evs.push((x: x1, y1: y1, y2: y2, d: 1))
        evs.push((x: x2, y1: y1, y2: y2, d: -1))
    let sy = raw_y.sorted()
    ys = []
    for y in sy:
        if ys.len == 0 or ys.last! != y:
            ys.push(y)
    let segs = max(1, ys.len - 1)
    cnt = List.filled(4 * segs + 4, 0)
    cov = List.filled(4 * segs + 4, 0)
    let sevs = evs.sorted()
    var area = 0
    var prev_x = sevs[0].x
    for ev in sevs:
        area += (ev.x - prev_x) * cov[1]
        let ql = y_id(ev.y1)
        let qr = y_id(ev.y2)
        if ql < qr:
            ct_update(1, 0, ys.len - 1, ql, qr, ev.d)
        prev_x = ev.x
    area

print(union_area(q) - union_area(q - 1))
