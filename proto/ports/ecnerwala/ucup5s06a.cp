// Universal Cup 5 Stage 6 (Hyakumanben) A: the answer is a coefficient of two
// logarithmic derivatives, x f'(x) / f(x), of series built from Catalan numbers.
use lib.math.poly.{Z, inv_trunc}

const MOD = 998244353
let comb = Comb<MOD>()

// [x^deg] of x p'(x) / p(x)
fn log_deriv_at(p: List<Z>, deg: Int) -> Z:
    let g = inv_trunc(p, deg + 1)
    sum(Z(i) * p[i] * g[deg - i] for i in 1..=deg)

@!multitest
let (n, k): (Int, Int) = read()
if k == 1:
    print(1)
    return

// 1 - sum of Catalan(i - 1) x^i for i < k
var f = List.filled(n + 1, Z(0))
f[0] = Z(1)
for i in 1..k: f[i] = -comb.catalan(i - 1)
let a = log_deriv_at(f, n)
f[1] -= Z(1)
print(Z(2) * a - log_deriv_at(f, n))
