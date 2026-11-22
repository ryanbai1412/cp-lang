// expect: ok, output "1 6"
use std.math.{mul_mod}
print(mul_mod(4_000_000_000_000_000_000, 4_000_000_000_000_000_000, 3), mul_mod(-1, 1, 7))
