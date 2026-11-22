// expect: ok, output "YES"
use std.matrix.{Mat}
print(yes(Mat<Int>.zeros(2, 0).mul_vec([]) == [0, 0]))
