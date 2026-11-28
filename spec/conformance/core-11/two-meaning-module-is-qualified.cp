// expect: ok, output "2 6 | 2"
// rule: 11.3.3
// `std.poly`'s series fns share names with float builtins, so the module is reached qualified.
use std.poly
type Z = Mod<998244353>
let a = [Z(1), Z(2), Z(3)]
print(" ".join(["{c.v}" for c in poly.deriv(a)]), "|", floor(sqrt(4.0)))
