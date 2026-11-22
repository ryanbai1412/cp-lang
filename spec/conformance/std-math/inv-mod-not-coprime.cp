// expect: ok, output "-1 4"
use std.math.{inv_mod}
print(inv_mod(6, 9) ?? -1, inv_mod(2, 7)!)
