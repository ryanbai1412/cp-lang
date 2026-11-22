// expect: R0401 at 4 (checked)
use std.mod.{Comb}
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.catalan(-1))
