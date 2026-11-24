// Source: CCO 2020 P1, A Game with Grundy, https://dmoj.ca/problem/cco20p1
// Problem: N <= 1e5 friends at (x_i, 0) with v_i, h_i <= 100; friend i sees (x, Y) iff
//   |x - x_i| v_i < h_i Y. For each i = 0..N print how many integers x in L..=R
//   (|L|, |R| <= 1e9, Y <= 1e6) are seen by at most i friends.
// Solution: friend i sees the interval x_i +- (h_i Y - 1) / v_i clipped to L..=R. Sort the
//   interval start/end events, sweep L..=R to count the positions seen by exactly c friends for
//   every c, then print prefix sums. O(N log N).
use std.prefix.{prefix_fold}

let n: 1..=1e5 = read()
let l: -1e9..=1e9 = read()
let r: l..=1e9 = read()
let y: 1..=1e6 = read()
let friends: [n](x: l..=r, v: 1..=100, h: 1..=100) = read()

var events: List<(pos: Int, delta: -1..=1)> = []
for f in friends:
    let reach = (f.h * y - 1) / f.v  // the largest |x - x_i| still strictly inside the view
    let (lo, hi) = (max(l, f.x - reach), min(r, f.x + reach))
    if lo <= hi:
        events.push((lo, 1))
        events.push((hi + 1, -1))
events.sort()

var seen_by: [0..=n]Int = 0  // positions watched by exactly this many friends
var (watching, from) = (0, l)
for e in events:
    seen_by[watching] += e.pos - from
    (watching, from) = (watching + e.delta, e.pos)
seen_by[0] += r + 1 - from
for total in prefix_fold(+, seen_by)[1..]: print(total)
