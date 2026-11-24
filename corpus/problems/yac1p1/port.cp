// Source: Yet Another Contest 1 P1 (Mixed Doubles), https://dmoj.ca/problem/yac1p1
// limits: 300 ms, 256 MB
// Problem: n <= 2e5 men with skills a_i and n women with skills b_i (<= 2e9). Pair them up and
//   add 1 to someone's skill k <= 1e9 times; a pair scores a_i * b_j. Print the largest total
//   score mod 1e9+7.
// Solution: pair in sorted order. An increment to one side of a pair gains the other side, and
//   the top pair's larger side is the most anything gains, so every increment goes to the top
//   pair: first level its two sides, then split the rest evenly. Sort: O(n log n).
// original: corpus/submissions/4376122.cpp

type Z = Mod<1_000_000_007>

let n: 1..=2e5 = read()
let k: 0..=1e9 = read()
let a = read<[n]1..=2e9>().sorted()
let b = read<[n]1..=2e9>().sorted()

var x = a.last!
var y = b.last!
let level = min(k, abs(x - y))
if x < y: x += level else: y += level
x += (k - level) / 2
y += (k - level + 1) / 2

print(sum(Z(a[i]) * Z(b[i]) for i in 0..n - 1) + Z(x) * Z(y))
