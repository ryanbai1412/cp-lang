// expect: R0401 at 3
use std.poly.{bell_table}
type Z = Mod<7>
let b: List<Z> = bell_table(7)
print(b.len)
