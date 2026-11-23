let n: 1..=200000 = read()
let a: [0..n]Int = read()
var dp: [0..n]Int = 0
for i in 0..n:
    dp[i] = a[i]
    if i > 0:
        dp[i] += dp[i - 1]
print(dp[n - 1])
var lv: [0..3, 0..3]Int = 1
for s in 0..3:
    for k in 0..3:
        lv[s, k] = if k > 0: lv[s, k - 1] elif s > 0: lv[s - 1, 2] else: 1
var cnt: [0..4, 0..4]Int = 0
cnt[0, 1] = 1
for i in 0..3:
    for j in 0..4:
        if j > 0:
            cnt[i + 1, j - 1] += cnt[i, j] * j
print(lv[2, 2], cnt[3, 0])
