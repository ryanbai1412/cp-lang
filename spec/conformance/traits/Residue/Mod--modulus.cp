// expect: ok, output "7"
use std.mod.{Residue}
type Z = Mod<7>
print(Z.modulus())
