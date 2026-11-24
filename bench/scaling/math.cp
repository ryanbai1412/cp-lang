// fixed: k, d, lcm, base, lo
// let: popcount = \log n
use std.math.{gcd, lcm, isqrt, ceil_sqrt, iroot, fixed, choose, choose2, ext_gcd, ceil_div, divmod, bit_len, digits, to_str, sign, Sign, bit, is_pow2, splitmix64, inv_mod, crt, mul_mod, pow_mod, is_prime, primitive_root, mod_sqrt, floor_sum, linear_diophantine, linear_congruence, jacobi, min_mod_in_range, continued_fraction, best_rational}
let n: Int = read()
var p = n
while not is_prime(p): p += 1
let g = primitive_root(p)
var acc = 0
var h = U64(0)
for i in 0..8 * n:
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
print(acc)
print(h)
