// Repro: stdlib/subset.md documents `{}` as the empty subset, but `{}` has type `()`
domain P = 0..3
var dp: [Subset<P>]Int = 0
print(dp[{}])
