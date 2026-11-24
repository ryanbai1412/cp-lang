// n: 32
// fixed: m, e, max_exp
use std.matrix.{Mat}
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
for _ in 0..4096:
    acc += Mat<Z>.of(rows).get(1, 1) // timed: matrix.Mat.of
for _ in 0..4096:
    acc += Mat<Z>.from_rows(rows).get(1, 1) // timed: matrix.Mat.from_rows
for _ in 0..12288:
    acc += Mat<Z>.from_flat(n, n, flat).get(1, 1) // timed: matrix.Mat.from_flat
for _ in 0..32768:
    acc += Mat<Z>.zeros(n, n).get(1, 1) // timed: matrix.Mat.zeros
for _ in 0..8192:
    acc += Mat<Z>.filled(n, n, Z(3)).get(1, 1) // timed: matrix.Mat.filled
for _ in 0..32768:
    acc += Mat<Z>.identity(n).get(1, 1) // timed: matrix.Mat.identity
for _ in 0..1280:
    acc += Mat([R, R] (r, c) => Z(Int(r) * Int(c) + 1)).get(1, 1) // timed: matrix.Mat.new
for _ in 0..16384:
    acc += a0.transpose().get(1, 1) // timed: matrix.Mat.transpose
for _ in 0..2048:
    acc += a0.mul_vec(vec)[0] // timed: matrix.Mat.mul_vec
for _ in 0..16384:
    acc += a0.sum() // timed: matrix.Mat.sum
for _ in 0..65536:
    acc += Mat<Z>.product(n).id.get(0, 0) // timed: matrix.Mat.product
var g = Mat<Z>.of(rows)
for i in 0..1 << 18:
    let (r, c) = (i % n, i * 7 % n)
    acc += g.get(r, c) // timed: matrix.Mat.get
    g.set(c, r, Z(i)) // timed: matrix.Mat.set
    g.at(r, r) += Z(1) // timed: matrix.Mat.at
var sk = Mat<Z>.zeros(n, n)
for i in 0..n:
    for j in i + 1..n:
        sk.set(i, j, Z(i * 7 + j + 1))
        sk.set(j, i, -Z(i * 7 + j + 1))
var val = 0.0
print(acc, k, val)
