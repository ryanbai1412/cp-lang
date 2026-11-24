// Source: NOI 2007 P2 (Cash Exchange), https://dmoj.ca/problem/noi07p2
// limits: 600 ms, 256 MB
// Problem: over n <= 1e5 days vouchers A and B are worth a_k and b_k (in (0, 10]); cash spent
//   on day k buys A and B in the amount ratio rate_k. Any day you may sell the same share of both
//   and buy with any cash. Starting with s dollars, print the most cash after day n (3 decimals).
// Solution: all-in trades suffice (the statement's hint). Cash f on day j buys
//   B = f / (a_j rate_j + b_j) and A = rate_j B, worth b_i (A x + B) on day i at x = a_i / b_i:
//   a line per day. best_i = max(best_(i-1), b_i * highest line at x_i), then add day i's line.
//   The ratios are known up front, so a Li Chao tree over the days sorted by ratio holds the
//   lines and no two reals are compared for equality: O(n log n). (The original runs the days
//   backwards with a dynamic hull.)
// original: corpus/submissions/3767248.cpp

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
print(best, precision: 3)
