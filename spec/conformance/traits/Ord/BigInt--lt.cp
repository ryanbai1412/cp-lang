// expect: ok, output "YES NO"
use std.math.{BigInt}
print(yes(BigInt.from_int(-5) < BigInt.from_int(3)), yes(BigInt.from_str("100000000000000000000") < BigInt.from_int(3)))
