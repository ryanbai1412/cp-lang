// Source: Codeforces 2124B (Minimise Sum), https://codeforces.com/contest/2124/problem/B
// limits: 1500 ms, 250 MB
// Problem: n <= 2e5 values in 0..=n. At most once, pick i < j, add a[j] to a[i] and set a[j] = 0.
//   Print the least possible sum of the prefix minima. Multitest, sum n <= 2e5.
// Solution: the first minimum a[0] can only grow and the second stays min(a[0], a[1]) unless a[1]
//   goes into a[0] (then a[0] + a[1] is no smaller), so the sum is >= a[0] + min(a[0], a[1]).
//   Moving a[1] into a[0] when a[1] <= a[0], else a[2] into a[1], zeroes every later minimum and
//   meets it. O(1).

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]0..=2 * n = read()
print(a[0] + min(a[0], a[1]))
