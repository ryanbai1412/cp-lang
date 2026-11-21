let (n, cap): (Int, Int) = read()
var dp: [0..cap + 1]Int = 0
dp[0] = 1
for _ in 0..n:
    let (w, _): (Int, Int) = read()
    for s in (w..=cap).rev:
        dp[s] += dp[s - w]
print(dp[cap])
