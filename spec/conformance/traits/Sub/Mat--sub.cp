// expect: ok, output "0 3 2"
use std.matrix.{Mat}
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
let s = a - Mat<Int>.identity(2)
print(s.get(0, 0), s.get(1, 1), s.get(0, 1))
