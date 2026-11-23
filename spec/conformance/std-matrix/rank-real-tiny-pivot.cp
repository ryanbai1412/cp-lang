// expect: ok, output "1"
use std.matrix.{rank_real}
print(rank_real(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1e-12]])))
