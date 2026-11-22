// expect: ok, output "1\nYES"
use std.math.{jacobi}
print(jacobi(2, 7))
print((0..100).last_true(x => x * x <= 50) == Some(7) and (0..100).first_true(x => x * x > 50) == Some(8))
