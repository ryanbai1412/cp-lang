// expect: ok, output "0 0 1 0"
use std.mod.{Comb}
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.choose(5, 6), c.choose(5, -1), c.choose(0, 0), c.choose(-3, 1))
