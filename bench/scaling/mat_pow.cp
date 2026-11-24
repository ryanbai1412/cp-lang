// n: 32
// fixed: m, e, max_exp
use std.matrix.{Mat, MatPow, simplex}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = Z(0)
var rnd = [1]
for i in 1..n * n: rnd.push(rnd[i - 1] * 48271 % 2147483647)
let rows = [[Z(rnd[i * n + j] % 1000003) for j in 0..n] for i in 0..n]
let vec = [Z(i + 1) for i in 0..n]
let a = Mat<Z>.of(rows)
let p = MatPow(a, 16)
for _ in 0..32:
    acc += MatPow(a, 16).apply_pow(5, vec)[0] // timed: matrix.MatPow.new
for i in 0..64 * n:
    acc += p.apply_pow(i % 16, vec)[0] // timed: matrix.MatPow.apply_pow
let lp = Mat<F64>.identity(n)
let ones = [1.0 for _ in 0..n]
var val = 0.0
for _ in 0..640:
    val += (simplex(lp, ones, ones))!.val // timed: matrix.simplex
print(acc, val)
