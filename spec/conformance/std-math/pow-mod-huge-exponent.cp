// expect: ok, output "1 1"
use std.math.{pow_mod}
print(pow_mod(2, 1_000_000_006, 1_000_000_007), pow_mod(10, 4_000_000_000_000_000_000, 3))
