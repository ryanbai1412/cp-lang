// Source: CCO 2020 P1, A Game with Grundy, https://dmoj.ca/problem/cco20p1
// Problem: N <= 1e5 friends at (x_i, 0) with v_i, h_i <= 100; friend i sees (x, Y) iff
//   |x - x_i| v_i < h_i Y. For each i = 0..N print how many integers x in L..=R
//   (|L|, |R| <= 1e9, Y <= 1e6) are seen by at most i friends.
// Solution: friend i sees the interval x_i +- (h_i Y - 1) / v_i clipped to L..=R. Sort the
//   interval start/end events, sweep to count the positions seen by exactly c friends for
//   every c, then print prefix sums. O(N log N).

use std.prefix.{prefix_fold}

let n: 1..=1e5 = read()
let l: -1e9..=1e9 = read()
let r: l..=1e9 = read()
let y: 1..=1e9 = read()
let friends: [n](x: l..=r, v: 1..=100, h: 1..=100) = read()

var events: List<(pos: Int, delta: Int)> = [(l, 0), (r + 1, 0)]
for (x, v, h) in friends:
    let reach = (h * y - 1) / v  // the largest |x - x_i| still strictly inside the view
    let lo = max(l, x - reach)
    let hi = min(r, x + reach)
    if lo <= hi:
        events.push((lo, 1))
        events.push((hi + 1, -1))
events.sort()

var seen_by: [0..=n]Int = 0  // positions watched by exactly this many friends
var cur = 0
var prev = l
for (pos, delta) in events:
    if pos > prev:
        seen_by[cur] += pos - prev
        prev = pos
    cur += delta
for total in prefix_fold(+, seen_by)[1..]: print(total)
