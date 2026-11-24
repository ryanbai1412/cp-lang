// let: k = \log n
use std.ntt.{convolve_all}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
acc ^= convolve_all([[Z(1), Z(i)] for i in 0..n / 2]).len // timed: ntt.convolve_all
print(acc)
