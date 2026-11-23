// cf1930d2 Sum over all Substrings (hard): f(t) is greedy from the left, so a '1'
// at i places one 1 at i + 1 and covers i..i+2. Walking i from the right,
// dp[i] = (n - i) + dp[i + 3] for a '1', else dp[i + 1]; the answer sums dp.
@!multitest
let n: 1..=1e6 = read()
let s = read<Str>()
var dp: [0..n + 3]Int = 0
for i in (0..n).rev:
    dp[i] = if s[i] == '1': n - i + dp[i + 3] else: dp[i + 1]
print(dp.sum())
