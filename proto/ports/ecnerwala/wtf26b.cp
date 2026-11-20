// AtCoder WTF 2026 B: 3D suffix DP tracking (value x, suffix length a, cut).
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
                for cut in 0..k:
                    let v = dp[y, k, cut]
                    if v == 0: continue
                    if x == y + 1:
                        if k <= i: ndp[x, k, cut] += v
                    else:
                        let nk = k - 1
                        let ncut = max(cut, k - (x + 1))
                        if ncut >= nk: ndp[x, 0, 0] += v
                        else: ndp[x, nk, ncut] += v
    dp = ndp

var ans: Z = 0
for x in 0..n: ans += dp[x, 0, 0]
print(ans)
