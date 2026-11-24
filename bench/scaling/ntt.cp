// let: k = \log n
use std.ntt.{ntt, convolve, convolve_2d, convolve_all, fwht, sos, subset_exp, subset_log}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
var a = [Z(i * 7 + 1) for i in 0..n]
let b = [i % 1000 for i in 0..n]
let c = [i % 997 for i in 0..n]
var s = [i % 13 for i in 0..n]
var w = [i % 11 for i in 0..n]
var z = [Z(i % 5) for i in 0..n]
z[0] = Z(0)
for _ in 0..16:
    ntt(mut a) // timed: ntt.ntt
    ntt(mut a, inverse: true)
for _ in 0..96:
    fwht(mut w) // timed: ntt.fwht
    fwht(mut w, inverse: true)
for _ in 0..32:
    sos(mut s) // timed: ntt.sos
    sos(mut s, inverse: true)
acc ^= convolve(b, c).len // timed: ntt.convolve
acc ^= convolve(c, b).len // timed: ntt.convolve
let g2 = [[Z(r * n + j) for j in 0..n / 8] for r in 0..4]
for _ in 0..4:
    acc ^= convolve_2d(g2, g2).len // timed: ntt.convolve_2d
acc ^= convolve_all([[Z(1), Z(i)] for i in 0..n / 2]).len // timed: ntt.convolve_all
let e = subset_exp(z) // timed: ntt.subset_exp
acc ^= subset_log(e)[n - 1].v // timed: ntt.subset_log
print(acc, a[1].v, s[3], w[5])
