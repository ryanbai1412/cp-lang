// expect: ok, output "rank 1: 4"
use std.matrix.{XorBasis}
var b = XorBasis()
b.add(4)
print(b)
