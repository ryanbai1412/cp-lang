// expect: R0401
use std.poly.{stirling1_row}
type Z = Mod<998244353>
let s: List<Z> = stirling1_row(-1) // <- here
print(s.len)
