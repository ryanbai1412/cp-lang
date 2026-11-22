// Codeforces 2255 F — Vieta Catalan Composition
//
// Let `vieta(t) = prod_linear(-a)` be the Vieta polynomial of `-a`.
// Pairing complementary subsets of sizes `k` and `n - k` gives
// `v2[k] = vieta[k] * vieta[n - k]` for `k in 0..=n / 2`.
//
// Substituting the Catalan generating function `C(x) = sum_i C_i x^i`
// (`cat[i] = comb.catalan(i)`, `cat1(x) = x * C(x)` shifted to `cat[1..]`)
// computes `ways(x) = v2(cat1(x)) * C(x)^(-n)` via NTT composition and power.
// Weighting each `ways[k]` by `1 / C(2k, k) = inv_fact(2k) * fact(k)^2` and
// multiplying by `2 * (n - 1)!` yields the answer in O(n log^2 n).

use std.ntt.{convolve}
use std.poly.{compose, pow, prod_linear}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]Z = read()
let h = n / 2
let vieta = prod_linear([-x for x in a])
let v2 = [vieta[k] * vieta[n - k] for k in 0..=h]
let cat = [comb.catalan(i) for i in 0..=h]
let cat1 = [Z(0)] + cat[1..=h]
let ways = convolve(compose(v2, cat1, h + 1), pow(cat, -n, h + 1))
let total = sum(ways[k] * comb.inv_fact(2 * k) * comb.fact(k) * comb.fact(k) for k in 0..=h)
print(total * comb.fact(n - 1) * 2)
