// Codeforces 2237 F: linear DP tracking valid start positions of length-M blocks
// via prefix-best arrays dp_1, dp_M, and previous occurrence per shift st = i - a[i].
@!multitest
let (n, m): (Int, Int) = read()
let a: [0..n]Int = read()
let neg_inf = -(2 * n + 2)
var dp1: [0..=n]Int = neg_inf
var dp_m: [0..=n]Int = neg_inf
var dp_use: [0..=n]Int = neg_inf
var prv: [0..=n - m]?Int = none

dp1[0] = 0
dp_m[m] = 0
for i in 0..n:
    dp1[i + 1] max= dp1[i]
    dp_m[i + 1] max= dp_m[i]
    let st = i - (a[i] - 1)
    if 0 <= st and st <= n - m:
        dp_use[i] max= dp1[st] + 1
        dp_use[i] max= dp_m[i] + 1
        if let p = prv[st]: dp_use[i] max= dp_use[p] + 1
        prv[st] = i
        dp_m[st + m] max= dp_use[i]
        dp1[i + 1] max= dp_use[i]
print(n - max(dp1[n - m], dp_m[n]))
