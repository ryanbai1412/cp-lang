// Codeforces 2262 E — Catalan-Squared Generating Function & Taylor Shift
//
// Compute `ans[k] = choose(2n, k) / (k + 1) * [u^(n+1)] (u * sum C_i^2 u^i)^(k+1)`
// modulo the runtime prime `m` (using `convolve_mod` over `Int` coefficient
// lists), then apply the binomial Taylor shift `ans(x - 1)`:
// `out[j] = sum_{k=j..n} ans[k] * choose(k, j) * (-1)^(k - j)`.

use std.ntt.{convolve_mod}

@!multitest
let (n, m): (1..=500, 2..=2 * 10 ** 9) = read()
type Z = Mod<m>
let comb = Comb<Z>()

let symbol = [if i == 0: 0 else: (comb.catalan(i - 1) * comb.catalan(i - 1)).v for i in 0..=n + 1]
var cur = [if i == 0: 1 % m else: 0 for i in 0..=n + 1]
var ans: [0..=n]Z = 0
for k in 0..=n:
    cur = convolve_mod(cur, symbol, m)[..n + 2].to_list()
    ans[k] = cur[n + 1] * comb.choose(2 * n, k) * comb.inv(k + 1)

fn shift(j: Int) -> Int:
    var tot: Z = 0
    for k in j..=n:
        let term = ans[k] * comb.choose(k, j)
        tot += if (k - j) % 2 == 0: term else: -term
    tot.v

print([shift(j) for j in 0..=n])
