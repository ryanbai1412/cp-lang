// expect: ok, output "factorials up to 5"
use std.mod.{Mod, Comb}
type Z = Mod<11>
print(Comb<Z>(5))
