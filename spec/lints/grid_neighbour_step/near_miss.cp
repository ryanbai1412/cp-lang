let n: 1..=200000 = read()
let a: [0..n]Int = read()
var dp: [0..n]Int = 0
for i in 0..n:
    dp[i] = a[i]
    if i > 0:
        dp[i] += dp[i - 1]
print(dp[n - 1])
