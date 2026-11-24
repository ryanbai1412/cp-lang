// Source: CEOI '19 Practice P1 (Count Squares), https://dmoj.ca/problem/ceoi19pp1
// limits: 1800 ms, 512 MB
// Problem: h, v <= 1500 horizontal and vertical lines at strictly increasing coordinates in
//   0..=2^30: count the squares whose whole boundary is drawn.
// Solution: a square is a pair of horizontal lines and a pair of vertical lines with equal gaps.
//   Tally every horizontal gap over the pairs of lines, then sum the tallies of every vertical
//   gap: O(h^2 + v^2) map operations.
// original: corpus/submissions/5343174.cpp
let n: 1..=1500 = read()
let m: 1..=1500 = read()
let a: [n]Int = read()
let b: [m]Int = read()
let gaps = a.pairs().map(p => p.1 - p.0).tally()
print(b.pairs().map(p => gaps[p.1 - p.0]).sum())
