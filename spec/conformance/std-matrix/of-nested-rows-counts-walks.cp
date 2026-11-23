// expect: ok, output "6 3\n0 0"
use std.matrix.{Mat}
use std.mod.{Mod}
type Z = Mod<1_000_000_007>
let adj = [[Z(0), Z(1), Z(1)], [Z(1), Z(0), Z(1)], [Z(1), Z(1), Z(0)]]
let w = Mat<Z>.of(adj) ** 4
let total = Mat<Int>.of([[1, 2], [3, 4]])
print(w.get(0, 0), total.get(1, 0))
let e = Mat<Int>.of([])
print(e.rows, e.cols)
