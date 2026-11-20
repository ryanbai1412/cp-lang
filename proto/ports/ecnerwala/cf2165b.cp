const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var freq: [0..n]Int = 0
for x in a: freq[x - 1] += 1

var dp: [0..=n]Z = 0
dp[0] = 1
var sz = 0
for f in freq:
    if f > 0:
        for i in (0..=sz).rev:
            dp[i + f] += dp[i] * f
        sz += f

let lo = max(freq)!
var ans: Z = 0
for i in lo..=n: ans += dp[i]
print(ans)
