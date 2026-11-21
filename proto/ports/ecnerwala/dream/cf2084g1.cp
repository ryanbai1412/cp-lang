// cf2084g1 as it should read: layered DP over values in increasing order.
@!multitest
let n: 1..=5000 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let half = n / 2
let even: [1..=n]?Bool = a.invert().map(i => i.label.even)

var dp: [0..=n, 0..=half]?Int = none @rolling(0)
dp[0, 0] = 0
for k in 0..n, c in 0..=min(k, half), z in [false, true]:
    if c + Int(z) <= half and even[k + 1].allows(z):
        let gain = (2 * c - k) * (2 * (half - c) - (n - k))
        dp[k + 1, c + Int(z)] max= dp[k, c] + gain
print((dp[n, half]! + n * (n + 1) * (n + 1) / 2) / 2)
