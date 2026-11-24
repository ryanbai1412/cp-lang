// expect: ok, output "3x5 grid"
use std.strings.{Hash2D}
domain R = 0..3
domain C = 0..5
print(Hash2D<R, C>((r, c) => r.ord + c.ord))
