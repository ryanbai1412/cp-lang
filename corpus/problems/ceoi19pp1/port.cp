// Source: CEOI '19 Practice P1 (Count Squares), https://dmoj.ca/problem/ceoi19pp1
// limits: 1800 ms, 512 MB
// Problem: h, v <= 1500 horizontal and vertical lines at strictly increasing coordinates in
//   0..=2^30: count the squares whose whole boundary is drawn.
// Solution: a square is a pair of horizontal lines and a pair of vertical lines with equal gaps.
//   Count every horizontal gap a[j] - a[i] in a Map, then sum the counts of every vertical gap:
//   O(h^2 + v^2) map operations.
// original: corpus/submissions/5343174.cpp
// Runnable on proto/cpc today.
let n: Int = read()
let m: Int = read()
let a: [0..n]Int = read()
let b: [0..m]Int = read()
var gaps = Map<Int, Int>()
for j in 0..n:
    for i in 0..j: gaps[a[j] - a[i]] = (gaps.get(a[j] - a[i]) ?? 0) + 1
var ans = 0
for j in 0..m:
    for i in 0..j: ans += gaps.get(b[j] - b[i]) ?? 0
print(ans)
