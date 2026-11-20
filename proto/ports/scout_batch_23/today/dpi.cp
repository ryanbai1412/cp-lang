// AtCoder EDPC I - Coins (runnable on proto/cp.py lib-run today)
let n: Int = read()
var p: List<Float> = []
for _ in 0..n:
    let x: Float = read()
    p.push(x)

var dp = List.filled(n + 1, 0.0)
dp[0] = 1.0
for i in 0..n:
    let pi = p[i]
    let qi = 1.0 - pi
    var h = i
    while h >= 0:
        dp[h + 1] += dp[h] * pi
        dp[h] = dp[h] * qi
        h -= 1

var ans = 0.0
for h in (n / 2 + 1)..=n:
    ans += dp[h]

print(ans)
