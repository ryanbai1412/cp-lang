// expect: R0401
use std.poly.{bell_table}
type Z = Mod<7>
let b: List<Z> = bell_table(7) // <- here
print(b.len)
