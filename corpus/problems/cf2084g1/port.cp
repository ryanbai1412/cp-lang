// Codeforces 2084 G1: only the parity of each value's position matters.
// Put the values in increasing order; when the k smallest are placed and c of
// them sit on the n / 2 even positions, the beauty gains
// (2c - k) * (2(n / 2 - c) - (n - k)), the product of the even-minus-odd
// imbalances of the small and the large values. So it is a DP over k and c,
// with a known value forced to its own parity; the beauty is
// (best + n(n + 1)^2 / 2) / 2. O(n^2).
@!multitest
let n: 1..=5000 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let half = n / 2
let at = a.inverse()

var dp: [0..=half]?Int = none
dp[0] = 0
for k in 0..n:
    let forced = at[k + 1].map(p => p.label % 2 == 0)
    var next: [0..=half]?Int = none
    for c in 0..=min(k, half):
        if let x = dp[c]:
            let gain = (2 * c - k) * (2 * (half - c) - (n - k))
            for z in [false, true] if c + Int(z) <= half and (forced == none or forced == some(z)):
                next[c + Int(z)] max= x + gain
    dp = next
print((dp[half]! + n * (n + 1) * (n + 1) / 2) / 2)
