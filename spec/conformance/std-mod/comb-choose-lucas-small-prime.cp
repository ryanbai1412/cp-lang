// expect: ok, output "1 0"
use std.mod.{Comb}
type Z = Mod<7>
let c = Comb<Z>()
print(c.choose(10, 3), c.choose(7, 1))
