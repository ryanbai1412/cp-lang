// expect: ok, output "7 13 11 12 12"
use std.algebra.{Affine}
print(Affine<Int>.id().eval(7), Affine(2, 3).eval(5), Affine.then(Affine(2, 0), Affine(1, 1)).eval(5), Affine.then(Affine(1, 1), Affine(2, 0)).eval(5), fold(Affine.monoid(), [Affine(1, 1), Affine(2, 0)]).eval(5))
