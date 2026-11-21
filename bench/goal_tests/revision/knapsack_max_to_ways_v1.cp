let (n, cap): (Int, Int) = read()
var dp: [0..cap + 1]Int = 0
for _ in 0..n:
    let (w, v): (Int, Int) = read()
    for s in (w..=cap).rev:
        dp[s] = max(dp[s], dp[s - w] + v)
print(dp[cap])
