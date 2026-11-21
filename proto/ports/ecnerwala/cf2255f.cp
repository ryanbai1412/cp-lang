// Codeforces 2255 F: Vieta product + Catalan composition / negative power via NTT.
use lib.math.poly.{Z, compose_trunc, convolve, pow_trunc, prod_linear}

let comb = Comb<Z>()

@!multitest
let n: Int = read()
let a: [0..n]Z = read()
let h = n / 2
let vieta = prod_linear([-x for x in a])
let v2 = [vieta[k] * vieta[n - k] for k in 0..=h]
let cat = [comb.fact(2 * i) * comb.inv_fact(i) * comb.inv_fact(i + 1) for i in 0..=h]
let cat1 = [Z(0)] + cat[1..=h]
let ways = convolve(compose_trunc(v2, cat1, h + 1), pow_trunc(cat, -n, h + 1))
let total = sum(ways[k] * comb.inv_fact(2 * k) * comb.fact(k) * comb.fact(k) for k in 0..=h)
print(total * comb.fact(n - 1) * 2)
