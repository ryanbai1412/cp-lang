// Source: CMT 2021 (Peter) P2, https://dmoj.ca/problem/cmt21peter2
// Problem: for each of n points, print the sum of its distances to all the others; an
//   answer within a few percent is accepted.
// Solution: a vector's length is pi/2 times the average length of its shadow over all
//   directions. Take K directions evenly spread over a half turn; along one direction
//   the sum of shadow distances from each point to all others is a sort and a running
//   sum, and pi/2 times the average over the K directions is off by about 1/K^2.
// original: corpus/submissions/4041223.cpp
// checker: corpus/problems/cmt21peter2/checker.py
use real.float.{PI, sin_cos}

const K = 30
let n: Int = read()
domain P = 0..n
let at: [P](x: F64, y: F64) = read()

var total: [P]F64 = 0.0
for k in 0..K:
    let dir = sin_cos(PI * k / K)
    let shadow = [P] p => at[p].x * dir.cos + at[p].y * dir.sin
    let all = sum(shadow[p] for p in P)
    var behind = 0.0  // the shadows already passed
    for (i, p) in sorted(P, by: p => shadow[p]).items:
        let s = shadow[p]
        total[p] += (i * s - behind) + (all - behind - (n - i) * s)
        behind += s
for p in P: print(total[p] / K * PI / 2, precision: 2)
