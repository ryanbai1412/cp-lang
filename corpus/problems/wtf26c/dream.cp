// AtCoder WTF 2026 C.
//
// For each g, `labelled[g]` counts the structures whose groups take labels
// from a set of g. It is `dp[n, 0]` of a 2D DP over the sorted `a`: after
// step `cur`, `dp[i, j]` has `i` closed and `j` open slots; a step either
// keeps a slot (weight `cur + 1 - i` or `j + 1`) or, weighted `a[cur] + 1`,
// joins the DP with the `claim` table of that g by a 2D convolution, where
// `claim[i, j] = C(i + j, i) * sum_{k < g} k^i (k + 1)^j`.
// Two binomial inversions then give the answer:
//   exact[h] = labelled[h] - sum_{g < h} C(h, g) exact[g]
//   ans[g]   = exact[g]    - sum_{h < g} C(n - h, g - h) ans[h]
use std.ntt.{convolve_2d}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

let n: Int = read()
let a: [0..n]Int = read()

var labelled: [0..=n]Z = 0
var power_sum: [0..=n, 0..=n]Z = 0
for g in 1..=n:
    for i in 0..n, j in 0..n - i: power_sum[i, j] += Z(g - 1) ** i * Z(g) ** j
    let claim = [0..=n, 0..=n] (i, j) => power_sum[i, j] * comb.choose(i + j, i)

    var dp: [0..=n, 0..=n]Z = 0
    dp[0, 0] = 1
    for cur in 0..n:
        let (rows, cols) = (cur + 1, n - cur)
        let joined = convolve_2d(claim, dp, shape: (rows, cols))
        var next: [0..=n, 0..=n]Z = 0
        for i in 0..rows, j in 0..cols:
            next[i, j] += dp[i, j] * (cur + 1 - i)
            next[i + 1, j] += dp[i, j + 1] * (j + 1) + joined[i, j] * (a[cur] + 1)
        dp = next
    labelled[g] = dp[n, 0]

let exact = labelled.binomial_inverse()
var ans = exact
for g in 1..=n, h in 1..g: ans[g] -= comb.choose(n - h, g - h) * ans[h]

print(ans[1..=n])
