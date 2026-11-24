// Source: SAC '22 Code Challenge 3 P3 (Bob Sort), https://dmoj.ca/problem/sac22cc3p3
// limits: 1000 ms, 256 MB
// Problem: n <= 1e5 distinct 1 <= A_i <= 1e9. Round r (r = 1 .. digits of max A) sorts the
//   array by d_r, its elements' r-th digit from the right (0 when missing), ties in any
//   order; print the array after each round.
// Solution: a stable sort by d_r each round is the statement itself, and it is an LSD radix
//   sort: after round r the array is ordered by the last r digits. O(digits * n log n).
// original: corpus/submissions/4331598.cpp
// Bounds: n: 1..=1e5; row: 1..=1e9

let n: 1..=1e5 = read()
var row: [n]1..=1e9 = read()
let top = row.max()!

var p = 1  // 10^(r - 1): d_r = x / p % 10
while p <= top:
    row.sort(by: x => x / p % 10)
    print(row)
    p *= 10
