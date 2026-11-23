let n: 1..=3000 = read()
let a: [0..n]Int = read()
var dp: [n, n]Int = 0
for i in 0..n:
    dp[i, i] = a[i]
for len in 2..=n:
    for l in 0..n - len + 1:
        let r = l + len - 1
        dp[l, r] = max(a[l] - dp[l + 1, r], a[r] - dp[l, r - 1])
print(dp[0, n - 1])
