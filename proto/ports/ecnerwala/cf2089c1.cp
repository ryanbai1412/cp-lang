const MOD = 1_000_000_007

@!multitest
let n: Int = read()
let l: Int = read()
let _k: Int = read()
var dp: [0..n]Mod<MOD> = 0
for i in 1..=l:
    let last_val = dp[n - 1]
    var j = n - 1
    while j > 0:
        dp[j] = dp[j - 1]
        j -= 1
    dp[0] = last_val + 1
    var pref: [0..=n]Mod<MOD> = 0
    for k in 0..n:
        pref[k + 1] = pref[k] + dp[k]
    let inv_i = (Mod<MOD>(i)).inv()
    for k in 0..n:
        let a = k + 1 - i
        let cnt = (n - a) / n
        let p = pref[a + n * cnt] - pref[n] * cnt
        dp[k] = (pref[k + 1] - p) * inv_i
print(dp)
