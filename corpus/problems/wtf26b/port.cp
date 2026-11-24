// Source: AtCoder World Tour Finals 2026 B (Window Records),
//   https://atcoder.jp/contests/awtf2026algo/tasks/awtf2026algo_b
// limits: 2000 ms, 2048 MB
// Problem: given A[1..N] (2 <= N <= 50, 1 <= A_i <= N), count sequences x with 1 <= x_i <= A_i
//   for which some permutation p of 1..2N-1 has exactly x_i prefix maxima in each window
//   p[i..i+N-1], mod 998244353.
// Solution: store x_i - 1, the records of window i after its first index. A fixed x is tested
//   by one right-to-left scan (editorial): x_i <= x_{i+1} + 1 is needed, and for the last window
//   an index N + j can be a record only before the reach c of the first window's maximum, or
//   when x_j <= x_{j+1}; the best maximum is the one with the largest c. This necessary
//   condition is also sufficient, as p can then be built window by window from the right, so
//   counting x means running that scan over all x at once. DP right to left: dp[x, k, cut]
//   counts valid suffixes starting at value x with an open decreasing run of length k (0 once
//   resolved) and largest cut threshold cut. The next value either resolves, extends the run
//   (x = y + 1) or shrinks it (x <= y), raising cut to k - x - 1 and resolving once
//   cut >= k - 1. Each step sums over the next value y >= x - 1 for x < a[i] only, so the cost
//   follows a: O(N^5) at worst, far less on small a.

const MOD = 998_244_353
type Z = Mod<MOD>

let n: 2..=50 = read()
let a: [0..n]1..=n = read()

var dp: [0..n, 0..n, 0..n]Z = 0
for x in 0..a[n - 1]: dp[x, x, 0] = 1

for i in (0..n - 1).rev:
    var ndp: [0..n, 0..n, 0..n]Z = 0
    for x in 0..a[i]:
        for y in max(0, x - 1)..n:
            ndp[x, 0, 0] += dp[y, 0, 0]
            for k in 1..=i + 1:
                for cut in 0..k:
                    let v = dp[y, k, cut]
                    if x == y + 1:
                        if k <= i: ndp[x, k, cut] += v
                    else:
                        let ncut = max(cut, k - x - 1)
                        if ncut >= k - 1: ndp[x, 0, 0] += v
                        else: ndp[x, k - 1, ncut] += v
    dp = ndp

print(sum(dp[x, 0, 0] for x in 0..n))
