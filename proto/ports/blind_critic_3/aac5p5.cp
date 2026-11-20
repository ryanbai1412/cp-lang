// original: 4240171 (aac5p5)
// Count rectangles formed by two horizontal and two vertical segments. Fix the lower
// horizontal level y1, keep the verticals that cross it in two Fenwick trees over x (count,
// and pairs sharing an x), and sweep the upper level y2 upward, dropping verticals that end.
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
let (X, xs) = compress(all_x)
let (Y, ys) = compress(all_y)
let K = X.len
fn cx(x: Int) -> Int: xs.first_ge(x)!.to_idx
var by_y: [Y]List<(Int, Int)> = []
for h in hori: by_y[ys.first_ge(h.y)!].push((cx(h.x1), cx(h.x2)))
fn c2(x: Int) -> Int: x * (x - 1) / 2

domain B = 0..=K
var cnt: [0..K]Int = 0
var b1: [B]Int = 0                      // Fenwick: verticals per x
var b2: [B]Int = 0                      // Fenwick: pairs of verticals on the same x
fn add(mut b: [B]Int, at: Int, d: Int):
    var i = at + 1
    while i <= K:
        b[B(i)] += d
        i += i & -i
fn upto(b: [B]Int, at: Int) -> Int:     // sum over x < at
    var i = at
    var s = 0
    while i > 0:
        s += b[B(i)]
        i -= i & -i
    s
fn upd(i: Int, d: Int):
    add(mut b1, i, d)
    let old = c2(cnt[i])
    cnt[i] += d
    add(mut b2, i, c2(cnt[i]) - old)

var ans = 0
for l1 in Y:
    let y1 = ys[l1]
    for i in 0..K: cnt[i] = 0
    for i in 0..=K:
        b1[B(i)] = 0
        b2[B(i)] = 0
    var dels: List<(Int, Int)> = []
    for v in vert:
        if v.y1 <= y1 and y1 < v.y2:
            upd(cx(v.x), 1)
            dels.push((v.y2, cx(v.x)))
    dels.sort()
    var j = 0
    for l2 in Y:
        let y2 = ys[l2]
        if y2 <= y1: continue
        while j < dels.len and dels[j].0 < y2:
            upd(dels[j].1, -1)
            j += 1
        for (a1, r1) in by_y[l1]:
            for (a2, r2) in by_y[l2]:
                let l = max(a1, a2)
                let r = min(r1, r2)
                if l <= r: ans += c2(upto(b1, r + 1) - upto(b1, l)) - (upto(b2, r + 1) - upto(b2, l))
print(ans)
