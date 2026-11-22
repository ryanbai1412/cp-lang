// expect: R0401 at 2 (checked)
use std.matrix.{Mat, det_real}
print(yes(det_real(Mat<F64>.from_rows([[1.0, 2.0]])) == 0.0))
