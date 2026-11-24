// expect: ok, output "1 2 1\n1 3 3"
use std.poly.{pow}
use std.mod.{Mod}
const P = 998244353
print(pow([Mod<P>(1), Mod<P>(1)], 2, 3))
print(pow([Mod<P>(1), Mod<P>(1)], 3, 3))
