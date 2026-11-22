// expect: ok, output "0 1"
use std.mod.{ModComb}
let c = ModComb(12)
print(c.choose(3, 4), c.choose(3, 0))
