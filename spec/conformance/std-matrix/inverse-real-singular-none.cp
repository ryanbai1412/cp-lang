// expect: ok, output "YES"
use std.matrix.{inverse_real}
print(yes(inverse_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]])) == None))
