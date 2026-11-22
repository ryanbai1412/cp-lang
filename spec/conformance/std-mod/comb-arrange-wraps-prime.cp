// expect: ok, output "2 0 0"
use std.mod.{Comb}
type Z = Mod<7>
let c = Comb<Z>()
print(c.arrange(9, 2), c.arrange(8, 2), c.arrange(3, 4))
