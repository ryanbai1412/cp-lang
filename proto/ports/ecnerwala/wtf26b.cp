// AtCoder WTF 2026 B: 3D suffix DP tracking (value x, suffix length k, cut).
const MOD = 998244353
type Z = Mod<MOD>

let n: Int = read()
let a: [0..n]Int = read()

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
