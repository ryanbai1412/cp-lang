// expect: ok, output "YES"
use std.matrix.{Mat, solve_real}
print(yes(solve_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]), [1.0, 0.0]) == None))
