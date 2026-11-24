// expect: ok, output "4 2"
use std.algebra.{MaxCount}
let s = MaxCount.of(4) + MaxCount.of(1) + MaxCount.of(4)
print(s.val, s.count)
