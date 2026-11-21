const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let (n, l, _k): (Int, Int, Int) = read()
var dp: [0..n]Z = 0
for i in 1..=l:
    let pref = prefix_n([0..n] j => if j == 0: dp[n - 1] + 1 else: dp[j - 1])
    let inv_i = comb.inv(i)
    for k in 0..n:
        let a = k + 1 - i
        let cnt = (n - a) / n
        dp[k] = (pref.prefix(k + 1) - (pref.prefix(a + n * cnt) - pref.prefix(n) * cnt)) * inv_i
print(dp)
