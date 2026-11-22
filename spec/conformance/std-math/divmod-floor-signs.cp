// expect: ok, output "-3 2\n-1 3"
use std.math.{divmod}
let (q, r) = (-7).divmod(3)
print(q, r)
let (q2, r2) = 7.divmod(-4)
print(q2, r2)
