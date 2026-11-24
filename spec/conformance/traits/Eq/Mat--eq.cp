// expect: ok, output "YES NO NO"
use std.matrix.{Mat}
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
print(yes(a == Mat<Int>.from_rows([[1, 2], [3, 4]])), yes(a == a.transpose()), yes(a == Mat<Int>.from_rows([[1, 2]])))
