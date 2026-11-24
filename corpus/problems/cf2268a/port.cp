// Source: Codeforces 2268A (K Is Important), https://codeforces.com/contest/2268/problem/A
// limits: 2000 ms, 256 MB
// Problem: n <= 1e5 values in 1..=1e9 and 1 <= k <= n. While at least k elements remain, remove
//   the k-th element or the k-th from the end, adding it to the score. Print the largest score.
//   Multitest, sum n <= 1e5.
// Solution: an element i < k - 1 places from the front can only go as the k-th from the end, when
//   the length is k + i, and so can its mirror a[n - 1 - i]: at most one of each such pair goes.
//   Counting the n - k + 1 removals, the best score is the middle block a[k - 1..n - k + 1] (empty
//   when 2 k - 2 > n) plus max(a[i], a[n - 1 - i]) for i < min(k - 1, n - k + 1). O(n).

@!multitest
let (n, k): (1..=200_000, 1..=n) = read()
let a: [0..n]1..=10 ** 9 = read()
let mid = if 2 * k - 2 <= n: a[k - 1..n - k + 1].sum() else: 0
let edge = min(k - 1, n - k + 1, n / 2)
print(mid + sum(max(a[i], a[n - 1 - i]) for i in 0..edge))
