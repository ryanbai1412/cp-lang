// Source: AtCoder World Tour Finals 2026 B (Window Records),
//   https://atcoder.jp/contests/awtf2026algo/tasks/awtf2026algo_b
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
//   cut >= k - 1. Suffix sums over the first value make each transition O(1): O(N^4) total.

const MOD = 998_244_353
type Z = Mod<MOD>

let n: 2..=50 = read()
let a: [0..n]1..=n = read()

var dp: [0..n, 0..n, 0..n]Z = 0
for x in 0..a[n - 1]: dp[x, x, 0] = 1

for i in (0..n - 1).rev:
    var suf = dp.copy()
    for y in (0..n - 1).rev:
        suf[y, 0, 0] += suf[y + 1, 0, 0]
        for k in 1..=i + 1:
            for cut in 0..k:
                suf[y, k, cut] += suf[y + 1, k, cut]

    var ndp: [0..n, 0..n, 0..n]Z = 0
    for x in 0..a[i]:
        ndp[x, 0, 0] += suf[max(0, x - 1), 0, 0]
        for k in 1..=i + 1:
            let ncut_base = k - x - 1
            for cut in 0..k:
                if x > 0 and k <= i:
                    ndp[x, k, cut] += dp[x - 1, k, cut]
                let v = suf[x, k, cut]
                if v != 0:
                    let ncut = max(cut, ncut_base)
                    if ncut >= k - 1: ndp[x, 0, 0] += v
                    else: ndp[x, k - 1, ncut] += v
    dp = ndp

print(sum(dp[x, 0, 0] for x in 0..n))
