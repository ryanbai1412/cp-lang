// expect: ok, output "YES 6"
use std.math.{ext_gcd}
let (g, x, y) = ext_gcd(240, -18)
print(yes(240 * x + -18 * y == g), g)
