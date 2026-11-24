// let: k = \log n
use std.ntt.{subset_exp, subset_log}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
var z = [Z(i % 5) for i in 0..n]
z[0] = Z(0)
let e = subset_exp(z) // timed: ntt.subset_exp
acc ^= subset_log(e)[n - 1].v // timed: ntt.subset_log
print(acc)
