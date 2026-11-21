// AtCoder WTF 2026 C: 2D DP accelerated per step by 1D NTT convolution over
// flattened (i, j) coordinates, followed by two inclusion-exclusion passes.
use std.ntt.{convolve}

const MOD = 998_244_353
type Z = Mod<MOD>
let comb = Comb<Z>()

let n: Int = read()
let a: [0..n]Int = read()

var ways_groups: [0..=n]Z = 0
var claim_sum: [0..n, 0..n]Z = 0
for g_cnt in 1..=n:
    let k = g_cnt - 1
    var pk: [0..n]Z = 1
    var pk1: [0..n]Z = 1
    for i in 1..n:
        pk[i] = pk[i - 1] * k
        pk1[i] = pk1[i - 1] * (k + 1)
    var claim: [0..n, 0..n]Z = 0
    for i in 0..n:
        for j in 0..n - i:
            claim_sum[i, j] += pk[i] * pk1[j]
            claim[i, j] = claim_sum[i, j] * comb.choose(i + j, i)

    var dp: [0..=n, 0..=n]Z = 0
    dp[0, 0] = 1
    for cur in 0..n:
        let cur_a = Z(a[cur] + 1)
        let stride = 2 * cur + 1
        let max_j = n - 1 - cur
        let sz = stride * (max_j + 1)
        var mul_a: [0..sz]Z = 0
        var mul_b: [0..sz]Z = 0
        var ndp: [0..=n, 0..=n]Z = 0
        for i in 0..=cur:
            for j in 0..=max_j:
                ndp[i, j] += dp[i, j] * (cur + 1 - i)
                ndp[i + 1, j] += dp[i, j + 1] * (j + 1)
                let idx = i + stride * j
                mul_a[idx] = claim[i, j] * cur_a
                mul_b[idx] = dp[i, j]
        let res = convolve([mul_a[t] for t in 0..sz], [mul_b[t] for t in 0..sz])
        for i in 0..=cur:
            for j in 0..=max_j:
                ndp[i + 1, j] += res[i + stride * j]
        dp = ndp
    ways_groups[g_cnt] = dp[n, 0]

for g in 1..=n:
    for h in g + 1..=n:
        ways_groups[h] -= comb.choose(h, g) * ways_groups[g]

for g in 1..=n:
    for h in 1..g:
        ways_groups[g] -= ways_groups[h] * comb.choose(n - h, g - h)

print(ways_groups[1..=n])
