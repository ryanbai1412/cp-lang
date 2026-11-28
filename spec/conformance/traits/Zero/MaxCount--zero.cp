// expect: ok, output "0 7 1"
// engines: next
use std.algebra.{MaxCount}
let z = MaxCount<Int>.zero
let m = z + MaxCount.of(3) + MaxCount.of(7)
print(z.count, m.val, m.count)
