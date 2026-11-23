// expect: ok, output "YES"
use std.matrix.{det_real}
print(yes(abs(det_real(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]))) < 1e-9))
