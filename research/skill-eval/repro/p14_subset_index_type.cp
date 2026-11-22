// Repro: indexing `[Subset<P>]T` with `Subset<P>.empty` or `Subset<P>.full`
// fails with `E0302: expected Subset<P>, found Subset<P>`
domain P = 0..3
var dp: [Subset<P>]Int = 0
dp[Subset<P>.empty] = 1
print(dp[Subset<P>.full])
