// expect: ok, output "5 6"
use std.mod.{Residue}
type Z = Mod<7>
print(Z(12).residue(), Z(-1).residue())
