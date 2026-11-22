// expect: R0401 at 2 (checked)
use std.matrix.{Mat, solve_real}
print(yes(solve_real(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1.0]]), [1.0]) == None))
