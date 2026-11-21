let (n, target): (Int, Int) = read()
let c: [0..n]Int = read()
let INF = 1_000_000_000
var dp: [0..target + 1]Int = INF
dp[0] = 0
for i in 0..n:
    for s in c[i]..=target:
        dp[s] = min(dp[s], dp[s - c[i]] + 1)
print(dp[target])
