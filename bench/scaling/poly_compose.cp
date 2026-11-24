// n: 2048
// fixed: P, k
use std.poly.{compose, compose_inv, pow_mod_poly, gcd_poly}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let a = [Z(i * 7 + 1) for i in 0..n]
let b = [Z(i * 13 + 5) for i in 0..n]
var z = [Z(i * 3 + 2) for i in 0..n]
z[0] = Z(0)
acc ^= compose(a, z, n).len // timed: poly.compose
acc ^= compose_inv(z, n).len // timed: poly.compose_inv
for _ in 0..4:
    acc ^= pow_mod_poly(a + b, 1 << 6, b).len // timed: poly.pow_mod_poly
for _ in 0..40:
    acc ^= gcd_poly(a, b).len // timed: poly.gcd_poly
print(acc)
