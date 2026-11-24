// n: 32
// fixed: m, e, max_exp
use std.matrix.{Mat}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = Z(0)
var k = 0
var rnd = [1]
for i in 1..n * n: rnd.push(rnd[i - 1] * 48271 % 2147483647)
let ints = [[rnd[i * n + j] % 1000003 for j in 0..n] for i in 0..n]
let rows = [[Z(x) for x in r] for r in ints]
let vec = [Z(i + 1) for i in 0..n]
let a = Mat<Z>.of(rows)
let ai = Mat<Int>.of(ints)
var sk = Mat<Z>.zeros(n, n)
for i in 0..n:
    for j in i + 1..n:
        sk.set(i, j, Z(i * 7 + j + 1))
        sk.set(j, i, -Z(i * 7 + j + 1))
for _ in 0..160:
    acc += a.det() // timed: matrix.Mat.det
    k += a.rank() // timed: matrix.Mat.rank
for _ in 0..96:
    k += a.rref().rank // timed: matrix.Mat.rref
    acc += (a.solve(vec))![0] // timed: matrix.Mat.solve
    acc += (a.solve_space(vec))!.0[0] // timed: matrix.Mat.solve_space
for _ in 0..128:
    acc += a.char_poly()[0] // timed: matrix.Mat.char_poly
    acc += sk.pfaffian() // timed: matrix.Mat.pfaffian
for _ in 0..40:
    acc += (a.inverse())!.get(0, 0) // timed: matrix.Mat.inverse
for _ in 0..48:
    k += ai.det_mod(1_000_000) // timed: matrix.Mat.det_mod
    acc += a.adjugate().get(0, 0) // timed: matrix.Mat.adjugate
print(acc, k)
