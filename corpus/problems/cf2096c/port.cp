// Source: Codeforces 2096C (Wonderful City), https://codeforces.com/contest/2096/problem/C
// limits: 2000 ms, 250 MB
// Problem: an n x n grid of heights; raising all of row i by 1 costs a[i], all of column j costs
//   b[j], each at most once. Print the least cost that leaves no two side-adjacent cells equal,
//   or -1. n <= 1000, sum of n <= 1000; multitest.
// Solution: a row raise keeps every horizontal difference and a column raise every vertical
//   one, so rows and columns are independent. Rows i - 1, i raised by p, q in {0, 1} clash when
//   some column has h[i, j] - h[i - 1, j] = p - q; mark those pairs, then a 2-state chain DP
//   gives the cheapest rows (and, the same way, columns). Answer: the sum, O(n^2).
// Bounds: n: 1..=1000; a, b: 0..=10 ** 9

@!multitest
let n: 1..=1000 = read()
let h: [0..n, 0..n]Int = read()
let a: [0..n]0..=10 ** 9 = read()
let b: [0..n]0..=10 ** 9 = read()

var row_bad: [0..n - 1, 0..3]Bool = false
var col_bad: [0..n - 1, 0..3]Bool = false
for i in 1..n:
    for j in 0..n:
        let d = h[i, j] - h[i - 1, j]
        if -1 <= d <= 1: row_bad[i - 1, d + 1] = true
for i in 0..n:
    for j in 1..n:
        let d = h[i, j] - h[i, j - 1]
        if -1 <= d <= 1: col_bad[j - 1, d + 1] = true

fn min_cost(c: [0..n]Int, bad: [0..n - 1, 0..3]Bool) -> ?Int:
    var dp: [0..2]?Int = [0, c[0]]
    for i in 1..n:
        var nxt: [0..2]?Int = None
        for p in 0..2:
            if let cur = dp[p]:
                for q in 0..2 if not bad[i - 1, 1 + p - q]:
                    nxt[q] min= cur + q * c[i]
        dp = nxt
    min(dp[0], dp[1])

if let ra = min_cost(a, row_bad) and let rb = min_cost(b, col_bad): print(ra + rb)
else: print(-1)
