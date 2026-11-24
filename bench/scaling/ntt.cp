// let: k = \log n
use std.ntt.{fwht}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
var a = [Z(i * 7 + 1) for i in 0..n]
var s = [i % 13 for i in 0..n]
var w = [i % 11 for i in 0..n]
for _ in 0..16:
    ntt(mut a) // timed: ntt.ntt
    ntt(mut a, inverse: true)
for _ in 0..96:
    fwht(mut w) // timed: ntt.fwht
    fwht(mut w, inverse: true)
for _ in 0..32:
    sos(mut s) // timed: ntt.sos
    sos(mut s, inverse: true)
print(acc, a[1].v, s[3], w[5])
