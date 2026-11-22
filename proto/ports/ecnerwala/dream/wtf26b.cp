// AtCoder WTF 2026 B — Suffix Value/Length/Cut DP
//
// Process positions from `n - 1` down to `0`, tracking `dp[x, k, cut]`: the
// number of valid suffixes starting with value `x in 0..a[i]`, active
// decreasing-run length `k` (0 once resolved), and maximum cut threshold `cut`.
//
// From `(y, k, cut)` to `x in 0..a[i]` with `y >= max(0, x - 1)`:
// - If `k == 0`: stay resolved at `(x, 0, 0)`.
// - If `x == y + 1`: extend the run at `(x, k, cut)` while `k <= i`.
// - Otherwise: shrink the run by 1 with `ncut = max(cut, k - x - 1)`; if
//   `ncut >= k - 1`, the run resolves to `(x, 0, 0)`, else `(x, k - 1, ncut)`.

const MOD = 998_244_353
type Z = Mod<MOD>

let n: 1..=200 = read()
let a: [0..n]1..=n = read()

var dp: [0..n, 0..n, 0..n]Z = 0
for x in 0..a[n - 1]: dp[x, x, 0] = 1

for i in (0..n - 1).rev:
    var ndp: [0..n, 0..n, 0..n]Z = 0
    for x in 0..a[i]:
        for y in max(0, x - 1)..n:
            ndp[x, 0, 0] += dp[y, 0, 0]
            for k in 1..=i + 1:
                for cut in 0..k if dp[y, k, cut] != 0:
                    let v = dp[y, k, cut]
                    if x == y + 1:
                        if k <= i: ndp[x, k, cut] += v
                    else:
                        let ncut = max(cut, k - x - 1)
                        if ncut >= k - 1: ndp[x, 0, 0] += v
                        else: ndp[x, k - 1, ncut] += v
    dp = ndp

print(sum(dp[x, 0, 0] for x in 0..n))
