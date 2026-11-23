// Source: Codeforces 2178C (First or Second), https://codeforces.com/contest/2178/problem/C
// Problem: n <= 2e5 children with niceness |a_i| <= 1e9 in a line. n - 1 times remove the
//   first child (X += a) or the second (X -= a); one child stays. Print the max X. Multitest.
// Solution: fix the child k that stays: a[0] is added (if k > 0), each of a[1..k] can be
//   added or subtracted freely, and every later child is subtracted. Relative to all -1:
//   `cur` is 2 a[0] + sum of max(0, 2x) over a[1..k], leaving a[k] adds a[k], and k = 0
//   gives a[0]; answer = best - a.sum(). One scan, O(n).

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()

var best = a[0]
var cur = 2 * a[0]
for x in a[1..n]:
    best max= cur + x
    cur += max(0, 2 * x)
print(best - a.sum())
