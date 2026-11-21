// original: 4240171 (aac5p5)
// Count rectangles formed by two horizontal and two vertical segments. Fix the lower
// horizontal level y1, keep the verticals that cross it in two Fenwick trees over x (count,
// and pairs sharing an x), and sweep the upper level y2 upward, dropping verticals that end.
use std.fenwick.{Fenwick}
use std.seq.{dedup}

type H = (y: Int, x1: Int, x2: Int)
type Vt = (x: Int, y1: Int, y2: Int)
let n: Int = read()
let m: Int = read()
let hori: [0..n]H = read()
let vert: [0..m]Vt = read()
var all_x: List<Int> = []
var all_y: List<Int> = []
for h in hori:
    all_x.push(h.x1)
    all_x.push(h.x2)
    all_y.push(h.y)
for v in vert: all_x.push(v.x)
var xs = all_x.sorted()
dedup(mut xs)
var ys = all_y.sorted()
dedup(mut ys)
domain X = 0..xs.len
domain Y = 0..ys.len
fn cx(x: Int) -> X: X(xs.first_ge(x)!)
fn cy(y: Int) -> Y: Y(ys.first_ge(y)!)
var by_y: [Y]List<(X, X)> = []
for h in hori: by_y[cy(h.y)].push((cx(h.x1), cx(h.x2)))
fn c2(x: Int) -> Int: x * (x - 1) / 2

var ans = 0
for l1 in Y:
    let y1 = ys[l1.ord]
    var cnt: [X]Int = 0
    var b1 = Fenwick<X>()               // verticals per x
    var b2 = Fenwick<X>()               // pairs of verticals on the same x
    fn upd(i: X, d: Int):
        b1.add(i, d)
        let old = c2(cnt[i])
        cnt[i] += d
        b2.add(i, c2(cnt[i]) - old)
    var dels: List<(Int, X)> = []
    for v in vert:
        if v.y1 <= y1 and y1 < v.y2:
            upd(cx(v.x), 1)
            dels.push((v.y2, cx(v.x)))
    dels.sort()
    var j = 0
    for l2 in Y:
        let y2 = ys[l2.ord]
        if y2 <= y1: continue
        while j < dels.len and dels[j].0 < y2:
            upd(dels[j].1, -1)
            j += 1
        for (a1, r1) in by_y[l1]:
            for (a2, r2) in by_y[l2]:
                let l = max(a1, a2)
                let r = min(r1, r2)
                if l <= r: ans += c2(b1.sum(l..=r)) - b2.sum(l..=r)
print(ans)
