// Source: Codeforces 2255F (Who Will Witness the End?),
//   https://codeforces.com/contest/2255/problem/F
// limits: 2000 ms, 256 MB
// Problem: n labeled values a[i] mod 998244353, 3 <= n <= 2e5. Over the (n - 1)! circular
//   orders p (rotations equal, reflections not), sum prod (a[p_i] + a[p_(i+1)]), mod
//   998244353. Multitest, sum of n <= 2e5.
// Solution: expand the product: each edge picks one end, so every value is picked 0, 1 or 2
//   times, with as many 0s as 2s (k of each). The code builds the elementary symmetric sums
//   e_k of a with prod_linear, takes v2[k] = e_k * e_(n-k) for k <= n/2, composes v2 with
//   x * C(x) (C the Catalan series), multiplies by C(x)^-n, weights term k by k!^2 / (2k)!
//   and the total by 2 (n - 1)!. NTT compose and pow: O(n log^2 n).
// Bounds: n: 1..=200_000

use std.poly.{compose, prod_linear}

// Tuned: C^-n, the Catalan series to the power -n, by its two-term coefficient recurrence
// instead of the generic series pow.

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
// [x^k] C^a is a polynomial in a with c(k + 1) (k + 1)(k + a + 1) = c(k) (2k + a)(2k + a + 1),
// so C^-n comes in O(n) instead of a series power; k + 1 - n is never 0 for k < h.
var cat_neg = [Z(1)]
for k in 0..h:
    cat_neg.push(cat_neg[k] * Z(2 * k - n) * Z(2 * k - n + 1) * comb.inv(k + 1) / Z(k + 1 - n))
let ways = convolve(compose(v2, cat1, h + 1), cat_neg)
let total = sum(ways[k] * comb.inv_fact(2 * k) * comb.fact(k) * comb.fact(k) for k in 0..=h)
print(total * comb.fact(n - 1) * 2)
