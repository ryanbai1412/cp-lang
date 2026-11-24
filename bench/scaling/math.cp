// fixed: k, d, lcm, base, lo
// let: popcount = \log n
use std.math.{gcd, lcm, isqrt, ceil_sqrt, iroot, fixed, choose, choose2, ext_gcd, ceil_div, divmod, bit_len, digits, to_str, sign, Sign, bit, is_pow2, splitmix64, factor, divisors, inv_mod, crt, crt_all, mul_mod, pow_mod, is_prime, phi, primitive_root, mod_log, mod_sqrt, floor_sum, primes_in, prime_count, floor_quotients, each_divisor, submasks, dirichlet_convolve, dirichlet_inv, sum_phi, sum_mobius, sum_multiplicative, sieve_multiplicative, linear_diophantine, linear_congruence, mod_kth_root, mult_order, jacobi, carmichael, min_mod_in_range, continued_fraction, best_rational}
let n: Int = read()
var p = n
while not is_prime(p): p += 1
let g = primitive_root(p)
var acc = 0
var h = U64(0)
for i in 0..5 * n:
    let x = n + i % n
    let y = n + i * 7919 % n
    acc += gcd(x, y) // timed: math.gcd
    acc += lcm(x, y) // timed: math.lcm
    acc += x.isqrt() // timed: math.isqrt
    acc += ceil_sqrt(x) // timed: math.ceil_sqrt
    acc += iroot(x, 3) // timed: math.iroot
    acc += fixed(x, 7, 6).len // timed: math.fixed
    acc += choose(x, 3) // timed: math.choose
    acc += choose2(x) // timed: math.choose2
    acc += ext_gcd(x, y).g // timed: math.ext_gcd
    acc += x.ceil_div(7) // timed: math.ceil_div
    let (q, r) = x.divmod(7) // timed: math.divmod
    acc += q + r
    acc += bit_len(x) // timed: math.bit_len
    acc += x.digits().len // timed: math.digits
    acc += x.to_str().len // timed: math.to_str
    if x.sign() == Sign.Pos: acc += 1 // timed: math.sign
    if x.bit(i % 20): acc += 1 // timed: math.bit
    if x.is_pow2(): acc += 1 // timed: math.is_pow2
    h ^= splitmix64(h + U64(x)) // timed: math.splitmix64
    acc += inv_mod(x, p) ?? 0 // timed: math.inv_mod
    if crt(x % 7, 7, y % p, p) != None: acc += 1 // timed: math.crt
    acc += mul_mod(x, y, p) // timed: math.mul_mod
    acc += pow_mod(x, y, p) // timed: math.pow_mod
    if is_prime(x): acc += 1 // timed: math.is_prime
    acc += floor_sum(x, y, 3, 1) // timed: math.floor_sum
    acc += jacobi(x, 2 * y + 1) // timed: math.jacobi
    if linear_diophantine(x, y, x + y) != None: acc += 1 // timed: math.linear_diophantine
    if linear_congruence(x, y, p) != None: acc += 1 // timed: math.linear_congruence
    acc += min_mod_in_range(x, p, 0, n) ?? 0 // timed: math.min_mod_in_range
    acc += continued_fraction(x, y).len // timed: math.continued_fraction
    acc += best_rational(x, y, n).floor // timed: math.best_rational
for i in 0..n / 2:
    let x = n + i % n
    acc += mod_sqrt(x % p, p) ?? 0 // timed: math.mod_sqrt
for i in 0..n:
    let x = n + i * 31 % n
    acc += factor(x).len // timed: math.factor
    acc += x.divisors().len // timed: math.divisors
    acc += phi(x) // timed: math.phi
    acc += mult_order(x % (p - 1) + 1, p) ?? 0 // timed: math.mult_order
    acc += carmichael(x) // timed: math.carmichael
for i in 0..n / 32:
    let x = n + i * 31 % n
    acc += mod_log(g, x % p, p) ?? 0 // timed: math.mod_log
    acc += mod_kth_root(x % p, 3, p) ?? 0 // timed: math.mod_kth_root
for i in 0..n / 8:
    let x = n + i * 31 % n
    acc += primitive_root(p) // timed: math.primitive_root
for i in 0..n / 4:
    let x = n + i * 31 % n
    for e in each_divisor(x): acc += e // timed: math.each_divisor
for i in 0..n / 32:
    let x = n + i * 31 % n
    for f in floor_quotients(x): acc += f.q // timed: math.floor_quotients
let eqs = [(5, 7) for i in 0..n]
let ones = [if i == 0: 0 else: 1 for i in 0..=n]
var pi = List.filled(n + 1, 0)
for v in 2..=n: pi[v] = pi[v - 1] + (if is_prime(v): 1 else: 0)
var vs: List<Int> = []
for f in floor_quotients(n): vs.push(f.q)
let ps = [pi[v] for v in vs.rev]
for _ in 0..32:
    if crt_all(eqs) != None: acc += 1 // timed: math.crt_all
    acc += sieve_multiplicative(n, (q, e, qe) => e + 1).len // timed: math.sieve_multiplicative
    for s in submasks(n - 1): acc += s // timed: math.submasks
    acc += dirichlet_convolve(ones, ones, n).len // timed: math.dirichlet_convolve
    acc += dirichlet_inv(ones, n).len // timed: math.dirichlet_inv
for _ in 0..128:
    acc += primes_in(0, n).len // timed: math.primes_in
    acc += sum_phi(n) // timed: math.sum_phi
    acc += sum_mobius(n) // timed: math.sum_mobius
for _ in 0..512:
    acc += prime_count(n) // timed: math.prime_count
    acc += sum_multiplicative(n, (q, e, qe) => 1, ps) // timed: math.sum_multiplicative
print(acc)
print(h)
