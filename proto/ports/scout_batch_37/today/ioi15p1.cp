// IOI 2015 Day 1 P1, Boxes with Souvenirs (buildable today-form)
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
