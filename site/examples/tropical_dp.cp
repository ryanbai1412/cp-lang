let (n, x): (1..=100, 1..=1000) = read()
let coins: [0..n]Int = read()

var dp: [0..=x]?Int = none
dp[0] = 0
for s in 1..=x:
    for c in coins:
        if c <= s:
            dp[s] min= dp[s - c] + 1
print(dp[x] ?? -1)
