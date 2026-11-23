// AtCoder WTF 2026 B, tuned: port.cp's DP over three tables allocated once
// and swapped, instead of a copy and a fresh n^3 table per step. A step only
// reads cells with k <= i + 1, so the suffix sums and the clear of the next
// table stay inside that region.
const MOD = 998_244_353
type Z = Mod<MOD>

let n: 1..=200 = read()
let a: [0..n]1..=n = read()

var dp: [0..n, 0..n, 0..n]Z = 0
var suf: [0..n, 0..n, 0..n]Z = 0
var ndp: [0..n, 0..n, 0..n]Z = 0
for x in 0..a[n - 1]: dp[x, x, 0] = 1

for i in (0..n - 1).rev:
    unchecked:
        suf[n - 1, 0, 0] = dp[n - 1, 0, 0]
        for k in 1..=i + 1:
            for cut in 0..k: suf[n - 1, k, cut] = dp[n - 1, k, cut]
        for y in (0..n - 1).rev:
            suf[y, 0, 0] = dp[y, 0, 0] + suf[y + 1, 0, 0]
            for k in 1..=i + 1:
                for cut in 0..k: suf[y, k, cut] = dp[y, k, cut] + suf[y + 1, k, cut]
        for x in 0..n:
            ndp[x, 0, 0] = 0
            for k in 1..=i + 1:
                for cut in 0..k: ndp[x, k, cut] = 0
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
    swap(mut dp, mut ndp)

print(sum(dp[x, 0, 0] for x in 0..n))
