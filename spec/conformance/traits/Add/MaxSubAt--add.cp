// expect: ok, output "3"
use std.algebra.{MaxSubAt}
domain D = 0..3
let s = MaxSubAt.of(D(0), 2) + MaxSubAt.of(D(1), -5) + MaxSubAt.of(D(2), 3)
print(s.try_best!)
