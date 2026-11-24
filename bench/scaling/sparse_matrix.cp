// n: 256
use std.matrix.{sparse_det, sparse_solve}
type Z = Mod<998_244_353>
let n: Int = read()
var e: List<(Int, Int, Z)> = []
for i in 0..n:
    e.push((i, i, Z(i + 2)))
    for d in 1..5: e.push((i, (i * 7 + d * 13) % n, Z(i * d + 1)))
let b = [Z(i + 1) for i in 0..n]
var acc = Z(0)
for _ in 0..4:
    acc += sparse_det(n, e) // timed: matrix.sparse_det
    acc += (sparse_solve(n, e, b))![0] // timed: matrix.sparse_solve
print(acc)
