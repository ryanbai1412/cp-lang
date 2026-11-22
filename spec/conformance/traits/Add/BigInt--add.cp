// expect: ok, output "100000000000000000000"
use std.math.{BigInt}
print(BigInt.from_str("99999999999999999999") + BigInt.from_int(1))
