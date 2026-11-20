// Codeforces 2268 E: per bit, the sign prefix products a[0..=n] contribute
// sum over gaps d of weight[d] * sum_i a[i] * a[i + d], an autocorrelation.
use lib.math.poly.{Z, convolve}

const MOD = 998244353
let comb = Comb<MOD>()

@!multitest
let n: Int = read()
let xs: [0..n]Int = read()
let weight = [if d == 0: Z(0) else: comb.catalan(d) * comb.catalan(n - d) for d in 0..n]
var ans = Z(0)
for b in 0..18:
    var a = [Z(1) for _ in 0..=n]
    for i in 1..=n: a[i] = if xs[i - 1] >> b & 1 == 1: -a[i - 1] else: a[i - 1]
    var cur = comb.catalan(n) * Z(n - 1)
    if a[n] == Z(1):
        // corr[n - d] = sum_i a[i] * a[i + d]
        let corr = convolve(a, [a[n - i] for i in 0..=n])
        for d in 1..n: cur -= weight[d] * corr[n - d]
    ans += cur * Z(1 << b)
print(ans)
