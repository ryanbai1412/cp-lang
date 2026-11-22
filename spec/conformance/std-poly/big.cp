// expect: ok, output "YES\nYES"
use std.poly.{interpolate, eval_all, inv}
use std.mod.{Mod}
type Z = Mod<998244353>
let xs: List<Z> = [Z(i) for i in 0..100]
let ys: List<Z> = [Z(i * i * i + 7) for i in 0..100]
print(yes(eval_all(interpolate(xs, ys), xs) == ys))
let a: List<Z> = [Z(i + 1) for i in 0..200]
let b = inv(a, 200)
var ok = true
for k in 0..200:
    var s = Z(0)
    for i in 0..=k: s = s + a[i] * b[k - i]
    if s != (if k == 0: Z(1) else: Z(0)): ok = false
print(yes(ok))
