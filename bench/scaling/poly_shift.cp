// n: 2048
// fixed: P, k
use std.poly.{taylor_shift, binomial_transform, transposed_mul}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let a = [Z(i * 7 + 1) for i in 0..n]
let b = [Z(i * 13 + 5) for i in 0..n]
for _ in 0..32:
    acc ^= taylor_shift(a, Z(3)).len // timed: poly.taylor_shift
    acc ^= binomial_transform(a).len // timed: poly.binomial_transform
    acc ^= transposed_mul(a + b, b).len // timed: poly.transposed_mul
print(acc)
