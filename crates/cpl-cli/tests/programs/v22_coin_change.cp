// The README's v2.2 example: `none` is the unreachable state, `+` passes
// it on and `min=` skips it.
let (n, x): (1..=100, 1..=1e6) = read()
let coins: [0..n]Int = read()

var dp: [0..=x]?Int = None
dp[0] = 0
for s in 1..=x:
    for c in coins:
        if c <= s: dp[s] min= dp[s - c] + 1
print(dp[x] ?? -1)
