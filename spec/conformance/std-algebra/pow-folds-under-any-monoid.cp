// expect: ok, output "1024 1 15 9 7"
// rule: std.algebra.pow

use std.algebra.{Affine}

print(2 ** 10, 5 ** 0, pow(3, 5, op: +), pow(9, 4, op: max), pow(Affine(2, 1), 3, op: Affine.monoid()).eval(0))
