// AtCoder WTF 2026 B — Suffix Value/Length/Cut DP
//
// Process positions from `n - 1` down to `0`, tracking `dp[x, k, cut]`: the
// number of valid suffixes starting with value `x in 0..a[i]`, active
// decreasing-run length `k` (0 once resolved), and maximum cut threshold `cut`.
//
// Base case at `n - 1`: starting at `x` needs `x` smaller values (`0..x`) to
// resolve, so `dp[x, x, 0] = 1` (`x = 0` is already resolved at `k = 0`).
//
// At each step `i`, precompute suffix sums `suf[y, k, cut] = sum_{y' >= y} dp[y', k, cut]`
// so transitioning to `x in 0..a[i]` takes O(1) per `(x, k, cut)`:
// - Resolved (`k == 0`): `ndp[x, 0, 0] += suf[max(0, x - 1), 0, 0]`.
// - Extend (`y = x - 1`, when `x >= 1` and `k <= i`): `ndp[x, k, cut] += dp[x - 1, k, cut]`.
// - Shrink (`y >= x`): `v = suf[x, k, cut]` with `ncut = max(cut, k - x - 1)`;
//   resolves to `(x, 0, 0)` if `ncut >= k - 1`, else `(x, k - 1, ncut)`.
// Complexity: O(n^4) (`~n^4 / 12`, ~1.3e8 ops at `n = 200`).

const MOD = 998_244_353
type Z = Mod<MOD>

let n: 1..=200 = read()
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
