// Source: IOI 2015 Day 1 P1 (Boxes with Souvenirs), https://dmoj.ca/problem/ioi15p1
// Problem: n teams (n <= 1e7) sit at sorted positions on a circle of l sections (l <= 1e9);
//   start at section 0, carry at most k souvenirs per trip, give one to every team and
//   finish back at 0. Print the least total walking distance (sections moved).
// Solution: some optimal plan serves the teams by position: a clockwise prefix and an
//   anticlockwise suffix by round trips, each taking the k farthest teams it has left, and
//   at most one full lap (cost l) for k consecutive teams between them. cw[i] (first i
//   teams clockwise) and ccw[i] (teams i.. anticlockwise) are O(1) recurrences; take the
//   best split i, with or without a lap over the k teams after it. O(n).
// original: corpus/submissions/5723345.cpp (grader function delivery)
// stdin here, as the IOI sample grader: n k l, then the n positions in order.
let n: 1..=10_000_000 = read()
let k: 1..=n = read()
let l: 1..=1_000_000_000 = read()
let pos: [0..n]Int = read()

// cw[i]: the first i teams served by clockwise round trips, k teams a trip;
// ccw[i]: teams i.. served by anticlockwise round trips
var cw: [0..=n]Int = 0
for i in 1..=n: cw[i] = 2 * pos[i - 1] + cw[max(0, i - k)]
var ccw: [0..=n]Int = 0
for i in (0..n).rev: ccw[i] = 2 * (l - pos[i]) + ccw[min(n, i + k)]

// split the teams at i, with at most one full lap taking the k teams after it
var best = cw[n]
for i in 0..=n: best min= cw[i] + min(ccw[i], l + ccw[min(n, i + k)])
print(best)
