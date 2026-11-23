let (n, x): (1..=100, 1..=1e5) = read()
let c: [0..n]1..=1e5 = read()
var dp: [0..=x]?Int = None
dp[0] = Some(0)
for s in 1..=x:
    for v in c if v <= s:
        if let d = dp[s - v]: dp[s] min= d + 1
print(dp[x] ?? -1)
