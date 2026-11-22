// expect: ok, output "3 1 YES"
use std.matrix.{Mat}
let t = Mat<Int>.from_rows([[1, 2, 3]]).transpose()
print(t.rows, t.cols, yes(t.get(2, 0) == 3))
