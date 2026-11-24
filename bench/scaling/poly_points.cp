// n: 2048
// fixed: P, k
use std.poly.{prod_linear, eval_all, interpolate, chirp_z, sample_points_shift, power_sums}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let a = [Z(i * 7 + 1) for i in 0..n]
var z = [Z(i * 3 + 2) for i in 0..n]
z[0] = Z(0)
let xs = [Z(i + 1) for i in 0..n]
for _ in 0..12:
    acc ^= sample_points_shift(a, Z(n * 5), n).len // timed: poly.sample_points_shift
for _ in 0..2:
    acc ^= eval_all(a, xs).len // timed: poly.eval_all
acc ^= interpolate(xs, a).len // timed: poly.interpolate
for _ in 0..32:
    acc ^= chirp_z(a, Z(3), n).len // timed: poly.chirp_z
for _ in 0..2:
    acc ^= power_sums(xs, n).len // timed: poly.power_sums
for _ in 0..4:
    acc ^= prod_linear(xs).len // timed: poly.prod_linear
print(acc)
