// n: 2048
// fixed: P, k
use std.poly.{deriv, integral, inv, log, exp, pow, sqrt, eval, div_mod, prod_linear, taylor_shift, binomial_transform, eval_all, compose, lagrange_at, interpolate, transposed_mul, compose_inv, pow_mod_poly, gcd_poly, chirp_z, sample_points_shift, power_sums, stirling1_row, stirling2_row, stirling1_col, stirling2_col, bell_table, bernoulli_table, partition_table}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let a = [Z(i * 7 + 1) for i in 0..n]
let b = [Z(i * 13 + 5) for i in 0..n]
var z = [Z(i * 3 + 2) for i in 0..n]
z[0] = Z(0)
let xs = [Z(i + 1) for i in 0..n]
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
    acc ^= sample_points_shift(a, Z(n * 5), n).len // timed: poly.sample_points_shift
    acc ^= stirling1_row<998_244_353>(n).len // timed: poly.stirling1_row
    acc ^= stirling2_row<998_244_353>(n).len // timed: poly.stirling2_row
    acc ^= stirling1_col<998_244_353>(3, n).len // timed: poly.stirling1_col
    acc ^= stirling2_col<998_244_353>(3, n).len // timed: poly.stirling2_col
    acc ^= bell_table<998_244_353>(n).len // timed: poly.bell_table
    acc ^= bernoulli_table<998_244_353>(n).len // timed: poly.bernoulli_table
    acc ^= partition_table<998_244_353>(n).len // timed: poly.partition_table
acc ^= eval_all(a, xs).len // timed: poly.eval_all
acc ^= compose(a, z, n).len // timed: poly.compose
acc ^= interpolate(xs, a).len // timed: poly.interpolate
acc ^= compose_inv(z, n).len // timed: poly.compose_inv
for _ in 0..1024:
    acc ^= deriv(a).len // timed: poly.deriv
for _ in 0..16:
    acc ^= taylor_shift(a, Z(3)).len // timed: poly.taylor_shift
    acc ^= binomial_transform(a).len // timed: poly.binomial_transform
    acc ^= transposed_mul(a + b, b).len // timed: poly.transposed_mul
    acc ^= chirp_z(a, Z(3), n).len // timed: poly.chirp_z
for _ in 0..2:
    acc ^= power_sums(xs, n).len // timed: poly.power_sums
for _ in 0..4:
    acc ^= prod_linear(xs).len // timed: poly.prod_linear
    acc ^= pow_mod_poly(a + b, 1 << 6, b).len // timed: poly.pow_mod_poly
for _ in 0..8:
    acc ^= gcd_poly(a, b).len // timed: poly.gcd_poly
print(acc)
