// expect: ok, output "37 118 1 0"
use std.matrix.{Mat}
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
let s = a ** 3
let i = a ** 0
print(s.get(0, 0), s.get(1, 1), i.get(0, 0), i.get(0, 1))
