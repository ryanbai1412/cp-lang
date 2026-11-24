// n: 2048
// fixed: P, k
use std.poly.{deriv, integral, inv, log, exp, pow, sqrt, eval, div_mod, lagrange_at}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let a = [Z(i * 7 + 1) for i in 0..n]
let b = [Z(i * 13 + 5) for i in 0..n]
var z = [Z(i * 3 + 2) for i in 0..n]
z[0] = Z(0)
for _ in 0..512:
    acc ^= integral(a).len // timed: poly.integral
    acc ^= a.eval(Z(3)).v // timed: poly.eval
    acc ^= lagrange_at(a, Z(n * 3)).v // timed: poly.lagrange_at
for _ in 0..12:
    acc ^= inv(a, n).len // timed: poly.inv
    acc ^= log(a, n).len // timed: poly.log
    acc ^= exp(z, n).len // timed: poly.exp
    acc ^= pow(a, 5, n).len // timed: poly.pow
    acc ^= sqrt(a, n)!.len // timed: poly.sqrt
    acc ^= div_mod(a + b, b).q.len // timed: poly.div_mod
for _ in 0..1024:
    acc ^= deriv(a).len // timed: poly.deriv
print(acc)
