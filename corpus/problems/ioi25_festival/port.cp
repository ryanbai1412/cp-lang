// Source: IOI 2025 Day 2 (Festival), https://github.com/ioi/task-archive/tree/master/2025/day2/festival
// limits: 1000 ms, 2048 MB
// Problem: n <= 2e5 coupons; coupon i costs p[i] tokens and turns the tokens left into
//   (A - p[i]) * t[i], t[i] in 1..=4. Starting from A tokens, buy as many distinct coupons as
//   possible, never going negative. Input and output as the IOI sample grader gives them: n A,
//   then the coupons; print the count, then the coupons in the order bought.
// Solution: exchange argument: multipliers (t > 1) go in increasing p * t / (t - 1), plain
//   coupons last, cheapest first. Buy multipliers while one doesn't shrink the tokens; past
//   that the gap between the next key and the tokens at least doubles per buy, so fewer than
//   33 more fit and only the cheapest 32 left of each type matter. A DP over (prefix, count)
//   of those keeps the most tokens, and each count adds the plain coupons its tokens cover.
//   O(n log n).
// original: corpus/problems/ioi25_festival/orig.cpp (the official model solution plus the sample grader)
// The judge form is dream.cp's `@grader fn max_coupons`, linked with dream.tests/grader.cpp;
// the port reads the sample grader's stdin so `cpl stress` and the --max gate run it alone.
// Bounds: n: 1..=2e5; a0: 1..=1e9; coupons.p: 1..=1e9; coupons.t: 1..=4
let (n, a0) = read<(1..=2e5, 1..=1e9)>()
domain C = 0..n
let coupons: [C](p: 1..=1e9, t: 1..=4) = read()
let cap = 2e14  // every price combined: n · max p = 2e5 · 1e9
fn buy(a: Int, c: C) -> Int = min((a - coupons[c].p) * coupons[c].t, cap)

let mult = C.filter(c => coupons[c].t > 1).sorted(
    by: c => Ratio(coupons[c].p * coupons[c].t, coupons[c].t - 1)
)
let plain = C.filter(c => coupons[c].t == 1).sorted(by: c => coupons[c].p)

var a: Int = a0
var order: List<C> = []  // the free run: buys that don't shrink a
var left = mult[..]
while let c = left.first and buy(a, c) >= a:
    a = buy(a, c)
    order.push(c)
    left = left.drop(1)
var cheapest = Bitset<C>.empty()  // the cheapest 32 left of each type
for t in 2..=4:
    for c in left.filter(c => coupons[c].t == t).take(32): cheapest.add(c)
let rest = List(left.filter(c => cheapest.contains(c)))

domain K = 0..=rest.len
var best: [0..=rest.len][K]?Int = None  // most tokens after k of the first i
best[0][0] = a
for (i, c) in rest.items:
    for k in K:
        best[i + 1][k] max= best[i][k]
        if let j = k.prev and let t = best[i][j] and t >= coupons[c].p: best[i + 1][k] max= buy(
            t,
            c,
        )

let cost = plain.map(c => coupons[c].p).scan(+)
fn covered(k: K) -> Int =  // the plain coupons k's tokens pay for
    (0..plain.len).first_true(i => cost.cut(i + 1) > best[rest.len][k]!) ?? plain.len
let k = K.filter(k => best[rest.len][k] != None).max(by: k => k.ord + covered(k))!

var took: List<C> = []  // walk back: a skip that keeps best was optimal
var j = k
for i in (0..rest.len).rev:
    if best[i + 1][j] != best[i][j]:
        took.push(rest[i])
        j = j.prev!
order += took.reversed() + plain.take(covered(k))
print(order.len)
print(order)
