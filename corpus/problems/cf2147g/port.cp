// Source: Codeforces 2147G (Modular Tetration), https://codeforces.com/contest/2147/problem/G
// limits: 2000 ms, 256 MB
// Problem: m = x y z with x, y, z <= 1e6 (multitest). a is m-tetrative when the tower b_0 = 1,
//   b_n = a^(b_(n-1)) is eventually 1 mod m. Print the density of m-tetrative integers a, mod
//   998244353.
// Solution: the tower's exponent grows without bound, so a qualifies iff gcd(a, m) = 1 and every
//   prime q of ord_m(a) divides a. Over the units mod m, q misses the order with probability
//   q^-v_q(phi(m)); the p^(k-1) parts of phi fold into the start prod (p - 1) / m over the primes
//   p | m, leaving e = v_q(prod (p - 1)). Otherwise q | a has probability 0 if q | m, else 1/q by
//   CRT. Factor x, y, z and each p - 1: a product over the primes q.
// Bounds: xs: 1..=10 ** 6


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
    let has_q = xs.any(x => x % q == 0)
    tot *= prob_no_q + (1 - prob_no_q) * (if has_q: 0 else: inv_q)
print(tot)
