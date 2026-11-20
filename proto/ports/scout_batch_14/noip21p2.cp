// NOIP 2021 Sequence: count weighted length-n sequences over 0..=m whose sum of
// 2^{a_i} has popcount at most k, modulo 998244353.
// original: corpus/submissions/5445677.cpp
use std.mod.{Comb}  // WISH P7
const P = 998244353
type Z = Mod<P>
let n: 1..=30 = read()
let m: 0..=100 = read()
let k: 1..=30 = read()
assert k <= n
let val: [m + 1]0..=P - 1 = read()
let comb = Comb<P>.new(n)  // WISH P7

// Layer DP over bit positions 0..=m: dp[used, carry, ones] is the weighted sum
// of ways to place `used` items in lower bits with carry `carry` and `ones` set
// bits finalized so far.
type Layer = [0..=n, 0..=n, 0..=k]Z
var dp: Layer = 0
dp[0, 0, 0] = 1
for w in val:
    var next: Layer = 0
    for used in 0..=n:
        for carry in 0..=used:
            for ones in 0..=k if dp[used, carry, ones] != 0:  // WISH P9
                let cur = dp[used, carry, ones]
                var pw = Z(1)
                for take in 0..=n - used:
                    let bits = carry + take
                    let n_ones = ones + (bits & 1)
                    if n_ones <= k:
                        next[
                            used + take,
                            bits / 2,
                            n_ones,
                        ] += cur * pw * comb.choose(n - used, take)
                    pw *= w
    dp = next

var ans = Z(0)
for carry in 0..=n:
    for ones in 0..=k if ones + popcount(carry) <= k:  // WISH P11, P9
        ans += dp[n, carry, ones]
print(ans)
