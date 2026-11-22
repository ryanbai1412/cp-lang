// expect: ok, output "0 1"
use std.mod.{PrimePowerComb}
let c = PrimePowerComb(3, 2)
print(c.choose(5, 7), c.choose(5, 5))
