let n: Int = read()
let a: [0..n]Int = read()
var dp: [0..n]Int = 1
for i in 0..n:
    for j in 0..i:
        if a[j] <= a[i]:
            dp[i] = max(dp[i], dp[j] + 1)
print(max(dp) ?? 0)
