use lib.math.poly.{Z, convolve, compose_trunc, pow_trunc, prod_linear}

const MOD = 998244353

@!multitest
let n: Int = read()
let a: [0..n]Z = read()
let comb = Comb<MOD>()
let vieta = prod_linear([-a[i] for i in 0..n])
let h = n / 2
let v2: List<Z> = [vieta[k] * vieta[n - k] for k in 0..=h]
let cat: List<Z> = [comb.fact(2 * i) * comb.inv_fact(i) * comb.inv_fact(i + 1) for i in 0..=h]
var cat1 = cat
cat1[0] = 0
let ways = convolve(compose_trunc(v2, cat1, h + 1), pow_trunc(cat, -n, h + 1))
var ans: Z = 0
for k in 0..=h:
    ans += ways[k] * comb.fact(n) * comb.inv_fact(2 * k) * comb.fact(k) * comb.fact(k) * 2
let zn: Z = n
print(ans / zn)
