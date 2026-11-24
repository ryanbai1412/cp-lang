// Source: Codeforces 2096B (Wonderful Gloves), https://codeforces.com/contest/2096/problem/B
// limits: 1000 ms, 250 MB
// Problem: colour i has l_i left and r_i right gloves (n <= 2e5, counts <= 1e9, multitest).
//   Print the fewest gloves drawn blind that guarantee matching pairs in at least k <= n colours.
// Solution: the worst draw takes max(l_i, r_i) of every colour (no pair) and min(l_i, r_i) of
//   the k - 1 colours with the largest min (k - 1 pairs); one more glove makes a k-th. Select the
//   top k - 1 mins in place: sum of maxes + top k - 1 mins + 1, O(n).
// Bounds: n, k: 1..=200_000; l, r: 0..=10 ** 9

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let l: [0..n]0..=10 ** 9 = read()
let r: [0..n]0..=10 ** 9 = read()
var mins = [min(l[i], r[i]) for i in 0..n]
if k > 1: mins.select(n - k + 1)
print(sum(max(l[i], r[i]) for i in 0..n) + mins[n - k + 1..n].sum() + 1)
