// NOIP 2021 Sequence: count weighted length-n sequences over 0..=m whose sum of
// 2^{a_i} has popcount at most k, modulo 998244353.
// original: corpus/submissions/5445677.cpp
const P = 998244353
type Z = Mod<P>
let n: 1..=30 = read()
let m: 0..=100 = read()
let k: 1..=30 = read() where k <= n
let val: [m + 1]0..=P - 1 = read()

// Pascal table for binomial coefficients choose(a, b) mod P.
var comb: [0..=n, 0..=n]Z = 0
for i in 0..=n:
    comb[i, 0] = 1
    for j in 1..=i: comb[i, j] = comb[i - 1, j - 1] + comb[i - 1, j]

fn popcount(x: Int) -> Int:
    var v = x
    var c = 0
    while v > 0:
        c += v & 1
        v = v / 2
    c

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
            for ones in 0..=k:
                let cur = dp[used, carry, ones]
                if cur == 0: continue
                var pw = Z(1)
                for take in 0..=n - used:
                    let bits = carry + take
                    let n_ones = ones + (bits & 1)
                    if n_ones <= k:
                        next[used + take, bits / 2, n_ones] += cur * pw * comb[n - used, take]
                    pw *= w
    dp = next

var ans = Z(0)
for carry in 0..=n:
    let extra = popcount(carry)
    for ones in 0..=k:
        if ones + extra <= k: ans += dp[n, carry, ones]
print(ans)
