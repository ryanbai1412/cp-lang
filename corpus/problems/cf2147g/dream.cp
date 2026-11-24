// Codeforces 2147 G — Multiplicative Order Prime-Factor Probability
//
// For `M = xs[0] * xs[1] * xs[2]`, let `pset` be the distinct prime divisors
// of `M`. Starting from `phi(M) / M = prod_{p in pset} (p - 1) / M`, each
// prime `q` dividing `prod_{p in pset} (p - 1)` with total multiplicity `e`
// independently fails to divide the random order with probability `q^(-e)`,
// and otherwise contributes an extra factor `0` (if `q | M`) or `q^(-1)`.


const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let xs: [0..3]1..=10 ** 6 = read()
var pset: Set<Int> = Set()
for x in xs:
    for (p, _) in factor(x):
        pset.insert(p)

var tot: Z = 1
var tot_qs: Map<Int, Int> = Map()
for p in pset:
    tot *= p - 1
    for (q, eq) in factor(p - 1):
        tot_qs[q] += eq

for x in xs: tot /= x
for (q, e) in tot_qs:
    let inv_q = Z(q).inv()
    let prob_no_q = inv_q ** e
    let has_q = any(x % q == 0 for x in xs)
    tot *= prob_no_q + (1 - prob_no_q) * (if has_q: 0 else: inv_q)
print(tot)
