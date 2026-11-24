// Source: CCC 2019 S5 (Triangle: The Data Structure), https://dmoj.ca/problem/ccc19s5
// limits: 1000 ms, 512 MB
// Problem: a triangle of n <= 3000 rows, row i holding i numbers in 0..=1e9, and 1 <= k <= n.
//   Print the sum over every sub-triangle of size k of its largest element.
// Solution: with m[i][j] the max of the size-s sub-triangle topped at (i, j), size s + d for d <=
//   (s + 1) / 2 is the max of the three size-s ones at (i, j), (i + d, j), (i + d, j + d), which
//   cover it. Grow s by about 1.5x up to k, updating one table in place (the
//   apex rows read only rows below them): O(n^2 log k).
// original: corpus/submissions/5119931.cpp
let n: 1..=3000 = read()
let k: 1..=n = read()
// best[i, j] (j <= i): the largest entry of the current-side triangle with apex (i, j)
var best: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in 0..=i: best[i, j] = read()

var side = 1
while side < k:
    let grown = min(k, (side * 3 + 1) / 2)
    let d = grown - side
    for i in 0..=n - grown:
        for j in 0..=i: best[i, j] max= max(best[i + d, j], best[i + d, j + d])
    side = grown
print(sum(best[i, j] for i in 0..=n - k for j in 0..=i))
