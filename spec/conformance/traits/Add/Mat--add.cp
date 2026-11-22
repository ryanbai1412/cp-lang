// expect: ok, output "4 6"
use std.matrix.{Mat}
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
let s = a + a
print(s.get(0, 1), s.get(1, 0))
