// expect: ok, output "0 4 2"
// engines: next
use std.algebra.{MinCount}
let z = MinCount<Int>.zero
let m = z + MinCount.of(4) + MinCount.of(4)
print(z.count, m.val, m.count)
