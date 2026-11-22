// expect: ok, output "4 7"
use std.algebra.{MaxSub}
let a = MaxSub.of(3) + MaxSub.of(-5)
print((a + MaxSub.of(4)).try_best!, (a.rev() + MaxSub.of(4)).try_best!)
