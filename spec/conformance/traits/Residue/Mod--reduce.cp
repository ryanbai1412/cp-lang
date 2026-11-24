// expect: ok, output "5 0"
use std.mod.{Residue}
type Z = Mod<7>
print(Z.reduce(40), Z.reduce(7))
