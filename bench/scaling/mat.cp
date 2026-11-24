// n: 32
// fixed: m, e, max_exp
use std.matrix.{Mat, MatPow, simplex}
type Z = Mod<998_244_353>
let n: Int = read()
domain R = 0..n
var acc = Z(0)
var k = 0
var rnd = [1]
for i in 1..n * n: rnd.push(rnd[i - 1] * 48271 % 2147483647)
let ints = [[rnd[i * n + j] % 1000003 for j in 0..n] for i in 0..n]
let rows = [[Z(x) for x in r] for r in ints]
let flat = [Z(i * 37 % 1009) for i in 0..n * n]
let vec = [Z(i + 1) for i in 0..n]
let a0 = Mat<Z>.of(rows)
for _ in 0..2048:
    acc += Mat<Z>.of(rows).get(1, 1) // timed: matrix.Mat.of
for _ in 0..2048:
    acc += Mat<Z>.from_rows(rows).get(1, 1) // timed: matrix.Mat.from_rows
for _ in 0..6144:
    acc += Mat<Z>.from_flat(n, n, flat).get(1, 1) // timed: matrix.Mat.from_flat
for _ in 0..32768:
    acc += Mat<Z>.zeros(n, n).get(1, 1) // timed: matrix.Mat.zeros
for _ in 0..8192:
    acc += Mat<Z>.filled(n, n, Z(3)).get(1, 1) // timed: matrix.Mat.filled
for _ in 0..32768:
    acc += Mat<Z>.identity(n).get(1, 1) // timed: matrix.Mat.identity
for _ in 0..1280:
    acc += Mat([R, R] (r, c) => Z(Int(r) * Int(c) + 1)).get(1, 1) // timed: matrix.Mat.new
for _ in 0..4096:
    acc += a0.transpose().get(1, 1) // timed: matrix.Mat.transpose
for _ in 0..2048:
    acc += a0.mul_vec(vec)[0] // timed: matrix.Mat.mul_vec
for _ in 0..8192:
    acc += Mat<Z>.product(n).id.get(0, 0) // timed: matrix.Mat.product
var g = Mat<Z>.of(rows)
for i in 0..1 << 18:
    let (r, c) = (i % n, i * 7 % n)
    acc += g.get(r, c) // timed: matrix.Mat.get
    g.set(c, r, Z(i)) // timed: matrix.Mat.set
    g.at(r, r) += Z(1) // timed: matrix.Mat.at
let a = Mat<Z>.of(rows)
let ai = Mat<Int>.of(ints)
var sk = Mat<Z>.zeros(n, n)
for i in 0..n:
    for j in i + 1..n:
        sk.set(i, j, Z(i * 7 + j + 1))
        sk.set(j, i, -Z(i * 7 + j + 1))
let p = MatPow(a, 16)
for _ in 0..160:
    acc += a.det() // timed: matrix.Mat.det
    k += a.rank() // timed: matrix.Mat.rank
for _ in 0..96:
    k += a.rref().rank // timed: matrix.Mat.rref
    acc += (a.solve(vec))![0] // timed: matrix.Mat.solve
    acc += (a.solve_space(vec))!.0[0] // timed: matrix.Mat.solve_space
for _ in 0..64:
    acc += a.char_poly()[0] // timed: matrix.Mat.char_poly
    acc += sk.pfaffian() // timed: matrix.Mat.pfaffian
for _ in 0..40:
    acc += (a.inverse())!.get(0, 0) // timed: matrix.Mat.inverse
for _ in 0..24:
    k += ai.det_mod(1_000_000) // timed: matrix.Mat.det_mod
    acc += a.adjugate().get(0, 0) // timed: matrix.Mat.adjugate
for _ in 0..16:
    acc += MatPow(a, 16).apply_pow(5, vec)[0] // timed: matrix.MatPow.new
for i in 0..64 * n:
    acc += p.apply_pow(i % 16, vec)[0] // timed: matrix.MatPow.apply_pow
let lp = Mat<F64>.identity(n)
let ones = [1.0 for _ in 0..n]
var val = 0.0
for _ in 0..320:
    val += (simplex(lp, ones, ones))!.val // timed: matrix.simplex
print(acc, k, val)
