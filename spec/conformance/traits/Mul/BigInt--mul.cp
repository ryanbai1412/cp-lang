// expect: ok, output "10000000000000000000000"
use std.math.{BigInt}
print(BigInt.from_str("100000000000") * BigInt.from_str("100000000000"))
