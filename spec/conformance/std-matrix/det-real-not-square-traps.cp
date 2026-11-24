// expect: R0401 (checked)
use std.matrix.{det_real}
print(yes(det_real(Mat<F64>.from_rows([[1.0, 2.0]])) == 0.0)) // <- here
