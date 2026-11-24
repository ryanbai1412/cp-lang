// expect: R0401 (checked)
use std.matrix.{Mat}
let m = Mat<Int>.of([[1, 1], [1, 0]])
print(m.get(0, 2)) // <- here
