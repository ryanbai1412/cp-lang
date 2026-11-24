// Source: NOIP 2021 P2 (Sequence), https://dmoj.ca/problem/noip21p2
// limits: 1000 ms, 512 MB
// Problem: n <= 30, m <= 100, k <= n and weights v_0..v_m. Over all arrays a of n values in
//   0..=m with at most k one bits in 2^a_1 + ... + 2^a_n, sum v_a_1 * ... * v_a_n mod 998244353.
// Solution: add bits 0..=m in order. dp[used, carry, ones]: weighted ways to give `used` of the
//   n positions values below the current bit, with that carry into it and `ones` one bits fixed.
//   Giving `take` more positions bit b multiplies by C(n - used, take) v_b^take and fixes bit
//   (carry + take) & 1; at the end the carry adds its popcount. O(m n^3 k), about 1e8.
// Bounds: n, k: 1..=30; m: 0..=100
// original: corpus/submissions/5445677.cpp
const P = 998244353
type Z = Mod<P>

let (n, m, k): (1..=30, 0..=100, 1..=30) = read()
let weight: [m + 1]Z = read()
let comb = Comb<Z>()

// dp[used, carry, ones]; the carry into bit b is at most used / 2, so it stays in 0..=n / 2
type Layer = [0..=n, 0..=n / 2, 0..=k]Z
var dp: Layer = 0
dp[0, 0, 0] = 1
for w in weight:
    var next: Layer = 0
    for used in 0..=n:
        for carry in 0..=used / 2:
            for ones in 0..=k:
                let cur = dp[used, carry, ones]
                if cur == 0: continue
                var power = Z(1)  // w ** take
                for take in 0..=n - used:
                    let bits = carry + take
                    let fixed = ones + bits % 2
                    if fixed <= k: next[used + take, bits / 2, fixed] += cur * power * comb.choose(n - used, take)
                    power *= w
    dp = next
print(sum(dp[n, carry, ones] for carry in 0..=n / 2 for ones in 0..=k if ones + popcount(carry) <= k))
