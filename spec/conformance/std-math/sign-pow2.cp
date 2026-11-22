// expect: ok, output "YES"
use std.math.{Sign}
print((-5).sign == Sign.Neg and 0.sign == Sign.Zero and Int(7.sign) == 1 and 8.is_pow2 and 1.is_pow2 and not 6.is_pow2 and not 0.is_pow2)
