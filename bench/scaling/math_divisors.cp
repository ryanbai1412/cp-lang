// fixed: k, d, lcm, base, lo
// let: popcount = \log n
use std.math.{factor, divisors, is_prime, phi, primitive_root, mod_log, floor_quotients, each_divisor, mod_kth_root, mult_order, carmichael}
let n: Int = read()
var p = n
while not is_prime(p): p += 1
let g = primitive_root(p)
var acc = 0
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
print(acc)
