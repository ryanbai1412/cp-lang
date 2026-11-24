// n: 4096
use std.ntt.{RelaxedConvolve, SemiRelaxedConvolve}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let b = [Z(i % 7) for i in 0..n]
for _ in 0..1:
    var rc = RelaxedConvolve<998_244_353>()
    var sc = SemiRelaxedConvolve<998_244_353>(b)
    for i in 0..n:
        acc ^= rc.push(Z(i), Z(i + 1)).v // timed: ntt.RelaxedConvolve.push
        acc ^= sc.push(Z(i)).v // timed: ntt.SemiRelaxedConvolve.push
for i in 0..20 * n:
    var r = RelaxedConvolve<998_244_353>() // timed: ntt.RelaxedConvolve.new
    acc ^= r.push(Z(i), Z(1)).v
for _ in 0..2048:
    var r = SemiRelaxedConvolve<998_244_353>(b) // timed: ntt.SemiRelaxedConvolve.new
    acc ^= r.push(Z(1)).v
print(acc)
