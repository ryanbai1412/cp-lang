type Z = Mod<1_000_000_007>
let (n, target): (Int, Int) = read()
let c: [0..n]Int = read()
var dp: [0..target + 1]Z = 0
dp[0] = 1
for i in 0..n:
    for s in c[i]..=target:
        dp[s] += dp[s - c[i]]
print(dp[target])
