// Codeforces 2262 E: ans[k] = choose(2n, k) / (k + 1) * [u^(n+1)] (u * sum catalan(i)^2 u^i)^(k+1),
// then a Taylor shift by -1. The modulus is read per test, so values are plain
// `Int`s reduced by hand and series multiply through `convolve_mod`.
use lib.math.poly.{convolve_mod}

fn pow_mod(b: Int, e: Int, m: Int) -> Int:
    var (r, b, e) = (1 % m, b % m, e)
    while e > 0:
        if e % 2 == 1: r = r * b % m
        b = b * b % m
        e /= 2
    r

@!multitest
let (n, m): (Int, Int) = read()
let top = 2 * n + 1
var fact = [1 % m for _ in 0..=top]
for i in 1..=top: fact[i] = fact[i - 1] * i % m
var ifact = fact
ifact[top] = pow_mod(fact[top], m - 2, m)
for i in (1..=top).rev: ifact[i - 1] = ifact[i] * i % m

var symbol = [0 for _ in 0..n + 2]
for i in 0..=n:
    let cat = fact[2 * i] * ifact[i] % m * ifact[i + 1] % m
    symbol[i + 1] = cat * cat % m

var cur = [0 for _ in 0..n + 2]
cur[0] = 1 % m
var ans = [0 for _ in 0..=n]
for k in 0..=n:
    cur = convolve_mod(cur, symbol, m)[..n + 2]
    ans[k] = cur[n + 1] * fact[2 * n] % m * ifact[k + 1] % m * ifact[2 * n - k] % m

// ans(x - 1): out[j] = sum over k >= j of ans[k] * choose(k, j) * (-1)^(k - j)
var out = [0 for _ in 0..=n]
for j in 0..=n:
    var s = 0
    for k in j..=n:
        let term = ans[k] * fact[k] % m * ifact[k - j] % m
        s += if (k - j) % 2 == 0: term else: m - term
    out[j] = s % m * ifact[j] % m
print(out)
