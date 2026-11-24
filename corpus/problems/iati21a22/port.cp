// Source: IATI 2021 Senior Day 2 Task 2, https://dmoj.ca/problem/iati21a22
// Problem: n <= 2e5 points with |x|, |y| <= 1e9. From (x, y) one may step to a point whose
//   rotated coordinates u = x + y, v = x - y satisfy u' <= u and v' > v. Print the most points
//   on one chain (the statement is not mirrored here; this follows the accepted solution).
// Solution: rotate, sort by u descending with ties v descending (so equal u never chain), and
//   take the longest strictly increasing subsequence of v. O(n log n).
// original: corpus/submissions/5053473.cpp
// Bounds: n: 1..=2e5; pts.x, pts.y: -1e9..=1e9
let n: 1..=2e5 = read()
let pts: [n](x: -1e9..=1e9, y: -1e9..=1e9) = read()
let order = [(u: p.x + p.y, v: p.x - p.y) for p in pts].sorted(by: p => (desc(p.u), desc(p.v)))
print(order.lis(by: p => p.v).len)
