// Source: Codeforces 2211G (Rational Bubble Sort), https://codeforces.com/contest/2211/problem/G
// limits: 2000 ms, 256 MB
// Problem: n <= 1e6 integers 0 <= a_i <= 1e6. An operation replaces a_i and a_(i+1) both by their
//   average. Print whether a can be made non-decreasing. Multitest, sum of n <= 1e6.
// Solution: operations keep the sum. With diff(i) = n * (sum of a[..i]) - tot * i: if some proper
//   prefix averages strictly below the mean (diff(i) < 0), the answer is Yes. Otherwise a sorted
//   result must be constant, which needs the exact-mean cuts (diff(i) = 0) to split a into
//   blocks of length <= 2, each levelled by one operation: every element is next to such a cut.
//   Exact integers on prefix sums, O(n).
// Bounds: n: 1..=1_000_000; a: 0..=1_000_000

@!multitest
let n: 1..=1_000_000 = read()
let a: [0..n]0..=1_000_000 = read()
let tot = a.sum()
var run = 0  // a[..i + 1].sum()
var below = false
var no_gap = true
var zero_before = true  // diff(0) is 0
for i in 0..n:
    run += a[i]
    let d = run * n - tot * (i + 1)  // diff(i + 1): a[..i + 1].sum() * n - tot * (i + 1)
    if i + 1 < n and d < 0: below = true
    if not zero_before and d != 0: no_gap = false
    zero_before = d == 0
print(yes(below or no_gap, answers: ("Yes", "No")))
