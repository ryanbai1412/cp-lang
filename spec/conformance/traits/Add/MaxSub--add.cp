// expect: ok, output "4"
use std.algebra.{MaxSub}
print((MaxSub.of(2) + MaxSub.of(-5) + MaxSub.of(3) + MaxSub.of(1)).try_best!)
