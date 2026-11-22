// expect: ok, output "YES YES\nYES YES\nYES YES YES YES\nYES"
use std.poly.{inv}
use std.poly.{eval, taylor_shift, lagrange_at, interpolate}
use std.mod.{Mod}
type Z = Mod<998244353>
let a1: List<Z> = [1, -1]
let a2: List<Z> = [2]
print(inv(a1, 4) == [Z(1), Z(1), Z(1), Z(1)], inv(a2, 2) == [Z(1) / Z(2), Z(0)])
print([1, 2, 3].eval(2) == 17, [5].eval(100) == 5)
let ys: List<Z> = [1, 4, 9]
let xs: List<Z> = [0, 1, 2]
let x2: List<Z> = [0, 1]
let y2: List<Z> = [5, 5]
print(lagrange_at(ys, Z(3)) == Z(16), lagrange_at(ys, Z(1)) == Z(4), interpolate(xs, ys) == [Z(1), Z(2), Z(1)], interpolate(x2, y2) == [Z(5), Z(0)])
let t: List<Z> = [0, 0, 1]
print(taylor_shift(t, Z(1)) == [Z(1), Z(2), Z(1)])
