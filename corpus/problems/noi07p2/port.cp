// Source: NOI 2007 P2 (Cash Exchange), https://dmoj.ca/problem/noi07p2
// limits: 600 ms, 256 MB
// Problem: over n <= 1e5 days vouchers A and B are worth a_k and b_k (in (0, 10]); cash spent
//   on day k buys A and B in the amount ratio rate_k. Any day you may sell the same share of both
//   and buy with any cash. Starting with s dollars, print the most cash after day n (3 decimals).
// Solution: all-in trades suffice (the statement's hint). Cash f on day j buys
//   B = f / (a_j rate_j + b_j) and A = rate_j B, worth b_i (A x + B) on day i at x = a_i / b_i:
//   a line per day. best_i = max(best_(i-1), b_i * highest line at x_i), then add day i's line.
//   The ratios are known up front, so a Li Chao tree over the days sorted by ratio holds the
//   lines and no two reals are compared for equality: a std `LiChao` over the ratio ranks,
//   O(n log n). (The original runs the days backwards with a dynamic hull.)
// original: corpus/submissions/3767248.cpp
// Bounds: n: 1..=1e5
let n: 1..=1e5 = read()
let money: F64 = read()
domain Day = 0..n
let days: [Day](a: F64, b: F64, rate: F64) = read()

let by_ratio = sorted(Day, by: d => days[d].a / days[d].b)
let ratio = [days[d].a / days[d].b for d in by_ratio]
var rank: [Day]Int = 0  // a day's place among the ratios
for (i, d) in by_ratio.items: rank[d] = i

// a day's line over the ratio ranks: worth k * ratio + m
struct Worth(k: F64, m: F64):
    fn at(self, x: Int) -> F64 = k * ratio[x] + m

var lines = LiChao<Worth, F64>(max, over: 0..n)
var best = money  // the most money held so far
for d in Day:
    let t = days[d]
    best max= t.b * (lines.at(rank[d]) ?? 0.0)
    let coupons_b = best / (t.a * t.rate + t.b)
    lines.add(Worth(coupons_b * t.rate, coupons_b))
print(best, precision: 3)
