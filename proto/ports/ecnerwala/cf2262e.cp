// Codeforces 2262 E: ans[k] = choose(2n, k) / (k + 1) * [u^(n+1)] (u * sum catalan(i)^2 u^i)^(k+1),
// then a Taylor shift by -1. The modulus is read per test, so `Z` and `Comb`
// take it at runtime; the series stay plain `Int`s for `convolve_mod`.
use lib.math.poly.{convolve_mod}

@!multitest
let (n, m): (Int, Int) = read()
type Z = Mod<m>
let comb = Comb<m>()

var symbol = List.filled(n + 2, 0)
for i in 0..=n: symbol[i + 1] = (comb.catalan(i) * comb.catalan(i)).v

var cur = List.filled(n + 2, 0)
cur[0] = 1 % m
var ans = List.filled(n + 1, Z(0))
for k in 0..=n:
    cur = convolve_mod(cur, symbol, m)[..n + 2]
    ans[k] = Z(cur[n + 1]) * comb.fact(2 * n) * comb.inv_fact(k + 1) * comb.inv_fact(2 * n - k)

// ans(x - 1): out[j] = sum over k >= j of ans[k] * choose(k, j) * (-1)^(k - j)
let out = [
    (
        sum(
            (
                if (k - j) % 2 == 0: 1 else: -1
            ) * ans[k] * comb.fact(k) * comb.inv_fact(k - j) for k in j..=n
        ) * comb.inv_fact(j)
    ).v for j in 0..=n
]
print(out)
