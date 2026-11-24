// Source: Codeforces 2084G1 (Wish Upon a Satellite, Easy Version),
//   https://codeforces.com/contest/2084/problem/G1
// limits: 4000 ms, 500 MB
// Problem: f(c) is the final c_1 when Turtle (maximiser, first) and Piggy alternately merge an
//   adjacent pair into its min (Turtle) or max (Piggy). Fill the zeros of a partial
//   permutation of length n <= 5000 to maximise the sum of f over all subarrays; print that
//   sum. Multitest, sum n <= 5000.
// Solution: only the parity of each value's position matters. Place values in increasing order;
//   with the k smallest placed, c of them on the n / 2 even positions, the beauty gains
//   (2c - k)(2(n / 2 - c) - (n - k)), the product of the small and large values' even-minus-odd
//   imbalances. DP over (k, c), a known value forced to its parity; the beauty is
//   (best + n(n + 1)^2 / 2) / 2. O(n^2).
// Bounds: n: 1..=5000
@!multitest
let n: 1..=5000 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let half = n / 2
let at = a.inverse()

var dp: [0..=half]?Int = None
dp[0] = 0
for k in 0..n:
    let forced = at[k + 1].map(p => p.label % 2 == 0)
    var next: [0..=half]?Int = None
    for c in 0..=min(k, half):
        if let x = dp[c]:
            let gain = (2 * c - k) * (2 * (half - c) - (n - k))
            for z in [false, true] if c + Int(z) <= half and (forced == None or forced == Some(z)):
                next[c + Int(z)] max= x + gain
    dp = next
print((dp[half]! + n * (n + 1) * (n + 1) / 2) / 2)
