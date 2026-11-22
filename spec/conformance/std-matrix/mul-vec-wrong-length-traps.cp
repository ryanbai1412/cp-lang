// expect: R0401 at 2 (checked)
use std.matrix.{Mat}
print(Mat<Int>.from_rows([[1, 2], [3, 4]]).mul_vec([1]).len)
