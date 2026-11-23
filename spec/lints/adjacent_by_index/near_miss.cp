let n: 2..=100 = read()
let a: [0..n]Int = read()
var cnt = 0
for i in 0..n:
    if i + 1 < n:
        cnt += 1
    if a[i] > 0:
        cnt += 1
print(cnt)
// A loop that writes the array it reads at `i + 1` is a DP over neighbours.
var dp: [0..n]Int = read()
for i in 0..n:
    if i + 1 < n:
        dp[i] min= dp[i + 1] + a[i]
print(dp)
