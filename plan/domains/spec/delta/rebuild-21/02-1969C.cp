// CF 1969C Minimizing the Sum: dp over (prefix cut, operations used); each final block is
// one original element copied over its block, costing len - 1 operations.
@!multitest
let n = read<1..=3e5>()
let k = read<0..=10>()
let a = read<[n]1..=1e9>()

// dp[i, j]: least sum of the first i elements using at most j operations
var dp: [0..=n, 0..=k]?Int = None
for j in 0..=k: dp[0, j] = 0
for i in 1..=n:
    var lo = a[i - 1]
    for len in 1..=min(i, k + 1):
        lo = min(lo, a[i - len])
        for j in len - 1..=k:
            dp[i, j] min= dp[i - len, j - (len - 1)] + len * lo
print(dp[n, k]!)
