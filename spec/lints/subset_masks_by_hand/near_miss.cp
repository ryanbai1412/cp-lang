let n: 1..=20 = read()
let w: [0..(1 << n)]Int = read()
var dp = [0..(1 << n)] _ => 0
for mask in 0..(1 << n):
    dp[mask] = w[mask] + (if mask > 0: dp[mask & (mask - 1)] else: 0)
print(dp)
// A bit that chooses between two values, or feeds arithmetic, is not picking a subset.
let a: [0..n]Int = read()
let b: [0..n]Int = read()
var best = 0
for mask in 0..(1 << n):
    best max= sum(if ((mask >> i) & 1) == 0: a[i] else: b[i] for i in 0..n)
    let bit = (mask >> 0) & 1
    best max= bit * a[0]
print(best)
