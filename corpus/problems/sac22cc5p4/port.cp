// Source: SAC '22 Code Challenge 5 P4 (Querying Intervals), https://dmoj.ca/problem/sac22cc5p4
// Problem: N <= 1e5 integer intervals [l_i, r_i] and Q <= 1e5 queries [L, R], endpoints in
//   [-1e9, 1e9]; print Y if every integer of [L, R] lies in some interval, else N.
// Solution: an IntervalSet over the coordinates merges overlapping and touching intervals into
//   maximal runs; a query is Y iff one run covers it. O((N + Q) log N).
// original: corpus/submissions/4575559.cpp
// Bounds: n, q: 1..=1e5; l, r: -1e9..=1e9
use std.interval.{IntervalSet}

let n: 1..=1e5 = read()
let q: 1..=1e5 = read()
domain X = -1e9..=1e9

var covered = IntervalSet<X>()
for _ in 0..n:
    let (l, r): (X, X) = read()
    covered.add(l..=r)
for _ in 0..q:
    let (l, r): (X, X) = read()
    print(if covered.contains_range(l..=r): "Y" else: "N")
