// expect: ok, output "0 0"
use std.mod.{ModComb}
let c = ModComb(1)
print(c.choose(5, 2), c.choose(0, 0))
