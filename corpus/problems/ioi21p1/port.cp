// Source: IOI 2021 Day 1 Distributing Candies, https://dmoj.ca/problem/ioi21p1
// limits: 2000 ms, 512 MB
// Problem: n <= 2e5 boxes with capacities c <= 1e9, all empty, and q <= 2e5 days l r v
//   (|v| <= 1e9, v != 0): every box in l..=r gets p -> min(c, p + v) or max(0, p + v). Print the
//   final counts.
// Solution: sweep the boxes, keeping a segment tree over days of each day's change to the current
//   box, with each node's end, highest and lowest prefix sums. For box capacity c, walk down for
//   the latest day p from which the walk spans at least c: from then on the walls bind once, so
//   the answer is c - (top - end) or end - low of the days after p by the sign of v[p], and
//   end - low of all days if no such p. O((n + q) log q).
// original: corpus/submissions/4698803.cpp (grader function distribute_candies)
// stdin: n, the n capacities, q, then q lines l r v. stdout: the final counts.
// Bounds: n, q: 1..=200_000; cap: 1..=1_000_000_000; days.l, days.r: 0..=n - 1;
//   days.v: -1_000_000_000..=1_000_000_000
let n: 1..=200_000 = read()
domain Box = 0..n
let cap: [Box]1..=1_000_000_000 = read()
let q: 1..=200_000 = read()
domain Day = 0..q
let days: [Day](l: Box, r: Box, v: -1_000_000_000..=1_000_000_000) = read()

// A stretch of days as the walk it makes from 0 without walls: where it ends, and the highest
// and lowest points it touches, the start included.
struct Walk(end: Int, top: Int, low: Int)
fn then(a: Walk, b: Walk) -> Walk = Walk(a.end + b.end, max(a.top, a.end + b.top), min(a.low, a.end + b.low))
fn step(v: Int) -> Walk = Walk(v, max(v, 0), min(v, 0))

var opens: [Box]List<Day> = []
var closes: [Box]List<Day> = []
for d in Day:
    opens[days[d].l].push(d)
    closes[days[d].r].push(d)

// sweep the boxes, holding per day the change that day makes to the current box
var walk = SegTree<Day, Walk>(Monoid(Walk(0, 0, 0), then))
var ans: [Box]Int = 0
for b in Box:
    for d in opens[b]: walk.set(d, step(days[d].v))
    // the days after the latest one from which the walk spans the box: from that day on both
    // walls bind, so it fixes the box full or empty and the days after only shift it
    let from = walk.left_while(Day.cuts.last!, w => w.top - w.low < cap[b])
    let tail = walk.fold(from..Day.cuts.last!)
    ans[b] = if let p = Day.at(Int(from) - 1) and days[p].v > 0: cap[b] - (tail.top - tail.end) else: tail.end - tail.low
    for d in closes[b]: walk.set(d, step(0))
print(ans)
