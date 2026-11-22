// expect: ok, output "3 4 0 3037000499 4 4 0 2 3 5 0 1000000"
use std.math.{isqrt, ceil_sqrt, iroot}
print(isqrt(15), isqrt(16), isqrt(0), isqrt(9_223_372_036_854_775_807), ceil_sqrt(15), ceil_sqrt(16), ceil_sqrt(0), iroot(26, 3), iroot(27, 3), iroot(5, 1), iroot(0, 5), iroot(1_000_000_000_000_000_000, 3))
