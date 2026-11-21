// Codeforces 2237 F: linear DP tracking valid start positions of length-M blocks
// via prefix-best arrays dp1, dp_m, and previous occurrence per shift st = i - (a[i] - 1).
@!multitest
let (n, m): (Int, Int) = read()
let a: [0..n]Int = read()
var dp1: [0..=n]?Int = none
var dp_m: [0..=n]?Int = none
var dp_use: [0..=n]?Int = none
var prv: [0..=n - m]?Int = none

dp1[0] = 0
dp_m[m] = 0
for i in 0..n:
    dp1[i + 1] max= dp1[i]
    dp_m[i + 1] max= dp_m[i]
    let st = i - (a[i] - 1)
    if 0 <= st <= n - m:
        if let v = dp1[st]: dp_use[i] max= v + 1
        if let v = dp_m[i]: dp_use[i] max= v + 1
        if let p = prv[st], let v = dp_use[p]: dp_use[i] max= v + 1
        prv[st] = i
        dp_m[st + m] max= dp_use[i]
        dp1[i + 1] max= dp_use[i]
print(n - max(dp1[n - m] ?? 0, dp_m[n] ?? 0))
