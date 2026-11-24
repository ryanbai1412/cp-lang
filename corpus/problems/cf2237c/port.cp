// Source: Codeforces 2237C (Duck Surplus), https://codeforces.com/contest/2237/problem/C
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 piles of 1..=1e9 ducks. While unsorted, pick adjacent a[i] > a[i + 1] and
//   replace them with (a[i + 1], a[i] + a[i + 1]). Print the least possible largest pile at the
//   end. Multitest, sum n <= 2e5.
// Solution: scan left to right keeping cur, the least final maximum of the prefix. A pile x >= cur
//   stays and becomes the maximum; a pile x < cur must pass the largest pile at some point, making
//   it cur + x, and nothing then exceeds that. So cur becomes x if x >= cur, else cur + x; print
//   cur, O(n).
// Bounds: n: 1..=200_000; a: 1..=10 ** 9

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
var cur = a[0]
for x in a[1..n]: cur = if x >= cur: x else: cur + x
print(cur)
