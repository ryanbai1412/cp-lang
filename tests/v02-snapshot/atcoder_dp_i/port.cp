// Source: AtCoder Educational DP Contest I (Coins), https://atcoder.jp/contests/dp/tasks/dp_i
// Problem: N coins, N odd and <= 2999; coin i shows heads with probability p_i. Print the
//   probability of more heads than tails, to absolute error 1e-9.
// Solution: dp[h] = probability of h heads among the coins so far; each coin updates it in
//   place from high h down (dp[h + 1] += dp[h] p, dp[h] *= 1 - p). Sum dp[h] over h > N / 2.
//   O(N^2).
// Runnable on proto/cp.py lib-run today.
let n: Int = read()
var p: List<F64> = []
for _ in 0..n:
    let x: F64 = read()
    p.push(x)

var dp = List.filled(n + 1, 0.0)
dp[0] = 1.0
for i in 0..n:
    let pi = p[i]
    let qi = 1.0 - pi
    var h = i
    while h >= 0:
        dp[h + 1] += dp[h] * pi
        dp[h] *= qi
        h -= 1

var ans = 0.0
for h in (n / 2 + 1)..=n:
    ans += dp[h]

print(ans)
