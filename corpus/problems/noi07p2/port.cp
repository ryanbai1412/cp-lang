// NOI 2007 Cash: two kinds of coupon, A and B, have a price each day.
// Any day you may sell all your coupons, and may spend all your money on
// coupons, which come as rate[day] of A for every one of B. Starting with
// `money`, print the most money you can hold after the last day.
//
// It never pays to trade a part. Money f spent on day j buys
// f / (a_j rate_j + b_j) of B and rate_j times that of A, and is worth
// (A's * a_i + B's * b_i) on day i: a line in a_i / b_i, scaled by b_i. So
// each day asks for the highest line at its own price ratio, then adds its
// line. The ratios are known up front, so the lines live in a Li Chao tree
// over the days in order of ratio, and no two reals are ever compared for
// equality. (The original runs the days backwards with a dynamic hull.)
// original: corpus/submissions/3767248.cpp
use real.float.{fixed}

let n: Int = read()
let money: F64 = read()
domain Day = 0..n
var a: [Day]F64 = 0.0
var b: [Day]F64 = 0.0
var rate: [Day]F64 = 0.0
for d in Day:
    let (pa, pb, r): (F64, F64, F64) = read()
    (a[d], b[d], rate[d]) = (pa, pb, r)

let by_ratio = sorted(Day, by: d => a[d] / b[d])
var rank: [Day]Int = 0  // a day's place among the ratios
var ratio: List<F64> = []
for d in by_ratio:
    rank[d] = ratio.len
    ratio.push(a[d] / b[d])

// Li Chao tree for the highest line at ratio[0..n]: node `at` holds the line
// that wins at the middle of its span.
struct Line(k: F64, m: F64):
    fn at(self, x: F64) -> F64 = k * x + m
var held: List<?Line> = List.filled(4 * n, None)

fn add(fresh: Line):
    var (at, lo, hi, cur) = (1, 0, n - 1, fresh)
    while true:
        let mid = (lo + hi) / 2
        let old = held[at] else:
            held[at] = cur
            return
        if cur.at(ratio[mid]) > old.at(ratio[mid]): (held[at], cur) = (cur, old)
        if lo == hi: return
        if cur.at(ratio[lo]) > held[at]!.at(ratio[lo]): (at, hi) = (2 * at, mid)
        elif cur.at(ratio[hi]) > held[at]!.at(ratio[hi]): (at, lo) = (2 * at + 1, mid + 1)
        else: return

fn highest(i: Int) -> F64:
    var (at, lo, hi, best) = (1, 0, n - 1, 0.0)
    while true:
        let line = held[at] else: return best
        best max= line.at(ratio[i])
        if lo == hi: return best
        let mid = (lo + hi) / 2
        if i <= mid: (at, hi) = (2 * at, mid) else: (at, lo) = (2 * at + 1, mid + 1)

var best = money  // the most money held so far
for d in Day:
    best max= b[d] * highest(rank[d])
    let coupons_b = best / (a[d] * rate[d] + b[d])
    add(Line(coupons_b * rate[d], coupons_b))
print(fixed(best, 3))
