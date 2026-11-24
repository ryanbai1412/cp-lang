// expect: ok, output "2 2"
use std.algebra.{MinCount}
let s = MinCount.of(2) + MinCount.of(5) + MinCount.of(2)
print(s.val, s.count)
