// expect: ok, output "5 5 0 2.5000000000 9223372036854775807\n2"
// rule: R12.14.1
// std.math's `abs` is an `@intrinsic` of its operand's type, on `Int` and `F64` alike.
use std.math.{gcd}
print((-5).abs, 5.abs, 0.abs, (-2.5).abs, (-9223372036854775807).abs)
print(gcd(-4, 6))
