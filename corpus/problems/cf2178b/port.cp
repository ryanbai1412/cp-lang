// Source: Codeforces 2178B (Impost or Sus), https://codeforces.com/contest/2178/problem/B
// Problem: a string r of 's' and 'u' (3 <= |r| <= 2e5; multitest, sum <= 2e5). It is suspicious
//   when 's' appears at least twice and every 'u' has its two nearest 's' equally far away.
//   Print the fewest letters to change to 's' to make it suspicious.
// Solution: equal distances force every 'u' to have an 's' on both sides, so the first and last
//   letters are 's' and no two 'u' touch. DP over the last letter kept: dp[S] and dp[U] are the
//   fewest changes for the prefix; a 'u' stays only after an 's' and otherwise costs 1, starting
//   from dp[U] = 0 so the first letter cannot stay 'u'; the answer is dp[S] at the end. O(|r|).

enum Last:
    U
    S

@!multitest
let s: Str = read()
var dp: [Last]?Int = None
dp[Last.U] = 0
for c in s:
    let m = min(dp[Last.U], dp[Last.S])!
    if c == 's':
        dp[Last.U] = None
        dp[Last.S] = m
    else:
        dp[Last.U] = dp[Last.S]
        dp[Last.S] = 1 + m
print(dp[Last.S]!)
